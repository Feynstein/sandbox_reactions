// The GPU FFT (m0_contrat.md §1.3.3, §1.4.2, §6.2.2): radix-2 complex transforms of 32 … 2048 points per axis, one
// row or one column per workgroup, held in workgroup memory, and the zero-padded convolution over the active box.
//
// Within WebGPU's defaults (§6.2.2): 256 invocations, and one 2048-point row of vec2<f32> is 16,384 B — the whole of a
// workgroup's storage — so `buf` is the only workgroup variable: the twiddles come from a storage table, the indices
// are computed. The twiddles are e^{−2πi k/2048} for k < 1024, taken in f64 on the CPU (step/fft.rs); a length n reads
// every (2048/n)-th, never a cos/sin per butterfly.
//
// Conventions as the twin (sr_physics::reference::fft): row-major, index = y · nx + x; forward X_k = Σ x_n e^{−2πi kn/N}
// unnormalised, the inverse conjugated and divided by N. Decimation in time takes its input bit-reversed and leaves
// the transform in natural order; decimation in frequency takes natural order and leaves it bit-reversed — so the
// convolution's columns go forward, multiply and back in one dispatch without a reordering pass.
//
// `abox` has the active box's layout (reduce.wgsl's ActiveBox): the box, then the FFT size per axis.

const MAX_N: u32 = 2048u;
const WG: u32 = 256u;

struct ActiveBox {
    x0: u32,
    y0: u32,
    w: u32,
    h: u32,
    fft_w: u32,
    fft_h: u32,
    from_step: u32,
    fits: u32,
}

struct FftParams {
    // the planes' row length (the world's width) and where each plane starts, in f32 elements
    stride: u32,
    src_offset: u32,
    dst_offset: u32,
    // 1: the inverse transform (rows_c2c, columns_c2c)
    inverse: u32,
}

@group(0) @binding(0) var<uniform> abox: ActiveBox;
@group(0) @binding(1) var<uniform> fp: FftParams;
@group(0) @binding(2) var<storage, read> twiddles: array<vec2<f32>>;
// complex work: fft_w wide, one row per transformed row
@group(0) @binding(3) var<storage, read_write> spec: array<vec2<f32>>;
@group(0) @binding(4) var<storage, read> src: array<f32>;
@group(0) @binding(5) var<storage, read_write> dst: array<f32>;
// the kernel's transform, fft_w × fft_h
@group(0) @binding(6) var<storage, read> kspec: array<vec2<f32>>;

var<workgroup> buf: array<vec2<f32>, MAX_N>;

fn bitrev(i: u32, log_n: u32) -> u32 {
    return reverseBits(i) >> (32u - log_n);
}

fn cmul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

// e^{∓2πi k / (2 half)}, from the table.
fn twiddle(k: u32, half: u32, inverse: bool) -> vec2<f32> {
    let t = twiddles[k * (MAX_N / (2u * half))];
    return select(t, vec2<f32>(t.x, -t.y), inverse);
}

// Decimation in time over buf[0 … n): bit-reversed in, natural order out.
fn dit(n: u32, lid: u32, inverse: bool) {
    workgroupBarrier();
    for (var half = 1u; half < n; half = half * 2u) {
        for (var b = lid; b < n / 2u; b = b + WG) {
            let k = b & (half - 1u);
            let i = (b - k) * 2u + k;
            let a = buf[i];
            let t = cmul(buf[i + half], twiddle(k, half, inverse));
            buf[i] = a + t;
            buf[i + half] = a - t;
        }
        workgroupBarrier();
    }
}

// Decimation in frequency over buf[0 … n): natural order in, bit-reversed out.
fn dif(n: u32, lid: u32, inverse: bool) {
    workgroupBarrier();
    for (var half = n / 2u; half > 0u; half = half / 2u) {
        for (var b = lid; b < n / 2u; b = b + WG) {
            let k = b & (half - 1u);
            let i = (b - k) * 2u + k;
            let a = buf[i];
            let c = buf[i + half];
            buf[i] = a + c;
            buf[i + half] = cmul(a - c, twiddle(k, half, inverse));
        }
        workgroupBarrier();
    }
}

// A 2D transform's rows, in place over spec (fft_w × fft_h): one row per workgroup.
@compute @workgroup_size(256)
fn rows_c2c(@builtin(workgroup_id) wg: vec3<u32>, @builtin(local_invocation_index) lid: u32) {
    let row = wg.x;
    if (row >= abox.fft_h) {
        return;
    }
    let n = abox.fft_w;
    let log_n = countTrailingZeros(n);
    let inverse = fp.inverse != 0u;
    for (var x = lid; x < n; x = x + WG) {
        buf[bitrev(x, log_n)] = spec[row * n + x];
    }
    dit(n, lid, inverse);
    let scale = select(1.0, 1.0 / f32(n), inverse);
    for (var x = lid; x < n; x = x + WG) {
        spec[row * n + x] = buf[x] * scale;
    }
}

// A 2D transform's columns, in place over spec: one column per workgroup.
@compute @workgroup_size(256)
fn columns_c2c(@builtin(workgroup_id) wg: vec3<u32>, @builtin(local_invocation_index) lid: u32) {
    let col = wg.x;
    if (col >= abox.fft_w) {
        return;
    }
    let nx = abox.fft_w;
    let n = abox.fft_h;
    let log_n = countTrailingZeros(n);
    let inverse = fp.inverse != 0u;
    for (var y = lid; y < n; y = y + WG) {
        buf[bitrev(y, log_n)] = spec[y * nx + col];
    }
    dit(n, lid, inverse);
    let scale = select(1.0, 1.0 / f32(n), inverse);
    for (var y = lid; y < n; y = y + WG) {
        spec[y * nx + col] = buf[y] * scale;
    }
}

// The convolution, 1 of 3: the box's rows of the real source, zero-padded to fft_w, forward. Rows h … fft_h − 1 are
// zero and never transformed: spec holds h rows.
@compute @workgroup_size(256)
fn conv_rows(@builtin(workgroup_id) wg: vec3<u32>, @builtin(local_invocation_index) lid: u32) {
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
        }
        buf[bitrev(x, log_n)] = vec2<f32>(v, 0.0);
    }
    dit(n, lid, false);
    for (var x = lid; x < n; x = x + WG) {
        spec[row * n + x] = buf[x];
    }
}

// The convolution, 2 of 3: each column forward over fft_h (rows ≥ h are zero), times the kernel's transform and
// 1 / (fft_w · fft_h), back by decimation in frequency; only rows < h are kept.
@compute @workgroup_size(256)
fn conv_columns(@builtin(workgroup_id) wg: vec3<u32>, @builtin(local_invocation_index) lid: u32) {
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
    let scale = 1.0 / f32(nx * n);
    for (var y = lid; y < n; y = y + WG) {
        buf[y] = cmul(buf[y], kspec[y * nx + col]) * scale;
    }
    dif(n, lid, true);
    for (var y = lid; y < abox.h; y = y + WG) {
        spec[y * nx + col] = buf[bitrev(y, log_n)];
    }
}

// The convolution, 3 of 3: each row back by decimation in frequency; the real part of the box's cells into dst.
@compute @workgroup_size(256)
fn conv_rows_inverse(@builtin(workgroup_id) wg: vec3<u32>, @builtin(local_invocation_index) lid: u32) {
    let row = wg.x;
    if (row >= abox.h) {
        return;
    }
    let n = abox.fft_w;
    let log_n = countTrailingZeros(n);
    for (var x = lid; x < n; x = x + WG) {
        buf[x] = spec[row * n + x];
    }
    dif(n, lid, true);
    let base = fp.dst_offset + (abox.y0 + row) * fp.stride + abox.x0;
    for (var x = lid; x < abox.w; x = x + WG) {
        dst[base + x] = buf[bitrev(x, log_n)].x;
    }
}
