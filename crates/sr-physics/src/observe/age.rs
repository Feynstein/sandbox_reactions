//! The age clock (m0_contrat.md §1.10.4; M0-T73).
//!
//! Display only: the physics never reads it (G-WATCH). Each summary an object's age grows by `max(Δφ, 0) · τ_s(M_suns)`,
//! φ the progress of the stage it shows. A stage with a fuel (§1.9.3: main_sequence H, he_core He, c_burn C, ne_burn Ne,
//! o_burn O, si_burn Si) measures φ = (X_f,c,start − X_f,c) / (X_f,c,start − 10⁻³); any other stage measures it in
//! calibrated sandbox time (`stage_durations_sb`, 10 t.u. where unmeasured); a remnant stage adds 10⁸ years per t.u.
//!
//! The clock integrates: the age is a running sum of the Δφ's, so it never decreases and never jumps (G-AGE). A mass
//! change moves τ_s, which acts on the Δφ still to come, never on what is banked. A stage that ends is banked up to the
//! frame that ends it before the next one starts. Time in a stage is likewise summed frame by frame at the rate the mass
//! gives *then*, so a mass jump moves no φ; a stage entered again goes on from where it stood.
//!
//! The "1 s ≈ …" rate is the age gained over the last wall second ([`AgeClock::rate`]).

use std::collections::{BTreeMap, VecDeque};

use super::stages::Stage;
use super::tracker::Object;
use super::translate::{MassScale, TranslateError};
use crate::calibration::Calibration;

/// A fuel is spent at this abundance at the centre (SP-E2's end of the main sequence, §1.10.4).
pub const SPENT: f64 = 1e-3;
/// A remnant stage adds this many years per t.u. (§1.10.4, opinion).
pub const REMNANT_YEARS_PER_TU: f64 = 1e8;
/// A stage the calibration has not measured lasts this long (t.u.).
pub const UNMEASURED_DURATION: f64 = 10.0;
/// The masses of §1.10.4's table, in Suns.
pub const TAU_SUNS: [f64; 4] = [1.0, 5.0, 15.0, 25.0];
/// The rate looks back this many wall seconds.
const RATE_WINDOW: f64 = 1.0;

/// §1.10.4's table: τ_s in years at 1, 5, 15 and 25 Suns. The remnant stages are not in it (a flat [`REMNANT_YEARS_PER_TU`]
/// per t.u.), and `collapsing` shares the cloud's row.
fn tau_row(stage: Stage) -> Option<[f64; 4]> {
    Some(match stage {
        Stage::Cloud | Stage::Collapsing => [1e6, 1e6, 1e6, 1e6],
        Stage::Protostar => [1e7, 1.8e5, 1.1e4, 3.2e3],
        Stage::MainSequence => [1e10, 1.8e8, 1.1e7, 1e7],
        Stage::HShell => [2e9, 3.6e7, 2.3e6, 6.4e5],
        Stage::HeCore => [1.2e8, 2.2e7, 2.0e6, 1e6],
        Stage::HeShell => [1.5e6, 1.5e6, 1.5e6, 1.5e6],
        Stage::PlanetaryNebula => [1e4, 1e4, 1e4, 1e4],
        Stage::CBurn => [2.0e3, 2.0e3, 2.0e3, 1e3],
        Stage::NeBurn => [0.7, 0.7, 0.7, 3.0],
        Stage::OBurn => [2.6, 2.6, 2.6, 0.3],
        Stage::SiBurn => [0.049, 0.049, 0.049, 0.0137],
        Stage::IronCore => [0.0027, 0.0027, 0.0027, 0.0027],
        Stage::CoreCollapse => [9.5e-11, 9.5e-11, 9.5e-11, 9.5e-11],
        Stage::Supernova => [3.2e-7, 3.2e-7, 3.2e-7, 3.2e-7],
        Stage::BlackHole | Stage::NeutronStar | Stage::WhiteDwarf | Stage::FailedStar => return None,
    })
}

/// The species whose abundance at the centre is the stage's fuel (§1.9.3, §1.5's order); none for a stage without one.
fn fuel(stage: Stage) -> Option<usize> {
    match stage {
        Stage::MainSequence => Some(0),
        Stage::HeCore => Some(1),
        Stage::CBurn => Some(2),
        Stage::OBurn => Some(3),
        Stage::NeBurn => Some(4),
        Stage::SiBurn => Some(6),
        _ => None,
    }
}

fn is_remnant(stage: Stage) -> bool {
    tau_row(stage).is_none()
}

/// ln y linear in ln x through the points `(x, y)` (x ascending), held at the end values beyond the first and last.
fn log_log(points: &[(f64, f64)], x: f64) -> f64 {
    let (first, last) = (points[0], points[points.len() - 1]);
    // NaN and everything at or under the first point (zero included) read the first value.
    if x.is_nan() || x <= first.0 {
        return first.1;
    }
    if x >= last.0 {
        return last.1;
    }
    let i = points.windows(2).position(|w| x < w[1].0).unwrap_or(points.len() - 2);
    let ((x0, y0), (x1, y1)) = (points[i], points[i + 1]);
    let t = (x / x0).ln() / (x1 / x0).ln();
    (y0.ln() + t * (y1 / y0).ln()).exp()
}

/// τ_s in years for a stage at `suns` (§1.10.4): the table's row log-log in mass, held beyond 1 and 25 Suns. A remnant
/// stage has none (it adds [`REMNANT_YEARS_PER_TU`] per t.u.).
pub fn tau_years(stage: Stage, suns: f64) -> Option<f64> {
    let row = tau_row(stage)?;
    let points: Vec<(f64, f64)> = TAU_SUNS.iter().copied().zip(row).collect();
    Some(log_log(&points, suns))
}

/// One stage's progress for one object: φ as last measured, and the fuel abundance it began from.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Progress {
    phi: f64,
    /// The most X_f,c the stage has shown (a fuel stage); fuel added raises it.
    x_start: f64,
}

/// What the clock keeps for one object.
#[derive(Clone, Debug, Default)]
struct Track {
    age: f64,
    /// The sim time of the last update, and the stage it showed (the one in effect over the interval since).
    last_now: f64,
    stage: Option<Stage>,
    progress: BTreeMap<Stage, Progress>,
    /// (wall seconds, age) at each update, the last [`RATE_WINDOW`] and one older.
    samples: VecDeque<(f64, f64)>,
}

/// The age of each object, in years (§1.10.4).
#[derive(Clone, Debug)]
pub struct AgeClock {
    scale: MassScale,
    /// Stage id → (mass_sb, t.u.) pairs ([`Calibration::stage_durations_sb`]).
    durations: BTreeMap<String, Vec<(f64, f64)>>,
    tracks: BTreeMap<u32, Track>,
}

impl AgeClock {
    /// From calibration's four mass anchors and its stage durations.
    pub fn new(
        (m_ign, m_ch, m_up, m_bh): (f64, f64, f64, f64),
        durations: BTreeMap<String, Vec<(f64, f64)>>,
    ) -> Result<AgeClock, TranslateError> {
        Ok(AgeClock { scale: MassScale::new(m_ign, m_ch, m_up, m_bh)?, durations, tracks: BTreeMap::new() })
    }

    pub fn from_calibration(cal: &Calibration) -> Result<AgeClock, TranslateError> {
        AgeClock::new((cal.m_ign_sb, cal.m_ch_sb, cal.m_up_sb, cal.m_bh_sb), cal.stage_durations_sb.clone())
    }

    /// The sim time (t.u.) the calibration gives `stage` at `mass_sb`: log-log in mass between its pairs, held beyond them,
    /// [`UNMEASURED_DURATION`] where it has none.
    pub fn duration(&self, stage: Stage, mass_sb: f64) -> f64 {
        match self.durations.get(stage.id()) {
            Some(points) if !points.is_empty() => log_log(points, mass_sb),
            _ => UNMEASURED_DURATION,
        }
    }

    /// One frame at sim time `now` and wall time `wall` (seconds): each object's age grows by what its stage progressed
    /// since the last frame. `shown` is the stage an object displays (`None` before it shows one: its age waits). An
    /// object absent from the frame is forgotten; one seen for the first time starts at age 0.
    pub fn update(&mut self, now: f64, wall: f64, objects: &[Object], shown: impl Fn(u32) -> Option<Stage>) {
        self.tracks.retain(|id, _| objects.iter().any(|o| o.id == *id));
        for o in objects {
            let Some(stage) = shown(o.id) else { continue };
            let suns = self.scale.to_suns(o.mass);
            let mut t = self.tracks.remove(&o.id).unwrap_or_else(|| Track { last_now: now, ..Track::default() });
            // The stage in effect over the interval since the last frame is banked up to this frame, whatever this frame shows.
            let banking = t.stage.unwrap_or(stage);
            let dt = (now - t.last_now).max(0.0);
            t.age += self.progress(&mut t, banking, o, suns, dt);
            if banking != stage {
                // The new stage begins from this frame's state: a fuel stage from its abundance now, a timed one from
                // where it stood. Nothing is banked for it yet.
                self.enter(&mut t, stage, o);
            }
            t.stage = Some(stage);
            t.last_now = now;
            t.samples.push_back((wall, t.age));
            while t.samples.len() > 1 && t.samples[1].0 <= wall - RATE_WINDOW {
                t.samples.pop_front();
            }
            self.tracks.insert(o.id, t);
        }
    }

    /// Stage progress over a frame of `dt` t.u., in years (≥ 0).
    fn progress(&self, t: &mut Track, stage: Stage, o: &Object, suns: f64, dt: f64) -> f64 {
        if is_remnant(stage) {
            return dt * REMNANT_YEARS_PER_TU;
        }
        let tau = tau_years(stage, suns).expect("a non-remnant stage has a table row");
        let prog = t.progress.entry(stage).or_insert_with(|| Progress { phi: 0.0, x_start: fuel(stage).map_or(0.0, |f| o.x_c[f]) });
        let phi = match fuel(stage) {
            Some(f) => {
                let x = o.x_c[f];
                // Fuel added raises the start, so φ stays in [0, 1] and the burning of it counts again.
                prog.x_start = prog.x_start.max(x);
                if prog.x_start <= SPENT { 1.0 } else { ((prog.x_start - x) / (prog.x_start - SPENT)).clamp(0.0, 1.0) }
            }
            None => (prog.phi + dt / self.duration(stage, o.mass)).clamp(0.0, 1.0),
        };
        let gained = (phi - prog.phi).max(0.0) * tau;
        prog.phi = phi;
        gained
    }

    /// A stage newly shown: a fuel stage's φ is set from the abundance now without banking it, so a stage entered again
    /// after the fuel moved elsewhere does not jump.
    fn enter(&self, t: &mut Track, stage: Stage, o: &Object) {
        if is_remnant(stage) {
            return;
        }
        let prog = t.progress.entry(stage).or_insert_with(|| Progress { phi: 0.0, x_start: fuel(stage).map_or(0.0, |f| o.x_c[f]) });
        if let Some(f) = fuel(stage) {
            let x = o.x_c[f];
            prog.x_start = prog.x_start.max(x);
            prog.phi = if prog.x_start <= SPENT { 1.0 } else { ((prog.x_start - x) / (prog.x_start - SPENT)).clamp(0.0, 1.0) };
        }
    }

    /// An object's age in years; `None` for an object the clock has not seen show a stage.
    pub fn age(&self, id: u32) -> Option<f64> {
        self.tracks.get(&id).map(|t| t.age)
    }

    /// The age gained per wall second over the last wall second up to `wall` — "1 s ≈ …" (§1.10.4); it falls to 0 while
    /// no frame comes (paused). Over a history shorter than a second, the span there is. `None` before two instants.
    pub fn rate(&self, id: u32, wall: f64) -> Option<f64> {
        let t = self.tracks.get(&id)?;
        let latest = *t.samples.back()?;
        let reference = t.samples.iter().rev().find(|s| s.0 <= wall - RATE_WINDOW).or_else(|| t.samples.front())?;
        let span = wall - reference.0;
        (span > 0.0).then(|| (latest.1 - reference.1) / span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Synthetic anchors M_ign < M_Ch < M_up < M_BH at 0.08, 1.44, 8 and 25 Suns: the sandbox mass of N Suns is read
    // through the scale, so a test names its star in Suns.
    const ANCHORS: (f64, f64, f64, f64) = (100.0, 1_000.0, 5_000.0, 10_000.0);

    fn durations() -> BTreeMap<String, Vec<(f64, f64)>> {
        BTreeMap::from([
            ("cloud".to_string(), vec![(1_000.0, 20.0), (10_000.0, 2.0)]),
            ("h_shell".to_string(), vec![(2_000.0, 40.0)]),
        ])
    }

    fn clock() -> AgeClock {
        AgeClock::new(ANCHORS, durations()).unwrap()
    }

    fn mass(suns: f64) -> f64 {
        MassScale::new(ANCHORS.0, ANCHORS.1, ANCHORS.2, ANCHORS.3).unwrap().to_sb(suns)
    }

    /// An object of `suns` whose centre holds `x` of species `f`.
    fn obj(id: u32, suns: f64, f: usize, x: f64) -> Object {
        let mut o = Object { id, mass: mass(suns), ..Object::default() };
        o.x_c[f] = x;
        o
    }

    fn step(c: &mut AgeClock, now: f64, wall: f64, o: Object, stage: Stage) -> f64 {
        c.update(now, wall, std::slice::from_ref(&o), |_| Some(stage));
        c.age(o.id).unwrap()
    }

    fn near(a: f64, b: f64) -> bool {
        (a - b).abs() <= 1e-9 * b.abs().max(1e-300)
    }

    #[test]
    fn the_table_at_one_five_fifteen_and_twenty_five_suns() {
        // §1.10.4's rows typed again here, not read from the unit.
        let table: [(Stage, [f64; 4]); 14] = [
            (Stage::Cloud, [1e6, 1e6, 1e6, 1e6]),
            (Stage::Protostar, [1e7, 1.8e5, 1.1e4, 3.2e3]),
            (Stage::MainSequence, [1e10, 1.8e8, 1.1e7, 1e7]),
            (Stage::HShell, [2e9, 3.6e7, 2.3e6, 6.4e5]),
            (Stage::HeCore, [1.2e8, 2.2e7, 2.0e6, 1e6]),
            (Stage::HeShell, [1.5e6, 1.5e6, 1.5e6, 1.5e6]),
            (Stage::PlanetaryNebula, [1e4, 1e4, 1e4, 1e4]),
            (Stage::CBurn, [2.0e3, 2.0e3, 2.0e3, 1e3]),
            (Stage::NeBurn, [0.7, 0.7, 0.7, 3.0]),
            (Stage::OBurn, [2.6, 2.6, 2.6, 0.3]),
            (Stage::SiBurn, [0.049, 0.049, 0.049, 0.0137]),
            (Stage::IronCore, [0.0027, 0.0027, 0.0027, 0.0027]),
            (Stage::CoreCollapse, [9.5e-11, 9.5e-11, 9.5e-11, 9.5e-11]),
            (Stage::Supernova, [3.2e-7, 3.2e-7, 3.2e-7, 3.2e-7]),
        ];
        for (stage, row) in table {
            for (suns, want) in [1.0, 5.0, 15.0, 25.0].into_iter().zip(row) {
                let got = tau_years(stage, suns).unwrap();
                assert!(near(got, want), "{} at {suns} Suns: {got} ≠ {want}", stage.id());
            }
        }
        assert!(near(tau_years(Stage::Collapsing, 7.0).unwrap(), 1e6), "collapsing shares the cloud's row");
        for s in [Stage::BlackHole, Stage::NeutronStar, Stage::WhiteDwarf, Stage::FailedStar] {
            assert_eq!(tau_years(s, 1.0), None, "{} is a remnant: 10⁸ years per t.u.", s.id());
        }
    }

    #[test]
    fn between_the_listed_masses_tau_is_log_log() {
        // main_sequence, 1 → 5 Suns: ln τ straight in ln M. At √5 the geometric mean; at 3 the straight line's value.
        let (a, b) = (1e10_f64, 1.8e8_f64);
        assert!(near(tau_years(Stage::MainSequence, 5f64.sqrt()).unwrap(), (a * b).sqrt()));
        let want = a * (b / a).powf(3f64.ln() / 5f64.ln());
        assert!(near(tau_years(Stage::MainSequence, 3.0).unwrap(), want));
        // And it is the (M/M☉)^-2.5 law's neighbour, not a linear blend: well under the linear midpoint.
        assert!(tau_years(Stage::MainSequence, 3.0).unwrap() < 0.5 * (a + b));
        // 5 → 15: protostar 1.8e5 → 1.1e4.
        let want = 1.8e5_f64 * (1.1e4_f64 / 1.8e5).powf((9.0f64 / 5.0).ln() / 3f64.ln());
        assert!(near(tau_years(Stage::Protostar, 9.0).unwrap(), want));
    }

    #[test]
    fn beyond_the_table_tau_is_held() {
        for s in [Stage::MainSequence, Stage::Protostar, Stage::NeBurn] {
            assert_eq!(tau_years(s, 0.01), tau_years(s, 1.0), "{} under 1 Sun", s.id());
            assert_eq!(tau_years(s, 0.0), tau_years(s, 1.0), "{} at zero", s.id());
            assert_eq!(tau_years(s, 400.0), tau_years(s, 25.0), "{} over 25 Suns", s.id());
        }
    }

    #[test]
    fn a_fuel_stage_gains_its_tau_as_the_fuel_goes() {
        let mut c = clock();
        // 1 Sun, hydrogen 0.7 at the start.
        assert_eq!(step(&mut c, 0.0, 0.0, obj(1, 1.0, 0, 0.7), Stage::MainSequence), 0.0);
        // Half way: (0.7 − x) / (0.7 − 0.001) = 0.5 at x = 0.3505.
        let half = step(&mut c, 1.0, 0.1, obj(1, 1.0, 0, 0.3505), Stage::MainSequence);
        assert!(near(half, 0.5 * 1e10), "{half}");
        // Spent: the whole τ, and a centre under 10⁻³ adds no more.
        let all = step(&mut c, 2.0, 0.2, obj(1, 1.0, 0, 0.0005), Stage::MainSequence);
        assert!(near(all, 1e10), "{all}");
        assert_eq!(step(&mut c, 3.0, 0.3, obj(1, 1.0, 0, 0.0), Stage::MainSequence), all);
    }

    #[test]
    fn every_fuel_stage_reads_its_own_species() {
        // (stage, species index of its fuel): H 0, He 1, C 2, O 3, Ne 4, Si 6.
        for (stage, f) in [(Stage::MainSequence, 0), (Stage::HeCore, 1), (Stage::CBurn, 2), (Stage::OBurn, 3), (Stage::NeBurn, 4), (Stage::SiBurn, 6)] {
            let mut c = clock();
            step(&mut c, 0.0, 0.0, obj(1, 5.0, f, 0.5), stage);
            let got = step(&mut c, 1.0, 0.1, obj(1, 5.0, f, 0.001), stage);
            assert!(near(got, tau_years(stage, 5.0).unwrap()), "{}: {got}", stage.id());
        }
    }

    #[test]
    fn a_mass_jump_keeps_the_age_and_changes_the_rate() {
        let mut c = clock();
        step(&mut c, 0.0, 0.0, obj(1, 1.0, 0, 0.7), Stage::MainSequence);
        let before = step(&mut c, 1.0, 0.1, obj(1, 1.0, 0, 0.5), Stage::MainSequence);
        // The cloud swallows a neighbour: 1 → 15 Suns in one frame, the centre unchanged.
        let after = step(&mut c, 2.0, 0.2, obj(1, 15.0, 0, 0.5), Stage::MainSequence);
        assert_eq!(after, before, "the age is kept through the jump");
        // The same burn now earns 15 Suns' τ: Δφ = 0.2 / 0.699 of 1.1e7 years.
        let later = step(&mut c, 3.0, 0.3, obj(1, 15.0, 0, 0.3), Stage::MainSequence);
        assert!(near(later - after, 0.2 / 0.699 * 1.1e7), "{}", later - after);
    }

    #[test]
    fn a_mass_jump_moves_no_timed_stage() {
        // A timed stage sums its frames at the rate the mass gives then, so a jump moves no φ and no age.
        let mut c = clock();
        step(&mut c, 0.0, 0.0, obj(1, 1.0, 0, 0.0), Stage::Protostar);
        let a = step(&mut c, 2.0, 0.1, obj(1, 1.0, 0, 0.0), Stage::Protostar);
        assert!(near(a, 0.2 * 1e7), "2 of the 10 unmeasured t.u.: {a}");
        let b = step(&mut c, 2.0, 0.2, obj(1, 25.0, 0, 0.0), Stage::Protostar);
        assert_eq!(b, a, "no time passed, only the mass changed");
    }

    #[test]
    fn a_falling_phi_is_clamped_and_the_age_never_goes_down() {
        let mut c = clock();
        step(&mut c, 0.0, 0.0, obj(1, 1.0, 0, 0.7), Stage::MainSequence);
        let a = step(&mut c, 1.0, 0.1, obj(1, 1.0, 0, 0.5), Stage::MainSequence);
        // Hydrogen painted in: the centre refills to 0.9.
        let b = step(&mut c, 2.0, 0.2, obj(1, 1.0, 0, 0.9), Stage::MainSequence);
        assert_eq!(b, a, "Δφ < 0 adds nothing");
        // It burns down again from 0.9: 0.9 → 0.6 is 0.3 / 0.899 of τ — the new fuel counts.
        let c2 = step(&mut c, 3.0, 0.3, obj(1, 1.0, 0, 0.6), Stage::MainSequence);
        assert!(near(c2 - b, 0.3 / 0.899 * 1e10), "{}", c2 - b);
    }

    #[test]
    fn a_timed_stage_runs_on_the_calibrated_duration_or_ten() {
        let mut c = clock();
        // cloud at the Sun-like 1,000 → 20 t.u.: 5 t.u. is a quarter of τ = 1e6.
        step(&mut c, 0.0, 0.0, obj(1, 1.44, 0, 0.0), Stage::Cloud);
        assert!(near(step(&mut c, 5.0, 0.1, obj(1, 1.44, 0, 0.0), Stage::Cloud), 0.25 * 1e6));
        // protostar has no pair: 10 t.u., so 5 t.u. is half of τ(1 Sun) = 1e7.
        let mut c = clock();
        step(&mut c, 0.0, 0.0, obj(2, 1.0, 0, 0.0), Stage::Protostar);
        assert!(near(step(&mut c, 5.0, 0.1, obj(2, 1.0, 0, 0.0), Stage::Protostar), 0.5 * 1e7));
        // Beyond the duration φ stops at 1: the stage is worth its τ, no more.
        assert!(near(step(&mut c, 100.0, 0.2, obj(2, 1.0, 0, 0.0), Stage::Protostar), 1e7));
        // The duration between two pairs is log-log, held beyond.
        let c = clock();
        assert!(near(c.duration(Stage::Cloud, 1_000.0), 20.0) && near(c.duration(Stage::Cloud, 10_000.0), 2.0));
        assert!(near(c.duration(Stage::Cloud, 1_000.0 * 10f64.sqrt()), (20.0f64 * 2.0).sqrt()));
        assert!(near(c.duration(Stage::Cloud, 1.0), 20.0) && near(c.duration(Stage::Cloud, 1e9), 2.0), "held beyond");
        assert!(near(c.duration(Stage::HShell, 5.0), 40.0), "one pair: held either side");
        assert!(near(c.duration(Stage::IronCore, 5.0), UNMEASURED_DURATION));
    }

    #[test]
    fn a_remnant_adds_a_hundred_million_years_per_time_unit() {
        let mut c = clock();
        step(&mut c, 0.0, 0.0, obj(1, 1.0, 0, 0.0), Stage::WhiteDwarf);
        assert!(near(step(&mut c, 3.0, 0.1, obj(1, 1.0, 0, 0.0), Stage::WhiteDwarf), 3e8));
        // Whatever the mass, and in the other three remnant stages.
        for s in [Stage::BlackHole, Stage::NeutronStar, Stage::FailedStar] {
            let mut c = clock();
            step(&mut c, 0.0, 0.0, obj(1, 40.0, 0, 0.0), s);
            assert!(near(step(&mut c, 0.5, 0.1, obj(1, 40.0, 0, 0.0), s), 5e7), "{}", s.id());
        }
    }

    #[test]
    fn a_stage_change_banks_the_old_stage_and_starts_the_next() {
        let mut c = clock();
        step(&mut c, 0.0, 0.0, obj(1, 1.0, 0, 0.7), Stage::MainSequence);
        let a = step(&mut c, 1.0, 0.1, obj(1, 1.0, 0, 0.35), Stage::MainSequence);
        assert!(a > 0.0);
        // The frame that shows h_shell has the hydrogen spent: the main sequence is banked whole, not reset.
        let b = step(&mut c, 2.0, 0.2, obj(1, 1.0, 0, 0.0), Stage::HShell);
        assert!(near(b, 1e10), "the main sequence's whole τ, not {b}");
        // h_shell has a single pair, 40 t.u.: 10 t.u. in is a quarter of τ = 2e9.
        let d = step(&mut c, 12.0, 0.3, obj(1, 1.0, 0, 0.0), Stage::HShell);
        assert!(near(d - b, 0.25 * 2e9), "{}", d - b);
        // Back to the main sequence with the hydrogen still gone: nothing is owed, nothing jumps.
        let e = step(&mut c, 13.0, 0.4, obj(1, 1.0, 0, 0.0), Stage::MainSequence);
        assert!(e >= d);
        assert!(near(e - d, 1.0 / 40.0 * 2e9), "h_shell's last frame banked: {}", e - d);
        assert_eq!(step(&mut c, 14.0, 0.5, obj(1, 1.0, 0, 0.0), Stage::MainSequence), e, "the main sequence is already whole");
    }

    #[test]
    fn the_age_never_decreases_over_a_wild_run() {
        // A deterministic scramble: stage, mass, fuel and sim time all jump about (the time sometimes back).
        let stages = [
            Stage::Cloud, Stage::Collapsing, Stage::Protostar, Stage::MainSequence, Stage::HShell, Stage::HeCore, Stage::HeShell,
            Stage::CBurn, Stage::NeBurn, Stage::OBurn, Stage::SiBurn, Stage::IronCore, Stage::CoreCollapse, Stage::Supernova,
            Stage::PlanetaryNebula, Stage::WhiteDwarf, Stage::NeutronStar, Stage::BlackHole, Stage::FailedStar,
        ];
        let mut seed = 0x2545_F491_4F6C_DD1Du64;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            (seed >> 11) as f64 / (1u64 << 53) as f64
        };
        let mut c = clock();
        let mut last = 0.0;
        for k in 0..2_000 {
            let stage = stages[(next() * stages.len() as f64) as usize % stages.len()];
            let mut o = Object { id: 1, mass: next() * 20_000.0, ..Object::default() };
            for x in o.x_c.iter_mut() {
                *x = next();
            }
            let now = next() * 500.0;
            c.update(now, k as f64 * 0.05, std::slice::from_ref(&o), |_| Some(stage));
            let age = c.age(1).unwrap();
            assert!(age >= last, "frame {k}: {age} < {last}");
            assert!(age.is_finite());
            last = age;
        }
        assert!(last > 0.0);
    }

    #[test]
    fn the_rate_is_the_age_gained_over_the_last_wall_second() {
        let mut c = clock();
        // A white dwarf: 1e8 years per t.u.; 0.5 t.u. per frame, a frame every 0.125 wall seconds → 4e8 years per second.
        for k in 0..30 {
            step(&mut c, k as f64 * 0.5, k as f64 * 0.125, obj(1, 1.0, 0, 0.0), Stage::WhiteDwarf);
        }
        assert!(near(c.rate(1, 29.0 * 0.125).unwrap(), 4e8), "{:?}", c.rate(1, 29.0 * 0.125));
        // The speed doubles: one t.u. a frame, and a second later the rate has followed.
        for k in 30..60 {
            step(&mut c, 14.5 + (k - 29) as f64, k as f64 * 0.125, obj(1, 1.0, 0, 0.0), Stage::WhiteDwarf);
        }
        assert!(near(c.rate(1, 59.0 * 0.125).unwrap(), 8e8), "{:?}", c.rate(1, 59.0 * 0.125));
        // Paused: no frame comes, the rate over the last wall second falls to 0.
        assert_eq!(c.rate(1, 9.0), Some(0.0));
    }

    #[test]
    fn a_short_history_gives_the_rate_over_what_there_is() {
        let mut c = clock();
        assert_eq!(c.rate(1, 0.0), None, "an unknown object");
        step(&mut c, 0.0, 0.0, obj(1, 1.0, 0, 0.0), Stage::WhiteDwarf);
        assert_eq!(c.rate(1, 0.0), None, "one instant");
        step(&mut c, 1.0, 0.25, obj(1, 1.0, 0, 0.0), Stage::WhiteDwarf);
        assert!(near(c.rate(1, 0.25).unwrap(), 1e8 / 0.25), "a quarter second of history");
    }

    #[test]
    fn objects_are_independent_new_ones_start_at_zero_and_the_gone_are_forgotten() {
        let mut c = clock();
        let (a, b) = (obj(1, 1.0, 0, 0.0), obj(2, 1.0, 0, 0.0));
        c.update(0.0, 0.0, &[a.clone(), b.clone()], |_| Some(Stage::WhiteDwarf));
        c.update(1.0, 0.1, &[a.clone()], |_| Some(Stage::WhiteDwarf));
        assert!(near(c.age(1).unwrap(), 1e8));
        assert_eq!(c.age(2), None, "object 2 left the frame");
        // A late arrival starts from nothing, not from the sim time already run.
        c.update(50.0, 0.2, &[a, obj(3, 1.0, 0, 0.0)], |_| Some(Stage::WhiteDwarf));
        assert_eq!(c.age(3), Some(0.0));
        assert!(near(c.age(1).unwrap(), 50e8), "50 t.u. of remnant");
        // No stage shown yet: the age waits.
        c.update(51.0, 0.3, &[obj(4, 1.0, 0, 0.0)], |_| None);
        assert_eq!(c.age(4), None);
    }

    #[test]
    fn time_running_back_gains_nothing() {
        // A dump loaded mid-run: the sim time restarts lower.
        let mut c = clock();
        step(&mut c, 100.0, 0.0, obj(1, 1.0, 0, 0.0), Stage::WhiteDwarf);
        let a = step(&mut c, 101.0, 0.1, obj(1, 1.0, 0, 0.0), Stage::WhiteDwarf);
        assert_eq!(step(&mut c, 7.0, 0.2, obj(1, 1.0, 0, 0.0), Stage::WhiteDwarf), a);
        assert!(near(step(&mut c, 8.0, 0.3, obj(1, 1.0, 0, 0.0), Stage::WhiteDwarf), a + 1e8), "it counts on from the new time");
    }

    #[test]
    fn the_clock_builds_from_a_calibration() {
        let cal = Calibration::test_calibration("test");
        let mut c = AgeClock::from_calibration(&cal).unwrap();
        let scale = MassScale::new(cal.m_ign_sb, cal.m_ch_sb, cal.m_up_sb, cal.m_bh_sb).unwrap();
        // main_sequence's single pair is the calibration's: 900 t.u., but a fuel stage never reads it.
        assert!(near(c.duration(Stage::MainSequence, 3351.0), 900.0));
        let o = Object { id: 1, mass: scale.to_sb(1.0), ..Object::default() };
        c.update(0.0, 0.0, std::slice::from_ref(&o), |_| Some(Stage::WhiteDwarf));
        assert_eq!(c.age(1), Some(0.0));
        // The anchors out of order are refused, as the translations refuse them.
        assert!(AgeClock::new((1.0, 1.0, 2.0, 3.0), BTreeMap::new()).is_err());
    }
}
