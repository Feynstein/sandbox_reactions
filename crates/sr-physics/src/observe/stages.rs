//! Stages and events (m0_contrat.md §1.9.3, §1.9.4; M0-T59).
//!
//! [`classify`] reads one object and its history and walks §1.9.3's rows in the contract's order, the first match
//! winning. [`StageTracker::update`] does it for every object of a frame: it keeps each object's history (the `ignited`
//! flag, the last time its row said core collapse, the most hydrogen its centre has shown), applies the display hold, and
//! returns the events (§1.9.4), each once per object. The classifier never reads the rung, the frame or the engine — the
//! object, its history and m_ch_sb, the heaviest object a white dwarf may hold and a collapsing core's must pass, only
//! (I2; G-WATCH keeps this module out of the step). [`ending_hold`] is how long a remnant stage must show before it is the object's ending (§1.9.5 as §0.5 [M0-D78] reads it), of [`hold_tau`] — the degenerate centre's τ, else τ_dyn (§0.5 [M0-D161]) — for the engine's `ending` event and the
//! readout's.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use super::tracker::{exact, read_exact, read_whole, Object};
use crate::registry::electron_fraction;

/// The display hold: a stage other than core collapse and supernova shows after this many identical results (§1.9.3).
const HOLD: u32 = 3;
/// A supernova row is open this long (t.u.) after the object last passed core collapse.
const SUPERNOVA_WINDOW: f64 = 20.0;
/// L_H between these shares of L ignites an object: a star shines what it burns (§1.9.4 as §0.5 [M0-D53] reads it; the lead at
/// M0-V18, «Burning matches light (Recommended)»). The main sequence no longer reads the window — §0.5 [M0-D141].
const HYDROGEN_SHARE: f64 = 0.5;
const HYDROGEN_SHARE_MAX: f64 = 2.0;
/// Or the centre has burned a tenth of its hydrogen: X_H,c at or under this share of the most it has shown, while hydrogen
/// burns (§1.9.4 as §0.5 [M0-D78] reads it; the lead at M0-V9, «Burnt fuel also counts (Recommended)») — a flash that
/// overshoots the light window between two frames still burned its fuel.
const BURNT_SHARE: f64 = 0.9;
/// A remnant stage is the ending once shown for the longer of this many t.u. and [`ENDING_HOLD_TAU`] τ_dyn (§1.9.5, §2.11
/// item 3 as §0.5 [M0-D78] reads them; the lead at M0-V9, «max(10 t.u., 20 τ_dyn) (Recommended)»).
pub const ENDING_HOLD_MIN: f64 = 10.0;
pub const ENDING_HOLD_TAU: f64 = 20.0;
/// An abundance at the centre under this is spent; a fuel at or above it still feeds its stage (§1.9.3).
const SPENT: f64 = 1e-3;
/// A group is a burning stage's when it is the largest and at least this share of L_nuc.
const BURN_SHARE: f64 = 0.1;
/// A neutron star's centre is neutron matter: Y_e,c at or under this. Iron alone is 26/56 = 0.464 and every other species
/// ≥ 0.5, so no white dwarf reaches it; N_Fe's gate holds a collapsed iron centre at 0.25–0.30 (§1.9.3 as §0.5 [M0-D79]
/// reads it; the lead at M0-R5, «Rate + NS row (Recommended)»; reports/neutronization.md).
const NEUTRON_Y_E: f64 = 0.35;

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

/// What an object remembers between frames: the `ignited` flag (§1.9.4), the last time its row said core collapse and the
/// most hydrogen its centre has shown.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct History {
    /// First turned on by [`ignites`], never off again.
    pub ignited: bool,
    /// The sim time (t.u.) of the last frame whose result was core collapse.
    pub last_collapse: Option<f64>,
    /// The most X_H,c the object's centre has shown at any frame of its id, this one included (0 before the first).
    pub x_h_max: f64,
}

/// An object's quantities as the rows read them.
struct Ctx<'a> {
    o: &'a Object,
    h: &'a History,
    now: f64,
    /// √(GM/R) = R / τ_dyn.
    v_dyn: f64,
    /// m_ch_sb (§2.3.2): an object holding more is no white dwarf and sheds no planetary nebula (§0.5 [M0-D81]); one holding
    /// at most this is no collapsing core (§0.5 [M0-D92]).
    m_ch: f64,
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

/// At rest and pressure-supported, near virial: |v_r| ≤ 0.05 √(GM/R) and 2P/|W| ≥ 0.8 — the protostar's test, the failed
/// star's since §0.5 [M0-D134] and the white dwarf's and planetary nebula's since §0.5 [M0-D140]: a cold cloud's centre is held
/// mostly by the cold pressure (d_c 0.55–0.65 at the drop) while it falls in, and the Sun-like's still does while it re-expands
/// after its flash, so d_c alone makes no remnant. The support counts the ordered rotation about c beside the pressure,
/// 2(P + K_t)/|W| ≥ 0.8 — §0.5 [M0-D160]: m_up_sb's light end leaves a cold iron core inside a Keplerian disk, 2P/|W| 0.356
/// and 2(P + K_t)/|W| 0.93–0.95, where the Sun-like re-expanding after its flash has K_t ≈ 10⁻⁵ |W| (RTX 4090).
fn settled(c: &Ctx) -> bool {
    c.o.v_r.abs() <= 0.05 * c.v_dyn && c.o.potential != 0.0 && 2.0 * (c.o.pressure + c.o.kinetic_t) / c.o.potential.abs() >= 0.8
}

/// A white dwarf's centre at or above this degeneracy is held by the cold pressure alone (§0.5 [M0-D168]).
const DEGENERATE_AT_REST: f64 = 0.9;

/// At rest on a wholly degenerate centre: |v_r| ≤ 0.05 √(GM/R) and d_c ≥ 0.9 — the white dwarf's other way to be settled, §0.5
/// [M0-D168] (the lead, «Degenerate at rest (Recommended)»): a remnant of a few cells reads 2P/|W| 0.70–0.79 on the grid however
/// still it is — m_up_sb's 3,351 sb life on the burn clock leaves a 7 × 7-cell C/O remnant at d_c 1.000, |v_r| 0.043 √(GM/R),
/// 2(P + K_t)/|W| 0.779 — where M0-D140's false white dwarf, the Sun-like re-expanding after its flash, has d_c 0.59–0.61.
fn degenerate_at_rest(c: &Ctx) -> bool {
    c.o.v_r.abs() <= 0.05 * c.v_dyn && c.o.d_c >= DEGENERATE_AT_REST
}

/// Group `g` is the largest burning group at c itself (q̇_c, lit only from its ladder temperature on), its fuel at the
/// centre still there — §0.5 [M0-D169] (the lead, «Ladder down + core labels», then «Centre cell's rates»): the star's L_g let
/// the hydrogen shell (L_H 0.6–1.8 × 10⁵) hide the Massive's oxygen and silicon burning centre (L_O to 5.5 × 10⁴) for its
/// whole late life, and c's own block burns its H and He shells too.
fn core_burns(c: &Ctx, g: usize, fuel: usize) -> bool {
    let o = c.o;
    let q = o.q_c[g];
    let largest = (0..G_N).all(|k| o.q_c[k] <= q);
    q > 0.0 && largest && o.x_c[fuel] >= SPENT
}

/// The species each burning group's records burn, H … Si (assets/reactions.json: O_burn and Mg_burn book into O, Si_burn and
/// S_burn into Si).
const GROUP_FUELS: [&[usize]; 6] = [&[0], &[1], &[2], &[4], &[3, 5], &[6, 7]];

/// Some fuel still burns at c: a group lit there (q̇_c > 0) with one of its fuels at c ≥ 10⁻³ (§0.5 [M0-D169]).
fn centre_burns(o: &Object) -> bool {
    GROUP_FUELS.iter().enumerate().any(|(g, fuels)| o.q_c[g] > 0.0 && fuels.iter().any(|&f| o.x_c[f] >= SPENT))
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

/// §1.9.3's rows, in the table's order: the first that matches wins. The neutron star reads the neutronized, degenerate
/// centre and sits after supernova (a supernova's 20 t.u. still shows) and before core collapse (the gate re-opening for a
/// frame no longer restarts the remnant's hold) — §0.5 [M0-D79]; its |v_r| term returns with M0-D71's well-balanced P4. It
/// also needs the object to have passed core collapse: a centre neutronized inside one frame shows core_collapse first, its
/// event fires and its supernova can follow — §0.5 [M0-D88].
/// White dwarf and planetary nebula read an object of at most m_ch_sb: a heavier star's quiet, degenerate centre is its iron
/// core on the way to collapse, never a remnant whose hold races the collapse — §0.5 [M0-D81]. Core collapse reads one of more
/// (none refused where the run has no ceiling): a lighter remnant whose cold iron centre overshoots into N_Fe's gate bounces
/// back to a white dwarf — §0.5 [M0-D92]. White dwarf and planetary nebula are also settled: the Sun-like, re-expanding at
/// +0.2 √(GM/R) after its flash with its hydrogen unburnt, is no remnant — §0.5 [M0-D140]; a white dwarf may instead be at rest
/// on a wholly degenerate centre — §0.5 [M0-D168].
/// Iron core and the c/ne/o/si_burn rows read the burning at c itself, not the whole star's — §0.5 [M0-D169].
/// Main sequence reads an ignited object whose largest burning group, over the whole object, is hydrogen: the
/// Sun-like after its flash burns from nothing to 86 times what it shines for 1,700 steps, never inside [0.5 L, 2 L], which now
/// holds the ignition event alone — §0.5 [M0-D141].
const ROWS: [Row; 19] = [
    (Stage::BlackHole, |c| c.o.sinks > 0),
    (Stage::Supernova, |c| {
        c.h.last_collapse.is_some_and(|t| c.now - t <= SUPERNOVA_WINDOW) && c.o.unbound >= 0.1 * c.o.mass
    }),
    (Stage::NeutronStar, |c| {
        electron_fraction(&c.o.x_c) <= NEUTRON_Y_E && c.o.d_c >= 0.5 && c.o.sinks == 0 && c.h.last_collapse.is_some()
    }),
    (Stage::CoreCollapse, |c| c.o.l_group[G_N] < 0.0 && (c.o.mass > c.m_ch || c.m_ch.is_infinite())),
    (Stage::WhiteDwarf, |c| {
        c.h.ignited
            && c.o.d_c >= 0.5
            && c.o.l_nuc < 0.01 * c.o.luminosity
            && c.o.unbound < 0.05 * c.o.mass
            && c.o.x_c[X_N] < 0.5
            && c.o.mass <= c.m_ch
            && (settled(c) || degenerate_at_rest(c))
    }),
    (Stage::FailedStar, |c| !c.h.ignited && c.o.d_c >= 0.5 && c.o.l_group[G_H] < 0.01 * c.o.luminosity && settled(c)),
    (Stage::PlanetaryNebula, |c| {
        c.h.ignited
            && c.o.d_c >= 0.5
            && c.o.l_nuc < 0.1 * c.o.luminosity
            && c.o.unbound >= 0.05 * c.o.mass
            && c.o.mass <= c.m_ch
            && settled(c)
    }),
    (Stage::IronCore, |c| c.o.x_c[X_FE] >= 0.5 && !centre_burns(c.o)),
    (Stage::SiBurn, |c| core_burns(c, G_SI, FUEL_SI)),
    (Stage::OBurn, |c| core_burns(c, G_O, FUEL_O)),
    (Stage::NeBurn, |c| core_burns(c, G_NE, FUEL_NE)),
    (Stage::CBurn, |c| core_burns(c, G_C, FUEL_C)),
    (Stage::HeCore, |c| c.o.l_group[G_HE] > 0.0 && c.o.l_group[G_HE] >= 0.1 * c.o.luminosity && c.o.x_c[X_HE] >= SPENT),
    (Stage::HeShell, |c| c.h.ignited && c.o.x_c[X_HE] < SPENT && c.o.x_c[X_H] < SPENT && !carbon_burns(c.o)),
    (Stage::HShell, |c| c.h.ignited && c.o.x_c[X_H] < SPENT && c.o.l_group[G_HE] < 0.1 * c.o.luminosity),
    (Stage::MainSequence, |c| c.h.ignited && burns(c, G_H, X_H)),
    (Stage::Collapsing, |c| !c.h.ignited && c.o.v_r <= -0.1 * c.v_dyn),
    (Stage::Protostar, |c| !c.h.ignited && settled(c)),
    (Stage::Cloud, |_| true),
];

/// The stages in the table's order, for the readers that list them.
pub fn stage_order() -> [Stage; 19] {
    ROWS.map(|(s, _)| s)
}

/// The object's stage this frame by §1.9.3's rows, given its history (`h.ignited` already read at this frame), the sim time
/// and m_ch_sb.
pub fn classify(o: &Object, h: &History, now: f64, m_ch: f64) -> Stage {
    let v_dyn = if o.tau_dyn > 0.0 { o.radius / o.tau_dyn } else { 0.0 };
    let c = Ctx { o, h, now, v_dyn, m_ch };
    ROWS.iter().find(|(_, test)| test(&c)).map(|(s, _)| *s).unwrap_or(Stage::Cloud)
}

/// What turned an object's `ignited` flag on: the ignition event's `by` (§2.12.2 as §0.5 [M0-D78] reads it).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ignition {
    /// 0.5 L ≤ L_H ≤ 2 L (§0.5 [M0-D53]).
    Light,
    /// The centre burned a tenth of its hydrogen (§0.5 [M0-D78]).
    Fuel,
}

impl Ignition {
    pub fn id(self) -> &'static str {
        match self {
            Ignition::Light => "light",
            Ignition::Fuel => "fuel",
        }
    }
}

/// Whether this frame turns the object's `ignited` flag on, and by which rule (§1.9.4 as §0.5 [M0-D78] reads it): L_H between
/// half and twice L, or X_H,c at or under 0.9 × `h.x_h_max` (already this frame's most, and at least 10⁻³) while hydrogen
/// burns — a centre that moved onto painted helium burned nothing.
pub fn ignites(o: &Object, h: &History) -> Option<Ignition> {
    if hydrogen_leads(o) {
        Some(Ignition::Light)
    } else if h.x_h_max >= SPENT && o.x_c[X_H] <= BURNT_SHARE * h.x_h_max && o.l_group[G_H] > 0.0 {
        Some(Ignition::Fuel)
    } else {
        None
    }
}

/// Whether showing `stage` would take the display back before the life it has shown (§0.5 [M0-D134]; the lead, «Display holds
/// (Recommended)»): an ignited object never shows cloud, collapsing or protostar again, and before ignition those go only
/// forward — cloud, collapsing, protostar — a cloud never following a failed star either. The rows are unchanged: the raw stage
/// still reads them, only the shown one stays.
pub fn steps_back(stage: Stage, shown: Option<Stage>, ignited: bool) -> bool {
    let rank = |s: Stage| match s {
        Stage::Cloud => Some(0),
        Stage::Collapsing => Some(1),
        Stage::Protostar => Some(2),
        _ => None,
    };
    let Some(r) = rank(stage) else { return false };
    match shown {
        _ if ignited => true,
        Some(Stage::FailedStar) => r == 0,
        Some(s) => rank(s).is_some_and(|q| r < q),
        None => false,
    }
}

/// The τ an object's ending hold reads: its degenerate centre's τ_deg, else its τ_dyn (§0.5 [M0-D161]; the lead at M0-D161,
/// «(a) Core's τ (Recommended)», «R₉₀ as today (Recommended)»). m_up_sb's light end is a cold core of 4–5 cells inside a
/// Keplerian disk: τ_dyn of the disk's R₉₀ held the white dwarf 420–990 t.u., which the disk's |v_r| crossing 0.05 √(GM/R)
/// every few hundred steps never let it reach. The engine's `ending` event, summary.json's readout and the app's readout read
/// [`ending_hold`] of this.
pub fn hold_tau(o: &Object) -> f64 {
    o.tau_deg.unwrap_or(o.tau_dyn)
}

/// How long (t.u.) a remnant stage must show before it is the object's ending: max(10 t.u., 20 τ_dyn), τ_dyn the object's at
/// the frame read (§1.9.5 as §0.5 [M0-D78] reads it). A compact remnant keeps 10 t.u.; an extended one waits out its dynamics.
pub fn ending_hold(tau_dyn: f64) -> f64 {
    ENDING_HOLD_MIN.max(ENDING_HOLD_TAU * tau_dyn)
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

/// An event: which object, which kind, the sim time (t.u.) of the frame that saw it, and for an ignition the rule that lit it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Event {
    pub object: u32,
    pub kind: EventKind,
    pub time: f64,
    pub by: Option<Ignition>,
}

/// One object's stage as the readout wants it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ObjectStage {
    /// This frame's result, before the hold.
    pub raw: Stage,
    /// The displayed stage: none until a stage has held 3 frames or core collapse / supernova came; then the latest held that
    /// does not step back ([`steps_back`]).
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
pub struct StageTracker {
    states: BTreeMap<u32, State>,
    /// m_ch_sb for the run's physics, which the white dwarf and planetary nebula rows read.
    m_ch: f64,
}

impl StageTracker {
    /// A tracker whose rows read `m_ch`, m_ch_sb as §2.11 item 1 computes it for the run's physics (§0.5 [M0-D81]).
    pub fn new(m_ch: f64) -> StageTracker {
        StageTracker { states: BTreeMap::new(), m_ch }
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
            let mut fire = |state: &mut State, kind: EventKind, by: Option<Ignition>| {
                if !state.fired[kind as usize] {
                    state.fired[kind as usize] = true;
                    events.push(Event { object: o.id, kind, time: now, by });
                }
            };
            state.history.x_h_max = state.history.x_h_max.max(o.x_c[X_H]);
            if let Some(by) = ignites(o, &state.history).filter(|_| !state.history.ignited) {
                state.history.ignited = true;
                fire(state, EventKind::Ignition, Some(by));
            }
            let stage = classify(o, &state.history, now, self.m_ch);
            if stage == Stage::CoreCollapse {
                state.history.last_collapse = Some(now);
                fire(state, EventKind::CoreCollapse, None);
            }
            if stage == Stage::Supernova {
                fire(state, EventKind::Supernova, None);
            }
            if o.sinks > 0 {
                fire(state, EventKind::SinkFormed, None);
            }
            state.streak = if stage == state.raw { state.streak + 1 } else { 1 };
            state.raw = stage;
            if (stage.is_immediate() || state.streak >= HOLD) && !steps_back(stage, state.shown, state.history.ignited) {
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
    /// core collapse, [M0-D78]'s `x_h_max`), the last result and its streak, the shown stage and the events fired.
    pub fn snapshot(&self) -> Value {
        let states: Vec<Value> = self
            .states
            .iter()
            .map(|(id, s)| {
                json!({
                    "id": id,
                    "ignited": s.history.ignited,
                    "last_collapse": s.history.last_collapse.map_or(Value::Null, exact),
                    "x_h_max": exact(s.history.x_h_max),
                    "raw": s.raw.id(),
                    "streak": s.streak,
                    "shown": s.shown.map(Stage::id),
                    "fired": s.fired,
                })
            })
            .collect();
        Value::Array(states)
    }

    /// The stages [`StageTracker::snapshot`] wrote, the rows reading `m_ch` as [`StageTracker::new`]; one without `x_h_max`
    /// (written before §0.5 [M0-D78]) is refused, never read as a centre that showed no hydrogen.
    pub fn restore(v: &Value, m_ch: f64) -> Result<StageTracker, String> {
        let mut states = BTreeMap::new();
        for s in v.as_array().ok_or_else(|| format!("stages: not a list: {v}"))? {
            let id = u32::try_from(read_whole(s, "id")?).map_err(|_| "stages.id: past u32".to_string())?;
            let ignited = s["ignited"].as_bool().ok_or_else(|| format!("stages.ignited: not true or false: {}", s["ignited"]))?;
            let last_collapse = if s["last_collapse"].is_null() { None } else { Some(read_exact(&s["last_collapse"], "stages.last_collapse")?) };
            let x_h_max = read_exact(&s["x_h_max"], "stages.x_h_max")?;
            let shown = if s["shown"].is_null() { None } else { Some(Stage::from_id(&s["shown"])?) };
            let fired: Vec<bool> = s["fired"].as_array().into_iter().flatten().filter_map(Value::as_bool).collect();
            let fired: [bool; 4] = fired.try_into().map_err(|_| format!("stages.fired: not 4 flags: {}", s["fired"]))?;
            let streak = u32::try_from(read_whole(s, "streak")?).map_err(|_| "stages.streak: past u32".to_string())?;
            states.insert(id, State { history: History { ignited, last_collapse, x_h_max }, raw: Stage::from_id(&s["raw"])?, streak, shown, fired });
        }
        Ok(StageTracker { states, m_ch })
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
            q_c: [0.0; 7],
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
            kinetic_t: 0.0,
            pressure: 1.0,
            potential: -10.0,
            unbound: 0.0,
            core_group: None,
            r_core: None,
            envelope_mass: None,
            tau_deg: None,
        }
    }

    fn ignited() -> History {
        History { ignited: true, ..History::default() }
    }

    fn fresh() -> History {
        History::default()
    }

    /// Ignited, its last core-collapse frame at t 50 (past the supernova window at the tests' t 100).
    fn collapsed() -> History {
        History { ignited: true, last_collapse: Some(50.0), ..History::default() }
    }

    fn with_x(o: &mut Object, species: usize, x: f64) {
        o.x_c = [0.0; 10];
        o.x_c[species] = x;
    }

    /// An object whose burning is `l` in each group, at c as much as over the object, L = their sum, H and He fuel gone.
    fn burning(l: [f64; 7]) -> Object {
        let mut o = cloud();
        o.l_group = l;
        o.q_c = l;
        o.l_nuc = l.iter().filter(|v| **v > 0.0).sum();
        o.luminosity = o.l_nuc.max(1.0);
        o.x_c = [0.0; 10];
        o
    }

    /// m_ch_sb for the shipped physics (assets/calibration.json, physics_hash 56df8ade…, the same value as 8ca855fb…'s); the test objects weigh 100.
    const M_CH: f64 = 3216.4199210979236;

    fn stage_of(o: &Object, h: &History) -> Stage {
        classify(o, h, 100.0, M_CH)
    }

    /// N_Fe converting in an object past m_ch_sb, the only one whose core collapses (§0.5 [M0-D92]); the unbound masses the
    /// tests give it are shares of its mass.
    fn collapsing() -> Object {
        let mut o = burning([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0]);
        o.mass = 2.0 * M_CH;
        o
    }

    #[test]
    fn the_rows_are_in_the_contracts_order() {
        let ids: Vec<&str> = stage_order().iter().map(|s| s.id()).collect();
        assert_eq!(
            ids,
            [
                "black_hole", "supernova", "neutron_star", "core_collapse", "white_dwarf", "failed_star", "planetary_nebula", "iron_core",
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

    /// A collapsed iron centre as N_Fe's gate leaves it (§0.5 [M0-D79]): X_n 0.46, the rest iron — Y_e,c 0.54 × 26/56 = 0.251,
    /// degenerate, its grid flow at 0.49 √(GM/R) (M0-R5's M1 at a_n_fe 100).
    fn neutronized() -> Object {
        let mut o = cloud();
        o.x_c = [0.0; 10];
        (o.x_c[X_FE], o.x_c[X_N]) = (0.54, 0.46);
        o.d_c = 0.9999;
        o.v_r = -4.9;
        o
    }

    /// The centre's X as helium and neutron matter: Y_e,c = X_He / 2, exact in f64 at the row's 0.35.
    fn with_y_e(o: &mut Object, x_he: f64) {
        o.x_c = [0.0; 10];
        (o.x_c[X_HE], o.x_c[X_N]) = (x_he, 1.0 - x_he);
    }

    #[test]
    fn row_neutron_star() {
        let mut o = neutronized();
        assert_eq!(stage_of(&o, &collapsed()), Stage::NeutronStar, "the gate's centre, moving: no |v_r| term");
        let unlit = History { ignited: false, ..collapsed() };
        assert_eq!(stage_of(&o, &unlit), Stage::NeutronStar, "the row reads no ignition");
        with_y_e(&mut o, 0.7);
        assert_eq!(electron_fraction(&o.x_c), NEUTRON_Y_E);
        assert_eq!(stage_of(&o, &collapsed()), Stage::NeutronStar, "Y_e,c exactly 0.35 is within");
        with_y_e(&mut o, 0.7002);
        assert_ne!(stage_of(&o, &collapsed()), Stage::NeutronStar, "Y_e,c 0.3501: not neutron matter enough");
        let mut o = neutronized();
        o.d_c = 0.5;
        assert_eq!(stage_of(&o, &collapsed()), Stage::NeutronStar, "d_c exactly 0.5 is within");
        o.d_c = 0.49;
        assert_ne!(stage_of(&o, &collapsed()), Stage::NeutronStar, "not degenerate");
    }

    #[test]
    fn a_neutron_star_needs_a_passed_core_collapse() {
        // The 6,702.064 life's frame at step 1,553 (RTX 4090, M0-D81's trace): Y_e,c 0.342 a frame after 0.464, N_Fe converting.
        let mut o = neutronized();
        o.mass = 6_631.5;
        (o.x_c[X_FE], o.x_c[X_N]) = (0.7366, 0.2634);
        o.l_group[G_N] = -5.22e7;
        assert!(electron_fraction(&o.x_c) <= NEUTRON_Y_E);
        assert_eq!(stage_of(&o, &ignited()), Stage::CoreCollapse, "never collapsed: the collapse shows first");
        assert_eq!(stage_of(&o, &collapsed()), Stage::NeutronStar, "passed core collapse: the remnant");
        o.l_group[G_N] = 0.0;
        assert_ne!(stage_of(&o, &ignited()), Stage::NeutronStar, "no collapse behind it, none converting: no neutron star");
    }

    #[test]
    fn a_white_dwarf_never_matches_the_neutron_star_row() {
        // Carbon-oxygen (Y_e 0.5) and pure iron (0.464), cold, wholly degenerate and settled (§0.5 [M0-D140]): white dwarfs.
        let mut o = degenerate();
        o.d_c = 1.0;
        for (species, y_e) in [(2, 0.5), (3, 0.5), (X_FE, 26.0 / 56.0)] {
            with_x(&mut o, species, 1.0);
            assert_eq!(electron_fraction(&o.x_c), y_e);
            assert_eq!(stage_of(&o, &ignited()), Stage::WhiteDwarf, "species {species}");
        }
        // The stall M0-R5 measured at a_n_fe 10⁻³: X_n,c 0.011, Y_e,c 0.459 — an iron white dwarf, not a neutron star.
        (o.x_c[X_FE], o.x_c[X_N]) = (0.989, 0.011);
        assert_eq!(stage_of(&o, &ignited()), Stage::WhiteDwarf);
    }

    #[test]
    fn a_sink_makes_the_black_hole_before_the_neutron_star() {
        let mut o = neutronized();
        o.sinks = 1;
        assert_eq!(stage_of(&o, &collapsed()), Stage::BlackHole);
    }

    #[test]
    fn a_supernova_shows_before_the_neutron_star() {
        let mut o = neutronized();
        o.unbound = 10.0;
        let h = History { ignited: true, last_collapse: Some(90.0), ..History::default() };
        assert_eq!(classify(&o, &h, 100.0, M_CH), Stage::Supernova, "inside the 20 t.u. with 0.1 M unbound");
        assert_eq!(classify(&o, &h, 110.5, M_CH), Stage::NeutronStar, "past the window the remnant shows");
    }

    #[test]
    fn the_neutron_star_shows_before_core_collapse() {
        // The gate re-opens for a frame as the core sloshes (L_N < 0): the neutron star holds, its ending's hold not restarted.
        let mut o = neutronized();
        o.mass = 2.0 * M_CH;
        o.l_group[G_N] = -620.0;
        assert_eq!(stage_of(&o, &collapsed()), Stage::NeutronStar);
        o.d_c = 0.49;
        assert_eq!(stage_of(&o, &collapsed()), Stage::CoreCollapse, "not yet degenerate: still collapsing");
    }

    #[test]
    fn row_supernova_inside_the_window_with_unbound_mass() {
        let mut o = cloud();
        o.unbound = 10.0;
        let h = History { ignited: true, last_collapse: Some(90.0), ..History::default() };
        assert_eq!(classify(&o, &h, 100.0, M_CH), Stage::Supernova);
        assert_ne!(classify(&o, &h, 110.5, M_CH), Stage::Supernova, "past 20 t.u.");
        assert_eq!(classify(&o, &h, 110.0, M_CH), Stage::Supernova, "exactly 20 t.u. is within");
        o.unbound = 9.9;
        assert_ne!(classify(&o, &h, 100.0, M_CH), Stage::Supernova, "under 0.1 M unbound");
        o.unbound = 10.0;
        assert_ne!(classify(&o, &ignited(), 100.0, M_CH), Stage::Supernova, "never passed core collapse");
    }

    #[test]
    fn row_core_collapse_n_fe_converting() {
        let mut o = collapsing();
        assert_eq!(stage_of(&o, &ignited()), Stage::CoreCollapse);
        o.l_group[6] = 0.0;
        assert_ne!(stage_of(&o, &ignited()), Stage::CoreCollapse);
    }

    #[test]
    fn a_core_collapses_only_past_m_ch_sb() {
        let mut o = collapsing();
        o.mass = M_CH.next_up();
        assert_eq!(stage_of(&o, &ignited()), Stage::CoreCollapse, "just past m_ch_sb");
        o.mass = M_CH;
        assert_ne!(stage_of(&o, &ignited()), Stage::CoreCollapse, "exactly m_ch_sb is a white dwarf's, no collapsing core");
        assert_eq!(classify(&o, &ignited(), 100.0, f64::INFINITY), Stage::CoreCollapse, "no ceiling in the run: none refused");
        // M0-D92, RTX 4090: the M₀ life (3,351.032) at step 133,534, t 1512.08 — its flashed iron centre, cold, overshooting
        // into N_Fe's gate (Σ_c 347.2) while the envelope leaves; the same physics then bounced to a 2,364 white dwarf.
        let mut o = degenerate();
        with_x(&mut o, X_FE, 0.9888);
        o.x_c[X_N] = 0.0062;
        (o.mass, o.unbound, o.d_c) = (2_922.0, 474.7, 1.0);
        (o.luminosity, o.l_nuc) = (3.363e5, 6_908.0);
        o.l_group[G_N] = -2.337e5;
        assert_eq!(stage_of(&o, &ignited()), Stage::PlanetaryNebula, "a sub-Chandrasekhar remnant shedding its shell");
    }

    #[test]
    fn supernova_before_core_collapse() {
        let mut o = burning([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0]);
        o.unbound = 50.0;
        let h = History { ignited: true, last_collapse: Some(99.0), ..History::default() };
        assert_eq!(classify(&o, &h, 100.0, M_CH), Stage::Supernova);
    }

    /// Cold-pressure supported and settled: 2P/|W| = 0.8 at rest.
    fn degenerate() -> Object {
        let mut o = cloud();
        o.d_c = 0.5;
        o.pressure = 4.0;
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
    fn a_failed_star_is_settled() {
        // §0.5 [M0-D134]: the preset cloud's centre at the drop and in free fall reads d_c 0.55–0.65 (M0-D134's lives).
        let mut o = degenerate();
        o.d_c = 0.65;
        o.v_r = -0.6;
        assert_eq!(stage_of(&o, &fresh()), Stage::Collapsing, "falling in: a collapse, however cold");
        o.v_r = -0.075;
        assert_eq!(stage_of(&o, &fresh()), Stage::Cloud, "between the collapse and rest");
        o.v_r = 0.05;
        assert_eq!(stage_of(&o, &fresh()), Stage::FailedStar, "|v_r| = 0.05 √(GM/R) is within");
        o.v_r = 0.0;
        o.pressure = 3.9;
        assert_eq!(stage_of(&o, &fresh()), Stage::Cloud, "at rest, short of virial: the dropped cloud (2P/|W| ≈ 0.49)");
    }

    /// `frames` frames of `o` at sim times from `t0`: the shown stage after them.
    fn shown_after(t: &mut StageTracker, o: &Object, t0: f64, frames: u32) -> Option<Stage> {
        for f in 0..frames {
            t.update(&[o.clone()], t0 + f64::from(f));
        }
        t.stage(o.id).unwrap().shown
    }

    #[test]
    fn the_shown_stage_never_steps_back() {
        // §0.5 [M0-D134]: before ignition cloud → collapsing → protostar, forward only; the raw stage still reads the rows.
        let mut falling = cloud();
        falling.v_r = -1.0;
        let mut supported = cloud();
        supported.pressure = 4.0;
        let mut t = StageTracker::new(M_CH);
        assert_eq!(shown_after(&mut t, &cloud(), 0.0, 3), Some(Stage::Cloud));
        assert_eq!(shown_after(&mut t, &falling, 3.0, 3), Some(Stage::Collapsing));
        assert_eq!(shown_after(&mut t, &cloud(), 6.0, 5), Some(Stage::Collapsing), "a cloud after the collapse");
        assert_eq!(t.stage(1).unwrap().raw, Stage::Cloud);
        assert_eq!(shown_after(&mut t, &supported, 11.0, 3), Some(Stage::Protostar));
        assert_eq!(shown_after(&mut t, &falling, 14.0, 5), Some(Stage::Protostar), "a collapse after the protostar");
        assert_eq!(shown_after(&mut t, &cloud(), 19.0, 5), Some(Stage::Protostar));
        // A failed star: no cloud after it; a new collapse shows (its ending hold restarts, §1.9.5).
        let mut t = StageTracker::new(M_CH);
        assert_eq!(shown_after(&mut t, &degenerate(), 0.0, 3), Some(Stage::FailedStar));
        assert_eq!(shown_after(&mut t, &cloud(), 3.0, 5), Some(Stage::FailedStar), "a cloud after the failed star");
        assert_eq!(shown_after(&mut t, &falling, 8.0, 3), Some(Stage::Collapsing));
        // Ignited: none of the three again, whatever the rows fall through to.
        let mut lit = cloud();
        lit.l_group = [10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        lit.l_nuc = 10.0;
        let mut t = StageTracker::new(M_CH);
        assert_eq!(shown_after(&mut t, &lit, 0.0, 3), Some(Stage::MainSequence));
        let mut dim = lit.clone();
        dim.l_group[0] = 0.0;
        dim.l_nuc = 0.0;
        assert_eq!(shown_after(&mut t, &dim, 3.0, 5), Some(Stage::MainSequence), "burning nothing: the rows fall through to cloud");
        assert_eq!(t.stage(1).unwrap().raw, Stage::Cloud);
        // An ignition before any stage held: the cloud never shows.
        let mut t = StageTracker::new(M_CH);
        t.update(&[lit.clone()], 0.0);
        assert_eq!(shown_after(&mut t, &dim, 1.0, 5), None);
    }

    #[test]
    fn steps_back_reads_only_the_three_early_labels() {
        for s in stage_order() {
            let early = matches!(s, Stage::Cloud | Stage::Collapsing | Stage::Protostar);
            assert_eq!(steps_back(s, Some(Stage::MainSequence), true), early, "{s:?} once ignited");
            assert!(!steps_back(s, None, false), "{s:?}: nothing shown yet");
        }
        assert!(!steps_back(Stage::Collapsing, Some(Stage::Cloud), false));
        assert!(!steps_back(Stage::Protostar, Some(Stage::Collapsing), false));
        assert!(steps_back(Stage::Cloud, Some(Stage::Protostar), false));
        assert!(!steps_back(Stage::Protostar, Some(Stage::FailedStar), false));
    }

    /// The Sun-like preset's post-flash frames on the RTX 4090 (logs/M0-D140.measure.log; lavapipe the same): ignited, its
    /// hydrogen unburnt (X_H,c 0.7318), degenerate by the cold pressure, L_nuc 0 — raw white_dwarf from step 401 to 657 before
    /// §0.5 [M0-D140]. `v_r` in √(GM/R) (the cloud's is 1), `virial` its 2P/|W| (pressure / 5 against W = −10).
    fn re_expanding(d_c: f64, v_r: f64, virial: f64, luminosity: f64) -> Object {
        let mut o = degenerate();
        (o.mass, o.d_c, o.v_r, o.pressure, o.luminosity) = (3134.0, d_c, v_r, 5.0 * virial, luminosity);
        o.x_c[X_H] = 0.7318;
        o
    }

    #[test]
    fn a_white_dwarf_is_settled() {
        // §0.5 [M0-D140]: step 433, re-expanding — shown «White dwarf» before; step 545, near rest but short of virial.
        assert_ne!(stage_of(&re_expanding(0.5947, 0.1801, 0.877, 7161.0), &ignited()), Stage::WhiteDwarf, "step 433, +0.18 √(GM/R)");
        assert_ne!(stage_of(&re_expanding(0.6052, 0.0149, 0.787, 758.4), &ignited()), Stage::WhiteDwarf, "step 545, 2P/|W| 0.787");
        let mut o = degenerate();
        o.v_r = 0.05;
        assert_eq!(stage_of(&o, &ignited()), Stage::WhiteDwarf, "|v_r| = 0.05 √(GM/R) and 2P/|W| = 0.8 are within");
        o.v_r = -0.0501;
        assert_ne!(stage_of(&o, &ignited()), Stage::WhiteDwarf, "falling at 0.0501 √(GM/R)");
        o.v_r = 0.0;
        o.pressure = 3.99;
        assert_ne!(stage_of(&o, &ignited()), Stage::WhiteDwarf, "2P/|W| 0.798");
    }

    /// m_up_sb's light end at step 600,000 (the 3,351.032 life, RTX 4090, laserax-ai-server, 2026-11-06): the checkpoint's
    /// snapshot of object 1, its K_t read 16 steps on (logs/M0-D160.probe_remnant.log) — a cold iron core inside a Keplerian
    /// disk, 2P/|W| 0.356, 2(P + K_t)/|W| 0.934.
    fn spinning_remnant() -> Object {
        let mut o = cloud();
        (o.mass, o.gas_mass, o.centre, o.radius, o.tau_dyn) = (309.054, 309.054, [299.5, 201.5], 85.2209, 44.7509);
        (o.luminosity, o.luminosity_own, o.sigma_c, o.t_c, o.d_c, o.v_r) = (0.42997, 0.0, 1.6228, 0.053935, 0.99604, 2.4358e-4);
        (o.thermal, o.kinetic, o.kinetic_t, o.pressure, o.potential) = (96.797, 655.23, 622.38, 383.77, -2155.67);
        o.x_c = [1.9083e-3, 9.087e-4, 4.017e-5, 9.141e-4, 9.6e-11, 2.5e-11, 5.5e-7, 2.4e-7, 0.99623, 0.0];
        o
    }

    #[test]
    fn a_spinning_remnant_is_settled() {
        // §0.5 [M0-D160]: held by its rotation, a white dwarf — before, iron_core for 467,455 steps and calibrate's exit 6.
        let mut o = spinning_remnant();
        assert_eq!(stage_of(&o, &ignited()), Stage::WhiteDwarf, "2(P + K_t)/|W| 0.934");
        // Its centre under §0.5 [M0-D168]'s d_c 0.9, so only the support can make it a remnant.
        o.d_c = 0.89;
        assert_eq!(stage_of(&o, &ignited()), Stage::WhiteDwarf, "2(P + K_t)/|W| 0.934 at d_c 0.89");
        o.kinetic_t = 0.0;
        assert_eq!(stage_of(&o, &ignited()), Stage::IronCore, "its rotation left out: 2P/|W| 0.356");
        // Each edge: 2(P + K_t)/|W| = 0.8 is within, a hair under is not, and motion that is no rotation counts for nothing.
        let mut o = degenerate();
        (o.pressure, o.kinetic_t) = (1.0, 3.0);
        assert_eq!(stage_of(&o, &ignited()), Stage::WhiteDwarf, "2(P + K_t)/|W| = 0.8");
        o.unbound = 5.0;
        assert_eq!(stage_of(&o, &ignited()), Stage::PlanetaryNebula, "the nebula's row reads the same test");
        o.unbound = 0.0;
        assert_eq!(stage_of(&o, &fresh()), Stage::FailedStar, "the failed star's too");
        o.d_c = 0.0;
        assert_eq!(stage_of(&o, &fresh()), Stage::Protostar, "and the protostar's");
        o.d_c = 0.5;
        (o.kinetic_t, o.kinetic) = (2.99, 100.0);
        assert_ne!(stage_of(&o, &ignited()), Stage::WhiteDwarf, "2(P + K_t)/|W| 0.798, whatever K");
    }

    #[test]
    fn the_sun_like_s_false_white_dwarf_has_no_rotation() {
        // §0.5 [M0-D160] on M0-D140's frames (RTX 4090, logs/M0-D160.probe_lives.log): near rest at step 577, 2P/|W| 0.7979,
        // K 0.00452 |W| — all motion counted it read 0.807, a white dwarf with its hydrogen unburnt — and K_t 10⁻⁵ |W|.
        let mut o = re_expanding(0.6061, -0.0376, 0.7979, 410.7);
        (o.kinetic, o.kinetic_t) = (0.0452, 1.0e-4);
        assert_ne!(stage_of(&o, &ignited()), Stage::WhiteDwarf, "step 577: 2(P + K_t)/|W| 0.7979");
    }

    /// m_up_sb's 3,351.032 life on the burn clock at step 450,001 (t 3,464.13; M0-D165's stopped calibration, RTX 4090,
    /// laserax-ai-server): the checkpoint's snapshot of object 1 (tasks/M0-D168.md) — a 7 × 7-cell C/O remnant, at rest and wholly
    /// degenerate, 2(P + K_t)/|W| 0.779, a hydrogen trace burning 3.85 against L 1.79 × 10⁵.
    fn compact_remnant() -> Object {
        let mut o = cloud();
        (o.mass, o.gas_mass, o.centre, o.radius, o.tau_dyn) = (2489.2328, 2489.2328, [299.5, 199.5], 2.0, 0.05669075);
        (o.luminosity, o.luminosity_own, o.sigma_c, o.t_c, o.d_c, o.v_r) = (178969.36, 178969.36, 243.0163, 0.050049, 0.99994, -1.524513);
        o.l_group = [3.854963, 0.872294, 9.504e-4, 0.0688577, 0.356757, 0.0, 0.0];
        o.l_nuc = 5.153822;
        (o.thermal, o.kinetic, o.kinetic_t, o.pressure, o.potential) = (127169.44, 4608.346, 17.26895, 829872.33, -2131445.5);
        o.x_c = [0.0279036, 0.0362437, 0.0223456, 0.861197, 0.0308624, 0.0129707, 0.0059885, 0.0024883, 2.348e-9, 0.0];
        o
    }

    #[test]
    fn a_compact_remnant_at_rest_is_a_white_dwarf() {
        // §0.5 [M0-D168]: short of virial but at rest on a wholly degenerate centre — before, main_sequence on its hydrogen trace.
        let mut o = compact_remnant();
        assert_eq!(stage_of(&o, &ignited()), Stage::WhiteDwarf, "d_c 0.99994, |v_r| 0.043 √(GM/R), 2(P + K_t)/|W| 0.779");
        o.d_c = 0.9;
        assert_eq!(stage_of(&o, &ignited()), Stage::WhiteDwarf, "d_c 0.9 is within");
        o.d_c = 0.8999;
        assert_eq!(stage_of(&o, &ignited()), Stage::MainSequence, "d_c 0.8999: settled() alone, which it fails");
        o.d_c = 0.99994;
        o.v_r = -0.0501 * o.radius / o.tau_dyn;
        assert_eq!(stage_of(&o, &ignited()), Stage::MainSequence, "falling at 0.0501 √(GM/R)");
        o.v_r = 0.05 * o.radius / o.tau_dyn;
        assert_eq!(stage_of(&o, &ignited()), Stage::WhiteDwarf, "|v_r| = 0.05 √(GM/R) is within");
        o.v_r = 0.0;
        o.l_nuc = 0.01 * o.luminosity;
        assert_ne!(stage_of(&o, &ignited()), Stage::WhiteDwarf, "burning 0.01 L");
        // The other remnant rows keep settled() alone: the same object shedding 5 % of its mass is no planetary nebula.
        let mut o = compact_remnant();
        o.unbound = 0.05 * o.mass;
        assert_ne!(stage_of(&o, &ignited()), Stage::PlanetaryNebula, "planetary_nebula reads settled() alone");
    }

    #[test]
    fn a_planetary_nebula_is_settled() {
        // §0.5 [M0-D140]: the remnant shedding its shell is settled too; moving, or short of virial, it is not.
        let mut o = degenerate();
        o.unbound = 5.0;
        o.v_r = 0.05;
        assert_eq!(stage_of(&o, &ignited()), Stage::PlanetaryNebula, "|v_r| = 0.05 √(GM/R) is within");
        o.v_r = 0.0501;
        assert_ne!(stage_of(&o, &ignited()), Stage::PlanetaryNebula, "moving at 0.0501 √(GM/R)");
        o.v_r = 0.0;
        o.pressure = 3.99;
        assert_ne!(stage_of(&o, &ignited()), Stage::PlanetaryNebula, "2P/|W| 0.798");
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
    fn a_white_dwarf_holds_at_most_m_ch_sb() {
        let mut o = degenerate();
        o.mass = M_CH;
        assert_eq!(stage_of(&o, &ignited()), Stage::WhiteDwarf, "exactly m_ch_sb is within");
        o.mass = M_CH.next_up();
        assert_ne!(stage_of(&o, &ignited()), Stage::WhiteDwarf, "past m_ch_sb");
        with_x(&mut o, X_FE, 1.0);
        assert_eq!(stage_of(&o, &ignited()), Stage::IronCore, "an iron centre past m_ch_sb is the star's iron core");
    }

    #[test]
    fn a_planetary_nebula_holds_at_most_m_ch_sb() {
        let mut o = degenerate();
        (o.mass, o.unbound) = (M_CH, 0.05 * M_CH);
        assert_eq!(stage_of(&o, &ignited()), Stage::PlanetaryNebula, "exactly m_ch_sb is within");
        o.mass = M_CH.next_up();
        assert_ne!(stage_of(&o, &ignited()), Stage::PlanetaryNebula, "past m_ch_sb");
        with_x(&mut o, X_FE, 1.0);
        assert_eq!(stage_of(&o, &ignited()), Stage::IronCore);
    }

    #[test]
    fn a_heavy_stars_flashed_iron_core_is_no_white_dwarf() {
        // M0-D81, RTX 4090: the 4,572.394 life at step 321 — the whole star, its centre burnt to iron in one flash, quiet,
        // degenerate, pulsing at v_r 0.42 √(GM/R) — and at step 1,217, shedding 6 % of its mass; it collapsed at step 1,521.
        let mut o = degenerate();
        with_x(&mut o, X_FE, 1.0);
        (o.mass, o.d_c, o.l_nuc, o.luminosity, o.unbound) = (4570.5, 0.588, 8.9e-13, 1.0, 0.0243 * 4570.5);
        (o.radius, o.tau_dyn, o.v_r) = (10.0, 2.705, 0.419 * 10.0 / 2.705);
        assert_eq!(stage_of(&o, &ignited()), Stage::IronCore);
        (o.mass, o.d_c, o.unbound) = (4543.3, 0.764, 0.0615 * 4543.3);
        assert_eq!(stage_of(&o, &ignited()), Stage::IronCore);
        // m_up_sb's light side ended white_dwarf at 1,978.8 (the 3,639.109 life, RTX 5090, 2026-10-21): settled, it still does
        // (§0.5 [M0-D140]); pulsing at 0.42 √(GM/R) it is no remnant yet.
        (o.mass, o.unbound) = (1978.8, 0.0);
        assert_ne!(stage_of(&o, &ignited()), Stage::WhiteDwarf, "pulsing: not settled");
        o.v_r = 0.0;
        assert_eq!(stage_of(&o, &ignited()), Stage::WhiteDwarf);
    }

    #[test]
    fn row_iron_core() {
        let mut o = cloud();
        with_x(&mut o, X_FE, 0.5);
        assert_eq!(stage_of(&o, &ignited()), Stage::IronCore);
        // §0.5 [M0-D169]: an iron centre while a fuel still burns at c is not yet an iron core; the star's L_nuc is not read.
        (o.l_nuc, o.luminosity) = (10.0, 10.0);
        assert_eq!(stage_of(&o, &ignited()), Stage::IronCore, "the star burning its shells");
        o.q_c[5] = 1.0;
        o.x_c[FUEL_SI] = 0.001;
        assert_eq!(stage_of(&o, &ignited()), Stage::SiBurn, "silicon lit at c, its fuel at 10⁻³");
        o.x_c[FUEL_SI] = 0.0009;
        assert_eq!(stage_of(&o, &ignited()), Stage::IronCore, "silicon spent at c");
        o.x_c[7] = 0.001;
        assert_ne!(stage_of(&o, &ignited()), Stage::IronCore, "sulphur still burning in the silicon group");
        o.x_c[7] = 0.0;
        o.q_c[6] = -5.0;
        assert_eq!(stage_of(&o, &ignited()), Stage::IronCore, "N_Fe absorbing burns no fuel");
    }

    #[test]
    fn the_burning_rows_read_the_centre_under_a_brighter_shell() {
        // §0.5 [M0-D169]: the Massive's centre burning oxygen (L_O to 5.5 × 10⁴) under a hydrogen shell of 1.2 × 10⁵ (RTX 4090,
        // reports/heavy_end.md § 1.2) — shown he_shell on the star's L_g, o_burn on what c itself burns.
        let mut o = cloud();
        o.l_group = [1.2e5, 2.0e3, 0.0, 0.0, 5.5e4, 1.0e3, 0.0];
        o.l_nuc = 1.78e5;
        o.luminosity = 1.8e5;
        o.q_c = [0.0, 0.0, 0.0, 0.0, 3.0, 0.6, 0.0];
        o.x_c = [0.0; 10];
        o.x_c[FUEL_O] = 0.6;
        o.x_c[FUEL_SI] = 0.3;
        assert_eq!(stage_of(&o, &ignited()), Stage::OBurn);
        // Silicon leads at c: si_burn, the shell still the star's largest.
        o.q_c[5] = 3.5;
        assert_eq!(stage_of(&o, &ignited()), Stage::SiBurn);
        // A centre that burns nothing (cold, or under every lit rung): no burning row.
        o.q_c = [0.0; 7];
        assert_eq!(stage_of(&o, &ignited()), Stage::HeShell);
        // Helium leads at c: not a late burning row, whatever the star's oxygen does.
        o.q_c = [0.0, 5.0, 0.0, 0.0, 2.0, 0.0, 0.0];
        o.l_group[1] = 3.0e4;
        o.x_c[X_HE] = 0.2;
        assert_eq!(stage_of(&o, &ignited()), Stage::HeCore);
        // The iron centre (X_Fe,c 0.999 from t 250 under dn2) with nothing left to burn at c: iron_core, the shells burning on.
        o.x_c = [0.0; 10];
        o.x_c[X_FE] = 0.999;
        o.x_c[FUEL_SI] = 0.0009;
        o.q_c = [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        assert_eq!(stage_of(&o, &ignited()), Stage::IronCore, "the star's L_nuc is not read");
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
        // §0.5 [M0-D141]: ignited, hydrogen the largest burning group and at least a tenth of L_nuc, X_H,c ≥ 10⁻³.
        let mut o = cloud();
        o.l_group = [5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        o.l_nuc = 5.0;
        o.luminosity = 10.0;
        assert_eq!(stage_of(&o, &ignited()), Stage::MainSequence);
        assert_ne!(stage_of(&o, &fresh()), Stage::MainSequence, "never ignited");
        with_x(&mut o, X_H, 0.0009);
        assert_ne!(stage_of(&o, &ignited()), Stage::MainSequence, "the centre's hydrogen spent");
        with_x(&mut o, X_H, 0.001);
        assert_eq!(stage_of(&o, &ignited()), Stage::MainSequence, "X_H,c = 10⁻³ is within");
        o.l_group = [0.0; 7];
        o.l_nuc = 0.0;
        assert_ne!(stage_of(&o, &ignited()), Stage::MainSequence, "burning nothing");
        o.luminosity = 100.0;
        o.l_group = [1.0, 1.1, 0.0, 0.0, 0.0, 0.0, 0.0];
        o.l_nuc = 2.1;
        assert_ne!(stage_of(&o, &ignited()), Stage::MainSequence, "helium the larger group (L_He under 0.1 L, no he_core)");
        o.l_group = [1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        o.l_nuc = 2.0;
        assert_eq!(stage_of(&o, &ignited()), Stage::MainSequence, "a tie with helium is still the largest");
        o.l_group = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, -50.0];
        o.l_nuc = 1.0;
        assert_eq!(stage_of(&o, &ignited()), Stage::MainSequence, "L_N absorbs, it burns nothing");
    }

    #[test]
    fn the_main_sequence_reads_its_burning_not_its_light() {
        // §0.5 [M0-D141]: the Sun-like after its flash on the RTX 4090 (logs/M0-D140.iterate.log), ignited at step 273, burning
        // hydrogen outside [0.5 L, 2 L] — raw cloud on every frame before; L_H = L_nuc, nothing else burns.
        let frame = |l: f64, l_nuc: f64, x_h: f64| {
            let mut o = cloud();
            (o.luminosity, o.l_nuc, o.d_c, o.v_r) = (l, l_nuc, 0.57, 0.1);
            o.l_group[G_H] = l_nuc;
            o.x_c[X_H] = x_h;
            o
        };
        assert_eq!(stage_of(&frame(1.015e5, 2.100e4, 0.7326), &ignited()), Stage::MainSequence, "step 289, L_H 0.21 L");
        assert_eq!(stage_of(&frame(1.362e4, 9.646e1, 0.7318), &ignited()), Stage::MainSequence, "step 401, L_H 0.007 L");
        assert_eq!(stage_of(&frame(1.871e1, 1.605e3, 0.7317), &ignited()), Stage::MainSequence, "step 769, L_H 86 L");
        assert_eq!(stage_of(&frame(7.161e3, 0.0, 0.7318), &ignited()), Stage::Cloud, "step 433, burning nothing");
        // The window still ignites (`ignites`), not the row: the same burning never ignited is no main sequence.
        assert_ne!(stage_of(&frame(1.015e5, 2.100e4, 0.7326), &fresh()), Stage::MainSequence);
    }

    #[test]
    fn a_flash_inside_a_dark_cloud_does_not_ignite() {
        // M0-V8's step 225: L_H = 277 against L = 2.1 × 10⁻⁷ — no ignition, the cloud's rows still open.
        let mut t = StageTracker::new(M_CH);
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
        let mut t = StageTracker::new(M_CH);
        flash.luminosity = 138.4;
        assert!(t.update(&[flash.clone()], 1.0).is_empty(), "L_H just over 2 L");
        flash.luminosity = 554.0;
        assert_eq!(kinds(&t.update(&[flash.clone()], 2.0)), vec![EventKind::Ignition], "L_H = 0.5 L");
        let mut t = StageTracker::new(M_CH);
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
        assert_eq!(classify(&o, &fresh(), 0.0, M_CH), classify(&o, &fresh(), 1e6, M_CH));
    }

    fn id(mut o: Object, id: u32) -> Object {
        o.id = id;
        o
    }

    #[test]
    fn a_stage_shows_after_three_identical_results() {
        let mut t = StageTracker::new(M_CH);
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
        let mut t = StageTracker::new(M_CH);
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
        let mut t = StageTracker::new(M_CH);
        let mut o = collapsing();
        o.x_c[X_H] = 0.5;
        t.update(&[cloud()], 0.0);
        t.update(&[o.clone()], 1.0);
        assert_eq!(t.stage(1).unwrap().shown, Some(Stage::CoreCollapse));
        o.unbound = 0.5 * o.mass;
        t.update(&[o], 2.0);
        assert_eq!(t.stage(1).unwrap().shown, Some(Stage::Supernova));
    }

    fn kinds(events: &[Event]) -> Vec<EventKind> {
        events.iter().map(|e| e.kind).collect()
    }

    #[test]
    fn ignition_fires_once_and_the_flag_stays_on() {
        let mut t = StageTracker::new(M_CH);
        assert!(t.update(&[cloud()], 0.0).is_empty());
        let mut lit = cloud();
        lit.l_group = [5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        lit.l_nuc = 5.0;
        let events = t.update(&[lit.clone()], 1.0);
        assert_eq!(events, vec![Event { object: 1, kind: EventKind::Ignition, time: 1.0, by: Some(Ignition::Light) }]);
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
        let mut t = StageTracker::new(M_CH);
        let mut o = collapsing();
        let first = t.update(&[o.clone()], 1.0);
        assert_eq!(kinds(&first), vec![EventKind::CoreCollapse]);
        assert!(t.update(&[o.clone()], 2.0).is_empty(), "core collapse again: nothing");
        o.unbound = 0.5 * o.mass;
        let boom = t.update(&[o.clone()], 3.0);
        assert_eq!(kinds(&boom), vec![EventKind::Supernova], "the collapse row is shadowed by supernova while it holds");
        assert!(t.update(&[o.clone()], 4.0).is_empty());
        // Collapse again later: still once.
        let mut o2 = o;
        o2.unbound = 0.0;
        assert!(t.update(&[o2], 5.0).is_empty());
    }

    #[test]
    fn a_centre_neutronized_inside_one_frame_still_collapses_then_explodes() {
        // The 6,702.064 life's frames (RTX 4090, M0-D81's trace): an iron core at step 1,537, Y_e,c 0.342 at 1,553 with N_Fe
        // converting and 16.7 % unbound, 0.233 at 1,569; neutron_star once the ejecta are under 0.1 M.
        let mut t = StageTracker::new(M_CH);
        let mut iron = degenerate();
        with_x(&mut iron, X_FE, 1.0);
        iron.mass = 6_631.5;
        iron.unbound = 0.160 * iron.mass;
        t.update(&[iron.clone()], 11.306);
        let mut o = neutronized();
        o.mass = iron.mass;
        o.unbound = 0.167 * o.mass;
        (o.x_c[X_FE], o.x_c[X_N]) = (0.7366, 0.2634);
        o.l_group[G_N] = -5.22e7;
        assert_eq!(kinds(&t.update(&[o.clone()], 11.379)), vec![EventKind::CoreCollapse]);
        assert_eq!(t.stage(1).unwrap().shown, Some(Stage::CoreCollapse));
        (o.x_c[X_FE], o.x_c[X_N]) = (0.50, 0.50);
        assert_eq!(kinds(&t.update(&[o.clone()], 11.451)), vec![EventKind::Supernova]);
        assert_eq!(t.stage(1).unwrap().shown, Some(Stage::Supernova));
        o.unbound = 0.05 * o.mass;
        for k in 0..3 {
            t.update(&[o.clone()], 21.0 + f64::from(k));
        }
        assert_eq!(t.stage(1).unwrap().shown, Some(Stage::NeutronStar), "the remnant after the blast");
    }

    #[test]
    fn the_supernova_needs_a_collapse_in_the_last_20_tu() {
        let mut t = StageTracker::new(M_CH);
        let mut o = cloud();
        o.unbound = 50.0;
        // Unbound mass with no collapse behind it: no supernova.
        assert!(!kinds(&t.update(&[o.clone()], 1.0)).contains(&EventKind::Supernova));
        t.update(&[collapsing()], 2.0);
        assert!(kinds(&t.update(&[o.clone()], 30.0)).is_empty(), "28 t.u. later");
        assert_ne!(t.stage(1).unwrap().raw, Stage::Supernova);
        let mut t = StageTracker::new(M_CH);
        t.update(&[collapsing()], 2.0);
        assert!(kinds(&t.update(&[o], 12.0)).contains(&EventKind::Supernova));
    }

    #[test]
    fn sink_formed_fires_once_per_object_and_events_are_per_object() {
        let mut t = StageTracker::new(M_CH);
        let mut a = cloud();
        a.sinks = 1;
        let b = id(cloud(), 2);
        let events = t.update(&[a.clone(), b.clone()], 1.0);
        assert_eq!(events, vec![Event { object: 1, kind: EventKind::SinkFormed, time: 1.0, by: None }]);
        assert!(t.update(&[a, b.clone()], 2.0).is_empty());
        let mut b2 = b;
        b2.sinks = 1;
        assert_eq!(t.update(&[b2], 3.0), vec![Event { object: 2, kind: EventKind::SinkFormed, time: 3.0, by: None }]);
        assert!(t.stage(1).is_none(), "an object gone from the frame is forgotten");
    }

    #[test]
    fn a_frame_with_ignition_classifies_with_the_flag_on() {
        // The frame L_H reaches half L: this frame's result already sees `ignited` (main sequence, not failed/cloud rows).
        let mut t = StageTracker::new(M_CH);
        let mut o = cloud();
        o.l_group = [5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        o.l_nuc = 5.0;
        t.update(&[o], 1.0);
        assert_eq!(t.stage(1).unwrap().raw, Stage::MainSequence);
    }

    /// M0-V9's 3.74 M₀ cloud (tasks/M0-V9.md, P2): falling in, its centre at X_H,c 0.7346 burning L_H 18,681 against L 1.55 —
    /// past the light window at every frame.
    fn flashing(x_h: f64) -> Object {
        let mut o = cloud();
        o.luminosity = 1.55;
        o.l_group = [18_681.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        o.l_nuc = 18_681.0;
        o.v_r = -11.0;
        o.x_c[X_H] = x_h;
        o
    }

    /// The flash's burnt core: cold, degenerate, dark, its centre's hydrogen gone.
    fn burnt_core() -> Object {
        let mut o = degenerate();
        o.d_c = 0.9999;
        o.l_group = [0.0; 7];
        o.x_c = [0.0; 10];
        o.x_c[X_HE] = 0.98;
        o
    }

    #[test]
    fn a_flashed_centre_ignites_by_its_burnt_fuel_and_its_core_is_no_failed_star() {
        // §0.5 [M0-D78]: X_H,c at or under 0.9 × the most it has shown, while hydrogen burns, ignites — `by` fuel.
        let mut t = StageTracker::new(M_CH);
        assert!(t.update(&[flashing(0.7346)], 4.0).is_empty(), "L_H > 2 L: no ignition by the light");
        assert!(t.update(&[flashing(0.70)], 4.1).is_empty(), "burned under a tenth: not yet");
        assert_eq!(t.stage(1).unwrap().history.x_h_max, 0.7346, "the most the centre has shown");
        assert_eq!(t.stage(1).unwrap().raw, Stage::Collapsing, "a never-ignited cloud falling in");
        let lit = t.update(&[flashing(0.66)], 4.2);
        assert_eq!(lit, vec![Event { object: 1, kind: EventKind::Ignition, time: 4.2, by: Some(Ignition::Fuel) }], "0.66 ≤ 0.9 × 0.7346");
        assert!(t.stage(1).unwrap().history.ignited);
        assert!(t.update(&[flashing(0.0)], 4.3).is_empty(), "ignites once");
        for frame in 0..3 {
            t.update(&[burnt_core()], 5.0 + f64::from(frame));
        }
        assert_eq!(t.stage(1).unwrap().shown, Some(Stage::WhiteDwarf), "the burnt core of a flash is a white dwarf");
        // The same core with no flash behind it: never ignited, a failed star.
        let mut t = StageTracker::new(M_CH);
        for frame in 0..3 {
            t.update(&[burnt_core()], f64::from(frame));
        }
        assert_eq!(t.stage(1).unwrap().shown, Some(Stage::FailedStar), "fuel that never burned");
        // Exactly 0.9 × the most is within; a hair over it is not.
        let mut t = StageTracker::new(M_CH);
        t.update(&[flashing(1.0)], 1.0);
        assert!(t.update(&[flashing(0.900_000_1)], 2.0).is_empty());
        assert_eq!(kinds(&t.update(&[flashing(0.9)], 3.0)), vec![EventKind::Ignition]);
    }

    #[test]
    fn hydrogen_gone_without_burning_does_not_ignite() {
        // A centre that moved onto painted helium burned nothing: X_H,c 0.73 → 0 with L_H = 0.
        let mut t = StageTracker::new(M_CH);
        let mut o = flashing(0.73);
        o.l_group = [0.0; 7];
        o.l_nuc = 0.0;
        t.update(&[o.clone()], 1.0);
        o.x_c[X_H] = 0.0;
        assert!(t.update(&[o.clone()], 2.0).is_empty(), "X_H,c fell with L_H = 0");
        assert!(!t.stage(1).unwrap().history.ignited);
        for frame in 3..6 {
            t.update(&[burnt_core()], f64::from(frame));
        }
        assert_eq!(t.stage(1).unwrap().shown, Some(Stage::FailedStar));
        // A centre under 10⁻³ throughout never ignites by its fuel, burning or not.
        let mut t = StageTracker::new(M_CH);
        assert!(t.update(&[flashing(0.0009)], 1.0).is_empty());
        assert!(t.update(&[flashing(0.0)], 2.0).is_empty(), "the most it showed is under 10⁻³");
        assert!(!t.stage(1).unwrap().history.ignited);
    }

    #[test]
    fn a_flash_burning_under_a_tenth_does_not_ignite() {
        // M0-V8's step 225 ([M0-D53]): L_H 277 against L 2.1 × 10⁻⁷ burned 0.04 % of the centre's hydrogen.
        let mut t = StageTracker::new(M_CH);
        let mut flash = flashing(0.7346);
        flash.luminosity = 2.1e-7;
        flash.l_group[0] = 277.0;
        flash.l_nuc = 277.0;
        t.update(&[flash.clone()], 1.0);
        flash.x_c[X_H] = 0.7346 * (1.0 - 4e-4);
        assert!(t.update(&[flash], 2.0).is_empty(), "0.04 % burned is no star");
        assert!(!t.stage(1).unwrap().history.ignited);
    }

    #[test]
    fn the_light_window_still_ignites_by_light() {
        let mut t = StageTracker::new(M_CH);
        let mut lit = cloud();
        lit.l_group = [5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        lit.l_nuc = 5.0;
        assert_eq!(t.update(&[lit], 1.0)[0].by, Some(Ignition::Light));
        assert_eq!((Ignition::Light.id(), Ignition::Fuel.id()), ("light", "fuel"));
    }

    #[test]
    fn the_ending_hold_is_ten_tu_or_twenty_dynamical_times() {
        // §0.5 [M0-D78]: max(10 t.u., 20 τ_dyn).
        assert_eq!(ending_hold(0.25), 10.0, "a compact remnant keeps 10 t.u.");
        assert_eq!(ending_hold(0.5), 10.0, "20 τ_dyn = 10 t.u.");
        assert_eq!(ending_hold(1.0), 20.0, "τ_dyn 1: 20 t.u.");
        assert_eq!(ending_hold(0.0), 10.0);
    }

    #[test]
    fn the_spun_up_remnant_holds_for_its_degenerate_centre_s_time() {
        // §0.5 [M0-D161]: m_up_sb's life 08 (3,492.101), s_200000 resumed 16 steps on the RTX 4090 (logs/M0-D161.probe_remnant.log):
        // white_dwarf, a cold core of r_deg 4.119 holding 357.96 inside a disk of R₉₀ 128.29 — τ_dyn 38.02, τ_deg 0.4418 (W scaled
        // to −1,000, 2P/|W| 0.3685 and 2(P + K_t)/|W| 0.9030 as read).
        let mut o = cloud();
        (o.mass, o.gas_mass, o.radius, o.tau_dyn, o.tau_deg) = (1460.94, 1460.94, 128.292, 38.0173, Some(4.119 / 9.32242));
        (o.luminosity, o.luminosity_own, o.l_nuc, o.d_c) = (267.4, 267.4, 0.0, 0.9985);
        (o.pressure, o.kinetic_t, o.potential) = (184.25, 267.25, -1000.0);
        o.v_r = 0.02027 * o.radius / o.tau_dyn;
        o.x_c = [2.972e-6, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.7986, 0.2014];
        assert_eq!(stage_of(&o, &ignited()), Stage::WhiteDwarf);
        assert_eq!(ending_hold(hold_tau(&o)), 10.0, "20 τ_deg = 8.8 t.u.: the 10 t.u. floor");
        assert!((ending_hold(o.tau_dyn) - 760.346).abs() < 1e-9, "the disk's R₉₀ held it 760 t.u.");
        // At step 206,016 the cold disk's inner part read degenerate too: r_deg 35.3, τ_deg 8.03.
        o.tau_deg = Some(8.03);
        assert!((ending_hold(hold_tau(&o)) - 160.6).abs() < 1e-9);
        // No degenerate block: τ_dyn, as before (the lead, «R₉₀ as today»).
        o.tau_deg = None;
        assert_eq!(hold_tau(&o), 38.0173);
    }

    #[test]
    fn the_snapshot_round_trips_the_history_and_refuses_one_without_it() {
        let mut t = StageTracker::new(M_CH);
        t.update(&[flashing(0.7346)], 4.0);
        t.update(&[flashing(0.70), id(burnt_core(), 2)], 4.1);
        let snapshot = t.snapshot();
        let mut back = StageTracker::restore(&snapshot, M_CH).expect("its own snapshot");
        assert_eq!(back.snapshot(), snapshot, "every field round-trips");
        assert_eq!(back.stage(1).unwrap().history, t.stage(1).unwrap().history);
        // The restored history goes on as the unbroken one: the centre's most carried, the fuel test fires.
        assert_eq!(back.update(&[flashing(0.66)], 4.2)[0].by, Some(Ignition::Fuel));
        // A snapshot from before §0.5 [M0-D78] (no `x_h_max`) is refused, never read as a centre that showed no hydrogen.
        let mut old = snapshot;
        for s in old.as_array_mut().unwrap() {
            s.as_object_mut().unwrap().remove("x_h_max");
        }
        let refused = StageTracker::restore(&old, M_CH).err().expect("a snapshot without x_h_max is refused");
        assert!(refused.contains("x_h_max"), "{refused}");
    }
}
