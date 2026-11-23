//! The zoom (m0_contrat.md §1.11 [M0-R12], §4.2's Centre, §4.10; M0-T129), as a view model: the world drawn at the integer scale
//! z · s, z from 1 (the whole world, the default) to 8, around a view centre. The wheel steps z by one keeping the cell under the
//! cursor under it, the middle button drags the view (clamped so the zoomed world covers the viewport), Home returns to z = 1.
//! The viewport is where the world sits at z = 1 (`legend::layout`'s world rect): the zoomed image is clipped to it, so the legend
//! and the hint stay where they are. `image` is the zoomed world's rectangle on screen and `cell_at` the one pointer-to-cell
//! mapping the tools, the preset drop, the selection and the inspector read; at z = 1 both are today's. The egui layer
//! (desktop.rs) draws what this gives.

use eframe::egui;
use sr_engine::state::WorldConfig;

use crate::ui::left;

/// The largest z (§1.11).
pub const MAX_Z: u32 = 8;
/// Wheel points that make one step of z when the wheel reports points (a touchpad); a notch reported in lines is one step.
const POINTS_PER_STEP: f32 = 50.0;

/// The view: z and the world point (in cells, x to the right and y down) at the viewport's centre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Zoom {
    z: u32,
    centre: [f32; 2],
}

impl Zoom {
    /// z = 1, the whole world.
    pub fn home(world: WorldConfig) -> Self {
        Self { z: 1, centre: [world.width as f32 / 2.0, world.height as f32 / 2.0] }
    }

    #[cfg(test)]
    pub fn z(&self) -> u32 {
        self.z
    }

    /// The view centre `centre` clamped so the zoomed world covers the viewport: the visible half-extent is (W, H) / 2z cells.
    fn clamped(z: u32, centre: [f32; 2], world: WorldConfig) -> Self {
        let half = [world.width as f32 / (2.0 * z as f32), world.height as f32 / (2.0 * z as f32)];
        let clamp = |c: f32, half: f32, size: u32| c.clamp(half, size as f32 - half);
        Self { z, centre: [clamp(centre[0], half[0], world.width), clamp(centre[1], half[1], world.height)] }
    }

    /// Home (§4.10): back to the whole world.
    pub fn reset(&mut self, world: WorldConfig) {
        *self = Self::home(world);
    }

    /// The wheel's `steps` (positive in, negative out) with the pointer at `pos` over `viewport`: z moves by that many, kept within
    /// 1 to 8, and the world point under the pointer stays under it. A pointer outside the viewport zooms about its centre.
    pub fn step(&mut self, steps: i32, pos: egui::Pos2, viewport: egui::Rect, world: WorldConfig) {
        let z = (self.z as i32 + steps).clamp(1, MAX_Z as i32) as u32;
        if z == self.z {
            return;
        }
        // The pointer's place across the viewport, -½ to ½ each way; the world point there is centre + rel · (W, H) / z.
        let rel = if viewport.contains(pos) {
            [(pos.x - viewport.center().x) / viewport.width(), (pos.y - viewport.center().y) / viewport.height()]
        } else {
            [0.0, 0.0]
        };
        let extent = |size: u32, z: u32| size as f32 / z as f32;
        let under = [
            self.centre[0] + rel[0] * extent(world.width, self.z),
            self.centre[1] + rel[1] * extent(world.height, self.z),
        ];
        let centre = [under[0] - rel[0] * extent(world.width, z), under[1] - rel[1] * extent(world.height, z)];
        *self = Self::clamped(z, centre, world);
    }

    /// A middle-button drag of `delta` screen points over `viewport`: the world follows the pointer.
    pub fn pan(&mut self, delta: egui::Vec2, viewport: egui::Rect, world: WorldConfig) {
        let per_cell = viewport.width() / world.width as f32 * self.z as f32;
        *self = Self::clamped(self.z, [self.centre[0] - delta.x / per_cell, self.centre[1] - delta.y / per_cell], world);
    }

    /// z around cell (the capture's `{"zoom": z, "at": [x, y]}`): that cell's centre at the view's centre, as far as the clamp allows.
    pub fn centre_on(&mut self, z: u32, cell: [u32; 2], world: WorldConfig) {
        *self = Self::clamped(z.clamp(1, MAX_Z), [cell[0] as f32 + 0.5, cell[1] as f32 + 0.5], world);
    }

    /// The zoomed world's rectangle on screen for `viewport` (the world at z = 1): z times as large, the view centre at the
    /// viewport's centre, its corner on a physical pixel so nearest filtering shows every cell whole, and never short of
    /// covering the viewport. At z = 1 it is `viewport` itself.
    pub fn image(&self, viewport: egui::Rect, world: WorldConfig, pixels_per_point: f32) -> egui::Rect {
        if self.z == 1 {
            return viewport;
        }
        let size = viewport.size() * self.z as f32;
        let per_cell = viewport.width() / world.width as f32 * self.z as f32;
        let ideal = viewport.center() - egui::vec2(self.centre[0], self.centre[1]) * per_cell;
        let snap = |v: f32| (v * pixels_per_point).round() / pixels_per_point;
        let min = egui::pos2(
            snap(ideal.x).clamp(viewport.max.x - size.x, viewport.min.x),
            snap(ideal.y).clamp(viewport.max.y - size.y, viewport.min.y),
        );
        egui::Rect::from_min_size(min, size)
    }

    /// The part of the world's image the viewport shows, as the texture's uv rectangle.
    pub fn uv(&self, viewport: egui::Rect, image: egui::Rect) -> egui::Rect {
        egui::Rect::from_min_max(
            egui::pos2((viewport.min.x - image.min.x) / image.width(), (viewport.min.y - image.min.y) / image.height()),
            egui::pos2((viewport.max.x - image.min.x) / image.width(), (viewport.max.y - image.min.y) / image.height()),
        )
    }

    /// The world cell under `pos` through the zoom, none outside the viewport or the world — the one mapping every tool, the
    /// preset drop, the selection and the inspector read (§1.11).
    pub fn cell_at(&self, pos: egui::Pos2, viewport: egui::Rect, world: WorldConfig, pixels_per_point: f32) -> Option<[u32; 2]> {
        // The viewport's far edges are outside it (egui's `contains` takes them), as the world's are at z = 1.
        if !viewport.contains(pos) || pos.x >= viewport.max.x || pos.y >= viewport.max.y {
            return None;
        }
        left::cell_at(pos, self.image(viewport, world, pixels_per_point), world)
    }
}

/// The wheel's notches, summed until they make a step of z.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Wheel {
    points: f32,
}

impl Wheel {
    /// The steps the wheel's `delta` (up is positive) makes: a notch reported in `lines` is one step each, points sum up to
    /// `POINTS_PER_STEP` a step, and the remainder waits for the next event.
    pub fn steps(&mut self, delta: f32, lines: bool) -> i32 {
        self.points += if lines { delta.signum() * POINTS_PER_STEP * delta.abs().ceil() } else { delta };
        let steps = (self.points / POINTS_PER_STEP).trunc();
        self.points -= steps * POINTS_PER_STEP;
        steps as i32
    }
}

#[cfg(test)]
mod zoom_tests {
    use super::*;

    /// The default view (M0-D204): the default world where the default window draws it, and its points a cell at z = 1 (s).
    fn view() -> (WorldConfig, egui::Rect, f32) {
        let (world, vp) = crate::ui::legend::default_view();
        (world, vp, vp.width() / world.width as f32)
    }

    #[test]
    fn at_z_one_the_mapping_is_todays() {
        let (world, vp, s) = view();
        let zoom = Zoom::home(world);
        assert_eq!(zoom.image(vp, world, 1.0), vp);
        for pos in [
            vp.min,
            vp.min + egui::vec2(s - 0.1, s - 0.1),
            vp.min + egui::vec2(s, s),
            vp.center() + egui::vec2(-0.5, 0.5),
            vp.max - egui::vec2(0.1, 0.1),
            vp.max,
            egui::pos2(vp.min.x - 1.0, vp.min.y + 40.0),
        ] {
            assert_eq!(zoom.cell_at(pos, vp, world, 1.0), left::cell_at(pos, vp, world), "{pos:?}");
        }
        assert_eq!(zoom.uv(vp, vp), egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)));
    }

    #[test]
    fn the_wheel_keeps_the_cell_under_the_cursor() {
        let (world, vp, s) = view();
        // A cursor off the middle, mid-cell: each step in, the cell under it stays the same.
        let pos = vp.min + egui::vec2(150.5 * s, 50.5 * s);
        let mut zoom = Zoom::home(world);
        let before = zoom.cell_at(pos, vp, world, 1.0);
        assert_eq!(before, Some([150, 50]));
        for z in 2..=MAX_Z {
            zoom.step(1, pos, vp, world);
            assert_eq!(zoom.z(), z);
            assert_eq!(zoom.cell_at(pos, vp, world, 1.0), before, "z = {z}");
        }
        // And out again, down to the whole world.
        for z in (1..MAX_Z).rev() {
            zoom.step(-1, pos, vp, world);
            assert_eq!(zoom.z(), z);
            if z > 1 {
                assert_eq!(zoom.cell_at(pos, vp, world, 1.0), before, "z = {z}");
            }
        }
        assert_eq!(zoom, Zoom::home(world));
    }

    #[test]
    fn z_stops_at_one_and_eight() {
        let (world, vp, _) = view();
        let pos = vp.center();
        let mut zoom = Zoom::home(world);
        zoom.step(-3, pos, vp, world);
        assert_eq!(zoom.z(), 1);
        zoom.step(20, pos, vp, world);
        assert_eq!(zoom.z(), MAX_Z);
        zoom.step(1, pos, vp, world);
        assert_eq!(zoom.z(), MAX_Z);
    }

    #[test]
    fn the_image_always_covers_the_viewport() {
        let (world, vp, _) = view();
        // Zoomed at a corner, then dragged far every way: the image never leaves a gap.
        let mut zoom = Zoom::home(world);
        zoom.step(7, vp.min, vp, world);
        for delta in [egui::vec2(5000.0, 5000.0), egui::vec2(-9000.0, -9000.0), egui::vec2(-300.0, 4000.0)] {
            zoom.pan(delta, vp, world);
            let image = zoom.image(vp, world, 1.0);
            assert!(image.contains_rect(vp), "{delta:?}: {image:?} leaves a gap in {vp:?}");
            assert_eq!(image.size(), vp.size() * 8.0);
        }
        // The far corner of the world is the last cell the drag reaches.
        zoom.pan(egui::vec2(-1e6, -1e6), vp, world);
        assert_eq!(zoom.cell_at(vp.max - egui::vec2(0.1, 0.1), vp, world, 1.0), Some([world.width - 1, world.height - 1]));
    }

    #[test]
    fn a_drag_moves_the_world_with_the_pointer() {
        let (world, vp, s) = view();
        let mut zoom = Zoom::home(world);
        zoom.step(3, vp.center(), vp, world); // z = 4: 4 s px a cell
        let cell = 4.0 * s;
        let pos = vp.center() + egui::vec2(-100.3, -70.3);
        let under = zoom.cell_at(pos, vp, world, 1.0).unwrap();
        zoom.pan(egui::vec2(10.0 * cell, -5.0 * cell), vp, world); // ten cells right, five up
        assert_eq!(zoom.cell_at(pos, vp, world, 1.0), Some([under[0] - 10, under[1] + 5]));
        // At z = 1 there is nothing to drag.
        let mut whole = Zoom::home(world);
        whole.pan(egui::vec2(300.0, 300.0), vp, world);
        assert_eq!(whole, Zoom::home(world));
    }

    #[test]
    fn home_returns_to_the_whole_world_and_is_the_default() {
        let (world, vp, _) = view();
        let mut zoom = Zoom::home(world);
        zoom.step(5, vp.min + egui::vec2(70.0, 40.0), vp, world);
        zoom.pan(egui::vec2(-50.0, 20.0), vp, world);
        assert_ne!(zoom, Zoom::home(world));
        zoom.reset(world);
        assert_eq!(zoom, Zoom::home(world));
        assert_eq!(zoom.z(), 1);
    }

    #[test]
    fn nothing_outside_the_viewport_or_the_world_is_a_cell() {
        let (world, vp, _) = view();
        let mut zoom = Zoom::home(world);
        zoom.step(7, vp.center(), vp, world);
        assert_eq!(zoom.cell_at(egui::pos2(vp.min.x - 1.0, vp.min.y + 40.0), vp, world, 1.0), None);
        assert_eq!(zoom.cell_at(egui::pos2(vp.max.x, vp.min.y + 40.0), vp, world, 1.0), None);
        assert!(zoom.cell_at(vp.center(), vp, world, 1.0).is_some());
    }

    #[test]
    fn the_image_corner_sits_on_a_physical_pixel() {
        let (world, vp, _) = view();
        let mut zoom = Zoom::home(world);
        zoom.step(5, vp.min + egui::vec2(103.3, 51.1), vp, world);
        zoom.pan(egui::vec2(-13.37, 7.77), vp, world);
        for ppp in [1.0, 1.5, 2.0] {
            let image = zoom.image(vp, world, ppp);
            for v in [image.min.x, image.min.y] {
                assert_eq!(v * ppp, (v * ppp).round(), "ppp {ppp}: {v}");
            }
        }
    }

    #[test]
    fn centre_on_puts_the_cell_in_the_middle_as_far_as_the_world_allows() {
        let (world, vp, _) = view();
        let mut zoom = Zoom::home(world);
        let middle = [world.width / 2, world.height / 2];
        zoom.centre_on(8, middle, world);
        assert_eq!(zoom.z(), 8);
        assert_eq!(zoom.cell_at(vp.center(), vp, world, 1.0), Some(middle));
        // A cell at the corner cannot be centred: the clamp keeps it in the view.
        zoom.centre_on(8, [0, 0], world);
        assert_eq!(zoom.cell_at(vp.min, vp, world, 1.0), Some([0, 0]));
        zoom.centre_on(99, [world.width - 1, world.height - 1], world);
        assert_eq!(zoom.z(), MAX_Z);
    }

    #[test]
    fn a_brush_at_z_eight_edits_the_cell_the_zoom_names() {
        use sr_engine::step::edit::Edit;
        use sr_physics::registry::Elements;

        let (world, vp, s) = view();
        let tools = left::Tools::new(&Elements::shipped().unwrap());
        let mut zoom = Zoom::home(world);
        let [cx, cy] = [world.width / 2, world.height / 2];
        zoom.centre_on(MAX_Z, [cx, cy], world);
        // 8 s px a cell, the view centred on the middle cell: the pointer 3 cells right and 2 up of the centre, a hair into the cell.
        let px = MAX_Z as f32 * s;
        let pos = vp.center() + egui::vec2(3.0 * px + 1.0, -2.0 * px + 1.0);
        let cell = zoom.cell_at(pos, vp, world, 1.0);
        assert_eq!(cell, Some([cx + 3, cy - 2]));
        // The edit the frame queues (what the state will paint) names that cell, not the one z = 1 would give the same pointer.
        let Some(Edit::Paint { cell: painted, .. }) = left::frame_edit(&tools, cell) else { panic!("no paint edit") };
        assert_eq!(painted, [cx + 3, cy - 2]);
        assert_ne!(Some(painted), left::cell_at(pos, vp, world));
    }

    #[test]
    fn the_wheel_sums_points_into_steps_and_a_line_is_a_step() {
        let mut wheel = Wheel::default();
        assert_eq!(wheel.steps(1.0, true), 1);
        assert_eq!(wheel.steps(-1.0, true), -1);
        assert_eq!(wheel.steps(20.0, false), 0);
        assert_eq!(wheel.steps(20.0, false), 0);
        assert_eq!(wheel.steps(20.0, false), 1);
        assert_eq!(wheel.steps(-120.0, false), -2);
    }
}
