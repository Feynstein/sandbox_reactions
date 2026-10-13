//! `sandbox-reactions headless` (m0_contrat.md §3.3, §6.3): load a scene, run N steps on a surfaceless device, write the
//! run summary (§2.12.2). Today: `--scene` reads a scene file or `preset:<name>` through [`crate::scene`] (M0-T25; every
//! preset is the nominal Sun-like disk until M0-T74), `--world` and `--edge` stand over the scene's `world`, `--steps N`
//! runs [`Step`] N times and `--timing` adds the steps' cost (M0-T23); `--until`, the ledger, events and objects arrive
//! with their blocks. Of the scene's switches, `gravity` turns P2 on (M0-T31) and `hydro` P4 (M0-T34) under the run's edge
//! mode (M0-T35); the other switches are recorded, not yet applied (no pass reads them).
//!
//! State dumps (§2.12.3, [`crate::dump`], M0-T26): with `--out`, DIR/dumps/s_<step>.bin at the end of every run, and with
//! `--dump-every N` at every step index that N divides — the index counts from the scene, so a resumed run's dumps fall
//! where a straight run's do. `--load-dump FILE` starts from a dump instead of the scene's objects: its world (a
//! `--world` other than it is refused) and cells, its step index and the step's records; the scene still gives the
//! constants, the edge and the switches. `--steps N` is then N more steps. A dump refused is exit 4.
//!
//! stdout: the first line `SR-ADAPTER …`; the last, when the run ends without an error (exit 0 or 2), `SR-HEADLESS DONE
//! steps=<n> sim_time=<t> until=<met|unmet|none>`; every error line starts `SR-ERROR`, every warning `SR-WARN`.
//! Exit codes: 0 done · 2 `--steps` not reached within `--max-steps` · 3 no adapter or a GPU error · 4 bad arguments
//! or scene · 5 a non-finite value in the state, found by P9's guard every 64 steps and read after each submission
//! (the message names the step, cell and channel) · 101 a panic (Rust's own). 6 (calibration) arrives with its check.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use serde_json::json;
use sr_physics::registry::Elements;

use crate::dump::{dump_state, load_dump, Dump};
use crate::gpu::Gpu;
use crate::scene::{EdgeMode, Scene};
use crate::state::{Booking, SideFields, State, WorldConfig};
use crate::step::boxfit::BoxMode;
use crate::step::{CostSource, EosGpu, Pass, Passes, Step, StepCost, Timing};

pub const EXIT_DONE: i32 = 0;
pub const EXIT_UNMET: i32 = 2;
pub const EXIT_GPU: i32 = 3;
pub const EXIT_ARGS: i32 = 4;
pub const EXIT_NON_FINITE: i32 = 5;

/// §3.3: `--max-steps` defaults to 2,000,000.
pub const DEFAULT_MAX_STEPS: u64 = 2_000_000;
/// Steps recorded per submission: the queue never holds more than this many at once.
const CHUNK: u64 = 256;

#[derive(Debug, PartialEq)]
pub struct Args {
    pub scene: String,
    pub steps: u64,
    /// `--world` and `--edge`: over the scene's own `world` when given.
    pub world: Option<WorldConfig>,
    pub edge: Option<EdgeMode>,
    pub max_steps: u64,
    pub out: Option<PathBuf>,
    pub adapter: Option<String>,
    /// `--timing`: summary.json gets `timing` (§2.12.2), the steps' cost per pass and per step.
    pub timing: bool,
    /// `--dump-every N` (needs `--out`): a dump at every step index N divides.
    pub dump_every: Option<u64>,
    /// `--load-dump FILE`: the run starts from this dump.
    pub load_dump: Option<PathBuf>,
}

/// Parses what follows `headless`. `--flag value` or `--flag=value`, `--timing` alone; anything else is an error
/// naming it.
pub fn parse_args(args: &[String]) -> Result<Args, String> {
    let (mut scene, mut steps, mut world, mut edge, mut max_steps, mut out, mut adapter) =
        (None, None, None, None, DEFAULT_MAX_STEPS, None, None);
    let (mut dump_every, mut load_dump) = (None, None);
    let mut timing = false;
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        if arg == "--timing" {
            timing = true;
            continue;
        }
        let (flag, inline) = match arg.split_once('=') {
            Some((f, v)) => (f, Some(v.to_string())),
            None => (arg.as_str(), None),
        };
        let known = ["--scene", "--steps", "--world", "--edge", "--max-steps", "--out", "--adapter", "--dump-every", "--load-dump"];
        if !known.contains(&flag) {
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
            "--dump-every" => dump_every = Some(count(&value)?),
            "--load-dump" => load_dump = Some(PathBuf::from(value)),
            "--edge" => edge = Some(EdgeMode::parse(&value).map_err(|e| format!("--{e}"))?),
            _ => {
                let size = value.split_once('x').and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?)));
                let (w, h) = size.ok_or_else(|| format!("--world takes WxH, got {value:?}"))?;
                world = Some(WorldConfig::new(w, h).map_err(|e| e.to_string())?);
            }
        }
    }
    let scene = scene.ok_or("--scene is required (a scene file, or preset:sun)")?;
    let steps = steps.ok_or("--steps is required (--until arrives with its block)")?;
    if steps == 0 {
        return Err("--steps must be at least 1".to_string());
    }
    if dump_every == Some(0) {
        return Err("--dump-every must be at least 1".to_string());
    }
    if dump_every.is_some() && out.is_none() {
        return Err("--dump-every needs --out (dumps go to DIR/dumps)".to_string());
    }
    Ok(Args { scene, steps, world, edge, max_steps, out, adapter, timing, dump_every, load_dump })
}

/// Reads `--scene` before any GPU is touched: `preset:<name>` (one preset at the world's centre), else a scene file
/// (§2.12.1, [`Scene::load`]). Any refusal is exit 4.
fn load_scene(arg: &str, world: Option<WorldConfig>) -> Result<Scene, String> {
    match arg.strip_prefix("preset:") {
        Some(name) => Scene::preset(name, world.unwrap_or_default()),
        None => Scene::load(Path::new(arg)),
    }
    .map_err(|e| format!("scene {arg:?}: {e}"))
}

/// The disk builder moved into the scene loader (M0-T25); kept here for the callers that name it so.
pub use crate::scene::sun_disk_planes;
/// The run's cost (`--timing`): each batch's means weighted by the steps it timed.
#[derive(Default)]
struct Costs {
    source: Option<CostSource>,
    steps: u64,
    step_ms: f64,
    pass_ms: [Option<f64>; 10],
}

impl Costs {
    fn add(&mut self, c: &StepCost) {
        let w = f64::from(c.steps);
        self.source = Some(c.source);
        self.steps += u64::from(c.steps);
        self.step_ms += c.step_ms * w;
        for (sum, p) in self.pass_ms.iter_mut().zip(c.pass_ms) {
            if let Some(ms) = p {
                *sum.get_or_insert(0.0) += ms * w;
            }
        }
    }

    /// §2.12.2's `timing`: `per_pass_ms` (each pass the steps recorded, by its label, ms per step), `step_ms_mean`,
    /// and `source` — `gpu-timestamp` or `wall`, the latter with no pass named; none while timing is off.
    fn summary(&self) -> Option<serde_json::Value> {
        let source = self.source?;
        let n = self.steps.max(1) as f64;
        let per_pass: serde_json::Map<String, serde_json::Value> = Pass::ORDER
            .iter()
            .zip(self.pass_ms)
            .filter_map(|(pass, ms)| Some((pass.label().to_string(), json!(ms? / n))))
            .collect();
        let mean = (self.steps > 0).then(|| self.step_ms / n);
        Some(json!({"per_pass_ms": per_pass, "step_ms_mean": mean, "source": source.name()}))
    }
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
    // A dump to start from, read and checked before anything else (exit 4).
    let dump = match args.load_dump.as_deref().map(Dump::read).transpose() {
        Ok(d) => d,
        Err(e) => {
            say(&format!("SR-ERROR bad dump: {e}"));
            return EXIT_ARGS;
        }
    };
    if let (Some(d), Some(w)) = (&dump, args.world) {
        if d.world != w {
            let text = format!("--world {}x{} against the dump's {}x{}", w.width, w.height, d.world.width, d.world.height);
            say(&format!("SR-ERROR bad arguments: {text}"));
            return EXIT_ARGS;
        }
    }
    // The scene, its world and constants and the state it starts from, all before any GPU is touched (exit 4); a dump
    // gives the world and the cells instead.
    let prepared = load_scene(&args.scene, args.world).and_then(|scene| {
        let physics = scene.physics()?;
        let elements = Elements::shipped().map_err(|e| e.to_string())?;
        let edge = args.edge.unwrap_or(scene.edge);
        if let Some(d) = &dump {
            return Ok((d.world, edge, physics, elements, Vec::new(), scene.warnings(), scene.switches));
        }
        let world = args.world.unwrap_or(scene.world);
        let (planes, mut warnings) = scene.planes(world, &physics, &elements)?;
        warnings.splice(0..0, scene.warnings());
        Ok((world, edge, physics, elements, planes, warnings, scene.switches))
    });
    let (world, edge, physics, elements, planes, warnings, switches) = match prepared {
        Ok(p) => p,
        Err(e) => {
            say(&format!("SR-ERROR bad scene: {e}"));
            return EXIT_ARGS;
        }
    };
    if let Some(dir) = &args.out {
        if let Err(e) = std::fs::create_dir_all(dir.join("dumps")) {
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

    let state = match State::new(&gpu.device, world) {
        Ok(s) => s,
        Err(e) => {
            say(&format!("SR-ERROR bad arguments: {e}"));
            return EXIT_ARGS;
        }
    };
    // The physics is the shipped constants with the scene's overrides; the ledger reads the booking (its lot).
    let booking = Booking::new(&gpu.device, world).expect("the state's world is valid");
    let eos = EosGpu::new(&gpu.device, &physics, &elements);
    let side = SideFields::new(&gpu.device, world).expect("the state's world is valid");
    let passes =
        Passes { mode: BoxMode::Fit, gravity: switches.gravity.then_some(&side), hydro: switches.hydro.then_some(edge) };
    let step = match Step::with_passes(&gpu.device, &state, &booking, &eos, &physics, &passes) {
        Ok(s) => s,
        Err(e) => {
            say(&format!("SR-ERROR bad arguments: gravity over a {}x{} world: {e}", world.width, world.height));
            return EXIT_ARGS;
        }
    };
    let first = match &dump {
        Some(d) => {
            if let Err(e) = load_dump(&gpu.queue, &state, &step, d) {
                say(&format!("SR-ERROR bad dump: {e}"));
                return EXIT_ARGS;
            }
            d.step
        }
        None => {
            state.upload(&gpu.queue, &planes).expect("the scene fills 14 planes of the world");
            0
        }
    };
    let timing = if args.timing { Timing::Auto } else { Timing::Off };
    let mut cost = Costs { source: step.set_timing(&gpu.device, &gpu.queue, timing), ..Costs::default() };
    // A dump of the state as it stands, named by its step index; a failed write is exit 4, as a bad --out.
    let write_dump = |dir: &Path| {
        let d = dump_state(&gpu.device, &gpu.queue, &state, &step);
        let path = dir.join("dumps").join(format!("s_{}.bin", d.step));
        d.write(&path).map_err(|e| format!("SR-ERROR bad arguments: --out {}: {e}", path.display()))
    };

    let target = args.steps.min(args.max_steps);
    let started = Instant::now();
    let mut done = 0u64;
    while done < target {
        let mut n = (target - done).min(CHUNK);
        if let Some(every) = args.dump_every {
            n = n.min(every - (first + done) % every);
        }
        step.run(&gpu.device, &gpu.queue, n as u32);
        if let Err(e) = gpu.device.poll(wgpu::PollType::wait_indefinitely()) {
            say(&format!("SR-ERROR GPU error after {done} steps: {e}"));
            return EXIT_GPU;
        }
        done += n;
        if let Some(c) = step.read_cost(&gpu.device, &gpu.queue) {
            cost.add(&c);
        }
        if let Some(bad) = step.read_dt(&gpu.device, &gpu.queue).non_finite {
            say(&format!(
                "SR-ERROR non-finite value in the state at step {}, cell ({}, {}), channel {}",
                bad.step, bad.x, bad.y, bad.channel
            ));
            return EXIT_NON_FINITE;
        }
        let at = first + done;
        if let (Some(every), Some(dir)) = (args.dump_every, &args.out) {
            if at.is_multiple_of(every) && done < target {
                if let Err(e) = write_dump(dir) {
                    say(&e);
                    return EXIT_ARGS;
                }
            }
        }
    }
    let wall_s = started.elapsed().as_secs_f64();
    // The final dump, always (§3.3).
    if let Some(dir) = &args.out {
        if let Err(e) = write_dump(dir) {
            say(&e);
            return EXIT_ARGS;
        }
    }

    let met = done == args.steps;
    // The sim time of every step run since the scene, its dump's included — the Δt record's, as the final dump writes it.
    let sim_time = step.read_dt(&gpu.device, &gpu.queue).sim_time;
    if let Some(dir) = &args.out {
        let mut summary = json!({
            "version": 1,
            "tool": format!("sandbox-reactions {}", env!("CARGO_PKG_VERSION")),
            "git": git_rev(),
            "adapter": {"name": gpu.info.name, "backend": format!("{:?}", gpu.info.backend), "driver": gpu.info.driver},
            "scene": args.scene,
            "world": {"width": world.width, "height": world.height, "edge": edge.name()},
            "steps": done,
            "sim_time": sim_time,
            "wall_s": wall_s,
            "ledger": {"start": {}, "end": {}},
            "events": [],
            "objects": [],
            "until": {"condition": format!("steps:{}", args.steps), "met": met},
        });
        if let Some(timing) = cost.summary() {
            summary["timing"] = timing;
        }
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
