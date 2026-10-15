// P7 neutrino heating (m0_contrat.md §1.3.2, §1.3.4, §2.5.4): appended to fft.wgsl, whose helpers (dit, dif, bitrev), workgroup
// row, bindings 0–6 and entry points `conv_rows` and `conv_rows_inverse` it shares; step/nuheat.rs builds the dispatches.
// M0-T53's twin (sr_physics::reference::nuheat) in f32: D(x) = Σ(x) · (S_ν ∗ K_ν)(x) over the active box, K_ν(r) = 1/(r² + 1),
// D = 0 where X_n ≥ 0.5, and q(x) = f_dep · (Σ_y S_ν(y)) · D(x) / Σ_x D(x), added to E over Δt_n and booked
// `neutrino_deposited` at the cell.
//
// K_ν is real and even in both axes, so its transform is stored as P2's is (`kquarter`, `koffsets`: grav.wgsl's layout, one
// real quarter per padded size the world's boxes can take, indexed by (log2 nx − 5) · ny_sizes + log2 ny − 5).
//
// The dispatches, in order (the first two fixed, the rest `Chained`: sized by `sizes`, which `nu_gate` writes and `nu_close`
// zeroes, so a step with nothing emitted runs none of them and a frame's latch, which stops `nu_gate`, stops them too):
// - `nu_tiles`, over the box's 8 × 8 tiles: each tile's Σ S_ν by a fixed tree, into `st.tile`;
// - `nu_gate`, one workgroup: the tiles' sum Σ S_ν in a fixed order; > 0 opens the sizes (rows, columns, tiles, one),
//   else they stay 0;
// - `conv_rows`, `nu_columns`, `conv_rows_inverse`: reach(x) = (S_ν ∗ K_ν)(x) into the box's cells of `dst`;
// - `nu_deposit`, over the tiles: D(x) in place of reach, each tile's Σ D by a fixed tree into `st.tile`'s second half;
// - `nu_share`, one workgroup: Σ D in a fixed order; the scale f_dep ΣS_ν / ΣD, 0 when ΣD = 0 (every cell with mass is a neutron
//   star: nothing is handed to anyone);
// - `nu_apply`, over the tiles: E += scale · D · Δt_n, booked at the cell;
// - `nu_close`, one invocation: the sizes back to 0.
// No atomic anywhere: the same state gives the same bits (§1.3.4).

struct NuParams {
    f_dep: f32,
    // the kernel table's lengths per y axis: 32, 64, … (ny_sizes of them)
    ny_sizes: u32,
    // cells in a plane
    cells: u32,
    // the world's 8 × 8 tiles: the slots of each half of `st.tile`
    tiles: u32,
}

// step/nuheat.rs's layout.
struct NuState {
    // `nu_gate` runs since the step was made
    steps: u32,
    // 1 when the last `nu_gate` opened the sizes
    open: u32,
    // `steps` as of the last `nu_share` that ran: equal to `steps` when the last gate's step ran the chain
    share_stamp: u32,
    _pad0: u32,
    // Σ S_ν, Σ D and the scale of the last step
    emitted: f32,
    total: f32,
    scale: f32,
    _pad1: f32,
    // Σ S_ν per tile, then Σ D per tile
    tile: array<f32>,
}

@group(0) @binding(7) var<uniform> np: NuParams;
@group(0) @binding(8) var<storage, read> kquarter: array<f32>;
@group(0) @binding(9) var<storage, read> koffsets: array<u32>;
@group(0) @binding(10) var<storage, read_write> st: NuState;
// The state's Σ, momenta and E planes (E is where the heat goes)
@group(0) @binding(11) var<storage, read_write> hydro: array<f32>;
// The state's species planes 9–13 (X_n is the fifth)
@group(0) @binding(12) var<storage, read> species_b: array<f32>;
// booking.neutrino_heating: neutrino_deposited's plane, one slot per cell
@group(0) @binding(13) var<storage, read_write> book: array<f32>;
// Δt_n, the record's first word
@group(0) @binding(14) var<uniform> dt_rec: vec4<f32>;
// The four indirect sizes, 3 words each: rows (box h), columns (fft_w), tiles (the box's), one
@group(0) @binding(15) var<storage, read_write> sizes: array<u32, 12>;

// §2.5.4: a cell with X_n at or above this is the neutron star, which receives nothing.
const X_N_STAR: f32 = 0.5;
// Where X_n sits in `species_b` (the planes 9–13 of §2.2: neutron matter is the fifth)
const X_N_PLANE: u32 = 4u;

var<workgroup> part: array<f32, 64>;
var<workgroup> lanes: array<f32, 256>;

// The 64 values of `part`, summed by a fixed tree into part[0].
fn tile_sum(li: u32) {
    workgroupBarrier();
    for (var s = 32u; s > 0u; s = s / 2u) {
        if (li < s) {
            part[li] = part[li] + part[li + s];
        }
        workgroupBarrier();
    }
}

// `n` consecutive floats of `st.tile` from `first`, summed in a fixed order: 256 strided partial sums, then a tree.
fn tiles_sum(first: u32, n: u32, lid: u32) -> f32 {
    var s = 0.0;
    for (var i = lid; i < n; i = i + WG) {
        s = s + st.tile[first + i];
    }
    lanes[lid] = s;
    workgroupBarrier();
    for (var h = WG / 2u; h > 0u; h = h / 2u) {
        if (lid < h) {
            lanes[lid] = lanes[lid] + lanes[lid + h];
        }
        workgroupBarrier();
    }
    return lanes[0];
}

// The box's 8 × 8 tiles across (the slot of tile (tx, ty) is ty · tiles_across + tx).
fn tiles_across() -> u32 {
    return (abox.w + 7u) / 8u;
}

// 1: each tile's Σ S_ν. `src` is side.s_nu, which P6 wrote (zeros where it burns no neutron star).
@compute @workgroup_size(8, 8)
fn nu_tiles(
    @builtin(local_invocation_id) lid: vec3<u32>,
    @builtin(workgroup_id) wg: vec3<u32>,
    @builtin(local_invocation_index) li: u32,
) {
    let bx = wg.x * 8u + lid.x;
    let by = wg.y * 8u + lid.y;
    var v = 0.0;
    if (bx < abox.w && by < abox.h) {
        v = src[fp.src_offset + (abox.y0 + by) * fp.stride + abox.x0 + bx];
    }
    part[li] = v;
    tile_sum(li);
    if (li == 0u) {
        st.tile[wg.y * tiles_across() + wg.x] = part[0];
    }
}

// 2: Σ S_ν, and the sizes of everything that follows — all 0 when nothing is emitted.
@compute @workgroup_size(256)
fn nu_gate(@builtin(local_invocation_index) lid: u32) {
    let tx = tiles_across();
    let ty = (abox.h + 7u) / 8u;
    let emitted = tiles_sum(0u, tx * ty, lid);
    if (lid == 0u) {
        let open = emitted > 0.0;
        st.steps = st.steps + 1u;
        st.open = select(0u, 1u, open);
        st.emitted = emitted;
        sizes[0] = select(0u, abox.h, open);
        sizes[1] = 1u;
        sizes[2] = 1u;
        sizes[3] = select(0u, abox.fft_w, open);
        sizes[4] = 1u;
        sizes[5] = 1u;
        sizes[6] = select(0u, tx, open);
        sizes[7] = select(0u, ty, open);
        sizes[8] = 1u;
        sizes[9] = select(0u, 1u, open);
        sizes[10] = 1u;
        sizes[11] = 1u;
    }
}

// 3: the convolution's columns — each forward over fft_h (rows ≥ h are zero), times K̂ and 1 / (fft_w · fft_h), back by
// decimation in frequency; fft.wgsl's `conv_rows` (before) and `conv_rows_inverse` (after) are its rows. K̂ is read at
// (min(kx, nx − kx), min(ky, ny − ky)) from the real quarter of the box's padded size.
@compute @workgroup_size(256)
fn nu_columns(@builtin(workgroup_id) wg: vec3<u32>, @builtin(local_invocation_index) lid: u32) {
    let col = wg.x;
    if (col >= abox.fft_w) {
        return;
    }
    let nx = abox.fft_w;
    let n = abox.fft_h;
    let log_n = countTrailingZeros(n);
    for (var y = lid; y < n; y = y + WG) {
        var v = vec2<f32>(0.0, 0.0);
        if (y < abox.h) {
            v = spec[y * nx + col];
        }
        buf[bitrev(y, log_n)] = v;
    }
    dit(n, lid, false);
    let row_len = nx / 2u + 1u;
    let base = koffsets[(countTrailingZeros(nx) - 5u) * np.ny_sizes + log_n - 5u] + min(col, nx - col);
    let scale = 1.0 / f32(nx * n);
    for (var y = lid; y < n; y = y + WG) {
        buf[y] = buf[y] * (kquarter[base + min(y, n - y) * row_len] * scale);
    }
    dif(n, lid, true);
    for (var y = lid; y < abox.h; y = y + WG) {
        spec[y * nx + col] = buf[bitrev(y, log_n)];
    }
}

// 4: D(x) = Σ(x) · reach(x) over the box's tiles, in place of reach: 0 where X_n ≥ 0.5, never below 0 (the transforms' round-off
// can leave −1e-17 where the exact sum is ≥ 0); each tile's Σ D.
@compute @workgroup_size(8, 8)
fn nu_deposit(
    @builtin(local_invocation_id) lid: vec3<u32>,
    @builtin(workgroup_id) wg: vec3<u32>,
    @builtin(local_invocation_index) li: u32,
) {
    let bx = wg.x * 8u + lid.x;
    let by = wg.y * 8u + lid.y;
    var d = 0.0;
    if (bx < abox.w && by < abox.h) {
        let cell = (abox.y0 + by) * fp.stride + abox.x0 + bx;
        if (species_b[X_N_PLANE * np.cells + cell] < X_N_STAR) {
            d = max(hydro[cell] * dst[cell], 0.0);
        }
        dst[cell] = d;
    }
    part[li] = d;
    tile_sum(li);
    if (li == 0u) {
        st.tile[np.tiles + wg.y * tiles_across() + wg.x] = part[0];
    }
}

// 5: Σ D, and the scale q = scale · D: f_dep ΣS_ν / ΣD — exactly f_dep of the emitted power whatever D is — or 0 when no cell
// outside the star has mass to take it.
@compute @workgroup_size(256)
fn nu_share(@builtin(local_invocation_index) lid: u32) {
    let total = tiles_sum(np.tiles, tiles_across() * ((abox.h + 7u) / 8u), lid);
    if (lid == 0u) {
        st.total = total;
        st.scale = select(0.0, np.f_dep * st.emitted / total, total > 0.0);
        st.share_stamp = st.steps;
    }
}

// 6: E += q Δt_n and the same into the booking, at every cell that gets any.
@compute @workgroup_size(8, 8)
fn nu_apply(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= abox.w || id.y >= abox.h) {
        return;
    }
    let cell = (abox.y0 + id.y) * fp.stride + abox.x0 + id.x;
    let q = st.scale * dst[cell];
    if (q > 0.0) {
        let e = q * dt_rec.x;
        hydro[3u * np.cells + cell] = hydro[3u * np.cells + cell] + e;
        book[cell] = book[cell] + e;
    }
}

// 7: the sizes back to 0, whatever the step did.
@compute @workgroup_size(1)
fn nu_close() {
    for (var i = 0u; i < 12u; i++) {
        sizes[i] = 0u;
    }
}
