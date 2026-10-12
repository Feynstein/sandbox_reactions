// P2 gravity (m0_contrat.md §1.3.2, §1.4.1–§1.4.2): appended to fft.wgsl, whose helpers (dit, dif, bitrev), workgroup
// row and bindings 0–5 it shares; step/grav.rs builds the dispatches. φ = K ∗ Σ_src over the active box — the box's own
// record is `abox`, so the FFT size per axis is the box's (§1.3.3) — then g = −∇φ by central differences, the scheme
// of the CPU twin (sr_physics::reference::grav).
//
// K is real and even in both axes, so K̂ is too: `kquarter` holds, for every padded size the world's boxes can take,
// K̂(kx, ky) for kx ≤ nx/2 and ky ≤ ny/2 — row-major in ky, (nx/2 + 1) per row — taken in f64 on the CPU; a size's
// quarter starts at koffsets[(log2 nx − 5) · ny_sizes + log2 ny − 5], and K̂(kx, ky) is read at
// (min(kx, nx − kx), min(ky, ny − ky)).

struct GravParams {
    sigma_vac: f32,
    // the table's lengths per y axis: 32, 64, … (ny_sizes of them)
    ny_sizes: u32,
    // cells in a plane: g's y plane starts there
    cells: u32,
    _pad: u32,
}

@group(0) @binding(7) var<uniform> gp: GravParams;
@group(0) @binding(8) var<storage, read> kquarter: array<f32>;
@group(0) @binding(9) var<storage, read> koffsets: array<u32>;
// φ as conv_rows_inverse wrote it into `dst` (the same buffer, never bound both ways in one dispatch)
@group(0) @binding(10) var<storage, read> phi: array<f32>;
// g = −∇φ: the x plane, then the y plane
@group(0) @binding(11) var<storage, read_write> g: array<f32>;

// 1 of 4: the box's rows of Σ_src — Σ, or 0 in vacuum (Σ < Σ_vac, §1.4.2) — zero-padded to fft_w, forward:
// conv_rows with the vacuum mask.
@compute @workgroup_size(256)
fn grav_rows(@builtin(workgroup_id) wg: vec3<u32>, @builtin(local_invocation_index) lid: u32) {
    let row = wg.x;
    if (row >= abox.h) {
        return;
    }
    let n = abox.fft_w;
    let log_n = countTrailingZeros(n);
    let base = fp.src_offset + (abox.y0 + row) * fp.stride + abox.x0;
    for (var x = lid; x < n; x = x + WG) {
        var v = 0.0;
        if (x < abox.w) {
            v = src[base + x];
            if (v < gp.sigma_vac) {
                v = 0.0;
            }
        }
        buf[bitrev(x, log_n)] = vec2<f32>(v, 0.0);
    }
    dit(n, lid, false);
    for (var x = lid; x < n; x = x + WG) {
        spec[row * n + x] = buf[x];
    }
}

// 2 of 4: each column forward over fft_h (rows ≥ h are zero), times K̂ and 1 / (fft_w · fft_h), back by decimation in
// frequency: conv_columns with the kernel's real quarter for the box's size. Then fft.wgsl's conv_rows_inverse (3 of 4)
// writes φ.
@compute @workgroup_size(256)
fn grav_columns(@builtin(workgroup_id) wg: vec3<u32>, @builtin(local_invocation_index) lid: u32) {
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
    let base = koffsets[(countTrailingZeros(nx) - 5u) * gp.ny_sizes + log_n - 5u] + min(col, nx - col);
    let scale = 1.0 / f32(nx * n);
    for (var y = lid; y < n; y = y + WG) {
        buf[y] = buf[y] * (kquarter[base + min(y, n - y) * row_len] * scale);
    }
    dif(n, lid, true);
    for (var y = lid; y < abox.h; y = y + WG) {
        spec[y * nx + col] = buf[bitrev(y, log_n)];
    }
}

// 4 of 4: g = −∇φ over the box's 8 × 8 tiles by central differences, one-sided on the box's outermost ring (the twin's
// rule; the box's margin keeps matter off it, §1.3.3).
@compute @workgroup_size(8, 8)
fn grav_gradient(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= abox.w || id.y >= abox.h) {
        return;
    }
    let x = abox.x0 + id.x;
    let y = abox.y0 + id.y;
    let s = fp.stride;
    let xl = select(x - 1u, x, id.x == 0u);
    let xh = select(x + 1u, x, id.x + 1u == abox.w);
    let yl = select(y - 1u, y, id.y == 0u);
    let yh = select(y + 1u, y, id.y + 1u == abox.h);
    var a = vec2<f32>(0.0, 0.0);
    if (xh > xl) {
        a.x = -(phi[y * s + xh] - phi[y * s + xl]) / f32(xh - xl);
    }
    if (yh > yl) {
        a.y = -(phi[yh * s + x] - phi[yl * s + x]) / f32(yh - yl);
    }
    let cell = y * s + x;
    g[cell] = a.x;
    g[gp.cells + cell] = a.y;
}
