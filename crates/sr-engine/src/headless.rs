//! `sandbox-reactions headless` (m0_contrat.md §3.3, §6.3): load a scene, run N steps on a surfaceless device, write the
//! run summary (§2.12.2). Today: `--scene` reads a scene file or `preset:<name>` through [`crate::scene`] (M0-T25; every
//! preset is the nominal Sun-like disk until M0-T74), `--world` and `--edge` stand over the scene's `world`, `--steps N`
//! runs [`Step`] N times and `--timing` adds the steps' cost (M0-T23); the ledger (§2.9, [`crate::ledger`], M0-T57) is
//! sampled after every chunk of steps and written as summary.json's `ledger`. The observers ([`crate::watch`], M0-T60) read
//! the same summary after every chunk — a frame: objects, stages, events and `ending` — and write summary.json's `events`
//! and `objects` and DIR/events.jsonl; `--until sim_time:<t> | event:<kind> | stage:<id> | ending` stops the run at the frame
//! whose summary meets it (`sim_time` at the first step past `t`; the others up to one frame of [`FRAME`] steps past the
//! step they first held — with `--until` a frame is that long, without it a chunk is [`CHUNK`]), `--max-steps` bounding it (exit 2, `until=unmet`). The ledger starts after the run's first step (the box and
//! φ exist only then), so that step is a chunk of its own. Of the scene's switches, `gravity` turns P2 on (M0-T31), `hydro` P4 (M0-T34) under the run's edge
//! mode (M0-T35) and `heat` P5 with the scene's `test` flags (M0-T42; a step whose implicit P5 hit its ceiling prints
//! `SR-WARN heat unconverged step <n> passes <k> rounds <r>` — read at the last step of each submission, M0-T119; every
//! step's is M0-T121's; a `calibrate` life counts them instead, M0-D74) and `reactions` P6 with the registry's enabled records and
//! the scene's `pin_temperature` (M0-T46), `sinks` P3 (M0-T51) and `neutrinos` P7 (M0-T54; M0-D47) — P7 reads the S_ν plane P6 writes,
//! so with `reactions` off it is not built, not a gate run every step over a zero plane.
//!
//! State dumps (§2.12.3, [`crate::dump`], M0-T26): with `--out`, DIR/dumps/s_<step>.bin at the end of every run, and with
//! `--dump-every N` at every step index that N divides — the index counts from the scene, so a resumed run's dumps fall
//! where a straight run's do. `--load-dump FILE` starts from a dump instead of the scene's objects: its world (a
//! `--world` other than it is refused) and cells, its step index and the step's records; the scene still gives the
//! constants, the edge and the switches. `--steps N` is then N more steps. A dump refused is exit 4.
//!
//! Beside every dump of a GPU run, the observers' snapshot `dumps/s_<step>.observers.json` (contract §0.5 [M0-D76]): the
//! [`Watch`], the [`Ledger`], the steps done since the run's start and its wall time, the next Δt and P5's unconverged
//! steps, under the dump's step, sim time, scene label and world. `--load-dump FILE --continue` reads it back and goes on as
//! the cut run would have — the same chunks and frames, summary.json the whole life's; `--steps` and `--max-steps` then
//! count from the life's start. A missing snapshot, or one of another step, scene or world, is exit 4.
//!
//! stdout: the first line `SR-ADAPTER …`; the last, when the run ends without an error (exit 0 or 2), `SR-HEADLESS DONE
//! steps=<n> sim_time=<t> until=<met|unmet|none>` (`none` for a run on `--steps`, which `--max-steps` may cut: `unmet`); every error line starts `SR-ERROR`, every warning `SR-WARN`.
//! Exit codes: 0 done · 2 `--steps` or `--until` not reached within `--max-steps` · 3 no adapter or a GPU error · 4 bad arguments
//! or scene · 5 a non-finite value in the state, found by P9's guard every 64 steps and read after each submission, or by
//! the scan of the run's last step (M0-D59) (the message names the step, cell and channel) · 6 a calibration out of date, invalid or missing for a scene that names a preset (§2.11, G-CAL, M0-T65:
//! [`calibration_gate`]; with no calibration embedded yet that gate has nothing to read) · 101 a panic (Rust's own).
//!
//! `--cpu-reference` (M0-T37) runs the CPU f64 twin ([`sr_physics::reference::Twin`]) from the same scene in place of the
//! GPU, worlds up to 128 × 128 (exit 4 above), and writes the same summary.json and dumps: no adapter is touched, so the
//! first line reads `SR-ADAPTER name=cpu-reference backend=cpu driver=sr_physics::reference`, and `--adapter`, `--timing`
//! `--load-dump` (the GPU's own) and `--until` (the twin has no block summary to observe) are refused. The twin has no active box: its dumps carry the box record of the whole
//! world, the GPU's `BoxMode::WholeWorld` form.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use serde_json::json;
use sr_physics::calibration;
use sr_physics::observe::tracker::{exact, read_exact, read_whole};
use sr_physics::reference::hydro::Edge;
use sr_physics::reference::{Switches as TwinSwitches, Twin, GUARD_EVERY};
use sr_physics::registry::{Elements, Physics, Reactions};

use crate::dump::{dump_state, load_dump, Dump, Resume};
use crate::gpu::Gpu;
use crate::ledger::{take_booked, At, Ledger};
use crate::scene::{EdgeMode, Object, Scene, Switches};
use crate::state::{Booking, SideFields, State, WorldConfig, CHANNELS};
use crate::step::boxfit::BoxMode;
use crate::step::heat::HeatSetup;
use crate::step::nuheat::NuHeatSetup;
use crate::step::react::ReactSetup;
use crate::step::sinks::SinkSetup;
use crate::step::{CostSource, EosGpu, Pass, Passes, Step, StepCost, Timing};
use crate::watch::{Until, Watch};

pub const EXIT_DONE: i32 = 0;
pub const EXIT_UNMET: i32 = 2;
pub const EXIT_GPU: i32 = 3;
pub const EXIT_ARGS: i32 = 4;
pub const EXIT_NON_FINITE: i32 = 5;
pub const EXIT_CALIBRATION: i32 = sr_physics::calibration::EXIT_CALIBRATION;

/// §3.3: `--max-steps` defaults to 2,000,000.
pub const DEFAULT_MAX_STEPS: u64 = 2_000_000;
/// Steps recorded per submission: the queue never holds more than this many at once — nor P5's log of unconverged steps.
const CHUNK: u64 = 256;
/// A frame while `--until` watches, in steps (untuned): the cold disk's whole collapse is some 250 steps, so a stage that
/// must show for 3 frames (§1.9.3) needs frames of a few steps — 16 shows `collapsing` at step 49, 64 never does; shorter
/// frames cost more (fill_world, 800 steps on the RTX 5090: 2.5 s at 256, 3.1 s at 16, 4.6 s at 4).
const FRAME: u64 = 16;
const _: () = assert!(CHUNK as usize <= crate::step::heat::LOG);

#[derive(Debug, PartialEq)]
pub struct Args {
    pub scene: String,
    /// `--steps N` or `--until <condition>`, one of the two.
    pub steps: Option<u64>,
    pub until: Option<Until>,
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
    /// `--continue` (needs `--load-dump`): the run goes on as the one that wrote the dump would have, from the observers'
    /// snapshot beside it (§0.5 [M0-D76]).
    pub continue_life: bool,
    /// `--cpu-reference`: the CPU f64 twin in place of the GPU.
    pub cpu_reference: bool,
}

/// Parses what follows `headless`. `--flag value` or `--flag=value`, `--timing`, `--continue` and `--cpu-reference` alone;
/// anything else is an error naming it.
pub fn parse_args(args: &[String]) -> Result<Args, String> {
    let (mut scene, mut steps, mut world, mut edge, mut max_steps, mut out, mut adapter) =
        (None, None, None, None, DEFAULT_MAX_STEPS, None, None);
    let (mut dump_every, mut load_dump, mut until) = (None, None, None);
    let (mut timing, mut cpu_reference, mut continue_life) = (false, false, false);
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        if arg == "--timing" {
            timing = true;
            continue;
        }
        if arg == "--cpu-reference" {
            cpu_reference = true;
            continue;
        }
        if arg == "--continue" {
            continue_life = true;
            continue;
        }
        let (flag, inline) = match arg.split_once('=') {
            Some((f, v)) => (f, Some(v.to_string())),
            None => (arg.as_str(), None),
        };
        let known = ["--scene", "--steps", "--world", "--edge", "--max-steps", "--out", "--adapter", "--dump-every", "--load-dump", "--until"];
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
            "--until" => until = Some(Until::parse(&value)?),
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
    if steps.is_some() == until.is_some() {
        return Err("give --steps N or --until <condition>, one of the two".to_string());
    }
    if steps == Some(0) {
        return Err("--steps must be at least 1".to_string());
    }
    if dump_every == Some(0) {
        return Err("--dump-every must be at least 1".to_string());
    }
    if dump_every.is_some() && out.is_none() {
        return Err("--dump-every needs --out (dumps go to DIR/dumps)".to_string());
    }
    if continue_life && load_dump.is_none() {
        return Err("--continue needs --load-dump (the dump it goes on from)".to_string());
    }
    if cpu_reference {
        let refused =
            [("--adapter", adapter.is_some()), ("--timing", timing), ("--load-dump", load_dump.is_some()), ("--until", until.is_some())];
        if let Some((flag, _)) = refused.iter().find(|(_, given)| *given) {
            return Err(format!("{flag} is the GPU's: it has no meaning with --cpu-reference"));
        }
    }
    Ok(Args { scene, steps, until, world, edge, max_steps, out, adapter, timing, dump_every, load_dump, continue_life, cpu_reference })
}

/// The observers' snapshot beside the dump at `dump` (§0.5 [M0-D76]): `dumps/s_<step>.observers.json`.
pub fn observers_path(dump: &Path) -> PathBuf {
    dump.with_extension("observers.json")
}

/// What a cut run goes on with under `--continue` (§0.5 [M0-D76]), read from the observers' snapshot beside its dump: the
/// steps it ran since its own start (the scene, or the dump it was loaded from), its wall time, the next Δt its chunks were
/// sized by, the steps that reached P5's ceiling, and the observers.
struct Carried {
    done: u64,
    wall_s: f64,
    dt_next: f64,
    heat_unconverged: u64,
    watch: Watch,
    ledger: Ledger,
}

/// The snapshot beside `path` for `dump`, refused unless it is the same step (index and sim time, bit for bit), scene label
/// and world as the run.
fn read_carried(path: &Path, dump: &Dump, scene: &str, physics: &Physics) -> Result<Carried, String> {
    let file = observers_path(path);
    let text = std::fs::read_to_string(&file).map_err(|e| format!("--continue: {}: {e}", file.display()))?;
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("--continue: {}: not JSON: {e}", file.display()))?;
    let refused = |what: String| format!("--continue: {} is {what}", file.display());
    if v["version"] != 1 {
        return Err(refused(format!("version {}, this build reads 1", v["version"])));
    }
    let sim_time = read_exact(&v["sim_time"], "sim_time").map_err(refused)?;
    if v["step"].as_u64() != Some(dump.step) || sim_time.to_bits() != dump.sim_time.to_bits() {
        return Err(refused(format!("step {} at {sim_time}, the dump's {} at {}", v["step"], dump.step, dump.sim_time)));
    }
    if v["scene"].as_str() != Some(scene) {
        return Err(refused(format!("scene {}, the run's {scene:?}", v["scene"])));
    }
    if v["world"] != json!([dump.world.width, dump.world.height]) {
        return Err(refused(format!("world {}, the dump's {}x{}", v["world"], dump.world.width, dump.world.height)));
    }
    let done = read_whole(&v, "done").map_err(refused)?;
    if done > dump.step {
        return Err(refused(format!("{done} steps done past the dump's step {}", dump.step)));
    }
    let world_width = dump.world.width;
    Ok(Carried {
        done,
        wall_s: read_exact(&v["wall_s"], "wall_s").map_err(refused)?,
        dt_next: read_exact(&v["dt_next"], "dt_next").map_err(refused)?,
        heat_unconverged: read_whole(&v, "heat_unconverged").map_err(refused)?,
        watch: Watch::restore(physics, world_width, &v["watch"]).map_err(refused)?,
        ledger: Ledger::restore(dump.world, physics, &v["ledger"]).map_err(refused)?,
    })
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
/// The run's cost (`--timing`): each batch's means weighted by the steps it ended — a step carried over submissions
/// (§0.5 [M0-TJ4] (6)) counted once, with its rounds in all of them.
#[derive(Default)]
struct Costs {
    source: Option<CostSource>,
    steps: u64,
    step_ms: f64,
    pass_ms: [Option<f64>; 10],
}

impl Costs {
    fn add(&mut self, c: &StepCost) {
        let w = f64::from(c.ended);
        self.source = Some(c.source);
        self.steps += u64::from(c.ended);
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

/// The lines `say` has printed since [`run_captured`] began, while it runs.
static CAPTURE: std::sync::Mutex<Option<Vec<String>>> = std::sync::Mutex::new(None);

/// The start of the warning line P5's ceiling prints (§0.5 [M0-TJ4] (6)).
const HEAT_UNCONVERGED_LINE: &str = "SR-WARN heat unconverged";

/// While a [`run_scene_captured`] life runs (M0-D74): its `SR-WARN heat unconverged` lines are counted here, not printed or kept —
/// a long life can hit the ceiling thousands of times and they bury `calibrate`'s own lines. `calibrate` says the count in its
/// progress and run lines instead, so it is never silent. [`unconverged_steps`] reads it; each captured run starts it at 0.
static HEAT_UNCONVERGED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Whether the run in progress counts its heat warnings instead of printing them: only a [`capturing`] run that asks for it.
static COUNT_HEAT_WARNINGS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The steps that reached P5's ceiling in the current or last [`capturing`] run that counted them.
pub fn unconverged_steps() -> u64 {
    HEAT_UNCONVERGED.load(std::sync::atomic::Ordering::Relaxed)
}

/// One line of the run's output: printed and kept while a run is captured — but a heat-unconverged warning of a run that counts
/// them is only counted ([`capturing`]).
pub fn say(line: &str) {
    if COUNT_HEAT_WARNINGS.load(std::sync::atomic::Ordering::Relaxed) && line.starts_with(HEAT_UNCONVERGED_LINE) {
        HEAT_UNCONVERGED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        return;
    }
    println!("{line}");
    if let Some(lines) = CAPTURE.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
        lines.push(line.to_string());
    }
}

/// What `f` said through [`say`], kept while it runs (one run at a time); `count_heat` has its `SR-WARN heat unconverged` lines
/// counted into [`unconverged_steps`] (from 0) instead of printed and kept (M0-D74).
pub fn capturing<R>(count_heat: bool, f: impl FnOnce() -> R) -> (R, Vec<String>) {
    *CAPTURE.lock().unwrap_or_else(|e| e.into_inner()) = Some(Vec::new());
    HEAT_UNCONVERGED.store(0, std::sync::atomic::Ordering::Relaxed);
    COUNT_HEAT_WARNINGS.store(count_heat, std::sync::atomic::Ordering::Relaxed);
    let result = f();
    COUNT_HEAT_WARNINGS.store(false, std::sync::atomic::Ordering::Relaxed);
    let lines = CAPTURE.lock().unwrap_or_else(|e| e.into_inner()).take().unwrap_or_default();
    (result, lines)
}

/// [`run`], and the stdout lines it printed — how the tests read the `SR-HEADLESS DONE` line (one run at a time).
pub fn run_captured(args: &[String]) -> (i32, Vec<String>) {
    capturing(false, || run(args))
}

pub fn git_rev() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// The exit-5 line (§3.3): the step the value was found at, its cell and its channel's §2.2 name.
fn non_finite_line(step: u64, x: u64, y: u64, channel: &str) -> String {
    format!("SR-ERROR non-finite value in the state at step {step}, cell ({x}, {y}), channel {channel}")
}

/// The first non-finite value in `planes` (§2.2's channel-major layout) by cell and then channel, as P9's guard finds
/// it: (cell, channel).
fn first_non_finite(planes: &[f32]) -> Option<(usize, usize)> {
    let n = planes.len() / CHANNELS.len();
    (0..n).find_map(|i| (0..CHANNELS.len()).find(|&c| !planes[c * n + i].is_finite()).map(|c| (i, c)))
}

/// The twin's state as a dump (§2.12.3): the planes in f32 as the GPU's are, the step index and sim time, and the step's
/// records — Δt_n and Δt_{n+1} as f32, no latch, and the box record of the whole world with its FFT size per axis (§1.3.3:
/// the smallest power of two ≥ 2 × the box, at least 32) and its indirect sizes, the twin having no active box; no F, the
/// twin running no P5 (M0-T48).
pub fn twin_dump(twin: &Twin, world: WorldConfig) -> Dump {
    let fft = |n: u32| (2 * n).next_power_of_two().max(32);
    let (w, h) = (world.width, world.height);
    Dump {
        world,
        step: twin.step_index(),
        sim_time: twin.sim_time(),
        sinks: Vec::new(),
        resume: Resume {
            dt: twin.dt() as f32,
            dt_next: twin.dt_next() as f32,
            latch_since: 0.0,
            box_record: [0, 0, w, h, fft(w), fft(h), 0, 0],
            box_args: [w.div_ceil(8), h.div_ceil(8), 1, (world.cells() as u32).div_ceil(256), 1, 1],
        },
        planes: twin.state.planes.iter().map(|&v| v as f32).collect(),
        flux: None,
    }
}

/// `--cpu-reference`: the twin from the scene's planes, step by step, with the GPU run's outputs.
#[allow(clippy::too_many_arguments)]
fn run_twin(
    args: &Args,
    world: WorldConfig,
    edge: EdgeMode,
    physics: &Physics,
    elements: &Elements,
    planes: &[f32],
    warnings: &[String],
    switches: Switches,
) -> i32 {
    let edge_of = match edge {
        EdgeMode::Leave => Edge::Leave,
        EdgeMode::Bounce => Edge::Bounce,
    };
    let planes = planes.iter().map(|&v| f64::from(v)).collect();
    let (w, h) = (world.width as usize, world.height as usize);
    // The twin's passes are gravity and hydro (its scope's expectation); the scene's heat and reactions are not applied here.
    let twin_switches = TwinSwitches { gravity: switches.gravity, hydro: switches.hydro, heat: false, reactions: false };
    let mut twin = match Twin::new(physics, elements, edge_of, twin_switches, None, w, h, planes) {
            Ok(t) => t,
            Err(e) => {
                say(&format!("SR-ERROR bad arguments: --cpu-reference: {e}"));
                return EXIT_ARGS;
            }
        };
    say("SR-ADAPTER name=cpu-reference backend=cpu driver=sr_physics::reference");
    for w in warnings {
        say(&format!("SR-WARN {w}"));
    }
    let write_dump = |twin: &Twin, dir: &Path| {
        let d = twin_dump(twin, world);
        let path = dir.join("dumps").join(format!("s_{}.bin", d.step));
        d.write(&path).map_err(|e| format!("SR-ERROR bad arguments: --out {}: {e}", path.display()))
    };
    // --until is refused with --cpu-reference (parse_args), so --steps is given.
    let wanted = args.steps.expect("--cpu-reference runs --steps");
    let target = wanted.min(args.max_steps);
    let started = Instant::now();
    // P9's guard names the first non-finite value, by cell then channel, at the step it found it.
    let non_finite = |twin: &Twin| {
        twin.first_non_finite().map(|(cell, channel)| non_finite_line(twin.step_index() - 1, (cell % w) as u64, (cell / w) as u64, CHANNELS[channel]))
    };
    for done in 0..target {
        twin.step();
        // P9's guard, on the step indices it runs on.
        if (twin.step_index() - 1).is_multiple_of(GUARD_EVERY) {
            if let Some(line) = non_finite(&twin) {
                say(&line);
                return EXIT_NON_FINITE;
            }
        }
        if let (Some(every), Some(dir)) = (args.dump_every, &args.out) {
            if twin.step_index().is_multiple_of(every) && done + 1 < target {
                if let Err(e) = write_dump(&twin, dir) {
                    say(&e);
                    return EXIT_ARGS;
                }
            }
        }
    }
    let wall_s = started.elapsed().as_secs_f64();
    // The last step's state is scanned too (M0-D59): a run ending between two of P9's scans still exits 5 on a non-finite world.
    if target > 0 {
        if let Some(line) = non_finite(&twin) {
            say(&line);
            return EXIT_NON_FINITE;
        }
    }
    if let Some(dir) = &args.out {
        if let Err(e) = write_dump(&twin, dir) {
            say(&e);
            return EXIT_ARGS;
        }
    }
    let met = target == wanted;
    let sim_time = twin.sim_time();
    if let Some(dir) = &args.out {
        let summary = json!({
            "version": 1,
            "tool": format!("sandbox-reactions {}", env!("CARGO_PKG_VERSION")),
            "git": git_rev(),
            "adapter": {"name": "cpu-reference", "backend": "cpu", "driver": "sr_physics::reference"},
            "scene": args.scene,
            "world": {"width": world.width, "height": world.height, "edge": edge.name()},
            "steps": target,
            "sim_time": sim_time,
            "wall_s": wall_s,
            "ledger": {"start": {}, "end": {}},
            "events": [],
            "objects": [],
            "until": {"condition": format!("steps:{wanted}"), "met": met},
        });
        let text = serde_json::to_string_pretty(&summary).expect("a json! value serialises");
        let written = std::fs::write(dir.join("summary.json"), text + "\n").and_then(|()| std::fs::write(dir.join("events.jsonl"), ""));
        if let Err(e) = written {
            say(&format!("SR-ERROR bad arguments: --out {}: {e}", dir.display()));
            return EXIT_ARGS;
        }
    }
    say(&format!("SR-HEADLESS DONE steps={target} sim_time={sim_time:.6} until={}", if met { "none" } else { "unmet" }));
    if met {
        EXIT_DONE
    } else {
        EXIT_UNMET
    }
}

/// §2.11's refusal as the run sees it: `embedded` is the calibration compiled in ([`calibration::EMBEDDED`]), `names_preset`
/// whether the scene names a preset. `Ok` for a run that needs none, and — until M0-V9 embeds a file — for a run with none
/// to check; otherwise [`calibration::resolve`]'s verdict as `(exit 6, the SR-ERROR line)`.
pub fn calibration_gate(embedded: Option<&str>, names_preset: bool) -> Result<(), (i32, String)> {
    if embedded.is_none() {
        return Ok(());
    }
    calibration::resolve(embedded, names_preset).map(|_| ()).map_err(|e| (e.exit_code(), e.line()))
}

/// Runs the command; returns the process's exit code.
pub fn run(args: &[String]) -> i32 {
    match parse_args(args) {
        Ok(a) => execute(a, None, None),
        Err(e) => {
            say(&format!("SR-ERROR bad arguments: {e}"));
            EXIT_ARGS
        }
    }
}

/// A run's progress hook (M0-D60): `tick(step, sim_time, stage)` is called each time the run's step count passes a multiple of
/// `every`, `stage` being the heaviest object's shown stage (`none` before one has held).
pub struct Progress<'a> {
    pub every: u64,
    pub tick: &'a mut dyn FnMut(u64, f64, &str),
    /// M0-D77: the run saves a checkpoint — a dump and the observers' snapshot, the last pair alone kept in `<out>/checkpoint/` —
    /// at the first frame boundary at or past each multiple of this many steps, apart from `every` and from `--dump-every`
    /// (no chunk is shortened: nothing the life computes moves). `saved(step, sim_time, stage)` follows each save.
    pub checkpoint_every: Option<u64>,
    pub saved: &'a mut dyn FnMut(u64, f64, &str),
}

/// `<out>/checkpoint/` (M0-D77): where a run's checkpoint pair lives, `s_<step>.bin` and `s_<step>.observers.json`.
pub const CHECKPOINT_DIR: &str = "checkpoint";

/// Removes every file in `folder` that does not belong to the pair of `step` — the older pairs, a temp file left by a crash —
/// once the pair of `step` is whole (the snapshot is renamed into place after its dump, so a reader finds this pair or the
/// previous one).
fn prune_checkpoints(folder: &Path, step: u64) {
    let keep = format!("s_{step}.");
    for entry in std::fs::read_dir(folder).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with("s_") && !name.starts_with(&keep) {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

/// A run of `scene`, built in code, under `args` ([`Args::scene`] is only the label summary.json records), and the stdout
/// lines it printed — `calibrate`'s way to a preset cloud of a given mass (M0-T68), which a scene file cannot name (§2.12.1),
/// with `progress` called along the way. The scene reads no calibration, so the gate of §2.11 is not applied: `calibrate`
/// runs on a stale one.
pub fn run_scene_captured(args: Args, scene: Scene, progress: Option<Progress>) -> (i32, Vec<String>) {
    // A life can reach P5's ceiling thousands of times: its warnings are counted (M0-D74), `calibrate` says the count.
    capturing(true, || execute(args, Some(scene), progress))
}

fn execute(args: Args, given: Option<Scene>, mut progress: Option<Progress>) -> i32 {
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
    let mut names_preset = false;
    let built = given.is_some();
    let prepared = given.map_or_else(|| load_scene(&args.scene, args.world), Ok).and_then(|scene| {
        names_preset = !built && scene.objects.iter().any(|o| matches!(o, Object::Preset { .. }));
        let physics = scene.physics()?;
        let elements = Elements::shipped().map_err(|e| e.to_string())?;
        let edge = args.edge.unwrap_or(scene.edge);
        if let Some(d) = &dump {
            return Ok((d.world, edge, physics, elements, Vec::new(), scene.warnings(), (scene.switches, scene.test)));
        }
        let world = args.world.unwrap_or(scene.world);
        let (planes, mut warnings) = scene.planes(world, &physics, &elements)?;
        warnings.splice(0..0, scene.warnings());
        Ok((world, edge, physics, elements, planes, warnings, (scene.switches, scene.test)))
    });
    let (world, edge, physics, elements, planes, warnings, (switches, test)) = match prepared {
        Ok(p) => p,
        Err(e) => {
            say(&format!("SR-ERROR bad scene: {e}"));
            return EXIT_ARGS;
        }
    };
    // `--continue`: the observers' snapshot beside the dump, of its step, scene and world (exit 4 otherwise).
    let carried = match (&dump, &args.load_dump) {
        (Some(d), Some(path)) if args.continue_life => match read_carried(path, d, &args.scene, &physics) {
            Ok(c) => Some(c),
            Err(e) => {
                say(&format!("SR-ERROR bad dump: {e}"));
                return EXIT_ARGS;
            }
        },
        _ => None,
    };
    // §2.11 (G-CAL): a scene that names a preset reads calibration, so a stale one stops the run here (exit 6) — before
    // the registry, the GPU or any output; a scene that names none never reads it, and runs without one.
    if let Err((code, line)) = calibration_gate(calibration::EMBEDDED, names_preset) {
        say(&line);
        return code;
    }
    let reactions = match Reactions::shipped(&elements, &physics) {
        Ok(r) => r,
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

    if args.cpu_reference {
        return run_twin(&args, world, edge, &physics, &elements, &planes, &warnings, switches);
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
    // The physics is the shipped constants with the scene's overrides; the ledger reads the booking.
    let booking = Booking::new(&gpu.device, world).expect("the state's world is valid");
    let eos = EosGpu::new(&gpu.device, &physics, &elements);
    let side = SideFields::new(&gpu.device, world).expect("the state's world is valid");
    let passes =
        Passes { mode: BoxMode::Fit, gravity: switches.gravity.then_some(&side), hydro: switches.hydro.then_some(edge) };
    let heat = switches.heat.then_some(HeatSetup { side: &side, test });
    let react = switches.reactions.then_some(ReactSetup { reactions: &reactions, test, neutrinos: switches.neutrinos, side: &side });
    // P3 reads φ from P2's plane; P7 reads S_ν from P6's, so it needs P6 (§2.12.1; M0-D47).
    let sink = switches.sinks.then_some(SinkSetup { side: &side });
    let nuheat = (switches.reactions && switches.neutrinos).then_some(NuHeatSetup { side: &side });
    let step = match Step::with_neutrino_heating(
        &gpu.device,
        &state,
        &booking,
        &eos,
        &physics,
        &passes,
        heat.as_ref(),
        react.as_ref(),
        sink.as_ref(),
        nuheat.as_ref(),
    ) {
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
    // A dump of the state as it stands, named by its step index, and beside it the observers' snapshot (§0.5 [M0-D76]):
    // `observers` with the dump's step and sim time, the scene label and the world, so that a reader never pairs a dump with
    // another step's; renamed into place once written. A failed write is exit 4, as a bad --out.
    // The adapter, its driver and the physics hash ride in the snapshot (§0.5 [M0-D77]): `calibrate`'s checkpoint is used
    // only on the adapter and the constants it was made on.
    let physics_hash = calibration::physics_hash().ok();
    let write_dump = |folder: &Path, mut observers: serde_json::Value| {
        let d = dump_state(&gpu.device, &gpu.queue, &state, &step);
        let path = folder.join(format!("s_{}.bin", d.step));
        observers["adapter"] = json!(gpu.info.name);
        observers["driver"] = json!(gpu.info.driver);
        observers["physics_hash"] = json!(physics_hash);
        observers["version"] = json!(1);
        observers["step"] = json!(d.step);
        observers["sim_time"] = exact(d.sim_time);
        observers["scene"] = json!(args.scene);
        observers["world"] = json!([d.world.width, d.world.height]);
        let (file, temp) = (observers_path(&path), path.with_extension("observers.json.tmp"));
        d.write(&path)
            .and_then(|()| std::fs::write(&temp, observers.to_string() + "\n"))
            .and_then(|()| std::fs::rename(&temp, &file))
            .map_err(|e| format!("SR-ERROR bad arguments: --out {}: {e}", path.display()))
    };

    let target = args.steps.unwrap_or(args.max_steps).min(args.max_steps);
    let started = Instant::now();
    // The run's sim time starts at the dump's, else 0. Under `--continue` the cut run's count, wall time, next Δt, P5's
    // unconverged steps and observers go on: `--steps` and `--max-steps` count the whole life, and step indices from its start.
    let start_sim = dump.as_ref().map_or(0.0, |d| d.sim_time);
    let fresh = || (0, 0.0, 0.0, 0, Watch::new(&physics, world.width, start_sim), Ledger::new(world, &physics));
    let continued = carried.is_some();
    let (mut done, wall_before, mut dt_next, mut unconverged, mut watch, mut ledger) =
        carried.map_or_else(fresh, |c| (c.done, c.wall_s, c.dt_next, c.heat_unconverged, c.watch, c.ledger));
    if COUNT_HEAT_WARNINGS.load(std::sync::atomic::Ordering::Relaxed) {
        HEAT_UNCONVERGED.fetch_add(unconverged, std::sync::atomic::Ordering::Relaxed);
    }
    let first = first - done;
    let mut until_met = false;
    // The Δt record's sim time after the last chunk: with the next Δt, it sizes the chunks of `--until sim_time`.
    let mut sim = start_sim;
    // The progress hook's last multiple of `every` (M0-D60): the run's start is not a tick.
    let mut ticked = progress.as_ref().map_or(0, |p| (first + done) / p.every.max(1));
    // The checkpoint's last multiple of `checkpoint_every`, likewise.
    let mut saved = progress.as_ref().and_then(|p| p.checkpoint_every).map_or(0, |e| (first + done) / e.max(1));
    // A continued life whose restored state already meets its `until` ends at once (M0-D77): the condition is otherwise read
    // on a frame's fresh events, and an event fired before the cut never fires again.
    if let (true, Some(until)) = (continued, args.until) {
        until_met = watch.holds(until, watch.events());
    }
    // What the observers' snapshot carries beside the observers.
    let carry = |done: u64, dt_next: f64, unconverged: u64, watch: &Watch, ledger: &Ledger| {
        let wall_s = wall_before + started.elapsed().as_secs_f64();
        json!({"done": done, "wall_s": exact(wall_s), "dt_next": exact(dt_next), "heat_unconverged": unconverged,
               "watch": watch.snapshot(), "ledger": ledger.snapshot()})
    };
    while done < target && !until_met {
        let mut n = (target - done).min(if args.until.is_some() { FRAME } else { CHUNK });
        // The ledger's start: the first step alone, then the state after it is the ledger's zero.
        if ledger.start().is_none() {
            n = 1;
        }
        // `--until sim_time:<t>` stops at the first step past t: a chunk covers at most half the time left at the next
        // step's Δt, so the last steps are single ones (a chunk longer than that is a Δt that doubled inside it).
        if let (Some(Until::SimTime(t)), true) = (args.until, dt_next > 0.0) {
            n = n.min(((t - sim) / (2.0 * dt_next)).floor().max(1.0) as u64);
        }
        if let Some(every) = args.dump_every {
            n = n.min(every - (first + done) % every);
        }
        step.run(&gpu.device, &gpu.queue, n as u32);
        if let Err(e) = gpu.device.poll(wgpu::PollType::wait_indefinitely()) {
            say(&format!("SR-ERROR GPU error after {done} steps: {e}"));
            return EXIT_GPU;
        }
        done += n;
        // Every step of the chunk that reached P5's ceiling (§0.5 [M0-TJ4] (6)); a chunk is never longer than the log.
        for w in step.heat_warnings(&gpu.device, &gpu.queue) {
            unconverged += 1;
            say(&w);
        }
        if let Some(c) = step.read_cost(&gpu.device, &gpu.queue) {
            cost.add(&c);
        }
        let dt = step.read_dt(&gpu.device, &gpu.queue);
        if let Some(bad) = dt.non_finite {
            say(&non_finite_line(bad.step, u64::from(bad.x), u64::from(bad.y), bad.channel));
            return EXIT_NON_FINITE;
        }
        // The ledger: the booked terms since the last chunk, the summary of the state after it.
        let booked = take_booked(&gpu.device, &gpu.queue, &booking);
        let summary = step.summarise(&gpu.device, &gpu.queue);
        let at_step = At { step: first + done, sim_time: dt.sim_time };
        if ledger.start().is_none() {
            ledger.restart(&summary, at_step);
        } else {
            ledger.update(&summary, &booked, at_step);
        }
        let fresh = watch.update(&summary, at_step);
        (sim, dt_next) = (dt.sim_time, f64::from(dt.dt_next));
        if let Some(p) = progress.as_mut() {
            if at_step.step / p.every.max(1) > ticked {
                ticked = at_step.step / p.every.max(1);
                (p.tick)(at_step.step, dt.sim_time, watch.heaviest_stage().unwrap_or("none"));
            }
        }
        if args.until.is_some_and(|u| watch.holds(u, &fresh)) {
            until_met = true;
            break;
        }
        let at = first + done;
        if let (Some(every), Some(dir)) = (args.dump_every, &args.out) {
            if at.is_multiple_of(every) && done < target {
                if let Err(e) = write_dump(&dir.join("dumps"), carry(done, dt_next, unconverged, &watch, &ledger)) {
                    say(&e);
                    return EXIT_ARGS;
                }
            }
        }
        // The checkpoint: after the `until` is read, so that no pair holds a state that already met it.
        if let (Some(p), Some(dir)) = (progress.as_mut(), &args.out) {
            if let Some(every) = p.checkpoint_every {
                if at / every.max(1) > saved {
                    saved = at / every.max(1);
                    let folder = dir.join(CHECKPOINT_DIR);
                    let written = std::fs::create_dir_all(&folder)
                        .map_err(|e| format!("SR-ERROR bad arguments: --out {}: {e}", folder.display()))
                        .and_then(|()| write_dump(&folder, carry(done, dt_next, unconverged, &watch, &ledger)));
                    if let Err(e) = written {
                        say(&e);
                        return EXIT_ARGS;
                    }
                    prune_checkpoints(&folder, at);
                    (p.saved)(at, dt.sim_time, watch.heaviest_stage().unwrap_or("none"));
                }
            }
        }
    }
    let wall_s = wall_before + started.elapsed().as_secs_f64();
    // The last step's state is scanned too (M0-D59): P9 scans only every 64th step, and a run ending between two scans
    // would exit 0 on a non-finite world. The scan finds what P9's would, by cell then channel, and names the last step.
    if done > 0 {
        let planes = state.readback(&gpu.device, &gpu.queue);
        if let Some((cell, channel)) = first_non_finite(&planes) {
            let w = world.width as usize;
            say(&non_finite_line(first + done - 1, (cell % w) as u64, (cell / w) as u64, CHANNELS[channel]));
            return EXIT_NON_FINITE;
        }
    }
    // The final dump, always (§3.3).
    if let Some(dir) = &args.out {
        if let Err(e) = write_dump(&dir.join("dumps"), carry(done, dt_next, unconverged, &watch, &ledger)) {
            say(&e);
            return EXIT_ARGS;
        }
    }

    let (condition, met) = match (args.until, args.steps) {
        (Some(u), _) => (u.condition(), until_met),
        (None, wanted) => {
            let wanted = wanted.expect("--steps or --until is given");
            (format!("steps:{wanted}"), done == wanted)
        }
    };
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
            "ledger": ledger.json(),
            "events": watch.events().iter().map(|e| e.json()).collect::<Vec<_>>(),
            "objects": watch.objects_json(),
            "max_gas_speed": watch.max_gas_speed(),
            "until": {"condition": condition, "met": met},
        });
        if let Some(timing) = cost.summary() {
            summary["timing"] = timing;
        }
        let text = serde_json::to_string_pretty(&summary).expect("a json! value serialises");
        let written = std::fs::write(dir.join("summary.json"), text + "\n")
            .and_then(|()| std::fs::write(dir.join("events.jsonl"), watch.events_jsonl()));
        if let Err(e) = written {
            say(&format!("SR-ERROR bad arguments: --out {}: {e}", dir.display()));
            return EXIT_ARGS;
        }
    }
    // `none`: a run on --steps has no condition to meet; with --until, met or unmet.
    let verdict = match (args.until, met) {
        (None, true) => "none",
        (_, true) => "met",
        (_, false) => "unmet",
    };
    say(&format!("SR-HEADLESS DONE steps={done} sim_time={sim_time:.6} until={verdict}"));
    if met {
        EXIT_DONE
    } else {
        EXIT_UNMET
    }
}
