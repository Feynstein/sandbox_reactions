//! The predicted ending and its close call (m0_contrat.md §1.9.5; M0-T72).
//!
//! Display only: the physics never reads it (G-WATCH). The prediction is read from the object's mass in Suns
//! ([`MassScale::to_suns`], §1.10.1): below 0.08 a failed star, below 8 a white dwarf, below 25 a supernova and then a
//! neutron star, otherwise a black hole. Those three cuts are calibration's M_ign, M_up and M_BH, so the "within ±10 %" of
//! a threshold is taken **in sandbox mass** — in Suns the squeezed mass ladder (§1.10.1) would widen or narrow the band at
//! each end. The band is closed: a mass exactly 10 % from a threshold is a close call.
//!
//! Once an object has *displayed* a remnant stage for [`stages::ending_hold`] t.u. — max(10 t.u., 20 τ_dyn), §0.5
//! [M0-D78] — the readout shows that ending instead of the prediction ([`readout`]).

use super::stages::{ending_hold, Stage};
use super::translate::{MassScale, TranslateError, MASS_SUNS};

/// The share of a threshold, in sandbox mass, either side of which the prediction is a close call (§1.9.5).
pub const CLOSE_CALL: f64 = 0.10;

/// What the prediction says the mass leads to (§1.9.5): the fate strings' four ids.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fate {
    FailedStar,
    WhiteDwarf,
    /// A supernova, then a neutron star.
    Supernova,
    BlackHole,
}

impl Fate {
    /// The strings table's key for this fate (§4.6): `fate.<id>`.
    pub fn key(self) -> &'static str {
        match self {
            Fate::FailedStar => "fate.failed_star",
            Fate::WhiteDwarf => "fate.white_dwarf",
            Fate::Supernova => "fate.supernova",
            Fate::BlackHole => "fate.black_hole",
        }
    }
}

/// What the object came to once its remnant stage held (§1.9.5): the ending strings' four ids.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ending {
    FailedStar,
    WhiteDwarf,
    NeutronStar,
    BlackHole,
}

impl Ending {
    /// The strings table's key for this ending (§4.6): `ending.<id>`.
    pub fn key(self) -> &'static str {
        match self {
            Ending::FailedStar => "ending.failed_star",
            Ending::WhiteDwarf => "ending.white_dwarf",
            Ending::NeutronStar => "ending.neutron_star",
            Ending::BlackHole => "ending.black_hole",
        }
    }

    /// The ending a displayed stage is, if it is a remnant stage (white_dwarf, neutron_star, black_hole, failed_star).
    pub fn of_stage(stage: Stage) -> Option<Ending> {
        match stage {
            Stage::FailedStar => Some(Ending::FailedStar),
            Stage::WhiteDwarf => Some(Ending::WhiteDwarf),
            Stage::NeutronStar => Some(Ending::NeutronStar),
            Stage::BlackHole => Some(Ending::BlackHole),
            _ => None,
        }
    }
}

/// The prediction for a mass: one fate, or two when the mass is within ±10 % of a threshold (lighter fate first).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prediction {
    Sure(Fate),
    /// "{a} or {b} (close call)" — `fate.close_call`.
    CloseCall(Fate, Fate),
}

/// What the readout shows for a star: the ending once its remnant has held, else the prediction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Readout {
    Predicted(Prediction),
    Ended(Ending),
}

/// A remnant stage the star displays and for how long (t.u.), with the star's τ_dyn at that frame (§1.9.2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Remnant {
    pub stage: Stage,
    pub held_t: f64,
    pub tau_dyn: f64,
}

/// The predictor: the mass scale and the three calibrated thresholds it cuts at.
#[derive(Clone, Debug, PartialEq)]
pub struct Predictor {
    scale: MassScale,
    /// M_ign, M_up and M_BH in sandbox mass: the thresholds of 0.08, 8 and 25 Suns, and what lies either side of each.
    cuts: [f64; 3],
}

impl Predictor {
    /// From calibration's four sandbox anchors, `M_ign < M_Ch < M_up < M_BH` (the scale refuses any other order).
    pub fn new(m_ign: f64, m_ch: f64, m_up: f64, m_bh: f64) -> Result<Predictor, TranslateError> {
        Ok(Predictor { scale: MassScale::new(m_ign, m_ch, m_up, m_bh)?, cuts: [m_ign, m_up, m_bh] })
    }

    /// The prediction for an object of `mass_sb`; `None` when the mass is not a number.
    pub fn predict(&self, mass_sb: f64) -> Option<Prediction> {
        if mass_sb.is_nan() {
            return None;
        }
        let suns = self.scale.to_suns(mass_sb);
        let fate = if suns < MASS_SUNS[0] {
            Fate::FailedStar
        } else if suns < MASS_SUNS[2] {
            Fate::WhiteDwarf
        } else if suns < MASS_SUNS[3] {
            Fate::Supernova
        } else {
            Fate::BlackHole
        };
        Some(match self.close_threshold(mass_sb) {
            Some(i) => {
                let (below, above) = [
                    (Fate::FailedStar, Fate::WhiteDwarf),
                    (Fate::WhiteDwarf, Fate::Supernova),
                    (Fate::Supernova, Fate::BlackHole),
                ][i];
                Prediction::CloseCall(below, above)
            }
            None => Prediction::Sure(fate),
        })
    }

    /// The threshold the mass is within ±10 % of, in sandbox mass (the nearer one, were two bands to meet).
    fn close_threshold(&self, mass_sb: f64) -> Option<usize> {
        let relative = |i: usize| (mass_sb - self.cuts[i]).abs() / self.cuts[i];
        (0..self.cuts.len())
            .filter(|&i| mass_sb >= (1.0 - CLOSE_CALL) * self.cuts[i] && mass_sb <= (1.0 + CLOSE_CALL) * self.cuts[i])
            .min_by(|&a, &b| relative(a).total_cmp(&relative(b)))
    }

    /// What the readout shows (§1.9.5): the remnant's ending when its stage is a remnant and has been displayed for
    /// [`ending_hold`] t.u. at the star's τ_dyn; otherwise the prediction for `mass_sb`.
    pub fn readout(&self, mass_sb: f64, remnant: Option<Remnant>) -> Option<Readout> {
        if let Some(r) = remnant {
            if let Some(ending) = Ending::of_stage(r.stage) {
                if r.held_t >= ending_hold(r.tau_dyn) {
                    return Some(Readout::Ended(ending));
                }
            }
        }
        self.predict(mass_sb).map(Readout::Predicted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Synthetic anchors, M_ign < M_Ch < M_up < M_BH with a squeezed upper ladder (M_BH only 2 × M_up): the Suns band of
    // ±10 % is then not the sandbox band, which is the case §1.9.5's "in sandbox mass" separates.
    const M_IGN: f64 = 100.0;
    const M_CH: f64 = 1_000.0;
    const M_UP: f64 = 5_000.0;
    const M_BH: f64 = 10_000.0;

    fn predictor() -> Predictor {
        Predictor::new(M_IGN, M_CH, M_UP, M_BH).unwrap()
    }

    fn sure(m: f64) -> Fate {
        match predictor().predict(m) {
            Some(Prediction::Sure(f)) => f,
            other => panic!("mass {m}: expected a sure fate, got {other:?}"),
        }
    }

    fn close(m: f64) -> (Fate, Fate) {
        match predictor().predict(m) {
            Some(Prediction::CloseCall(a, b)) => (a, b),
            other => panic!("mass {m}: expected a close call, got {other:?}"),
        }
    }

    #[test]
    fn each_band_reads_its_fate() {
        assert_eq!(sure(1.0), Fate::FailedStar);
        assert_eq!(sure(50.0), Fate::FailedStar);
        assert_eq!(sure(500.0), Fate::WhiteDwarf, "between M_ign and M_up");
        assert_eq!(sure(3_000.0), Fate::WhiteDwarf, "below 8 Suns");
        assert_eq!(sure(7_000.0), Fate::Supernova, "between M_up and M_BH");
        assert_eq!(sure(20_000.0), Fate::BlackHole);
        assert_eq!(sure(1.0e9), Fate::BlackHole, "the end segment extended");
        assert_eq!(sure(0.0), Fate::FailedStar, "an empty world's zero reads the lightest band");
        assert_eq!(predictor().predict(f64::NAN), None);
    }

    #[test]
    fn a_threshold_itself_is_a_close_call_between_its_two_fates() {
        assert_eq!(close(M_IGN), (Fate::FailedStar, Fate::WhiteDwarf));
        assert_eq!(close(M_UP), (Fate::WhiteDwarf, Fate::Supernova));
        assert_eq!(close(M_BH), (Fate::Supernova, Fate::BlackHole));
    }

    #[test]
    fn the_close_call_edges_are_exactly_ten_percent_in_sandbox_mass() {
        for (cut, pair) in [
            (M_IGN, (Fate::FailedStar, Fate::WhiteDwarf)),
            (M_UP, (Fate::WhiteDwarf, Fate::Supernova)),
            (M_BH, (Fate::Supernova, Fate::BlackHole)),
        ] {
            let low = 0.9 * cut;
            let high = 1.1 * cut;
            assert_eq!(close(low), pair, "−10 % exactly is a close call (cut {cut})");
            assert_eq!(close(high), pair, "+10 % exactly is a close call (cut {cut})");
            assert!(matches!(predictor().predict(low.next_down()), Some(Prediction::Sure(_))), "a hair under −10 % is not (cut {cut})");
            assert!(matches!(predictor().predict(high.next_up()), Some(Prediction::Sure(_))), "a hair over +10 % is not (cut {cut})");
        }
    }

    #[test]
    fn outside_the_band_the_fate_is_the_side_the_mass_is_on() {
        assert_eq!(sure((0.9 * M_UP).next_down()), Fate::WhiteDwarf);
        assert_eq!(sure((1.1 * M_UP).next_up()), Fate::Supernova);
        assert_eq!(sure((0.9 * M_BH).next_down()), Fate::Supernova);
        assert_eq!(sure((1.1 * M_BH).next_up()), Fate::BlackHole);
        assert_eq!(sure((0.9 * M_IGN).next_down()), Fate::FailedStar);
        assert_eq!(sure((1.1 * M_IGN).next_up()), Fate::WhiteDwarf);
    }

    #[test]
    fn the_band_is_taken_in_sandbox_mass_not_in_suns() {
        // 1.08 × M_up is inside the sandbox band; on this squeezed ladder it reads ≈ 9.1 Suns, 13 % over 8: outside a Suns band.
        let m = 1.08 * M_UP;
        assert!(predictor().scale.to_suns(m) > 1.1 * 8.0, "the fixture squeezes: ±10 % in Suns would not reach this mass");
        assert_eq!(close(m), (Fate::WhiteDwarf, Fate::Supernova));
    }

    #[test]
    fn the_nearer_threshold_names_the_pair_when_two_bands_meet() {
        // M_BH only 1.15 × M_up: 1.1 M_up and 0.9 M_BH overlap (1.1 M_up = 1.1 × 5,000 = 5,500; 0.9 × 5,750 = 5,175).
        let p = Predictor::new(100.0, 1_000.0, 5_000.0, 5_750.0).unwrap();
        assert_eq!(p.predict(5_200.0), Some(Prediction::CloseCall(Fate::WhiteDwarf, Fate::Supernova)), "nearer M_up");
        assert_eq!(p.predict(5_500.0), Some(Prediction::CloseCall(Fate::Supernova, Fate::BlackHole)), "nearer M_BH");
    }

    #[test]
    fn the_keys_name_the_strings_table() {
        assert_eq!(Fate::Supernova.key(), "fate.supernova");
        assert_eq!(Fate::FailedStar.key(), "fate.failed_star");
        assert_eq!(Ending::NeutronStar.key(), "ending.neutron_star");
        assert_eq!(Ending::of_stage(Stage::WhiteDwarf), Some(Ending::WhiteDwarf));
        assert_eq!(Ending::of_stage(Stage::Supernova), None, "the supernova is a stage, not an ending");
        assert_eq!(Ending::of_stage(Stage::MainSequence), None);
        assert_eq!(Ending::of_stage(Stage::PlanetaryNebula), None);
    }

    fn remnant(stage: Stage, held_t: f64, tau_dyn: f64) -> Option<Remnant> {
        Some(Remnant { stage, held_t, tau_dyn })
    }

    #[test]
    fn a_remnant_held_ten_tu_shows_the_ending_instead_of_the_prediction() {
        let p = predictor();
        let m = 3_000.0; // predicted: a white dwarf
        assert_eq!(p.readout(m, None), Some(Readout::Predicted(Prediction::Sure(Fate::WhiteDwarf))));
        let held = |t: f64| p.readout(m, remnant(Stage::WhiteDwarf, t, 0.1));
        assert_eq!(held(9.999), Some(Readout::Predicted(Prediction::Sure(Fate::WhiteDwarf))), "not yet");
        assert_eq!(held(10.0), Some(Readout::Ended(Ending::WhiteDwarf)), "at 10 t.u. exactly");
        assert_eq!(held(10.0 + 5.0), Some(Readout::Ended(Ending::WhiteDwarf)));
        // The ending is what the star came to, not what its mass pointed to; a close call yields too.
        let near = p.readout(M_UP, remnant(Stage::NeutronStar, 12.0, 0.1));
        assert_eq!(near, Some(Readout::Ended(Ending::NeutronStar)));
    }

    #[test]
    fn the_hold_is_the_stage_modules_function_of_tau_dyn() {
        let p = predictor();
        // τ_dyn 1.0: the hold is 20 t.u. (§0.5 [M0-D78]); ten is not enough, twenty is.
        assert!(matches!(p.readout(3_000.0, remnant(Stage::WhiteDwarf, 10.0, 1.0)), Some(Readout::Predicted(_))));
        assert_eq!(p.readout(3_000.0, remnant(Stage::WhiteDwarf, 20.0, 1.0)), Some(Readout::Ended(Ending::WhiteDwarf)));
    }

    #[test]
    fn only_a_remnant_stage_ends_the_prediction() {
        let p = predictor();
        for stage in [Stage::MainSequence, Stage::Supernova, Stage::CoreCollapse, Stage::PlanetaryNebula, Stage::HShell, Stage::Cloud] {
            assert!(
                matches!(p.readout(7_000.0, remnant(stage, 1.0e6, 0.1)), Some(Readout::Predicted(_))),
                "{stage:?} held for ever is not an ending"
            );
        }
        for (stage, ending) in [
            (Stage::FailedStar, Ending::FailedStar),
            (Stage::WhiteDwarf, Ending::WhiteDwarf),
            (Stage::NeutronStar, Ending::NeutronStar),
            (Stage::BlackHole, Ending::BlackHole),
        ] {
            assert_eq!(p.readout(7_000.0, remnant(stage, 10.0, 0.1)), Some(Readout::Ended(ending)));
        }
    }

    #[test]
    fn anchors_out_of_order_are_refused() {
        assert!(Predictor::new(100.0, 1_000.0, 10_000.0, 5_000.0).is_err());
    }
}
