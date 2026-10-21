//! Stages and events (m0_contrat.md §1.9.3, §1.9.4; M0-T59).
//!
//! [`classify`] reads one object and its history and walks §1.9.3's rows in the contract's order, the first match
//! winning. [`StageTracker::update`] does it for every object of a frame: it keeps each object's history (the `ignited`
//! flag, the last time its row said core collapse), applies the display hold, and returns the events (§1.9.4), each
//! once per object. The classifier never reads the rung, the frame or the engine — the object and its history only
//! (I2; G-WATCH keeps this module out of the step).

use std::collections::BTreeMap;

use serde_json::{json, Value};

use super::tracker::{exact, read_exact, read_whole, Object};

/// The display hold: a stage other than core collapse and supernova shows after this many identical results (§1.9.3).
const HOLD: u32 = 3;
/// A supernova row is open this long (t.u.) after the object last passed core collapse.
const SUPERNOVA_WINDOW: f64 = 20.0;
/// L_H between these shares of L ignites an object, and holds the main sequence: a star shines what it burns (§1.9.3,
/// §1.9.4 as §0.5 [M0-D53] reads them; the lead at M0-V18, «Burning matches light (Recommended)»).
const HYDROGEN_SHARE: f64 = 0.5;
const HYDROGEN_SHARE_MAX: f64 = 2.0;
/// An abundance at the centre under this is spent; a fuel at or above it still feeds its stage (§1.9.3).
const SPENT: f64 = 1e-3;
/// A group is a burning stage's when it is the largest and at least this share of L_nuc.
const BURN_SHARE: f64 = 0.1;

/// Species indices (§1.5) and group indices ([`super::GROUPS`]) the rows read.
const X_H: usize = 0;
const X_HE: usize = 1;
const X_FE: usize = 8;
const X_N: usize = 9;
const G_H: usize = 0;
const G_HE: usize = 1;
const G_C: usize = 2;
const G_NE: usize = 3;
const G_O: usize = 4;
const G_SI: usize = 5;
const G_N: usize = super::GROUP_N;

/// A stage of an object's life (§1.9.3), by the table's ids.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stage {
    BlackHole,
    NeutronStar,
    Supernova,
    CoreCollapse,
    WhiteDwarf,
    FailedStar,
    PlanetaryNebula,
    IronCore,
    SiBurn,
    OBurn,
    NeBurn,
    CBurn,
    HeCore,
    HeShell,
    HShell,
    MainSequence,
    Collapsing,
    Protostar,
    Cloud,
}

impl Stage {
    /// §1.9.3's id.
    pub fn id(self) -> &'static str {
        match self {
            Stage::BlackHole => "black_hole",
            Stage::NeutronStar => "neutron_star",
            Stage::Supernova => "supernova",
            Stage::CoreCollapse => "core_collapse",
            Stage::WhiteDwarf => "white_dwarf",
            Stage::FailedStar => "failed_star",
            Stage::PlanetaryNebula => "planetary_nebula",
            Stage::IronCore => "iron_core",
            Stage::SiBurn => "si_burn",
            Stage::OBurn => "o_burn",
            Stage::NeBurn => "ne_burn",
            Stage::CBurn => "c_burn",
            Stage::HeCore => "he_core",
            Stage::HeShell => "he_shell",
            Stage::HShell => "h_shell",
            Stage::MainSequence => "main_sequence",
            Stage::Collapsing => "collapsing",
            Stage::Protostar => "protostar",
            Stage::Cloud => "cloud",
        }
    }

    /// Core collapse and supernova show at once; every other stage waits out the hold (§1.9.3).
    pub fn is_immediate(self) -> bool {
        matches!(self, Stage::CoreCollapse | Stage::Supernova)
    }
}

/// What an object remembers between frames: the `ignited` flag (§1.9.4) and the last time its row said core collapse.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct History {
    /// First turned on at 0.5 L ≤ L_H ≤ 2 L, never off again.
    pub ignited: bool,
    /// The sim time (t.u.) of the last frame whose result was core collapse.
    pub last_collapse: Option<f64>,
}

/// An object's quantities as the rows read them.
struct Ctx<'a> {
    o: &'a Object,
    h: &'a History,
    now: f64,
    /// √(GM/R) = R / τ_dyn.
    v_dyn: f64,
}

/// L_H between half and twice L, with some light to share: a flash inside a dark cloud is not a star.
fn hydrogen_leads(o: &Object) -> bool {
    let l_h = o.l_group[G_H];
    o.luminosity > 0.0 && l_h >= HYDROGEN_SHARE * o.luminosity && l_h <= HYDROGEN_SHARE_MAX * o.luminosity
}

/// Group `g` is the largest of the burning groups and at least a tenth of L_nuc, its fuel at the centre still there.
fn burns(c: &Ctx, g: usize, fuel: usize) -> bool {
    let o = c.o;
    let l = o.l_group[g];
    let largest = (0..G_N).all(|k| o.l_group[k] <= l);
    l > 0.0 && largest && l >= BURN_SHARE * o.l_nuc && o.x_c[fuel] >= SPENT
}

fn carbon_burns(o: &Object) -> bool {
    let l = o.l_group[G_C];
    l > 0.0 && l >= BURN_SHARE * o.l_nuc
}

/// The species whose centre abundance is the fuel of each burning group.
const FUEL_SI: usize = 6;
const FUEL_O: usize = 3;
const FUEL_NE: usize = 4;
const FUEL_C: usize = 2;

type Row = (Stage, fn(&Ctx) -> bool);

/// §1.9.3's rows, in the table's order: the first that matches wins.
const ROWS: [Row; 19] = [
    (Stage::BlackHole, |c| c.o.sinks > 0),
    (Stage::NeutronStar, |c| c.o.x_c[X_N] >= 0.5 && c.o.v_r.abs() <= 0.05 * c.v_dyn && c.o.sinks == 0),
    (Stage::Supernova, |c| {
        c.h.last_collapse.is_some_and(|t| c.now - t <= SUPERNOVA_WINDOW) && c.o.unbound >= 0.1 * c.o.mass
    }),
    (Stage::CoreCollapse, |c| c.o.l_group[G_N] < 0.0),
    (Stage::WhiteDwarf, |c| {
        c.h.ignited && c.o.d_c >= 0.5 && c.o.l_nuc < 0.01 * c.o.luminosity && c.o.unbound < 0.05 * c.o.mass && c.o.x_c[X_N] < 0.5
    }),
    (Stage::FailedStar, |c| !c.h.ignited && c.o.d_c >= 0.5 && c.o.l_group[G_H] < 0.01 * c.o.luminosity),
    (Stage::PlanetaryNebula, |c| {
        c.h.ignited && c.o.d_c >= 0.5 && c.o.l_nuc < 0.1 * c.o.luminosity && c.o.unbound >= 0.05 * c.o.mass
    }),
    (Stage::IronCore, |c| c.o.x_c[X_FE] >= 0.5 && c.o.l_nuc < 0.1 * c.o.luminosity),
    (Stage::SiBurn, |c| burns(c, G_SI, FUEL_SI)),
    (Stage::OBurn, |c| burns(c, G_O, FUEL_O)),
    (Stage::NeBurn, |c| burns(c, G_NE, FUEL_NE)),
    (Stage::CBurn, |c| burns(c, G_C, FUEL_C)),
    (Stage::HeCore, |c| c.o.l_group[G_HE] > 0.0 && c.o.l_group[G_HE] >= 0.1 * c.o.luminosity && c.o.x_c[X_HE] >= SPENT),
    (Stage::HeShell, |c| c.h.ignited && c.o.x_c[X_HE] < SPENT && c.o.x_c[X_H] < SPENT && !carbon_burns(c.o)),
    (Stage::HShell, |c| c.h.ignited && c.o.x_c[X_H] < SPENT && c.o.l_group[G_HE] < 0.1 * c.o.luminosity),
    (Stage::MainSequence, |c| hydrogen_leads(c.o) && c.o.x_c[X_H] >= SPENT),
    (Stage::Collapsing, |c| !c.h.ignited && c.o.v_r <= -0.1 * c.v_dyn),
    (Stage::Protostar, |c| {
        !c.h.ignited && c.o.v_r.abs() <= 0.05 * c.v_dyn && c.o.potential != 0.0 && 2.0 * c.o.pressure / c.o.potential.abs() >= 0.8
    }),
    (Stage::Cloud, |_| true),
];

/// The stages in the table's order, for the readers that list them.
pub fn stage_order() -> [Stage; 19] {
    ROWS.map(|(s, _)| s)
}

/// The object's stage this frame by §1.9.3's rows, given its history (`h.ignited` already read at this frame) and the sim time.
pub fn classify(o: &Object, h: &History, now: f64) -> Stage {
    let v_dyn = if o.tau_dyn > 0.0 { o.radius / o.tau_dyn } else { 0.0 };
    let c = Ctx { o, h, now, v_dyn };
    ROWS.iter().find(|(_, test)| test(&c)).map(|(s, _)| *s).unwrap_or(Stage::Cloud)
}

/// Whether the object's L_H is between half and twice its L: the frame the `ignited` flag turns on (§1.9.4, §0.5 [M0-D53]).
pub fn ignites(o: &Object) -> bool {
    hydrogen_leads(o)
}

/// What happened to an object (§1.9.4), once each.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventKind {
    Ignition,
    CoreCollapse,
    Supernova,
    /// Logged only: the collapse latch covers its slow-down (§1.9.4).
    SinkFormed,
}

impl EventKind {
    pub fn id(self) -> &'static str {
        match self {
            EventKind::Ignition => "ignition",
            EventKind::CoreCollapse => "core_collapse",
            EventKind::Supernova => "supernova",
            EventKind::SinkFormed => "sink_formed",
        }
    }
}

/// An event: which object, which kind, the sim time (t.u.) of the frame that saw it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Event {
    pub object: u32,
    pub kind: EventKind,
    pub time: f64,
}

/// One object's stage as the readout wants it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ObjectStage {
    /// This frame's result, before the hold.
    pub raw: Stage,
    /// The displayed stage: none until a stage has held 3 frames or core collapse / supernova came; then the latest held.
    pub shown: Option<Stage>,
    pub history: History,
}

struct State {
    history: History,
    raw: Stage,
    streak: u32,
    shown: Option<Stage>,
    fired: [bool; 4],
}

/// The stages and events of a run's objects, from one frame's objects to the next.
#[derive(Default)]
pub struct StageTracker {
    states: BTreeMap<u32, State>,
}

impl StageTracker {
    pub fn new() -> StageTracker {
        StageTracker::default()
    }

    /// A frame's objects (from [`super::Tracker::update`]) at sim time `now`: each object's stage updated, the frame's
    /// events returned in object order, then ignition, core collapse, supernova, sink formed. An object gone from the
    /// frame is forgotten (ids are never reused).
    pub fn update(&mut self, objects: &[Object], now: f64) -> Vec<Event> {
        self.states.retain(|id, _| objects.iter().any(|o| o.id == *id));
        let mut events = Vec::new();
        for o in objects {
            let state = self.states.entry(o.id).or_insert(State {
                history: History::default(),
                raw: Stage::Cloud,
                streak: 0,
                shown: None,
                fired: [false; 4],
            });
            let mut fire = |state: &mut State, kind: EventKind| {
                if !state.fired[kind as usize] {
                    state.fired[kind as usize] = true;
                    events.push(Event { object: o.id, kind, time: now });
                }
            };
            if !state.history.ignited && ignites(o) {
                state.history.ignited = true;
                fire(state, EventKind::Ignition);
            }
            let stage = classify(o, &state.history, now);
            if stage == Stage::CoreCollapse {
                state.history.last_collapse = Some(now);
                fire(state, EventKind::CoreCollapse);
            }
            if stage == Stage::Supernova {
                fire(state, EventKind::Supernova);
            }
            if o.sinks > 0 {
                fire(state, EventKind::SinkFormed);
            }
            state.streak = if stage == state.raw { state.streak + 1 } else { 1 };
            state.raw = stage;
            if stage.is_immediate() || state.streak >= HOLD {
                state.shown = Some(stage);
            }
        }
        events
    }

    /// An object's stage after the last [`update`](StageTracker::update), none for an id not in that frame.
    pub fn stage(&self, id: u32) -> Option<ObjectStage> {
        self.states.get(&id).map(|s| ObjectStage { raw: s.raw, shown: s.shown, history: s.history })
    }

    /// What the next frame reads, for the observers' snapshot (§0.5 [M0-D76]): per object its history (`ignited`, the last
    /// core collapse), the last result and its streak, the shown stage and the events fired.
    pub fn snapshot(&self) -> Value {
        let states: Vec<Value> = self
            .states
            .iter()
            .map(|(id, s)| {
                json!({
                    "id": id,
                    "ignited": s.history.ignited,
                    "last_collapse": s.history.last_collapse.map_or(Value::Null, exact),
                    "raw": s.raw.id(),
                    "streak": s.streak,
                    "shown": s.shown.map(Stage::id),
                    "fired": s.fired,
                })
            })
            .collect();
        Value::Array(states)
    }

    /// The stages [`StageTracker::snapshot`] wrote.
    pub fn restore(v: &Value) -> Result<StageTracker, String> {
        let mut states = BTreeMap::new();
        for s in v.as_array().ok_or_else(|| format!("stages: not a list: {v}"))? {
            let id = u32::try_from(read_whole(s, "id")?).map_err(|_| "stages.id: past u32".to_string())?;
            let ignited = s["ignited"].as_bool().ok_or_else(|| format!("stages.ignited: not true or false: {}", s["ignited"]))?;
            let last_collapse = if s["last_collapse"].is_null() { None } else { Some(read_exact(&s["last_collapse"], "stages.last_collapse")?) };
            let shown = if s["shown"].is_null() { None } else { Some(Stage::from_id(&s["shown"])?) };
            let fired: Vec<bool> = s["fired"].as_array().into_iter().flatten().filter_map(Value::as_bool).collect();
            let fired: [bool; 4] = fired.try_into().map_err(|_| format!("stages.fired: not 4 flags: {}", s["fired"]))?;
            let streak = u32::try_from(read_whole(s, "streak")?).map_err(|_| "stages.streak: past u32".to_string())?;
            states.insert(id, State { history: History { ignited, last_collapse }, raw: Stage::from_id(&s["raw"])?, streak, shown, fired });
        }
        Ok(StageTracker { states })
    }
}

impl Stage {
    /// The stage of §1.9.3's id `v` (a JSON string).
    pub fn from_id(v: &Value) -> Result<Stage, String> {
        stage_order().into_iter().find(|s| Some(s.id()) == v.as_str()).ok_or_else(|| format!("not a stage id: {v}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A cold cloud at rest, unburnt: v = R/τ = 1, 2P/|W| = 0.2 → a cloud. Each test bends what its row reads.
    fn cloud() -> Object {
        Object {
            id: 1,
            mass: 100.0,
            gas_mass: 100.0,
            sink_mass: 0.0,
            sinks: 0,
            centre: [10.0, 10.0],
            radius: 10.0,
            tau_dyn: 10.0,
            luminosity: 10.0,
            luminosity_own: 10.0,
            l_group: [0.0; 7],
            l_nuc: 0.0,
            l_nu: 0.0,
            l_nu_deposited: 0.0,
            sigma_c: 1.0,
            t_c: 1.0,
            x_c: {
                let mut x = [0.0; 10];
                x[X_H] = 1.0;
                x
            },
            d_c: 0.0,
            v_r: 0.0,
            thermal: 1.0,
            kinetic: 0.0,
            pressure: 1.0,
            potential: -10.0,
            unbound: 0.0,
            core_group: None,
            r_core: None,
            envelope_mass: None,
        }
    }

    fn ignited() -> History {
        History { ignited: true, last_collapse: None }
    }

    fn fresh() -> History {
        History::default()
    }

    fn with_x(o: &mut Object, species: usize, x: f64) {
        o.x_c = [0.0; 10];
        o.x_c[species] = x;
    }

    /// An object whose burning is `l` in each group, L = their sum, H and He fuel gone.
    fn burning(l: [f64; 7]) -> Object {
        let mut o = cloud();
        o.l_group = l;
        o.l_nuc = l.iter().filter(|v| **v > 0.0).sum();
        o.luminosity = o.l_nuc.max(1.0);
        o.x_c = [0.0; 10];
        o
    }

    fn stage_of(o: &Object, h: &History) -> Stage {
        classify(o, h, 100.0)
    }

    #[test]
    fn the_rows_are_in_the_contracts_order() {
        let ids: Vec<&str> = stage_order().iter().map(|s| s.id()).collect();
        assert_eq!(
            ids,
            [
                "black_hole", "neutron_star", "supernova", "core_collapse", "white_dwarf", "failed_star", "planetary_nebula", "iron_core",
                "si_burn", "o_burn", "ne_burn", "c_burn", "he_core", "he_shell", "h_shell", "main_sequence", "collapsing", "protostar", "cloud"
            ]
        );
    }

    #[test]
    fn row_black_hole_a_sink_inside() {
        let mut o = cloud();
        o.sinks = 1;
        assert_eq!(stage_of(&o, &fresh()), Stage::BlackHole);
    }

    #[test]
    fn row_neutron_star() {
        let mut o = cloud();
        with_x(&mut o, X_N, 0.5);
        o.v_r = 0.05;
        assert_eq!(stage_of(&o, &fresh()), Stage::NeutronStar);
        o.v_r = 0.06;
        assert_ne!(stage_of(&o, &fresh()), Stage::NeutronStar, "moving too fast");
        o.v_r = 0.0;
        with_x(&mut o, X_N, 0.49);
        assert_ne!(stage_of(&o, &fresh()), Stage::NeutronStar, "not neutron matter enough");
    }

    #[test]
    fn a_sink_makes_the_black_hole_before_the_neutron_star() {
        let mut o = cloud();
        with_x(&mut o, X_N, 1.0);
        o.sinks = 1;
        assert_eq!(stage_of(&o, &fresh()), Stage::BlackHole);
    }

    #[test]
    fn row_supernova_inside_the_window_with_unbound_mass() {
        let mut o = cloud();
        o.unbound = 10.0;
        let h = History { ignited: true, last_collapse: Some(90.0) };
        assert_eq!(classify(&o, &h, 100.0), Stage::Supernova);
        assert_ne!(classify(&o, &h, 110.5), Stage::Supernova, "past 20 t.u.");
        assert_eq!(classify(&o, &h, 110.0), Stage::Supernova, "exactly 20 t.u. is within");
        o.unbound = 9.9;
        assert_ne!(classify(&o, &h, 100.0), Stage::Supernova, "under 0.1 M unbound");
        o.unbound = 10.0;
        assert_ne!(classify(&o, &ignited(), 100.0), Stage::Supernova, "never passed core collapse");
    }

    #[test]
    fn row_core_collapse_n_fe_converting() {
        let mut o = burning([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0]);
        assert_eq!(stage_of(&o, &ignited()), Stage::CoreCollapse);
        o.l_group[6] = 0.0;
        assert_ne!(stage_of(&o, &ignited()), Stage::CoreCollapse);
    }

    #[test]
    fn supernova_before_core_collapse() {
        let mut o = burning([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0]);
        o.unbound = 50.0;
        let h = History { ignited: true, last_collapse: Some(99.0) };
        assert_eq!(classify(&o, &h, 100.0), Stage::Supernova);
    }

    fn degenerate() -> Object {
        let mut o = cloud();
        o.d_c = 0.5;
        o.l_nuc = 0.0;
        o.luminosity = 10.0;
        o
    }

    #[test]
    fn row_white_dwarf() {
        let mut o = degenerate();
        assert_eq!(stage_of(&o, &ignited()), Stage::WhiteDwarf);
        o.l_nuc = 0.1;
        assert_ne!(stage_of(&o, &ignited()), Stage::WhiteDwarf, "still burning at 0.01 L");
        o.l_nuc = 0.0;
        o.unbound = 5.0;
        assert_ne!(stage_of(&o, &ignited()), Stage::WhiteDwarf, "0.05 M unbound");
        o.unbound = 0.0;
        o.d_c = 0.49;
        assert_ne!(stage_of(&o, &ignited()), Stage::WhiteDwarf, "not degenerate");
    }

    #[test]
    fn row_failed_star_only_when_never_ignited() {
        let o = degenerate();
        assert_eq!(stage_of(&o, &fresh()), Stage::FailedStar);
        assert_eq!(stage_of(&o, &ignited()), Stage::WhiteDwarf, "the same object, ignited once: a white dwarf");
        let mut o = degenerate();
        o.l_group[0] = 0.5;
        assert_ne!(stage_of(&o, &fresh()), Stage::FailedStar, "L_H at 0.05 L");
    }

    #[test]
    fn row_planetary_nebula() {
        let mut o = degenerate();
        o.unbound = 5.0;
        assert_eq!(stage_of(&o, &ignited()), Stage::PlanetaryNebula);
        o.l_nuc = 1.0;
        assert_ne!(stage_of(&o, &ignited()), Stage::PlanetaryNebula, "L_nuc at 0.1 L");
        o.l_nuc = 0.0;
        assert_ne!(stage_of(&o, &fresh()), Stage::PlanetaryNebula, "never ignited");
    }

    #[test]
    fn row_iron_core() {
        let mut o = cloud();
        with_x(&mut o, X_FE, 0.5);
        assert_eq!(stage_of(&o, &ignited()), Stage::IronCore);
        o.l_nuc = 1.0;
        assert_ne!(stage_of(&o, &ignited()), Stage::IronCore, "L_nuc at 0.1 L");
    }

    /// Powers H, He, C, Ne, O, Si, N.
    fn burn_stage(l: [f64; 7], fuel: usize, x: f64) -> Stage {
        let mut o = burning(l);
        o.x_c[fuel] = x;
        stage_of(&o, &ignited())
    }

    #[test]
    fn rows_si_o_ne_c_burn() {
        assert_eq!(burn_stage([0.0, 0.0, 0.0, 0.0, 0.0, 5.0, 0.0], FUEL_SI, 0.1), Stage::SiBurn);
        assert_eq!(burn_stage([0.0, 0.0, 0.0, 0.0, 5.0, 0.0, 0.0], FUEL_O, 0.1), Stage::OBurn);
        assert_eq!(burn_stage([0.0, 0.0, 0.0, 5.0, 0.0, 0.0, 0.0], FUEL_NE, 0.1), Stage::NeBurn);
        assert_eq!(burn_stage([0.0, 0.0, 5.0, 0.0, 0.0, 0.0, 0.0], FUEL_C, 0.1), Stage::CBurn);
    }

    #[test]
    fn a_burn_stage_needs_the_largest_group_and_its_fuel() {
        // Silicon is lit but oxygen is the larger power: o_burn wins, si_burn does not match.
        assert_eq!(burn_stage([0.0, 0.0, 0.0, 0.0, 6.0, 5.0, 0.0], FUEL_O, 0.1), Stage::OBurn);
        // Hydrogen larger than Si: not a burning stage of ours (main sequence needs X_H, so it falls on).
        assert_ne!(burn_stage([9.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0], FUEL_SI, 0.1), Stage::SiBurn);
        // The fuel at the centre spent: no stage.
        assert_ne!(burn_stage([0.0, 0.0, 0.0, 0.0, 0.0, 5.0, 0.0], FUEL_SI, 0.0009), Stage::SiBurn);
        assert_eq!(burn_stage([0.0, 0.0, 0.0, 0.0, 0.0, 5.0, 0.0], FUEL_SI, 0.001), Stage::SiBurn);
    }

    #[test]
    fn burn_rows_come_in_si_o_ne_c_order_on_a_tie() {
        assert_eq!(burn_stage([0.0, 0.0, 5.0, 5.0, 5.0, 5.0, 0.0], FUEL_SI, 0.1), Stage::SiBurn);
        let mut o = burning([0.0, 0.0, 5.0, 5.0, 5.0, 0.0, 0.0]);
        o.x_c[FUEL_O] = 0.1;
        o.x_c[FUEL_NE] = 0.1;
        o.x_c[FUEL_C] = 0.1;
        assert_eq!(stage_of(&o, &ignited()), Stage::OBurn);
    }

    #[test]
    fn row_he_core() {
        let mut o = cloud();
        o.l_group = [2.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        o.l_nuc = 3.0;
        o.luminosity = 10.0;
        with_x(&mut o, X_HE, 0.5);
        assert_eq!(stage_of(&o, &ignited()), Stage::HeCore);
        o.l_group[1] = 0.9;
        assert_ne!(stage_of(&o, &ignited()), Stage::HeCore, "L_He under 0.1 L");
        o.l_group[1] = 1.0;
        o.x_c[X_HE] = 0.0009;
        assert_ne!(stage_of(&o, &ignited()), Stage::HeCore, "no helium at the centre");
    }

    #[test]
    fn row_he_shell() {
        let mut o = cloud();
        o.x_c = [0.0; 10];
        assert_eq!(stage_of(&o, &ignited()), Stage::HeShell);
        assert_ne!(stage_of(&o, &fresh()), Stage::HeShell, "never ignited");
        o.l_group[2] = 1.0;
        o.l_nuc = 1.0;
        o.x_c[FUEL_C] = 0.0;
        // Carbon burning at the centre's shell: no he_shell (c_burn needs its fuel, so h_shell takes it).
        assert_ne!(stage_of(&o, &ignited()), Stage::HeShell);
        let mut o = cloud();
        o.x_c = [0.0; 10];
        o.x_c[X_HE] = 0.5;
        assert_ne!(stage_of(&o, &ignited()), Stage::HeShell, "helium at the centre");
    }

    #[test]
    fn row_h_shell() {
        let mut o = cloud();
        with_x(&mut o, X_HE, 0.5);
        assert_eq!(stage_of(&o, &ignited()), Stage::HShell);
        o.l_group = [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        o.l_nuc = 1.0;
        assert_ne!(stage_of(&o, &ignited()), Stage::HShell, "helium burning at 0.1 L is he_core's");
        assert_eq!(stage_of(&o, &ignited()), Stage::HeCore);
    }

    #[test]
    fn row_main_sequence() {
        let mut o = cloud();
        o.l_group = [5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        o.l_nuc = 5.0;
        o.luminosity = 10.0;
        assert_eq!(stage_of(&o, &ignited()), Stage::MainSequence);
        o.l_group[0] = 4.9;
        assert_ne!(stage_of(&o, &ignited()), Stage::MainSequence, "L_H under half of L");
        o.l_group[0] = 5.0;
        with_x(&mut o, X_H, 0.0009);
        assert_ne!(stage_of(&o, &ignited()), Stage::MainSequence, "the centre's hydrogen spent");
    }

    #[test]
    fn the_main_sequence_shines_what_it_burns() {
        // §0.5 [M0-D53]: 0.5 L ≤ L_H ≤ 2 L, both ends within.
        let mut o = cloud();
        o.l_group = [20.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        o.l_nuc = 20.0;
        assert_eq!(stage_of(&o, &ignited()), Stage::MainSequence, "L_H = 2 L");
        o.l_group[0] = 20.1;
        o.l_nuc = 20.1;
        assert_ne!(stage_of(&o, &ignited()), Stage::MainSequence, "burning more than twice the light it shines");
    }

    #[test]
    fn a_flash_inside_a_dark_cloud_does_not_ignite() {
        // M0-V8's step 225: L_H = 277 against L = 2.1 × 10⁻⁷ — no ignition, the cloud's rows still open.
        let mut t = StageTracker::new();
        let mut flash = cloud();
        flash.luminosity = 2.1e-7;
        flash.l_group = [277.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        flash.l_nuc = 277.0;
        flash.v_r = -1.0;
        assert!(t.update(&[flash.clone()], 1.0).is_empty(), "no ignition event");
        let s = t.stage(1).unwrap();
        assert!(!s.history.ignited);
        assert_eq!(s.raw, Stage::Collapsing, "a never-ignited cloud falling in");
        // The same burning once the light matches it: L_H = 2 L ignites.
        flash.luminosity = 138.5;
        assert_eq!(kinds(&t.update(&[flash.clone()], 2.0)), vec![EventKind::Ignition]);
        assert_eq!(t.stage(1).unwrap().raw, Stage::MainSequence);
        let mut t = StageTracker::new();
        flash.luminosity = 138.4;
        assert!(t.update(&[flash.clone()], 1.0).is_empty(), "L_H just over 2 L");
        flash.luminosity = 554.0;
        assert_eq!(kinds(&t.update(&[flash.clone()], 2.0)), vec![EventKind::Ignition], "L_H = 0.5 L");
        let mut t = StageTracker::new();
        flash.luminosity = 554.1;
        assert!(t.update(&[flash], 1.0).is_empty(), "L_H just under 0.5 L");
    }

    #[test]
    fn row_collapsing() {
        let mut o = cloud();
        o.v_r = -1.0;
        assert_eq!(stage_of(&o, &fresh()), Stage::Collapsing);
        o.v_r = -0.099;
        assert_ne!(stage_of(&o, &fresh()), Stage::Collapsing);
        o.v_r = -1.0;
        assert_ne!(stage_of(&o, &ignited()), Stage::Collapsing, "an ignited star falling in is no cloud");
    }

    #[test]
    fn row_protostar_and_the_cloud() {
        let mut o = cloud();
        assert_eq!(stage_of(&o, &fresh()), Stage::Cloud, "a cold cloud at rest: 2P/|W| = 0.2");
        o.pressure = 4.0;
        assert_eq!(stage_of(&o, &fresh()), Stage::Protostar, "2P/|W| = 0.8 at rest");
        o.pressure = 3.9;
        assert_eq!(stage_of(&o, &fresh()), Stage::Cloud);
        o.pressure = 10.0;
        o.v_r = 0.06;
        assert_eq!(stage_of(&o, &fresh()), Stage::Cloud, "pressure-supported but moving");
        o.v_r = 0.0;
        assert_ne!(stage_of(&o, &ignited()), Stage::Protostar, "ignited objects are never protostars");
    }

    #[test]
    fn collapsing_and_protostar_do_not_overlap_so_the_table_order_is_the_contracts() {
        // §1.9.3: collapsing has v_r ≤ −0.1 v and protostar |v_r| ≤ 0.05 v; a pressure-supported infall is a collapse.
        let mut o = cloud();
        o.pressure = 10.0;
        o.v_r = -0.5;
        assert_eq!(stage_of(&o, &fresh()), Stage::Collapsing);
        let order = stage_order();
        let pos = |s: Stage| order.iter().position(|x| *x == s).unwrap();
        assert!(pos(Stage::Collapsing) < pos(Stage::Protostar));
    }

    #[test]
    fn the_classifier_reads_no_rung_or_frame() {
        // The same object and history give the same stage at any time outside the supernova window.
        let o = cloud();
        assert_eq!(classify(&o, &fresh(), 0.0), classify(&o, &fresh(), 1e6));
    }

    fn id(mut o: Object, id: u32) -> Object {
        o.id = id;
        o
    }

    #[test]
    fn a_stage_shows_after_three_identical_results() {
        let mut t = StageTracker::new();
        let o = cloud();
        for frame in 1..=3u32 {
            t.update(&[o.clone()], frame as f64);
            let s = t.stage(1).unwrap();
            assert_eq!(s.raw, Stage::Cloud);
            assert_eq!(s.shown, if frame < 3 { None } else { Some(Stage::Cloud) }, "frame {frame}");
        }
    }

    #[test]
    fn a_different_result_restarts_the_hold() {
        let mut t = StageTracker::new();
        let mut protostar = cloud();
        protostar.pressure = 4.0;
        for _ in 0..3 {
            t.update(&[cloud()], 0.0);
        }
        assert_eq!(t.stage(1).unwrap().shown, Some(Stage::Cloud));
        t.update(&[protostar.clone()], 1.0);
        t.update(&[protostar.clone()], 2.0);
        assert_eq!(t.stage(1).unwrap().shown, Some(Stage::Cloud), "two frames of protostar: still the cloud");
        t.update(&[cloud()], 3.0);
        t.update(&[protostar.clone()], 4.0);
        t.update(&[protostar.clone()], 5.0);
        assert_eq!(t.stage(1).unwrap().shown, Some(Stage::Cloud), "a cloud frame between: the count restarted");
        t.update(&[protostar], 6.0);
        assert_eq!(t.stage(1).unwrap().shown, Some(Stage::Protostar));
    }

    #[test]
    fn core_collapse_and_supernova_show_at_once() {
        let mut t = StageTracker::new();
        let mut o = burning([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0]);
        o.x_c[X_H] = 0.5;
        t.update(&[cloud()], 0.0);
        t.update(&[o.clone()], 1.0);
        assert_eq!(t.stage(1).unwrap().shown, Some(Stage::CoreCollapse));
        o.unbound = 50.0;
        t.update(&[o], 2.0);
        assert_eq!(t.stage(1).unwrap().shown, Some(Stage::Supernova));
    }

    fn kinds(events: &[Event]) -> Vec<EventKind> {
        events.iter().map(|e| e.kind).collect()
    }

    #[test]
    fn ignition_fires_once_and_the_flag_stays_on() {
        let mut t = StageTracker::new();
        assert!(t.update(&[cloud()], 0.0).is_empty());
        let mut lit = cloud();
        lit.l_group = [5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        lit.l_nuc = 5.0;
        let events = t.update(&[lit.clone()], 1.0);
        assert_eq!(events, vec![Event { object: 1, kind: EventKind::Ignition, time: 1.0 }]);
        assert!(t.stage(1).unwrap().history.ignited);
        assert!(t.update(&[lit], 2.0).is_empty());
        // L_H falls away: ignited stays, no second ignition.
        let mut dim = cloud();
        dim.l_group = [0.0; 7];
        assert!(t.update(&[dim.clone()], 3.0).is_empty());
        assert!(t.stage(1).unwrap().history.ignited);
        let mut again = dim;
        again.l_group[0] = 9.0;
        again.l_nuc = 9.0;
        assert!(t.update(&[again], 4.0).is_empty(), "an object ignites once");
    }

    #[test]
    fn core_collapse_and_supernova_fire_once_each_in_order() {
        let mut t = StageTracker::new();
        let mut o = burning([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0]);
        let first = t.update(&[o.clone()], 1.0);
        assert_eq!(kinds(&first), vec![EventKind::CoreCollapse]);
        assert!(t.update(&[o.clone()], 2.0).is_empty(), "core collapse again: nothing");
        o.unbound = 50.0;
        let boom = t.update(&[o.clone()], 3.0);
        assert_eq!(kinds(&boom), vec![EventKind::Supernova], "the collapse row is shadowed by supernova while it holds");
        assert!(t.update(&[o.clone()], 4.0).is_empty());
        // Collapse again later: still once.
        let mut o2 = o;
        o2.unbound = 0.0;
        assert!(t.update(&[o2], 5.0).is_empty());
    }

    #[test]
    fn the_supernova_needs_a_collapse_in_the_last_20_tu() {
        let mut t = StageTracker::new();
        let mut o = cloud();
        o.unbound = 50.0;
        // Unbound mass with no collapse behind it: no supernova.
        assert!(!kinds(&t.update(&[o.clone()], 1.0)).contains(&EventKind::Supernova));
        let collapse = burning([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0]);
        t.update(&[collapse], 2.0);
        assert!(kinds(&t.update(&[o.clone()], 30.0)).is_empty(), "28 t.u. later");
        assert_ne!(t.stage(1).unwrap().raw, Stage::Supernova);
        let mut t = StageTracker::new();
        t.update(&[burning([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0])], 2.0);
        assert!(kinds(&t.update(&[o], 12.0)).contains(&EventKind::Supernova));
    }

    #[test]
    fn sink_formed_fires_once_per_object_and_events_are_per_object() {
        let mut t = StageTracker::new();
        let mut a = cloud();
        a.sinks = 1;
        let b = id(cloud(), 2);
        let events = t.update(&[a.clone(), b.clone()], 1.0);
        assert_eq!(events, vec![Event { object: 1, kind: EventKind::SinkFormed, time: 1.0 }]);
        assert!(t.update(&[a, b.clone()], 2.0).is_empty());
        let mut b2 = b;
        b2.sinks = 1;
        assert_eq!(t.update(&[b2], 3.0), vec![Event { object: 2, kind: EventKind::SinkFormed, time: 3.0 }]);
        assert!(t.stage(1).is_none(), "an object gone from the frame is forgotten");
    }

    #[test]
    fn a_frame_with_ignition_classifies_with_the_flag_on() {
        // The frame L_H reaches half L: this frame's result already sees `ignited` (main sequence, not failed/cloud rows).
        let mut t = StageTracker::new();
        let mut o = cloud();
        o.l_group = [5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        o.l_nuc = 5.0;
        t.update(&[o], 1.0);
        assert_eq!(t.stage(1).unwrap().raw, Stage::MainSequence);
    }
}
