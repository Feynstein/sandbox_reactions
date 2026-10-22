//! The observers in a headless run (m0_contrat.md §1.9, §2.12.2, §3.3; M0-T60).
//!
//! A run's frames — each chunk of steps `headless` summarises, [`crate::step::Step::summarise`] — go through
//! [`Watch::update`]: the block summary widened to f64 as a [`Report`], the objects of [`Tracker`], their stages and events
//! of [`StageTracker`], and the one event the engine adds, `ending` (an object holding a remnant stage for
//! [`ending_hold`] — max(10 t.u., 20 τ_dyn), §0.5 [M0-D78]). What it keeps becomes summary.json's `events`, `objects` and `until` and DIR/events.jsonl, and
//! [`Until`] is the condition `--until` stops on. Observers watch, never drive (I2): nothing here reaches into a step.
//!
//! Readings the contract leaves open, declared here:
//! - A frame is a chunk of steps (the ledger's own): 256 on a run of `--steps`, 16 with `--until` (`headless`'s FRAME,
//!   untuned); the run's first step is a frame of its own. Stages hold for 3 frames (§1.9.3), so a stage shows no sooner
//!   than the 3rd frame it matches in, and a condition on a stage or an event is read on a frame's summary: the run stops
//!   at that frame's end, up to one frame past the step it first held. `sim_time:<t>` is the exception — `headless` sizes its chunks so that it stops at the first step past `t`.
//! - An event's `step` and a stage's `from_step` are step indices from the scene, as the dumps' are (`s_<step>.bin`): a run
//!   resumed from a dump counts on from the dump's, while summary.json's `steps` is the steps this run ran.
//! - An object's `stage` is the displayed stage (after the hold), `null` until one has held; `stages` lists each stage
//!   displayed, from the frame that showed it. The objects listed are those of the last frame.
//! - The `ending` event's frame is the one at which the displayed remnant stage has been shown for max(10 t.u., 20 τ_dyn) of
//!   frames' sim time (the first frame showing it to the frame in question), τ_dyn the object's at the frame in question,
//!   the remnant being unchanged throughout (§0.5 [M0-D78]); its detail's `hold_t` is the hold it met. The ignition event's
//!   detail names its rule, `by` `light` or `fuel` (§0.5 [M0-D78]).

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{json, Value};
use sr_physics::observe::stages::{ending_hold, stage_order};
use sr_physics::observe::tracker::{exact, read_exact, read_list, read_whole, SUN_MASS_NOMINAL};
use sr_physics::observe::{BlockData, EventKind, Object, Report, SinkData, Stage, StageTracker, Tracker, TrackerConfig, GROUP_N};
use sr_physics::registry::Physics;

use crate::ledger::At;
use crate::summary::BlockSummary;

/// §1.10.2's Sun-like anchor: T_eff is read at the first frame whose object has central hydrogen at or under this.
pub const T_EFF_X_H: f64 = 0.35;
/// Hydrogen's slot in an object's central composition (§1.5's first species).
const X_H: usize = 0;
/// Iron's and the neutrons' slots (§1.5's order: H, He, C, O, Ne, Mg, Si, S, Fe, n).
const X_FE: usize = 8;
const X_N: usize = 9;

/// The heaviest object's numbers on a `calibrate` progress line (M0-D80; §1.9.2's quantities, sandbox units): M, R, τ_dyn, Σ_c,
/// T_c, d_c, Y_e,c, X_Fe,c, X_n,c, v_r and v_r/√(GM/R), M_unb, L, L_nuc, L_N (≤ 0, the neutronization's absorbed power), L_ν,
/// the sinks and their mass, the raw stage (this frame's result) and the shown one.
#[derive(Clone, Debug, PartialEq)]
pub struct Star {
    pub mass: f64,
    pub radius: f64,
    pub tau_dyn: f64,
    pub sigma_c: f64,
    pub t_c: f64,
    pub d_c: f64,
    pub ye_c: f64,
    pub x_fe_c: f64,
    pub x_n_c: f64,
    pub v_r: f64,
    pub v_r_dyn: f64,
    pub unbound: f64,
    pub luminosity: f64,
    pub l_nuc: f64,
    pub l_n: f64,
    pub l_nu: f64,
    pub sinks: usize,
    pub sink_mass: f64,
    pub raw: Option<&'static str>,
    pub shown: Option<&'static str>,
}

/// `--until`'s condition (§3.3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Until {
    /// The run's sim time has reached `t` (read at the step, not the frame).
    SimTime(f64),
    /// An event of this kind fired (never `ending`: that is its own condition).
    Event(EventKind),
    /// An object displays this stage.
    Stage(Stage),
    /// Any object holds a remnant stage for [`ending_hold`] of its τ_dyn.
    Ending,
    /// The first of the `ignition` event and the `ending` event: `calibrate`'s m_ign_sb run (§2.11 item 3), which stops at
    /// either side of the threshold. Built in code by `calibrate`; `--until` does not parse it.
    IgnitionOrEnding,
    /// The first frame where the `core_collapse` event fires or any object shows `neutron_star` or `black_hole`, else the
    /// `ending` event: `calibrate`'s m_up_sb runs (§2.11 item 4, M0-D75), whose side is known from the collapse on. Built in
    /// code by `calibrate`; `--until` does not parse it.
    CollapseOrEnding,
}

impl Until {
    /// `sim_time:<t>` · `event:<ignition|core_collapse|supernova|sink_formed>` · `stage:<stage id>` · `ending`.
    pub fn parse(text: &str) -> Result<Until, String> {
        let bad = |why: String| format!("--until {text:?}: {why}");
        let kinds = [EventKind::Ignition, EventKind::CoreCollapse, EventKind::Supernova, EventKind::SinkFormed];
        match text.split_once(':') {
            None if text == "ending" => Ok(Until::Ending),
            Some(("sim_time", t)) => match t.parse::<f64>() {
                Ok(t) if t.is_finite() && t >= 0.0 => Ok(Until::SimTime(t)),
                _ => Err(bad(format!("sim_time takes a time of 0 or more, got {t:?}"))),
            },
            Some(("event", k)) => kinds
                .into_iter()
                .find(|kind| kind.id() == k)
                .map(Until::Event)
                .ok_or_else(|| bad(format!("an event is one of {}, got {k:?}", kinds.map(EventKind::id).join("|")))),
            Some(("stage", s)) => stage_order()
                .into_iter()
                .find(|stage| stage.id() == s)
                .map(Until::Stage)
                .ok_or_else(|| bad(format!("a stage is one of {}, got {s:?}", stage_order().map(Stage::id).join("|")))),
            _ => Err(bad("expected sim_time:<t>, event:<kind>, stage:<id> or ending".to_string())),
        }
    }

    /// The condition as summary.json's `until.condition` writes it, the form [`Until::parse`] reads.
    pub fn condition(&self) -> String {
        match self {
            Until::SimTime(t) => format!("sim_time:{t}"),
            Until::Event(kind) => format!("event:{}", kind.id()),
            Until::Stage(stage) => format!("stage:{}", stage.id()),
            Until::Ending => "ending".to_string(),
            Until::IgnitionOrEnding => "ignition|ending".to_string(),
            Until::CollapseOrEnding => "collapse|ending".to_string(),
        }
    }
}

/// The event details' keys that hold a word, not a number: the ending's stage id and the ignition's rule (§0.5 [M0-D78]).
const DETAIL_WORDS: [&str; 2] = ["stage", "by"];

/// A stage is a remnant when it is an end of a life (§1.9.3's "remnant" rows).
pub fn is_remnant(stage: Stage) -> bool {
    matches!(stage, Stage::BlackHole | Stage::NeutronStar | Stage::WhiteDwarf | Stage::FailedStar)
}

/// An event as summary.json and events.jsonl hold it (§2.12.2): the frame's step count and sim time.
#[derive(Clone, Debug, PartialEq)]
pub struct RunEvent {
    pub step: u64,
    pub sim_time: f64,
    /// `ignition`, `core_collapse`, `supernova`, `sink_formed` or `ending`.
    pub kind: &'static str,
    pub object: u32,
    pub detail: Value,
}

impl RunEvent {
    pub fn json(&self) -> Value {
        json!({"step": self.step, "sim_time": self.sim_time, "kind": self.kind, "object": self.object, "detail": self.detail})
    }
}

/// A stage an object displayed, from the frame that showed it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StageEntry {
    pub stage: Stage,
    pub from_step: u64,
    pub sim_time: f64,
}

/// `summary` as the observers take it (§1.9.1): every f32 sum widened exactly, the frame's sim time `sim_dt`.
pub fn report(summary: &BlockSummary, world_width: u32, sim_dt: f64) -> Report {
    let wide = |v: [f32; 10]| v.map(f64::from);
    Report {
        sim_dt,
        world_width,
        origin: summary.origin,
        columns: summary.columns,
        blocks: summary
            .blocks
            .iter()
            .map(|b| BlockData {
                mass: f64::from(b.mass),
                mx: f64::from(b.mx),
                my: f64::from(b.my),
                momentum: b.momentum.map(f64::from),
                kinetic: f64::from(b.kinetic),
                thermal: f64::from(b.thermal),
                cold: f64::from(b.cold),
                pressure: f64::from(b.pressure),
                species: wide(b.species),
                phi_m: f64::from(b.phi_m),
                max_cell: b.max_cell,
                max_sigma: f64::from(b.max_sigma),
                max_t: f64::from(b.max_t),
                max_x: wide(b.max_x),
                cold_ratio: f64::from(b.cold_ratio),
                radiated: f64::from(b.radiated),
                nuclear: b.nuclear.map(f64::from),
                neutrino_lost: f64::from(b.neutrino_lost),
                neutrino_deposited: f64::from(b.neutrino_deposited),
                unbound: f64::from(b.unbound),
                sinks: b.sinks,
            })
            .collect(),
        sinks: summary.sinks.iter().map(|s| SinkData { x: s.x, y: s.y, vx: s.vx, vy: s.vy, m: s.m }).collect(),
    }
}

/// A remnant stage on display and the sim time of the first frame that showed it.
#[derive(Clone, Copy)]
struct Remnant {
    stage: Stage,
    since: f64,
}

/// The observers of one run.
pub struct Watch {
    world_width: u32,
    tracker: Tracker,
    stages: StageTracker,
    /// The sim time of the last frame (the run's start before the first): a frame's sim_dt is the difference.
    last_sim: f64,
    events: Vec<RunEvent>,
    /// Each object's displayed stages, by id.
    lives: BTreeMap<u32, Vec<StageEntry>>,
    remnants: BTreeMap<u32, Remnant>,
    ended: BTreeSet<u32>,
    /// Each object's L and R at the first frame its central hydrogen fell to [`T_EFF_X_H`] (§1.10.2's T_eff anchor).
    marks: BTreeMap<u32, XhMark>,
    /// The largest block rms gas speed any frame showed (§2.11 item 6): √(2 K / m) over the box's blocks, K being the
    /// block's kinetic energy and m its mass — a bound from below on the fastest cell, which the summary does not hold.
    max_gas_speed: f64,
}

/// An object's state at [`T_EFF_X_H`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct XhMark {
    pub sim_time: f64,
    pub luminosity: f64,
    pub radius: f64,
}

impl Watch {
    /// Observers for a run in a world `world_width` cells wide whose sim time starts at `start_sim_time`. The Sun's mass is
    /// the nominal Sun-like cloud's until calibration gives `preset_mass_sb.sun`.
    pub fn new(physics: &Physics, world_width: u32, start_sim_time: f64) -> Watch {
        Watch {
            world_width,
            tracker: Tracker::new(TrackerConfig::from_physics(physics, SUN_MASS_NOMINAL)),
            stages: StageTracker::new(),
            last_sim: start_sim_time,
            events: Vec::new(),
            lives: BTreeMap::new(),
            remnants: BTreeMap::new(),
            ended: BTreeSet::new(),
            marks: BTreeMap::new(),
            max_gas_speed: 0.0,
        }
    }

    /// A frame's summary in, taken `at` its end: the objects, stages and events it makes. Returns the frame's new events.
    pub fn update(&mut self, summary: &BlockSummary, at: At) -> Vec<RunEvent> {
        let sim_dt = at.sim_time - self.last_sim;
        self.last_sim = at.sim_time;
        let report = report(summary, self.world_width, sim_dt);
        let rms = report.blocks.iter().filter(|b| b.mass > 0.0).map(|b| (2.0 * b.kinetic / b.mass).sqrt());
        self.max_gas_speed = rms.fold(self.max_gas_speed, f64::max);
        let objects = self.tracker.update(&report).to_vec();
        for o in objects.iter().filter(|o| o.x_c[X_H] <= T_EFF_X_H) {
            self.marks.entry(o.id).or_insert(XhMark { sim_time: at.sim_time, luminosity: o.luminosity, radius: o.radius });
        }
        let mut fresh: Vec<RunEvent> = self
            .stages
            .update(&objects, at.sim_time)
            .into_iter()
            .map(|e| {
                let mass = objects.iter().find(|o| o.id == e.object).map(|o| o.mass);
                let mut detail = json!({"mass_sb": mass});
                if let Some(by) = e.by {
                    detail["by"] = json!(by.id());
                }
                RunEvent { step: at.step, sim_time: e.time, kind: e.kind.id(), object: e.object, detail }
            })
            .collect();
        self.lives.retain(|id, _| objects.iter().any(|o| o.id == *id));
        self.remnants.retain(|id, _| objects.iter().any(|o| o.id == *id));
        for o in &objects {
            let shown = self.stages.stage(o.id).and_then(|s| s.shown);
            let life = self.lives.entry(o.id).or_default();
            if let Some(stage) = shown {
                if life.last().is_none_or(|l| l.stage != stage) {
                    life.push(StageEntry { stage, from_step: at.step, sim_time: at.sim_time });
                }
            }
            match shown.filter(|s| is_remnant(*s)) {
                Some(stage) => {
                    let held = self.remnants.entry(o.id).or_insert(Remnant { stage, since: at.sim_time });
                    if held.stage != stage {
                        *held = Remnant { stage, since: at.sim_time };
                    }
                    let hold = ending_hold(o.tau_dyn);
                    if at.sim_time - held.since >= hold && self.ended.insert(o.id) {
                        let detail = json!({"stage": stage.id(), "held_t": at.sim_time - held.since, "hold_t": hold, "mass_sb": o.mass});
                        fresh.push(RunEvent { step: at.step, sim_time: at.sim_time, kind: "ending", object: o.id, detail });
                    }
                }
                None => {
                    self.remnants.remove(&o.id);
                }
            }
        }
        self.events.extend(fresh.iter().cloned());
        fresh
    }

    /// Whether `until` holds after the last frame, `fresh` being that frame's events. `SimTime` is the run's to read.
    pub fn holds(&self, until: Until, fresh: &[RunEvent]) -> bool {
        match until {
            Until::SimTime(t) => self.last_sim >= t,
            Until::Event(kind) => fresh.iter().any(|e| e.kind == kind.id()),
            Until::Stage(stage) => self.shows(stage),
            Until::Ending => fresh.iter().any(|e| e.kind == "ending"),
            Until::IgnitionOrEnding => fresh.iter().any(|e| e.kind == "ending" || e.kind == "ignition"),
            Until::CollapseOrEnding => {
                fresh.iter().any(|e| e.kind == "ending" || e.kind == "core_collapse") || self.shows(Stage::NeutronStar) || self.shows(Stage::BlackHole)
            }
        }
    }

    /// Whether any object displays `stage` now.
    fn shows(&self, stage: Stage) -> bool {
        self.tracker.objects().iter().any(|o| self.stages.stage(o.id).and_then(|s| s.shown) == Some(stage))
    }

    pub fn events(&self) -> &[RunEvent] {
        &self.events
    }

    /// summary.json's `max_gas_speed`: the fastest block rms gas speed of the run.
    pub fn max_gas_speed(&self) -> f64 {
        self.max_gas_speed
    }

    /// The stage the run's heaviest object shows now (`None` before one has held) — a progress line's `stage=`.
    pub fn heaviest_stage(&self) -> Option<&'static str> {
        let star = self.tracker.objects().iter().max_by(|a, b| a.mass.total_cmp(&b.mass))?;
        self.stages.stage(star.id).and_then(|s| s.shown).map(Stage::id)
    }

    /// The heaviest object's numbers now, for `calibrate`'s progress line (M0-D80); `None` while there is no object.
    pub fn heaviest_star(&self) -> Option<Star> {
        let o = self.tracker.objects().iter().max_by(|a, b| a.mass.total_cmp(&b.mass))?;
        let state = self.stages.stage(o.id);
        let v_dyn = (sr_physics::units::G * o.mass / o.radius).sqrt();
        Some(Star {
            mass: o.mass,
            radius: o.radius,
            tau_dyn: o.tau_dyn,
            sigma_c: o.sigma_c,
            t_c: o.t_c,
            d_c: o.d_c,
            ye_c: sr_physics::registry::electron_fraction(&o.x_c),
            x_fe_c: o.x_c[X_FE],
            x_n_c: o.x_c[X_N],
            v_r: o.v_r,
            v_r_dyn: if v_dyn > 0.0 { o.v_r / v_dyn } else { 0.0 },
            unbound: o.unbound,
            luminosity: o.luminosity,
            l_nuc: o.l_nuc,
            l_n: o.l_group[GROUP_N],
            l_nu: o.l_nu,
            sinks: o.sinks,
            sink_mass: o.sink_mass,
            raw: state.map(|s| s.raw.id()),
            shown: state.and_then(|s| s.shown).map(Stage::id),
        })
    }

    /// The objects the last frame found.
    pub fn object_count(&self) -> usize {
        self.tracker.objects().len()
    }

    /// The heaviest object's raw stage — this frame's result, before the hold (§1.9.3) — `None` while there is no object.
    pub fn heaviest_raw_stage(&self) -> Option<&'static str> {
        let star = self.tracker.objects().iter().max_by(|a, b| a.mass.total_cmp(&b.mass))?;
        self.stages.stage(star.id).map(|s| s.raw.id())
    }

    /// DIR/events.jsonl: one JSON object a line, none for a run without events.
    pub fn events_jsonl(&self) -> String {
        self.events.iter().map(|e| e.json().to_string() + "\n").collect()
    }

    /// Everything the next frame and summary.json read, for the observers' snapshot beside a dump (§0.5 [M0-D76]): the
    /// tracker, the stages, the last frame's sim time, the events, each object's stage trail, the remnant holds, the objects
    /// ended, the at_x_h marks and max_gas_speed — every f64 [`exact`].
    pub fn snapshot(&self) -> Value {
        let by_id = |entries: Vec<(u32, Value)>| entries.into_iter().map(|(id, v)| json!([id, v])).collect::<Vec<_>>();
        let events: Vec<Value> = self
            .events
            .iter()
            .map(|e| {
                // The detail's numbers as exact strings; its strings, a stage id and an ignition's rule, stay ones.
                let detail: serde_json::Map<String, Value> =
                    e.detail.as_object().into_iter().flatten().map(|(k, v)| (k.clone(), v.as_f64().map_or(v.clone(), exact))).collect();
                json!({"step": e.step, "sim_time": exact(e.sim_time), "kind": e.kind, "object": e.object, "detail": detail})
            })
            .collect();
        let lives = self.lives.iter().map(|(&id, life)| {
            (id, life.iter().map(|l| json!([l.stage.id(), l.from_step, exact(l.sim_time)])).collect::<Value>())
        });
        let remnants = self.remnants.iter().map(|(&id, r)| (id, json!([r.stage.id(), exact(r.since)])));
        let marks = self.marks.iter().map(|(&id, m)| (id, json!([exact(m.sim_time), exact(m.luminosity), exact(m.radius)])));
        json!({
            "tracker": self.tracker.snapshot(),
            "stages": self.stages.snapshot(),
            "last_sim": exact(self.last_sim),
            "events": events,
            "lives": by_id(lives.collect()),
            "remnants": by_id(remnants.collect()),
            "ended": self.ended,
            "marks": by_id(marks.collect()),
            "max_gas_speed": exact(self.max_gas_speed),
        })
    }

    /// The observers [`Watch::snapshot`] wrote, for a run in a world `world_width` cells wide under `physics`.
    pub fn restore(physics: &Physics, world_width: u32, v: &Value) -> Result<Watch, String> {
        let config = TrackerConfig::from_physics(physics, SUN_MASS_NOMINAL);
        let id = |v: &Value| v.as_u64().and_then(|n| u32::try_from(n).ok()).ok_or_else(|| format!("not an object id: {v}"));
        // `[id, value]` pairs into a map, each value read by `read`.
        fn by_id<T>(v: &Value, key: &str, read: impl Fn(&Value) -> Result<T, String>) -> Result<BTreeMap<u32, T>, String> {
            read_list(v, key)?
                .iter()
                .map(|pair| {
                    let id = pair[0].as_u64().and_then(|n| u32::try_from(n).ok()).ok_or_else(|| format!("{key}: not [id, …]: {pair}"))?;
                    Ok((id, read(&pair[1])?))
                })
                .collect()
        }
        let kinds = ["ignition", "core_collapse", "supernova", "sink_formed", "ending"];
        let events = read_list(v, "events")?
            .iter()
            .map(|e| {
                let kind = kinds.into_iter().find(|k| Some(*k) == e["kind"].as_str()).ok_or_else(|| format!("events.kind: {}", e["kind"]))?;
                let detail = e["detail"].as_object().ok_or_else(|| format!("events.detail: not an object: {}", e["detail"]))?;
                let detail = detail
                    .iter()
                    .map(|(k, x)| Ok((k.clone(), if DETAIL_WORDS.contains(&k.as_str()) || x.is_null() { x.clone() } else { json!(read_exact(x, "events.detail")?) })))
                    .collect::<Result<serde_json::Map<String, Value>, String>>()?;
                let at = read_exact(&e["sim_time"], "events.sim_time")?;
                Ok(RunEvent { step: read_whole(e, "step")?, sim_time: at, kind, object: id(&e["object"])?, detail: Value::Object(detail) })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let lives = by_id(v, "lives", |life| {
            life.as_array()
                .ok_or_else(|| format!("lives: not a list: {life}"))?
                .iter()
                .map(|l| {
                    let from_step = l[1].as_u64().ok_or_else(|| format!("lives: not [stage, step, time]: {l}"))?;
                    Ok(StageEntry { stage: Stage::from_id(&l[0])?, from_step, sim_time: read_exact(&l[2], "lives.sim_time")? })
                })
                .collect()
        })?;
        let remnants = by_id(v, "remnants", |r| Ok(Remnant { stage: Stage::from_id(&r[0])?, since: read_exact(&r[1], "remnants.since")? }))?;
        let marks = by_id(v, "marks", |m| {
            let n = |k: usize| read_exact(&m[k], "marks");
            Ok(XhMark { sim_time: n(0)?, luminosity: n(1)?, radius: n(2)? })
        })?;
        Ok(Watch {
            world_width,
            tracker: Tracker::restore(config, &v["tracker"])?,
            stages: StageTracker::restore(&v["stages"])?,
            last_sim: read_exact(&v["last_sim"], "last_sim")?,
            events,
            lives,
            remnants,
            ended: read_list(v, "ended")?.iter().map(id).collect::<Result<_, String>>()?,
            marks,
            max_gas_speed: read_exact(&v["max_gas_speed"], "max_gas_speed")?,
        })
    }

    /// summary.json's `objects` (§2.12.2): the last frame's, with the stages each displayed. `mass_suns`,
    /// `predicted_ending`, `ending` and `age_years` come with M0-T69, M0-T72 and M0-T73.
    pub fn objects_json(&self) -> Vec<Value> {
        self.tracker
            .objects()
            .iter()
            .map(|o: &Object| {
                let shown = self.stages.stage(o.id).and_then(|s| s.shown);
                let stages: Vec<Value> = self
                    .lives
                    .get(&o.id)
                    .into_iter()
                    .flatten()
                    .map(|l| json!({"stage": l.stage.id(), "from_step": l.from_step, "sim_time": l.sim_time}))
                    .collect();
                let mut object = json!({"id": o.id, "stage": shown.map(Stage::id), "mass_sb": o.mass, "stages": stages});
                if let Some(m) = self.marks.get(&o.id) {
                    object["at_x_h"] = json!({"x_h_c": T_EFF_X_H, "sim_time": m.sim_time, "luminosity": m.luminosity, "radius": m.radius});
                }
                object
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn until_parses_the_four_forms_and_refuses_the_rest() {
        assert_eq!(Until::parse("sim_time:5"), Ok(Until::SimTime(5.0)));
        assert_eq!(Until::parse("event:supernova"), Ok(Until::Event(EventKind::Supernova)));
        assert_eq!(Until::parse("stage:collapsing"), Ok(Until::Stage(Stage::Collapsing)));
        assert_eq!(Until::parse("ending"), Ok(Until::Ending));
        for bad in ["sim_time:-1", "sim_time:x", "sim_time:inf", "event:ending", "event:", "stage:nova", "stage", "later", ""] {
            assert!(Until::parse(bad).is_err(), "{bad:?} should be refused");
        }
    }
}
