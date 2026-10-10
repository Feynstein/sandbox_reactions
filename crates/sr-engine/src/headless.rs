//! `sandbox-reactions headless` (m0_contrat.md §3.3, §6.3): load a scene, run N steps on a surfaceless device, write the
//! run summary (§2.12.2). Today: `--scene preset:<name>` loads the nominal Sun-like disk (§2.1, §2.10) and `--steps N`
//! runs [`Step`] N times; scene files (M0-T25), `--until`, the ledger, events and objects arrive with their blocks.
//!
//! stdout: the first line `SR-ADAPTER …`; the last, when the run ends without an error (exit 0 or 2), `SR-HEADLESS DONE
//! steps=<n> sim_time=<t> until=<met|unmet|none>`; every error line starts `SR-ERROR`, every warning `SR-WARN`.
//! Exit codes: 0 done · 2 `--steps` not reached within `--max-steps` · 3 no adapter or a GPU error · 4 bad arguments
//! or scene · 101 a panic (Rust's own). 5 (a non-finite value) and 6 (calibration) arrive with their checks.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use serde_json::json;
use sr_physics::registry::{Elements, Physics};

use crate::gpu::Gpu;
use crate::state::{Booking, State, WorldConfig, N_CHANNELS};
use crate::step::{EosGpu, Step};

pub const EXIT_DONE: i32 = 0;
pub const EXIT_UNMET: i32 = 2;
pub const EXIT_GPU: i32 = 3;
pub const EXIT_ARGS: i32 = 4;

/// §3.3: `--max-steps` defaults to 2,000,000.
pub const DEFAULT_MAX_STEPS: u64 = 2_000_000;
/// Steps recorded per submission: the queue never holds more than this many at once.
const CHUNK: u64 = 256;

/// The presets §2.10 names. Their masses come from calibration (`preset_mass_sb`, §2.11) — until then all three load
/// the nominal Sun-like disk, and the run says so.
const PRESETS: [&str; 3] = ["sun", "massive", "giant"];

#[derive(Debug, PartialEq)]
pub struct Args {
    pub scene: String,
    pub steps: u64,
    pub world: WorldConfig,
    pub max_steps: u64,
    pub out: Option<PathBuf>,
    pub adapter: Option<String>,
}

/// Parses what follows `headless`. `--flag value` or `--flag=value`; anything else is an error naming it.
pub fn parse_args(args: &[String]) -> Result<Args, String> {
    let (mut scene, mut steps, mut world, mut max_steps, mut out, mut adapter) =
        (None, None, WorldConfig::default(), DEFAULT_MAX_STEPS, None, None);
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let (flag, inline) = match arg.split_once('=') {
            Some((f, v)) => (f, Some(v.to_string())),
            None => (arg.as_str(), None),
        };
        if !matches!(flag, "--scene" | "--steps" | "--world" | "--max-steps" | "--out" | "--adapter") {
            return Err(format!("unknown argument {arg:?}"));
        }
        let value = match inline {
            Some(v) => v,
            None => it.next().cloned().ok_or_else(|| format!("{flag} needs a value"))?,
        };
        let count = |v: &str| v.parse::<u64>().map_err(|_| format!("{flag} takes a whole number, got {v:?}"));
        match flag {
            "--scene" => scene = Some(value),
            "--steps" => steps = Some(count(&value)?),
            "--max-steps" => max_steps = count(&value)?,
            "--out" => out = Some(PathBuf::from(value)),
            "--adapter" => adapter = Some(value),
            _ => {
                let size = value.split_once('x').and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?)));
                let (w, h) = size.ok_or_else(|| format!("--world takes WxH, got {value:?}"))?;
                world = WorldConfig::new(w, h).map_err(|e| e.to_string())?;
            }
        }
    }
    let scene = scene.ok_or("--scene is required (a scene file, or preset:sun)")?;
    let steps = steps.ok_or("--steps is required (--until arrives with its block)")?;
    if steps == 0 {
        return Err("--steps must be at least 1".to_string());
    }
    Ok(Args { scene, steps, world, max_steps, out, adapter })
}

/// Checks `--scene` before any GPU is touched: the preset's name, or a refusal for a file (M0-T25).
fn check_scene(scene: &str) -> Result<Vec<String>, String> {
    let Some(name) = scene.strip_prefix("preset:") else {
        return Err(format!("scene {scene:?}: scene files arrive with M0-T25 — only preset:<name> loads today"));
    };
    if !PRESETS.contains(&name) {
        return Err(format!("scene {scene:?}: unknown preset (known: {})", PRESETS.join(", ")));
    }
    let mut warnings = Vec::new();
    if name != "sun" {
        warnings.push(format!("preset {name}: the nominal Sun-like disk until calibration gives preset_mass_sb (§2.11)"));
    }
    Ok(warnings)
}

/// §2.10's composition by mass: H 0.7346, He 0.2485, O 0.0077, C 0.0092, as (channel, fraction).
const SUN_MIX: [(usize, f64); 4] = [(4, 0.7346), (5, 0.2485), (7, 0.0077), (6, 0.0092)];
/// §2.1: the nominal cloud has Σc = 1 at radius a = 40 cells.
const SIGMA_C: f64 = 1.0;
const RADIUS: f64 = 40.0;
/// §2.10: the uniform temperature is set so that U_th = 0.1 |W|.
const THERMAL_OVER_W: f64 = 0.1;

/// The nominal Sun-like disk in the world's centre, at rest, in the state's canonical form (§2.2): Σ(r) = Σc√(1 − r²/a²)
/// inside r < a (cell centres), vacuum outside. The energy is a uniform specific thermal energy ε with Σ ε over the
/// cloud = 0.1 |W|, W = ½ Σ m φ by §1.4.2's kernel; E = Σ ε there (no cold part, no motion). Vacuum cells carry x_H = 1
/// so that every cell sums to 1, as P8 leaves them.
pub fn sun_disk_planes(world: WorldConfig) -> Vec<f32> {
    let (w, h) = (world.width as usize, world.height as usize);
    let cells = w * h;
    let (cx, cy) = (w as f64 / 2.0, h as f64 / 2.0);
    let mut planes = vec![0f32; N_CHANNELS * cells];
    let mut cloud = Vec::new(); // (x, y, mass) of the non-vacuum cells
    for y in 0..h {
        for x in 0..w {
            let (dx, dy) = (x as f64 + 0.5 - cx, y as f64 + 0.5 - cy);
            let r2 = (dx * dx + dy * dy) / (RADIUS * RADIUS);
            let i = y * w + x;
            if r2 < 1.0 {
                let sigma = SIGMA_C * (1.0 - r2).sqrt();
                planes[i] = sigma as f32;
                for (c, frac) in SUN_MIX {
                    planes[c * cells + i] = frac as f32;
                }
                cloud.push((x as f64, y as f64, sigma));
            } else {
                planes[4 * cells + i] = 1.0;
            }
        }
    }
    let self_term = -4.0 * (1.0 + 2f64.sqrt()).ln();
    let (mut w_sum, mut mass) = (0.0, 0.0);
    for &(xi, yi, mi) in &cloud {
        let phi: f64 = cloud
            .iter()
            .map(|&(xj, yj, mj)| {
                let r = ((xi - xj).powi(2) + (yi - yj).powi(2)).sqrt();
                mj * if r == 0.0 { self_term } else { -1.0 / r }
            })
            .sum();
        w_sum += 0.5 * mi * phi;
        mass += mi;
    }
    let eps = THERMAL_OVER_W * w_sum.abs() / mass;
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            planes[3 * cells + i] = (planes[i] as f64 * eps) as f32;
        }
    }
    planes
}

fn say(line: &str) {
    println!("{line}");
}

fn git_rev() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Runs the command; returns the process's exit code.
pub fn run(args: &[String]) -> i32 {
    let args = match parse_args(args) {
        Ok(a) => a,
        Err(e) => {
            say(&format!("SR-ERROR bad arguments: {e}"));
            return EXIT_ARGS;
        }
    };
    let warnings = match check_scene(&args.scene) {
        Ok(w) => w,
        Err(e) => {
            say(&format!("SR-ERROR bad scene: {e}"));
            return EXIT_ARGS;
        }
    };
    if let Some(dir) = &args.out {
        if let Err(e) = std::fs::create_dir_all(dir) {
            say(&format!("SR-ERROR bad arguments: --out {}: {e}", dir.display()));
            return EXIT_ARGS;
        }
    }

    let gpu = match pollster::block_on(Gpu::new_headless(args.adapter.as_deref())) {
        Ok(g) => g,
        Err(e) => {
            say(&format!("SR-ERROR {e}"));
            return EXIT_GPU;
        }
    };
    say(&gpu.info.sr_adapter_line());
    for w in &warnings {
        say(&format!("SR-WARN {w}"));
    }
    // A wgpu validation error would otherwise panic on a wgpu thread; it is the GPU-error exit.
    gpu.device.on_uncaptured_error(Arc::new(|e: wgpu::Error| {
        say(&format!("SR-ERROR GPU error: {e}"));
        std::process::exit(EXIT_GPU);
    }));

    let state = match State::new(&gpu.device, args.world) {
        Ok(s) => s,
        Err(e) => {
            say(&format!("SR-ERROR bad arguments: {e}"));
            return EXIT_ARGS;
        }
    };
    state.upload(&gpu.queue, &sun_disk_planes(args.world)).expect("the preset fills 14 planes of the world");
    // The shipped registries (a scene's overrides arrive with M0-T25); the ledger reads the booking (its lot).
    let (physics, elements) = match (Physics::shipped(), Elements::shipped()) {
        (Ok(p), Ok(e)) => (p, e),
        (Err(e), _) | (_, Err(e)) => {
            say(&format!("SR-ERROR {e}"));
            return EXIT_ARGS;
        }
    };
    let booking = Booking::new(&gpu.device, args.world).expect("the state's world is valid");
    let step = Step::new(&gpu.device, &state, &booking, &EosGpu::new(&gpu.device, &physics, &elements));

    let target = args.steps.min(args.max_steps);
    let started = Instant::now();
    let mut done = 0u64;
    while done < target {
        let n = (target - done).min(CHUNK);
        step.run(&gpu.device, &gpu.queue, n as u32);
        if let Err(e) = gpu.device.poll(wgpu::PollType::wait_indefinitely()) {
            say(&format!("SR-ERROR GPU error after {done} steps: {e}"));
            return EXIT_GPU;
        }
        done += n;
    }
    let wall_s = started.elapsed().as_secs_f64();

    let met = done == args.steps;
    let sim_time = 0.0; // P1 (the time step) arrives with M0-T9's lot; until then a step advances no time.
    if let Some(dir) = &args.out {
        let summary = json!({
            "version": 1,
            "tool": format!("sandbox-reactions {}", env!("CARGO_PKG_VERSION")),
            "git": git_rev(),
            "adapter": {"name": gpu.info.name, "backend": format!("{:?}", gpu.info.backend), "driver": gpu.info.driver},
            "scene": args.scene,
            "world": {"width": args.world.width, "height": args.world.height},
            "steps": done,
            "sim_time": sim_time,
            "wall_s": wall_s,
            "ledger": {"start": {}, "end": {}},
            "events": [],
            "objects": [],
            "until": {"condition": format!("steps:{}", args.steps), "met": met},
        });
        let text = serde_json::to_string_pretty(&summary).expect("a json! value serialises");
        if let Err(e) = std::fs::write(dir.join("summary.json"), text + "\n") {
            say(&format!("SR-ERROR bad arguments: --out {}: {e}", dir.display()));
            return EXIT_ARGS;
        }
    }
    say(&format!(
        "SR-HEADLESS DONE steps={done} sim_time={sim_time:.6} until={}",
        if met { "none" } else { "unmet" }
    ));
    if met {
        EXIT_DONE
    } else {
        EXIT_UNMET
    }
}
