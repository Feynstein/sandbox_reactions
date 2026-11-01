//! The right panel's star readout and the selection (m0_contrat.md §1.9.5, §1.9.6, §4.1's `star.*`, `fate.*`, `ending.*` and
//! `stage.*`, §4.2, §4.6, §4.12; M0-T92). `Stars` is the objects of the world as the readout wants them — the Tracker's
//! objects, their stages, the predicted ending, the age clock and the real-equivalent translations (M0-T69, M0-T72,
//! M0-T73) — fed the same frames the slow-down's observers see (ui::banner::Frames); `view` turns the selected one's `Facts`
//! into the panel's lines, byte for byte the string table's. The egui layer (desktop.rs) only draws what `view` gives.
//!
//! The selection is the tracker's id, never an index into the list (§1.9.6): the list shifts when an object merges or leaves.
//! Display only: nothing here reaches back into a step (G-WATCH).

use std::collections::BTreeMap;

use sr_engine::ledger::At;
use sr_engine::summary::BlockSummary;
use sr_engine::watch::report;
use sr_physics::calibration::Calibration;
use sr_physics::observe::age::AgeClock;
use sr_physics::observe::ending::{Ending, Prediction, Predictor, Readout, Remnant};
use sr_physics::observe::tracker::SUN_MASS_NOMINAL;
use sr_physics::observe::translate::Translations;
use sr_physics::observe::{Object, Stage, StageTracker, Tracker, TrackerConfig};
use sr_physics::registry::Physics;

use crate::format;
use crate::strings;

/// How near a right click must land to an object's centre to pick it, at least, in cells: a star of a few cells is still
/// a target.
const MIN_PICK_RADIUS: f64 = 3.0;

/// What the readout says of one star, in the units a reader sees (§4.12): everything `view` prints.
#[derive(Clone, Debug, PartialEq)]
pub struct Facts {
    pub stage: Stage,
    /// M ≥ M_up: the red-giant rows read "Supergiant" (§1.9.3).
    pub supergiant: bool,
    pub mass_suns: f64,
    /// The prediction, or the ending once a remnant has held (§1.9.5); none when the mass is not a number.
    pub readout: Option<Readout>,
    pub age_years: f64,
    /// Years gained per wall second (`star.rate`); none before the clock has two instants.
    pub rate: Option<f64>,
    pub surface_kelvin: f64,
    pub core_kelvin: f64,
}

/// The star readout of the right panel (§4.6): its heading, one line per quantity, and the note under them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StarPanel {
    pub heading: String,
    pub lines: Vec<String>,
    /// `star.none` with no object, `star.select_hint` with several; none with one.
    pub note: Option<String>,
}

/// The strings table's key of `stage`'s label: the Supergiant rows replace the red-giant ones for a star of M ≥ M_up.
fn stage_key(stage: Stage, supergiant: bool) -> String {
    match (stage, supergiant) {
        (Stage::HShell, true) => "stage.h_shell_super".to_string(),
        (Stage::HeCore, true) => "stage.he_core_super".to_string(),
        _ => format!("stage.{}", stage.id()),
    }
}

/// The template `key` with each `{name}` replaced by its value.
fn fill(key: &str, args: &[(&str, &str)]) -> String {
    let mut text = strings::get(key).to_string();
    for (name, value) in args {
        text = text.replace(&format!("{{{name}}}"), value);
    }
    text
}

fn fate_text(prediction: Prediction) -> String {
    match prediction {
        Prediction::Sure(fate) => strings::get(fate.key()).to_string(),
        Prediction::CloseCall(a, b) => fill("fate.close_call", &[("a", strings::get(a.key())), ("b", strings::get(b.key()))]),
    }
}

/// The readout's panel for the selected star (§4.6): the stage, `star.mass`, `star.fate` — or `star.ending` once a remnant
/// holds —, `star.age`, `star.rate` (not while `paused`), `star.surface`, `star.core`; `objects` is how many the world holds.
pub fn view(star: Option<&Facts>, objects: usize, paused: bool) -> StarPanel {
    let heading = strings::get("star.heading").to_string();
    let Some(star) = star else {
        return StarPanel { heading, lines: Vec::new(), note: Some(strings::get("star.none").to_string()) };
    };
    let mut lines = vec![strings::get(&stage_key(star.stage, star.supergiant)).to_string(), fill("star.mass", &[("mass", &format::mass(star.mass_suns))])];
    match star.readout {
        Some(Readout::Predicted(prediction)) => lines.push(fill("star.fate", &[("fate", &fate_text(prediction))])),
        Some(Readout::Ended(ending)) => lines.push(fill("star.ending", &[("ending", strings::get(ending.key()))])),
        None => {}
    }
    lines.push(fill("star.age", &[("age", &format::age(star.age_years))]));
    if let Some(rate) = star.rate.filter(|_| !paused) {
        lines.push(fill("star.rate", &[("rate", &format::rate(rate))]));
    }
    lines.push(fill("star.surface", &[("temperature", &format::temperature(star.surface_kelvin))]));
    lines.push(fill("star.core", &[("temperature", &format::temperature(star.core_kelvin))]));
    let note = (objects > 1).then(|| strings::get("star.select_hint").to_string());
    StarPanel { heading, lines, note }
}

/// The objects of a world and the readouts they feed (M0-T69, M0-T72, M0-T73), and which of them is selected.
pub struct Stars {
    physics: Physics,
    cal: Calibration,
    world_width: u32,
    tracker: Tracker,
    stages: StageTracker,
    ages: AgeClock,
    predictor: Predictor,
    translations: Translations,
    /// The sim time of the last frame (the world's start before the first): a frame's sim_dt is the difference.
    last_sim: f64,
    /// Each object's displayed remnant stage and the sim time of the first frame that showed it (§1.9.5).
    remnants: BTreeMap<u32, (Stage, f64)>,
    /// The tracker's id of the object last right-clicked (§1.9.6); none: the heaviest.
    selected: Option<u32>,
}

impl Stars {
    /// The observers of a new world `world_width` cells wide, its sim time at `start_sim_time`.
    pub fn new(physics: &Physics, cal: &Calibration, world_width: u32, start_sim_time: f64) -> Result<Stars, String> {
        Ok(Stars {
            physics: physics.clone(),
            cal: cal.clone(),
            world_width,
            // The same wisp threshold the slow-down's observers use (sr_engine::watch::Watch::new).
            tracker: Tracker::new(TrackerConfig::from_physics(physics, SUN_MASS_NOMINAL)),
            stages: StageTracker::new(cal.m_ch_sb),
            ages: AgeClock::from_calibration(cal).map_err(|e| e.to_string())?,
            predictor: Predictor::new(cal.m_ign_sb, cal.m_ch_sb, cal.m_up_sb, cal.m_bh_sb).map_err(|e| e.to_string())?,
            translations: Translations::new(cal, physics).map_err(|e| e.to_string())?,
            last_sim: start_sim_time,
            remnants: BTreeMap::new(),
            selected: None,
        })
    }

    /// A new world's observers: no object, no selection, the sim time running from `start_sim_time`.
    pub fn restart(&mut self, start_sim_time: f64) {
        *self = Stars::new(&self.physics, &self.cal, self.world_width, start_sim_time).expect("the same inputs built the observers once");
    }

    /// A frame's block summary in, taken `at` its end, at wall time `wall` (s, any monotonic clock): the objects, their stages
    /// and ages after it.
    pub fn update(&mut self, summary: &BlockSummary, at: At, wall: f64) {
        let sim_dt = at.sim_time - self.last_sim;
        self.last_sim = at.sim_time;
        let objects = self.tracker.update(&report(summary, self.world_width, sim_dt)).to_vec();
        self.stages.update(&objects, at.sim_time);
        let shown = |id: u32| self.stages.stage(id).and_then(|s| s.shown);
        self.ages.update(at.sim_time, wall, &objects, shown);
        self.remnants.retain(|id, _| objects.iter().any(|o| o.id == *id));
        for o in &objects {
            match shown(o.id).filter(|s| Ending::of_stage(*s).is_some()) {
                Some(stage) => {
                    let held = self.remnants.entry(o.id).or_insert((stage, at.sim_time));
                    if held.0 != stage {
                        *held = (stage, at.sim_time);
                    }
                }
                None => {
                    self.remnants.remove(&o.id);
                }
            }
        }
        if self.selected.is_some_and(|id| !objects.iter().any(|o| o.id == id)) {
            self.selected = None;
        }
    }

    /// How many objects the world holds.
    pub fn count(&self) -> usize {
        self.tracker.objects().len()
    }

    /// The selected object (§1.9.6): the one last right-clicked while it lasts, else the heaviest.
    fn current(&self) -> Option<&Object> {
        let objects = self.tracker.objects();
        self.selected
            .and_then(|id| objects.iter().find(|o| o.id == id))
            .or_else(|| objects.iter().max_by(|a, b| a.mass.total_cmp(&b.mass)))
    }

    /// The selected object's tracker id.
    #[cfg(test)]
    pub fn selected(&self) -> Option<u32> {
        self.current().map(|o| o.id)
    }

    /// A right click on world cell `cell` (§4.10): selects the object under it — the nearest centre among those whose
    /// radius (at least [`MIN_PICK_RADIUS`]) holds the cell. Over empty space the selection stays.
    pub fn select_at(&mut self, cell: [u32; 2]) {
        let at = [f64::from(cell[0]) + 0.5, f64::from(cell[1]) + 0.5];
        let picked = self
            .tracker
            .objects()
            .iter()
            .map(|o| (o, (o.centre[0] - at[0]).hypot(o.centre[1] - at[1])))
            .filter(|(o, d)| *d <= o.radius.max(MIN_PICK_RADIUS))
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(o, _)| o.id);
        if picked.is_some() {
            self.selected = picked;
        }
    }

    /// The selected star's readout at wall time `wall` (s): none with no object.
    pub fn facts(&self, wall: f64) -> Option<Facts> {
        let o = self.current()?;
        let shown = self.stages.stage(o.id).and_then(|s| s.shown);
        let remnant = self.remnants.get(&o.id).map(|&(stage, since)| Remnant { stage, held_t: self.last_sim - since, tau_dyn: o.tau_dyn });
        Some(Facts {
            // Until a stage has held (§1.9.3) the star is the cold cloud it began as.
            stage: shown.unwrap_or(Stage::Cloud),
            supergiant: o.mass >= self.cal.m_up_sb,
            mass_suns: self.translations.mass.to_suns(o.mass),
            readout: self.predictor.readout(o.mass, remnant),
            age_years: self.ages.age(o.id).unwrap_or(0.0),
            rate: self.ages.rate(o.id, wall),
            surface_kelvin: self.translations.surface_kelvin(o.luminosity, o.radius),
            core_kelvin: self.translations.temp.to_kelvin(o.t_c),
        })
    }

    /// The panel of the selected star at wall time `wall`, `paused` hiding the rate.
    pub fn panel(&self, wall: f64, paused: bool) -> StarPanel {
        view(self.facts(wall).as_ref(), self.count(), paused)
    }
}

#[cfg(test)]
mod star_tests {
    use super::*;
    use sr_engine::summary::Block;
    use sr_physics::calibration;
    use sr_physics::observe::ending::Fate;

    fn facts() -> Facts {
        Facts {
            stage: Stage::MainSequence,
            supergiant: false,
            mass_suns: 1.0,
            readout: Some(Readout::Predicted(Prediction::Sure(Fate::WhiteDwarf))),
            age_years: 4.57e9,
            rate: Some(12_300.0),
            surface_kelvin: 5770.0,
            core_kelvin: 15.7e6,
        }
    }

    fn lines(f: &Facts, paused: bool) -> Vec<String> {
        view(Some(f), 1, paused).lines
    }

    #[test]
    fn each_line_is_byte_exact_in_the_contracts_order() {
        let panel = view(Some(&facts()), 1, false);
        assert_eq!(panel.heading, "Star");
        assert_eq!(
            panel.lines,
            [
                "Main sequence: burning hydrogen",
                "Mass: 1.00 Suns",
                "Fate: white dwarf",
                "Age: 4.57 billion years",
                "1 s ≈ 12.3 thousand years",
                "Surface: 5,770 K",
                "Core: 15.7 million K",
            ]
        );
        assert_eq!(panel.note, None, "one star: nothing under the readout");
    }

    #[test]
    fn the_supergiant_rows_switch_at_m_up() {
        let at = |stage, supergiant| lines(&Facts { stage, supergiant, ..facts() }, false)[0].clone();
        assert_eq!(at(Stage::HShell, false), "Red giant: burning a hydrogen shell");
        assert_eq!(at(Stage::HShell, true), "Supergiant: burning a hydrogen shell");
        assert_eq!(at(Stage::HeCore, false), "Red giant: burning helium");
        assert_eq!(at(Stage::HeCore, true), "Supergiant: burning helium");
        // The other rows do not change with the mass: the heavy stages are Supergiant already, the Giant row has no twin.
        assert_eq!(at(Stage::HeShell, true), "Giant: burning a helium shell");
        assert_eq!(at(Stage::CBurn, false), "Supergiant: burning carbon");
        // And the star's own mass crosses at M_up exactly.
        let (physics, cal) = (Physics::shipped().unwrap(), shipped());
        let mut stars = Stars::new(&physics, &cal, 600, 0.0).unwrap();
        stars.update(&summary(&[(0, 0, cal.m_up_sb as f32)]), At { step: 1, sim_time: 0.1 }, 0.0);
        assert!(stars.facts(0.0).unwrap().supergiant, "M = M_up reads Supergiant");
        stars.restart(0.0);
        stars.update(&summary(&[(0, 0, (cal.m_up_sb * 0.99) as f32)]), At { step: 1, sim_time: 0.1 }, 0.0);
        assert!(!stars.facts(0.0).unwrap().supergiant, "below M_up it does not");
    }

    #[test]
    fn every_stage_has_a_label_row() {
        for stage in sr_physics::observe::stages::stage_order() {
            for supergiant in [false, true] {
                let label = lines(&Facts { stage, supergiant, ..facts() }, false)[0].clone();
                assert!(!label.is_empty(), "{stage:?}");
            }
        }
    }

    #[test]
    fn the_ending_replaces_the_fate() {
        let ended = Facts { stage: Stage::NeutronStar, readout: Some(Readout::Ended(Ending::NeutronStar)), ..facts() };
        let got = lines(&ended, false);
        assert_eq!(got[2], "Ending: neutron star");
        assert!(got.iter().all(|l| !l.starts_with("Fate:")), "no fate beside an ending: {got:?}");
        let supernova = Facts { readout: Some(Readout::Predicted(Prediction::Sure(Fate::Supernova))), ..facts() };
        assert_eq!(lines(&supernova, false)[2], "Fate: supernova, then a neutron star");
        for (ending, text) in [
            (Ending::FailedStar, "failed star"),
            (Ending::WhiteDwarf, "white dwarf"),
            (Ending::NeutronStar, "neutron star"),
            (Ending::BlackHole, "black hole"),
        ] {
            assert_eq!(lines(&Facts { readout: Some(Readout::Ended(ending)), ..facts() }, false)[2], format!("Ending: {text}"));
        }
    }

    #[test]
    fn a_close_call_names_both_fates() {
        let close = Facts { readout: Some(Readout::Predicted(Prediction::CloseCall(Fate::WhiteDwarf, Fate::Supernova))), ..facts() };
        assert_eq!(lines(&close, false)[2], "Fate: white dwarf or supernova, then a neutron star (close call)");
    }

    #[test]
    fn the_rate_is_hidden_while_paused_and_before_it_exists() {
        let running = lines(&facts(), false);
        let paused = lines(&facts(), true);
        assert!(running.iter().any(|l| l.starts_with("1 s ≈")));
        assert!(paused.iter().all(|l| !l.starts_with("1 s ≈")), "paused: {paused:?}");
        assert_eq!(paused.len() + 1, running.len(), "the rate is the only line paused takes away");
        let none = lines(&Facts { rate: None, ..facts() }, false);
        assert_eq!(none, paused, "no rate yet reads as paused");
    }

    #[test]
    fn no_object_reads_star_none_and_several_add_the_hint() {
        let none = view(None, 0, false);
        assert_eq!((none.heading.as_str(), none.lines.len()), ("Star", 0));
        assert_eq!(none.note.as_deref(), Some("No star yet. Paint gas or drop a star."));
        let many = view(Some(&facts()), 2, false);
        assert_eq!(many.note.as_deref(), Some("Right-click a star to select it."));
        assert_eq!(many.lines, view(Some(&facts()), 1, false).lines, "the hint sits under the same readout");
    }

    fn shipped() -> Calibration {
        calibration::resolve(calibration::EMBEDDED, true).unwrap().expect("the shipped calibration")
    }

    /// A summary of an 8-row box 48 cells wide (12 blocks a row, two rows): `stars` are (block column, block row, mass), each
    /// star one block holding gas with its densest cell at the block's (2, 2).
    fn summary(stars: &[(u32, u32, f32)]) -> BlockSummary {
        let mut x = [0.0f32; 10];
        (x[0], x[1]) = (0.7, 0.3);
        let empty = Block {
            mass: 0.0,
            mx: 0.0,
            my: 0.0,
            momentum: [0.0; 2],
            kinetic: 0.0,
            thermal: 0.0,
            cold: 0.0,
            pressure: 0.0,
            species: [0.0; 10],
            phi_m: 0.0,
            max_cell: 0,
            max_sigma: 0.0,
            max_t: 0.0,
            max_x: [0.0; 10],
            cold_ratio: 0.0,
            radiated: 0.0,
            nuclear: [0.0; 7],
            neutrino_lost: 0.0,
            neutrino_deposited: 0.0,
            unbound: 0.0,
            sinks: 0,
        };
        let mut blocks = vec![empty.clone(); 24];
        for &(col, row, mass) in stars {
            let (cx, cy) = (col * 4 + 2, row * 4 + 2);
            blocks[(row * 12 + col) as usize] = Block {
                mass,
                mx: mass * (cx as f32 + 0.5),
                my: mass * (cy as f32 + 0.5),
                thermal: 100.0,
                pressure: 50.0,
                species: x.map(|f| mass * f),
                phi_m: -1e4,
                max_cell: cy * 600 + cx,
                max_sigma: 200.0,
                max_t: 30.0,
                max_x: x,
                radiated: 1.0,
                nuclear: [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                ..empty.clone()
            };
        }
        BlockSummary { origin: [0, 0], size: [48, 8], columns: 12, blocks, sinks: Vec::new() }
    }

    fn stars() -> Stars {
        Stars::new(&Physics::shipped().unwrap(), &shipped(), 600, 0.0).unwrap()
    }

    fn frame(stars: &mut Stars, n: u64, held: &[(u32, u32, f32)]) {
        stars.update(&summary(held), At { step: n, sim_time: 0.1 * n as f64 }, n as f64 / 60.0);
    }

    #[test]
    fn the_default_is_the_heaviest_object() {
        let mut stars = stars();
        assert_eq!(stars.panel(0.0, false).note.as_deref(), Some("No star yet. Paint gas or drop a star."));
        frame(&mut stars, 1, &[(0, 0, 3000.0), (6, 0, 6000.0)]);
        assert_eq!(stars.count(), 2);
        let by_default = stars.facts(0.0).unwrap().mass_suns;
        stars.select_at([2, 2]);
        let lighter = stars.facts(0.0).unwrap().mass_suns;
        assert!(lighter < by_default, "{lighter} < {by_default}: the default was the heavier object");
        stars.select_at([26, 2]);
        assert_eq!(stars.facts(0.0).unwrap().mass_suns, by_default, "a click on the heavier one picks it back");
    }

    #[test]
    fn a_right_click_selects_the_object_under_the_cursor() {
        let mut stars = stars();
        frame(&mut stars, 1, &[(0, 0, 6000.0), (6, 0, 3000.0)]);
        let ids: Vec<u32> = stars.tracker.objects().iter().map(|o| o.id).collect();
        assert_eq!(ids.len(), 2);
        let (first, second) = (ids[0], ids[1]);
        assert_eq!(stars.selected(), Some(first), "the heavier object is the default");
        stars.select_at([26, 2]);
        assert_eq!(stars.selected(), Some(second), "the click at the second object's centre");
        stars.select_at([2, 2]);
        assert_eq!(stars.selected(), Some(first), "and back");
    }

    #[test]
    fn a_click_on_empty_space_keeps_the_selection() {
        let mut stars = stars();
        frame(&mut stars, 1, &[(0, 0, 6000.0), (6, 0, 3000.0)]);
        stars.select_at([26, 2]);
        let chosen = stars.selected();
        stars.select_at([300, 200]);
        assert_eq!(stars.selected(), chosen);
    }

    #[test]
    fn the_selection_holds_the_id_when_the_list_reorders() {
        let mut stars = stars();
        frame(&mut stars, 1, &[(0, 0, 6000.0), (6, 0, 3000.0)]);
        stars.select_at([26, 2]);
        let chosen = stars.selected().unwrap();
        assert_eq!(stars.tracker.objects()[1].id, chosen, "the second of two");
        // The first object leaves: the chosen one is now the only entry, at index 0 — an index kept from before would point
        // past the list.
        frame(&mut stars, 2, &[(6, 0, 3000.0)]);
        assert_eq!(stars.count(), 1);
        assert_eq!(stars.tracker.objects()[0].id, chosen);
        assert_eq!(stars.selected(), Some(chosen), "the same star, at its new place in the list");
        // When the chosen one itself leaves, the default takes over and nothing is kept for an object that is gone.
        frame(&mut stars, 3, &[(0, 0, 6000.0)]);
        assert_ne!(stars.selected(), Some(chosen));
        assert!(stars.selected().is_some());
    }

    #[test]
    fn a_new_world_starts_with_no_object_and_no_selection() {
        let mut stars = stars();
        frame(&mut stars, 1, &[(0, 0, 6000.0), (6, 0, 3000.0)]);
        stars.select_at([26, 2]);
        stars.restart(0.1);
        assert_eq!((stars.count(), stars.selected()), (0, None));
        assert_eq!(stars.panel(0.0, false).lines.len(), 0);
    }

    #[test]
    fn a_star_that_has_held_a_stage_reads_it_and_ages() {
        let mut stars = stars();
        for n in 1..=4 {
            frame(&mut stars, n, &[(0, 0, 3000.0)]);
        }
        let panel = stars.panel(4.0 / 60.0, false);
        assert_eq!(panel.lines[0], "Main sequence: burning hydrogen", "{panel:?}");
        assert!(panel.lines[1].starts_with("Mass: "), "{panel:?}");
        assert!(panel.lines.iter().any(|l| l.starts_with("Age: ")), "{panel:?}");
        assert_eq!(panel.note, None);
    }
}
