// The glow (m0_contrat.md §1.11, §4.11; M0-T81): hot matter glows and a supernova flashes. Four compute passes over the Glow
// view's cell image (the RGBA8 image render.wgsl wrote, sRGB-encoded):
//   glow_bright     the bright pass at half resolution — a half-resolution texel is the mean of its 2 × 2 cells, each kept (as its
//                   colour) when its luminance, 0.2126 R + 0.7152 G + 0.0722 B of the encoded values, is above THRESHOLD and
//                   zero otherwise (cells past the world's edge count as zero);
//   glow_blur_h/v   a separable 9-tap Gaussian (σ = SIGMA half-resolution texels, taps −4…4, the weights normalised to sum 1,
//                   zero outside the image);
//   glow_composite  the blurred bright pass upsampled bilinearly (the sampler's clamp at the edges) and added at INTENSITY to
//                   the cell image, clamped to 1.
// The half-resolution textures are RGBA16F: a lone cell's halo is under one level of the 8-bit image and must not be rounded
// away before the sum. render.rs `Renderer::render` runs the passes for the Glow view only.

const THRESHOLD: f32 = 0.8;
const SIGMA: f32 = 3.0;
const INTENSITY: f32 = 0.6;
const RADIUS: i32 = 4;

@group(0) @binding(0) var full_in: texture_2d<f32>;
@group(0) @binding(1) var half_in: texture_2d<f32>;
@group(0) @binding(2) var half_out: texture_storage_2d<rgba16float, write>;
@group(0) @binding(3) var full_out: texture_storage_2d<rgba8unorm, write>;

fn luminance(rgb: vec3<f32>) -> f32 {
    return dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn bright_cell(p: vec2<i32>) -> vec3<f32> {
    let size = vec2<i32>(textureDimensions(full_in));
    if (p.x >= size.x || p.y >= size.y) {
        return vec3<f32>(0.0);
    }
    let rgb = textureLoad(full_in, p, 0).rgb;
    return select(vec3<f32>(0.0), rgb, luminance(rgb) > THRESHOLD);
}

@compute @workgroup_size(8, 8)
fn glow_bright(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = vec2<u32>(textureDimensions(half_out));
    if (id.x >= size.x || id.y >= size.y) {
        return;
    }
    let p = vec2<i32>(id.xy) * 2;
    let sum = bright_cell(p) + bright_cell(p + vec2<i32>(1, 0)) + bright_cell(p + vec2<i32>(0, 1)) + bright_cell(p + vec2<i32>(1, 1));
    textureStore(half_out, vec2<i32>(id.xy), vec4<f32>(sum * 0.25, 1.0));
}

fn tap(k: i32) -> f32 {
    return exp(-f32(k * k) / (2.0 * SIGMA * SIGMA));
}

fn weight_sum() -> f32 {
    var s = 0.0;
    for (var k = -RADIUS; k <= RADIUS; k++) {
        s += tap(k);
    }
    return s;
}

fn blur(id: vec3<u32>, step: vec2<i32>) {
    let size = vec2<i32>(textureDimensions(half_out));
    if (i32(id.x) >= size.x || i32(id.y) >= size.y) {
        return;
    }
    let p = vec2<i32>(id.xy);
    var acc = vec3<f32>(0.0);
    for (var k = -RADIUS; k <= RADIUS; k++) {
        let q = p + step * k;
        if (q.x >= 0 && q.y >= 0 && q.x < size.x && q.y < size.y) {
            acc += tap(k) * textureLoad(half_in, q, 0).rgb;
        }
    }
    textureStore(half_out, p, vec4<f32>(acc / weight_sum(), 1.0));
}

@compute @workgroup_size(8, 8)
fn glow_blur_h(@builtin(global_invocation_id) id: vec3<u32>) {
    blur(id, vec2<i32>(1, 0));
}

@compute @workgroup_size(8, 8)
fn glow_blur_v(@builtin(global_invocation_id) id: vec3<u32>) {
    blur(id, vec2<i32>(0, 1));
}

fn halo_texel(q: vec2<i32>) -> vec3<f32> {
    let size = vec2<i32>(textureDimensions(half_in));
    return textureLoad(half_in, clamp(q, vec2<i32>(0), size - vec2<i32>(1)), 0).rgb;
}

@compute @workgroup_size(8, 8)
fn glow_composite(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(full_in);
    if (id.x >= size.x || id.y >= size.y) {
        return;
    }
    let p = vec2<i32>(id.xy);
    // The cell's centre in half-resolution texel coordinates (texel i is centred on i).
    let f = (vec2<f32>(p) + vec2<f32>(0.5)) * 0.5 - vec2<f32>(0.5);
    let f0 = floor(f);
    let t = f - f0;
    let i0 = vec2<i32>(f0);
    let top = mix(halo_texel(i0), halo_texel(i0 + vec2<i32>(1, 0)), t.x);
    let bottom = mix(halo_texel(i0 + vec2<i32>(0, 1)), halo_texel(i0 + vec2<i32>(1, 1)), t.x);
    let halo = mix(top, bottom, t.y);
    let base = textureLoad(full_in, p, 0).rgb;
    textureStore(full_out, p, vec4<f32>(clamp(base + INTENSITY * halo, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0));
}
