//! The remnant's ring (m0_contrat.md §0.5 [M0-D138], §4.2's Centre, §4.6; M0-D138), as a view model: from core collapse on
//! — a shown stage of core collapse, supernova, neutron star or black hole — each such object's centre (the tracker's c: its
//! heaviest sink, else its densest cell) is ringed over the world in every view, the ring the same size on screen at any zoom,
//! and the selected star's readout adds `star.remnant`. The ejecta of a supernova fill the world and saturate Glow white while
//! the remnant is a few cells (the lead at M0-TV3: «when the star exploses sometimes I cant see whats left inside»); the ring
//! changes no view's colours. The egui layer (desktop.rs) draws what `rings` gives. Display only (G-WATCH).

use eframe::egui;
use sr_engine::state::WorldConfig;
use sr_physics::observe::Stage;

use crate::ui::zoom::Zoom;

/// The ring's radius on screen, in points, at every z: a remnant of a few cells is still a target the eye finds.
pub const RING_RADIUS: f32 = 14.0;

/// Whether an object whose shown stage is `stage` is ringed: from core collapse on, the remnant inside its ejecta.
pub fn ringed(stage: Stage) -> bool {
    matches!(stage, Stage::CoreCollapse | Stage::Supernova | Stage::NeutronStar | Stage::BlackHole)
}

/// One ring over the world: its centre and radius in screen points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ring {
    pub centre: egui::Pos2,
    pub radius: f32,
}

/// The rings of the remnants at world points `centres` (cells, x to the right and y down) through `zoom` over `viewport`
/// (the world at z = 1): each centre's place on the zoomed image, none for a centre the viewport does not show.
pub fn rings(centres: &[[f64; 2]], zoom: &Zoom, viewport: egui::Rect, world: WorldConfig, pixels_per_point: f32) -> Vec<Ring> {
    let image = zoom.image(viewport, world, pixels_per_point);
    let per_cell = image.width() / world.width as f32;
    centres
        .iter()
        .map(|c| image.min + egui::vec2(c[0] as f32, c[1] as f32) * per_cell)
        // The viewport's far edges are outside it, as `Zoom::cell_at` reads them.
        .filter(|p| viewport.contains(*p) && p.x < viewport.max.x && p.y < viewport.max.y)
        .map(|centre| Ring { centre, radius: RING_RADIUS })
        .collect()
}

#[cfg(test)]
mod remnant_tests {
    use super::*;
    use crate::ui::right::{view, Facts};
    use sr_physics::observe::ending::{Ending, Readout};

    const WORLD: WorldConfig = WorldConfig { width: 600, height: 400 };

    fn viewport() -> egui::Rect {
        egui::Rect::from_min_size(egui::pos2(230.0, 60.0), egui::vec2(1200.0, 800.0))
    }

    /// The tracker's centre of a remnant in cell `cell`: the cell's centre, as c is the max-Σ cell's.
    fn centre_of(cell: [u32; 2]) -> [f64; 2] {
        [f64::from(cell[0]) + 0.5, f64::from(cell[1]) + 0.5]
    }

    #[test]
    fn the_ring_sits_on_the_remnants_cell_at_z_one() {
        let vp = viewport();
        let zoom = Zoom::home(WORLD);
        for cell in [[300, 200], [123, 77], [0, 0], [599, 399]] {
            let got = rings(&[centre_of(cell)], &zoom, vp, WORLD, 1.0);
            assert_eq!(got.len(), 1, "{cell:?}");
            assert_eq!(zoom.cell_at(got[0].centre, vp, WORLD, 1.0), Some(cell), "{cell:?}");
        }
    }

    #[test]
    fn the_ring_sits_on_the_remnants_cell_at_z_eight() {
        let vp = viewport();
        for ppp in [1.0, 1.5, 2.0] {
            for (view_at, cell) in [([300, 200], [300, 200]), ([300, 200], [303, 198]), ([120, 80], [131, 74]), ([0, 0], [2, 3])] {
                let mut zoom = Zoom::home(WORLD);
                zoom.centre_on(8, view_at, WORLD);
                let got = rings(&[centre_of(cell)], &zoom, vp, WORLD, ppp);
                assert_eq!(got.len(), 1, "ppp {ppp}, view at {view_at:?}, {cell:?}");
                assert_eq!(zoom.cell_at(got[0].centre, vp, WORLD, ppp), Some(cell), "ppp {ppp}, view at {view_at:?}");
                // Not where z = 1 would put it, unless the cell is the view's centre.
                if cell != view_at {
                    let home = rings(&[centre_of(cell)], &Zoom::home(WORLD), vp, WORLD, ppp)[0].centre;
                    assert_ne!(got[0].centre, home, "ppp {ppp}, {cell:?}: the ring follows the zoom");
                }
            }
        }
    }

    #[test]
    fn a_remnant_the_zoomed_view_does_not_show_has_no_ring() {
        let vp = viewport();
        let mut zoom = Zoom::home(WORLD);
        zoom.centre_on(8, [100, 100], WORLD);
        // 600 / 8 = 75 cells across: (300, 200) is far outside the view.
        assert_eq!(rings(&[centre_of([300, 200])], &zoom, vp, WORLD, 1.0), Vec::new());
        let both = rings(&[centre_of([300, 200]), centre_of([101, 99])], &zoom, vp, WORLD, 1.0);
        assert_eq!(both.len(), 1);
        assert_eq!(zoom.cell_at(both[0].centre, vp, WORLD, 1.0), Some([101, 99]));
    }

    #[test]
    fn no_remnant_no_ring() {
        let vp = viewport();
        assert_eq!(rings(&[], &Zoom::home(WORLD), vp, WORLD, 1.0), Vec::new());
        let ringed_stages: Vec<Stage> = sr_physics::observe::stages::stage_order().into_iter().filter(|s| ringed(*s)).collect();
        assert_eq!(ringed_stages.len(), 4, "{ringed_stages:?}");
        for stage in [Stage::CoreCollapse, Stage::Supernova, Stage::NeutronStar, Stage::BlackHole] {
            assert!(ringed(stage), "{stage:?}");
        }
        for stage in [Stage::IronCore, Stage::WhiteDwarf, Stage::PlanetaryNebula, Stage::FailedStar, Stage::MainSequence, Stage::Cloud] {
            assert!(!ringed(stage), "{stage:?}: no ring before core collapse, nor around a white dwarf");
        }
    }

    #[test]
    fn the_ring_keeps_its_size_on_screen_at_every_z() {
        let vp = viewport();
        for z in 1..=8 {
            let mut zoom = Zoom::home(WORLD);
            zoom.centre_on(z, [300, 200], WORLD);
            let got = rings(&[centre_of([300, 200])], &zoom, vp, WORLD, 1.0);
            assert_eq!(got.iter().map(|r| r.radius).collect::<Vec<_>>(), [RING_RADIUS], "z {z}");
        }
    }

    fn facts(stage: Stage, readout: Option<Readout>) -> Facts {
        Facts {
            stage,
            supergiant: true,
            mass_suns: 15.0,
            readout,
            age_years: 1.2e7,
            rate: Some(1.0e5),
            surface_kelvin: 4000.0,
            core_kelvin: 10.0,
        }
    }

    #[test]
    fn a_ringed_star_reads_star_remnant_under_its_fate_or_ending() {
        let ended = view(Some(&facts(Stage::NeutronStar, Some(Readout::Ended(Ending::NeutronStar)))), 1, false).lines;
        assert_eq!(&ended[2..4], ["Ending: neutron star", "Remnant: ringed in the world"]);
        let exploding = view(Some(&facts(Stage::Supernova, None)), 1, false).lines;
        assert_eq!(exploding[2], "Remnant: ringed in the world", "{exploding:?}");
        for stage in [Stage::IronCore, Stage::WhiteDwarf, Stage::MainSequence] {
            let lines = view(Some(&facts(stage, None)), 1, false).lines;
            assert!(lines.iter().all(|l| !l.starts_with("Remnant:")), "{stage:?}: {lines:?}");
        }
    }
}
