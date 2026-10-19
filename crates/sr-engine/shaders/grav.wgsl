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
    // G and c_sb², for the sinks' r_acc = max(2, 2GM/c_sb²) (§2.6) — used by `grav_sinks` alone
    grav: f32,
    c2: f32,
    _pad1: f32,
    _pad2: f32,
}

// P3's sink buffer as step/sinks.rs lays it out: the header (count first), then 16 sinks — the tiles' candidate slots
// that follow are P3's own. Read by `grav_sinks` alone.
struct Sink {
    pos: vec2<f32>,
    vel: vec2<f32>,
    m: f32,
    m_lo: f32,
    _pad: vec2<f32>,
}

struct SinkState {
    count: u32,
    formed: u32,
    merged: u32,
    _pad0: u32,
    accreted: f32,
    _pad1: f32,
    _pad2: f32,
    _pad3: f32,
    sinks: array<Sink, 16>,
}

const MAX_SINKS: u32 = 16u;

@group(0) @binding(7) var<uniform> gp: GravParams;
@group(0) @binding(8) var<storage, read> kquarter: array<f32>;
@group(0) @binding(9) var<storage, read> koffsets: array<u32>;
// φ as conv_rows_inverse wrote it into `dst` (the same buffer, never bound both ways in one dispatch)
@group(0) @binding(10) var<storage, read> phi: array<f32>;
// g = −∇φ: the x plane, then the y plane — the gas's alone when the step runs P3 (`gas_field`), else the field itself
@group(0) @binding(11) var<storage, read_write> g: array<f32>;
// the sinks (P3's buffer) and g with their pull added (the field P4 and Δt read), used by `grav_sinks` alone
@group(0) @binding(12) var<storage, read> sink_state: SinkState;
@group(0) @binding(13) var<storage, read_write> g_total: array<f32>;

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
    g[2u * gp.cells + cell] = phi[cell];
}

// An optional 5th, only in a step that runs P3 (§1.4.3): every sink's softened potential −G M / √(r² + r_acc²) added to φ, and its
// exact gradient −G M d / (r² + r_acc²)^{3/2} (d from the sink to the cell's centre) added to the gas's g into `g_total`,
// over the box's 8 × 8 tiles — the twin's `grav::add_sinks`, in the sinks' slot order. `g` keeps the gas's pull alone:
// P3's kick-drift-kick reads it, since a g with the sinks' own pull would make a sink pull on itself through the bilinear
// read. φ is rewritten whole by `conv_rows_inverse` every step, so adding in place never counts a sink twice.
@compute @workgroup_size(8, 8)
fn grav_sinks(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= abox.w || id.y >= abox.h) {
        return;
    }
    let x = abox.x0 + id.x;
    let y = abox.y0 + id.y;
    let cell = y * fp.stride + x;
    let c = vec2<f32>(f32(x) + 0.5, f32(y) + 0.5);
    var p = dst[cell];
    var a = vec2<f32>(g[cell], g[gp.cells + cell]);
    for (var k = 0u; k < min(sink_state.count, MAX_SINKS); k++) {
        let m = sink_state.sinks[k].m + sink_state.sinks[k].m_lo;
        let ra = max(2.0, 2.0 * gp.grav * m / gp.c2);
        let d = c - sink_state.sinks[k].pos;
        let soft = dot(d, d) + ra * ra;
        let inv = inverseSqrt(soft);
        p -= gp.grav * m * inv;
        a -= (gp.grav * m * inv / soft) * d;
    }
    dst[cell] = p;
    g_total[cell] = a.x;
    g_total[gp.cells + cell] = a.y;
    g_total[2u * gp.cells + cell] = p;
}
