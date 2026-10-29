//! M0-T82 — frames from headless runs (contract §3.3's `--frames-every N` and `--view`, §6.3, §1.11): DIR/frames/f_<step>.png,
//! the cell image of the state at every step index N divides, in the chosen view (Glow by default, with the glow passes),
//! drawn offscreen and written with `png`.
//!
//! The runs are `headless::run_captured` on a 104 × 64 world (a row of 416 bytes, so the copy's 256-byte row alignment pads
//! it) with a hot hydrogen disk the heat and the flow keep changing. The files are read back with the `png` decoder. The
//! adversarial case is the step in the name: every frame is compared, byte for byte, with the image `Renderer` draws from the
//! dump the same run wrote for that step (`--dump-every` on the same N) — a frame read back before the GPU finished, or named
//! for the wrong step, is another step's image — and the frames of consecutive steps are asserted to differ, so that the
//! comparison could tell two steps apart. Hot disks are hand-made: a star's frame is `NOT PROVEN (synthetic)`.

use super::device;
use sr_engine::dump::Dump;
use sr_engine::headless::run_captured;
use sr_engine::render::{Renderer, View, IMAGE_FORMAT};
use sr_engine::state::State;
use sr_engine::step::EosGpu;
use sr_physics::calibration::{Calibration, EMBEDDED};
use sr_physics::registry::{Elements, Physics};
use std::path::{Path, PathBuf};

const W: usize = 104;
const H: usize = 64;
/// The disk's centre and radius, in cells.
const CX: usize = 52;
const CY: usize = 32;
const RADIUS: usize = 22;

/// A hot hydrogen disk (T 12, near the Sun-like T_eff's 11.6) with the flow and the heat on: it keeps changing.
fn scene_text() -> String {
    format!(
        r#"{{"version": 1,
 "world": {{"width": {W}, "height": {H}, "edge": "leave"}},
 "slowdown": false,
 "switches": {{"gravity": false, "hydro": true, "heat": true, "reactions": false, "neutrinos": false, "sinks": false}},
 "objects": [
   {{"kind": "disk", "x": {CX}, "y": {CY}, "a": {RADIUS}, "sigma_c": 1.0, "t": 12.0, "composition": {{"H": 1.0}}, "v": [0, 0]}}
 ]}}"#
    )
}

/// A fresh folder for a case, with the scene in it.
fn folder(name: &str) -> (PathBuf, PathBuf) {
    let dir = std::env::temp_dir().join(format!("sr-frames-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let scene = dir.join("scene.json");
    std::fs::write(&scene, scene_text()).unwrap();
    (dir, scene)
}

fn args(scene: &Path, out: &Path, tail: &[&str]) -> Vec<String> {
    let mut a: Vec<String> = ["--scene", scene.to_str().unwrap(), "--out", out.to_str().unwrap()].map(String::from).to_vec();
    if let Some(adapter) = std::env::var("SR_TEST_ADAPTER").ok().filter(|s| !s.trim().is_empty()) {
        a.extend(["--adapter".to_string(), adapter]);
    }
    a.extend(tail.iter().map(|s| s.to_string()));
    a
}

fn run(scene: &Path, out: &Path, tail: &[&str]) {
    let (code, lines) = run_captured(&args(scene, out, tail));
    assert_eq!(code, 0, "headless {tail:?}: {lines:?}");
}

/// A frame as the PNG decoder reads it: (width, height, RGBA bytes).
fn decode(path: &Path) -> (u32, u32, Vec<u8>) {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let mut reader = png::Decoder::new(std::io::Cursor::new(bytes)).read_info().unwrap();
    let mut pixels = vec![0u8; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut pixels).unwrap();
    assert_eq!((info.color_type, info.bit_depth), (png::ColorType::Rgba, png::BitDepth::Eight), "{}", path.display());
    pixels.truncate(info.buffer_size());
    (info.width, info.height, pixels)
}

/// The names in `dir/frames`, sorted by step.
fn frame_steps(dir: &Path) -> Vec<u64> {
    let mut steps: Vec<u64> = std::fs::read_dir(dir.join("frames"))
        .unwrap()
        .map(|e| {
            let name = e.unwrap().file_name().into_string().unwrap();
            name.strip_prefix("f_").and_then(|n| n.strip_suffix(".png")).and_then(|n| n.parse().ok()).unwrap_or_else(|| panic!("{name}: not f_<step>.png"))
        })
        .collect();
    steps.sort_unstable();
    steps
}

fn frame_path(dir: &Path, step: u64) -> PathBuf {
    dir.join("frames").join(format!("f_{step}.png"))
}

/// The eight sandbox anchors the run uses: physics' own, and the embedded calibration's Sun-like T_eff.
fn anchors(physics: &Physics) -> [f64; 8] {
    let cal = Calibration::parse(EMBEDDED.expect("assets/calibration.json is embedded")).unwrap();
    let k = |key: &str| physics.get(key).unwrap();
    [k("t_floor"), cal.t_eff_sun_sb, k("t_h"), k("t_he"), k("t_c"), k("t_ne"), k("t_o"), k("t_si")]
}

/// The image `Renderer` draws from a dump's state — the frame the run should have written for that step.
fn render_dump(dump: &Dump, view: View) -> Vec<u8> {
    let gpu = device();
    let (physics, elements) = (Physics::shipped().unwrap(), Elements::shipped().unwrap());
    let state = State::new(&gpu.device, dump.world).unwrap();
    state.upload(&gpu.queue, &dump.planes).unwrap();
    let eos = EosGpu::new(&gpu.device, &physics, &elements);
    let renderer = Renderer::new(&gpu.device, &state, &state, &eos, anchors(&physics), &elements);
    let (w, h) = (dump.world.width, dump.world.height);
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("frames.image"),
        size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: IMAGE_FORMAT,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let row = (w * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("frames.staging"),
        size: u64::from(row * h),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frames") });
    renderer.render(view, 1.0, &texture.create_view(&wgpu::TextureViewDescriptor::default()), &mut encoder);
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyBufferInfo { buffer: &staging, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: Some(h) } },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
    gpu.queue.submit([encoder.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    staging.map_async(wgpu::MapMode::Read, .., move |r| {
        let _ = tx.send(r);
    });
    gpu.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    rx.recv().unwrap().unwrap();
    let mapped = staging.get_mapped_range(..).unwrap();
    let (w, row) = (w as usize * 4, row as usize);
    (0..h as usize).flat_map(|y| mapped[y * row..y * row + w].to_vec()).collect()
}

fn pixel(image: &[u8], x: usize, y: usize) -> [u8; 4] {
    let i = (y * W + x) * 4;
    [image[i], image[i + 1], image[i + 2], image[i + 3]]
}

fn brightness(p: [u8; 4]) -> u32 {
    p[..3].iter().map(|&c| u32::from(c)).sum()
}

#[test]
fn frames_land_at_every_n_steps_at_the_worlds_size() {
    let (dir, scene) = folder("steps");
    run(&scene, &dir, &["--steps", "50", "--frames-every", "10"]);
    assert_eq!(frame_steps(&dir), [10, 20, 30, 40, 50], "a frame at each multiple of 10 up to the run's end, none else");
    for step in [10, 20, 30, 40, 50] {
        let (w, h, pixels) = decode(&frame_path(&dir, step));
        assert_eq!((w as usize, h as usize), (W, H), "f_{step}.png");
        assert_eq!(pixels.len(), W * H * 4, "f_{step}.png");
    }
    // Every other multiple: 25 steps by 7 frames at 7, 14, 21 — the chunks are cut where the frames fall, the last step is no frame.
    let (dir, scene) = folder("steps7");
    run(&scene, &dir, &["--steps", "25", "--frames-every", "7"]);
    assert_eq!(frame_steps(&dir), [7, 14, 21]);
    // No `--frames-every`, no frames folder content.
    let (dir, scene) = folder("none");
    run(&scene, &dir, &["--steps", "20"]);
    assert!(!dir.join("frames").exists(), "frames/ is made only for a run with --frames-every");
}

#[test]
fn the_glow_frame_of_a_hot_disk_is_lit_and_brighter_at_its_centre() {
    let (dir, scene) = folder("glow");
    run(&scene, &dir, &["--steps", "10", "--frames-every", "10"]);
    let (_, _, image) = decode(&frame_path(&dir, 10));
    let lit = image.chunks_exact(4).filter(|p| brightness([p[0], p[1], p[2], p[3]]) > 0).count();
    assert!(lit > RADIUS * RADIUS, "the glow frame is not all black: {lit} lit pixels of {}", W * H);
    // The disk's centre against its rim (a cell inside the edge, on the same row), and the empty corner.
    let (centre, rim, corner) = (pixel(&image, CX, CY), pixel(&image, CX + RADIUS - 1, CY), pixel(&image, 0, 0));
    println!("centre {centre:?} rim {rim:?} corner {corner:?}");
    assert!(brightness(centre) > brightness(rim), "centre {centre:?} is brighter than rim {rim:?}");
    assert!(brightness(rim) > brightness(corner), "rim {rim:?} is brighter than the empty corner {corner:?}");
}

/// Every frame of a run that also dumps at the same N equals the image drawn from that step's dump, and consecutive frames differ.
fn frames_match_their_steps_dumps(name: &str, view_flag: Option<&str>, view: View) {
    let (dir, scene) = folder(name);
    let mut tail = vec!["--steps", "40", "--frames-every", "10", "--dump-every", "10"];
    if let Some(v) = view_flag {
        tail.extend(["--view", v]);
    }
    run(&scene, &dir, &tail);
    assert_eq!(frame_steps(&dir), [10, 20, 30, 40]);
    let mut seen: Vec<Vec<u8>> = Vec::new();
    for step in [10, 20, 30, 40] {
        let dump = Dump::read(&dir.join("dumps").join(format!("s_{step}.bin"))).unwrap();
        assert_eq!(dump.step, step);
        let (_, _, frame) = decode(&frame_path(&dir, step));
        assert!(frame == render_dump(&dump, view), "f_{step}.png is not the {view:?} image of s_{step}.bin");
        seen.push(frame);
    }
    for pair in seen.windows(2) {
        assert!(pair[0] != pair[1], "two consecutive frames are the same image: the comparison cannot tell steps apart");
    }
}

#[test]
fn each_frame_is_the_state_of_the_step_in_its_name_glow() {
    frames_match_their_steps_dumps("match-glow", None, View::Glow);
}

#[test]
fn the_view_flag_draws_the_chosen_view() {
    frames_match_their_steps_dumps("match-heat", Some("heat"), View::Heat);
    // The same run in the default view (Glow) is another image.
    let (heat, scene) = folder("heat-only");
    run(&scene, &heat, &["--steps", "10", "--frames-every", "10", "--view", "heat"]);
    let (glow, scene) = folder("glow-only");
    run(&scene, &glow, &["--steps", "10", "--frames-every", "10"]);
    assert!(decode(&frame_path(&heat, 10)).2 != decode(&frame_path(&glow, 10)).2, "--view heat and the default Glow differ");
}

#[test]
fn frames_leave_the_run_unchanged() {
    // The chunks are cut at the frames; the final dump is the straight run's, byte for byte.
    let (with, scene) = folder("with");
    run(&scene, &with, &["--steps", "50", "--frames-every", "7"]);
    let (without, scene) = folder("without");
    run(&scene, &without, &["--steps", "50"]);
    let (a, b) = (std::fs::read(with.join("dumps/s_50.bin")).unwrap(), std::fs::read(without.join("dumps/s_50.bin")).unwrap());
    assert!(a == b, "the dump of step 50 differs between a run with frames and one without");
}

#[test]
fn bad_frame_arguments_are_exit_4() {
    let (dir, scene) = folder("bad");
    let out = dir.join("out");
    let cases: [(&[&str], &str); 5] = [
        (&["--steps", "5", "--frames-every", "0"], "at least 1"),
        (&["--steps", "5", "--frames-every", "x"], "whole number"),
        (&["--steps", "5", "--frames-every", "5", "--view", "mass"], "glow, heat, element or density"),
        (&["--steps", "5", "--view", "heat"], "needs --frames-every"),
        (&["--steps", "5", "--frames-every", "5", "--cpu-reference"], "--cpu-reference"),
    ];
    for (tail, needle) in cases {
        let (code, lines) = run_captured(&args(&scene, &out, tail));
        assert_eq!(code, 4, "{tail:?}: {lines:?}");
        assert!(lines.iter().any(|l| l.starts_with("SR-ERROR") && l.contains(needle)), "{tail:?}: {lines:?}");
    }
    // Frames go to DIR/frames: with no --out there is nowhere to put them.
    let (code, lines) = run_captured(&["--scene".into(), scene.to_str().unwrap().into(), "--steps".into(), "5".into(), "--frames-every".into(), "5".into()]);
    assert_eq!(code, 4, "{lines:?}");
    assert!(lines.iter().any(|l| l.contains("--frames-every needs --out")), "{lines:?}");
}
