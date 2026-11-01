//! The live legend under the world (m0_contrat.md §4.2's Centre, §4.11's legend row, §4.12's ticks [M0-R12]; M0-T128), as a view
//! model: the active view's title, its 12 px bar, the ticks under it, and the marks — the Glow setting S and the hovered cell's
//! value. The Glow bar is the colour a Σ = 1 cell draws at each temperature over 10 K – 10 billion K at the current S, the glow
//! not added, by the render's own CPU form (the 512-entry blackbody table, the dimming table, the haze at Σ = 1); Heat and
//! Density are their map over the view's range, so the bar's colours are the maps' whatever the range, and the range shows in the
//! ticks and the marks; Element is the species' swatches with their names. `layout` places the world and the strip under it in
//! the centre area. The egui layer (desktop.rs) only draws what this gives.

use eframe::egui;
use sr_engine::render::{
    blackbody_lookup, blackbody_table, dimming_lookup, dimming_table, Display, View, GLOW_S_DEFAULT, GLOW_S_MAX, GLOW_S_MIN,
};
use sr_physics::registry::Elements;

use crate::format;
use crate::strings;
use crate::ui::right::Cell;

/// The strip's height, its gap under the world, the bar's height, a tick mark's length and the hint line's height, logical px (§4.11).
pub const STRIP_HEIGHT: f32 = 44.0;
pub const GAP: f32 = 8.0;
pub const BAR_HEIGHT: f32 = 12.0;
pub const TICK_LENGTH: f32 = 3.0;
pub const HINT_HEIGHT: f32 = 20.0;
/// A mark: a triangle this tall over a line this wide (§4.11).
pub const MARK_TRIANGLE: f32 = 10.0;
pub const MARK_LINE: f32 = 2.0;
/// The least space between two tick labels, logical px.
pub const LABEL_GAP: f32 = 6.0;
/// The Glow, Heat and Density bars are this many equal slices, each the colour at its middle.
pub const BAR_SAMPLES: usize = 256;

/// §4.11's heat and density stops, sRGB, evenly spaced over the view's range.
const HEAT_STOPS: [&str; 6] = ["#000004", "#3B0F70", "#8C2981", "#DE4968", "#FE9F6D", "#FCFDBF"];
const DENSITY_STOPS: [&str; 5] = ["#440154", "#3B528B", "#21918C", "#5EC962", "#FDE725"];
/// The Glow view's haze colour and gain: a Σ = 1 cell's haze is the whole of it (render.wgsl `glow_colour`).
const HAZE: &str = "#2A3550";
const HAZE_GAIN: f64 = 0.6;

/// What the strip's bar is: a gradient of equal slices, or the species' swatches.
#[derive(Clone, Debug, PartialEq)]
pub enum Bar {
    Gradient(Vec<[u8; 3]>),
    Swatches(Vec<[u8; 3]>),
}

/// A tick at `position` (0 … 1 along the bar); its label unless it would overlap the one on its left.
#[derive(Clone, Debug, PartialEq)]
pub struct Tick {
    pub position: f64,
    pub label: Option<String>,
}

/// A mark over the bar: where it stands (0 … 1) and what it says.
#[derive(Clone, Debug, PartialEq)]
pub struct Mark {
    pub position: f64,
    pub label: String,
}

/// The strip's content for the active view.
#[derive(Clone, Debug, PartialEq)]
pub struct Legend {
    pub title: String,
    pub bar: Bar,
    pub ticks: Vec<Tick>,
    /// The Glow setting S (`legend.white`): Glow only.
    pub white: Option<Mark>,
    /// The hovered cell's value (`legend.cursor`): none over vacuum, off the world, or in Element.
    pub cursor: Option<Mark>,
}

fn hex(s: &str) -> [f64; 3] {
    let v = |i: usize| f64::from(u8::from_str_radix(&s[1 + 2 * i..3 + 2 * i], 16).unwrap()) / 255.0;
    [v(0), v(1), v(2)]
}

fn linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn encode(c: f64) -> f64 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

fn byte(c: f64) -> u8 {
    (c.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// A map of `stops` at p ∈ [0, 1] along them, interpolated in linear RGB and sRGB-encoded (§4.11).
fn map(stops: &[&str], p: f64) -> [f64; 3] {
    let s = p.clamp(0.0, 1.0) * (stops.len() - 1) as f64;
    let i = (s.floor() as usize).min(stops.len() - 2);
    let (a, b, f) = (hex(stops[i]), hex(stops[i + 1]), s - i as f64);
    std::array::from_fn(|k| encode(linear(a[k]) * (1.0 - f) + linear(b[k]) * f))
}

/// The Glow bar's span, log₁₀ K (§4.2): 10 K – 10 billion K.
fn glow_span() -> [f64; 2] {
    [GLOW_S_MIN.log10(), GLOW_S_MAX.log10()]
}

/// The log₁₀ range the bar shows: Glow's span, or the view's range; none for Element.
pub fn range(view: View, display: &Display) -> Option<[f64; 2]> {
    match view {
        View::Glow => Some(glow_span()),
        View::Heat => Some(display.heat),
        View::Density => Some(display.density),
        View::Element => None,
    }
}

/// The Glow bar's colour at `t` kelvins for the setting `s`: what a Σ = 1 cell draws, the glow not added.
fn glow_colour(t: f64, s: f64, table: &[[f32; 3]], dimming: &[f32]) -> [u8; 3] {
    let b = (1.0 - (-(t / s.min(GLOW_S_DEFAULT)).powi(4)).exp()) * f64::from(dimming_lookup(dimming, t));
    let bb = blackbody_lookup(table, t);
    let haze = hex(HAZE);
    std::array::from_fn(|k| byte(f64::from(bb[k]) * b + haze[k] * HAZE_GAIN * (1.0 - b)))
}

/// The Glow bar's colours for the setting `s`, one per slice.
pub fn glow_bar(s: f64) -> Vec<[u8; 3]> {
    let (table, dimming) = (blackbody_table(), dimming_table(s));
    let [lo, hi] = glow_span();
    (0..BAR_SAMPLES)
        .map(|i| glow_colour(10f64.powf(lo + (hi - lo) * (i as f64 + 0.5) / BAR_SAMPLES as f64), s, &table, &dimming))
        .collect()
}

/// The Heat or Density bar: the map at the middle of each slice.
pub fn map_bar(stops: &[&str]) -> Vec<[u8; 3]> {
    (0..BAR_SAMPLES).map(|i| map(stops, (i as f64 + 0.5) / BAR_SAMPLES as f64).map(byte)).collect()
}

/// The ticks of §4.12: at every power of ten inside `range` (log₁₀ ends), both ends when fewer than two fall inside; a label
/// that would overlap the last one shown, at `width` px and `text_width`'s measure, is left out. `label` writes a value.
pub fn ticks(range: [f64; 2], width: f32, text_width: &dyn Fn(&str) -> f32, label: &dyn Fn(f64) -> String) -> Vec<Tick> {
    let [lo, hi] = range;
    let first = (lo - 1e-9).ceil() as i32;
    let last = (hi + 1e-9).floor() as i32;
    let at: Vec<(f64, String)> = if last - first >= 1 {
        (first..=last).map(|k| ((f64::from(k) - lo) / (hi - lo), label(10f64.powi(k)))).collect()
    } else {
        vec![(0.0, label(10f64.powf(lo))), (1.0, label(10f64.powf(hi)))]
    };
    spaced(at, width, text_width)
}

/// A label's left edge on a bar `width` px wide: its box centred on `position`, kept inside the bar so the first and the last
/// labels never leave the strip.
pub fn label_left(position: f64, label_width: f32, width: f32) -> f32 {
    (position as f32 * width - label_width / 2.0).clamp(0.0, (width - label_width).max(0.0))
}

/// Keeps each label whose box (`label_left`) clears the last kept one by `LABEL_GAP`.
fn spaced(at: Vec<(f64, String)>, width: f32, text_width: &dyn Fn(&str) -> f32) -> Vec<Tick> {
    let mut right = f32::NEG_INFINITY;
    at.into_iter()
        .map(|(position, text)| {
            let w = text_width(&text);
            let left = label_left(position, w, width);
            let shown = left >= right + LABEL_GAP;
            if shown {
                right = left + w;
            }
            Tick { position, label: shown.then_some(text) }
        })
        .collect()
}

/// The legend of `view` under `display` for a bar `width` px wide. `cursor` is the inspector's reading of the hovered cell.
pub fn legend(
    view: View,
    display: &Display,
    elements: &Elements,
    cursor: Option<&Cell>,
    bar: Vec<[u8; 3]>,
    width: f32,
    text_width: &dyn Fn(&str) -> f32,
) -> Legend {
    let title = strings::get(match view {
        View::Glow => "legend.glow",
        View::Heat => "legend.heat",
        View::Density => "legend.density",
        View::Element => "legend.element",
    })
    .to_string();
    let Some(range) = range(view, display) else {
        let names: Vec<(f64, String)> = elements
            .species()
            .iter()
            .enumerate()
            .map(|(i, s)| ((i as f64 + 0.5) / elements.species().len() as f64, strings::get(&format!("element.{}", s.key)).to_string()))
            .collect();
        let swatches = elements.species().iter().map(|s| s.color).collect();
        return Legend { title, bar: Bar::Swatches(swatches), ticks: spaced(names, width, text_width), white: None, cursor: None };
    };
    let along = |log10: f64| ((log10 - range[0]) / (range[1] - range[0])).clamp(0.0, 1.0);
    let labelled: &dyn Fn(f64) -> String = if view == View::Density { &format::tick_number } else { &format::tick_temperature };
    let white = (view == View::Glow)
        .then(|| Mark { position: along(display.glow_s.log10()), label: format::legend_white(display.glow_s) });
    let cursor = match (cursor, view) {
        (Some(Cell::Gas(c)), View::Glow | View::Heat) if c.kelvin > 0.0 && c.kelvin.is_finite() => {
            Some(Mark { position: along(c.kelvin.log10()), label: format::legend_cursor_temperature(c.kelvin) })
        }
        (Some(Cell::Gas(c)), View::Density) if c.density > 0.0 && c.density.is_finite() => {
            Some(Mark { position: along(c.density.log10()), label: format::legend_cursor_density(c.density) })
        }
        _ => None,
    };
    Legend { title, bar: Bar::Gradient(bar), ticks: ticks(range, width, text_width, labelled), white, cursor }
}

/// The bars, kept while the view and the Glow setting are the same: the Glow bar costs a luminance sum per slice.
#[derive(Default)]
pub struct Bars {
    glow: Option<(f64, Vec<[u8; 3]>)>,
    heat: Option<Vec<[u8; 3]>>,
    density: Option<Vec<[u8; 3]>>,
}

impl Bars {
    /// The bar of `view` under `display`; Element has none (its swatches are the species').
    pub fn of(&mut self, view: View, display: &Display) -> Vec<[u8; 3]> {
        match view {
            View::Glow => {
                if self.glow.as_ref().is_none_or(|(s, _)| *s != display.glow_s) {
                    self.glow = Some((display.glow_s, glow_bar(display.glow_s)));
                }
                self.glow.as_ref().map(|(_, bar)| bar.clone()).unwrap_or_default()
            }
            View::Heat => self.heat.get_or_insert_with(|| map_bar(&HEAT_STOPS)).clone(),
            View::Density => self.density.get_or_insert_with(|| map_bar(&DENSITY_STOPS)).clone(),
            View::Element => Vec::new(),
        }
    }
}

/// Where the centre area puts the world, the legend under it and the hint under that (§4.2): all in `area`'s logical px.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    pub world: egui::Rect,
    pub legend: egui::Rect,
    pub hint: egui::Rect,
}

/// The world's image at the integer scale s = max(1, ⌊min(width / W, (height − the strip and hint) / H)⌋) of the screen's pixels
/// (§1.11) — visible square cells — with the legend 8 px under it, as wide, and the hint line under that, the three centred as one
/// block, the world's corner on a pixel so nearest filtering shows every cell whole.
pub fn layout(area: egui::Rect, world: (u32, u32), pixels_per_point: f32) -> Layout {
    let (w, h) = (world.0 as f32, world.1 as f32);
    let below = GAP + STRIP_HEIGHT + HINT_HEIGHT;
    let scale = ((area.width() * pixels_per_point / w).min((area.height() - below) * pixels_per_point / h)).floor().max(1.0);
    let size = egui::vec2(w, h) * scale / pixels_per_point;
    let min = ((area.center() - egui::vec2(size.x, size.y + below) / 2.0) * pixels_per_point).round() / pixels_per_point;
    let world = egui::Rect::from_min_size(min, size);
    let legend = egui::Rect::from_min_size(egui::pos2(world.left(), world.bottom() + GAP), egui::vec2(size.x, STRIP_HEIGHT));
    let hint = egui::Rect::from_min_size(egui::pos2(world.left(), legend.bottom()), egui::vec2(size.x, HINT_HEIGHT));
    Layout { world, legend, hint }
}

#[cfg(test)]
mod legend_tests {
    use sr_engine::render::{blackbody_srgb, dimming, DENSITY_SPAN, HEAT_SPAN};

    use super::*;
    use crate::ui::right::CellFacts;

    /// A text 7 px a character, in place of the font.
    fn width(text: &str) -> f32 {
        7.0 * text.chars().count() as f32
    }

    fn elements() -> Elements {
        Elements::shipped().unwrap()
    }

    fn gas(kelvin: f64, density: f64) -> Cell {
        Cell::Gas(CellFacts { mix: vec![], kelvin, density, speed: 0.0, mach: 0.0 })
    }

    fn labels(ticks: &[Tick]) -> Vec<&str> {
        ticks.iter().filter_map(|t| t.label.as_deref()).collect()
    }

    fn glow_ticks(w: f32) -> Vec<Tick> {
        ticks(glow_span(), w, &width, &format::tick_temperature)
    }

    #[test]
    fn the_glow_ticks_are_ten_powers_of_ten_in_the_units() {
        let t = glow_ticks(1200.0);
        assert_eq!(
            labels(&t),
            ["10 K", "100 K", "1,000 K", "10,000 K", "100,000 K", "1 million K", "10 million K", "100 million K", "1 billion K", "10 billion K"]
        );
        assert_eq!(t.len(), 10);
    }

    #[test]
    fn ticks_stand_at_each_decade_not_linearly_in_t() {
        let t = glow_ticks(1200.0);
        for (k, tick) in t.iter().enumerate() {
            assert!((tick.position - k as f64 / 9.0).abs() < 1e-12, "tick {k} at {}", tick.position);
        }
        // 1,000 K is a third of the way along a 10 K – 10 billion K bar, not 1e-7 of it.
        assert!((t[2].position - 2.0 / 9.0).abs() < 1e-12);
    }

    #[test]
    fn a_narrow_range_ticks_only_the_powers_inside_or_both_ends() {
        // 10⁴ K – 10⁹ K: six decades.
        let six = ticks([4.0, 9.0], 1200.0, &width, &format::tick_temperature);
        assert_eq!(labels(&six), ["10,000 K", "100,000 K", "1 million K", "10 million K", "100 million K", "1 billion K"]);
        assert_eq!((six[0].position, six[5].position), (0.0, 1.0));
        // One power inside (10⁵ K at 4.5 – 5.5) is fewer than two: both ends, in the units.
        let one = ticks([4.5, 5.5], 1200.0, &width, &format::tick_temperature);
        assert_eq!(labels(&one), ["31,600 K", "316,000 K"]);
        assert_eq!((one[0].position, one[1].position), (0.0, 1.0));
        // None inside.
        let none = ticks([5.1, 5.9], 1200.0, &width, &format::tick_temperature);
        assert_eq!(labels(&none), ["126,000 K", "794,000 K"]);
    }

    #[test]
    fn density_ticks_follow_the_inspectors_rule() {
        let t = ticks(DENSITY_SPAN, 1200.0, &width, &format::tick_number);
        assert_eq!(labels(&t), ["1e-6", "1e-5", "1e-4", "0.001", "0.01", "0.1", "1", "10", "100", "1,000", "10,000"]);
    }

    #[test]
    fn a_label_that_would_overlap_its_left_neighbour_is_left_out_and_its_tick_kept() {
        // 400 px for ten ticks 44 px apart: "100,000 K" is 63 px wide.
        let t = glow_ticks(400.0);
        assert_eq!(t.len(), 10);
        let shown = labels(&t);
        assert!(shown.len() < 10 && !shown.is_empty(), "{shown:?}");
        assert_eq!(t[0].label.as_deref(), Some("10 K"));
        let mut right = f32::NEG_INFINITY;
        for tick in &t {
            if let Some(label) = &tick.label {
                let left = label_left(tick.position, width(label), 400.0);
                assert!(left >= right + LABEL_GAP, "{label} overlaps");
                right = left + width(label);
            }
        }
        // At full width nothing is dropped.
        assert_eq!(labels(&glow_ticks(1200.0)).len(), 10);
    }

    #[test]
    fn the_first_and_last_labels_stay_inside_the_bar() {
        let t = glow_ticks(1200.0);
        let last = t.last().unwrap();
        let w = width(last.label.as_deref().unwrap());
        assert_eq!(label_left(last.position, w, 1200.0) + w, 1200.0);
        assert_eq!(label_left(t[0].position, width("10 K"), 1200.0), 0.0);
        assert_eq!(label_left(0.5, 70.0, 1200.0), 565.0);
    }

    #[test]
    fn a_tick_label_has_no_trailing_zeros() {
        assert_eq!(format::tick_temperature(10.0), "10 K");
        assert_eq!(format::tick_temperature(1e6), "1 million K");
        assert_eq!(format::tick_temperature(1e10), "10 billion K");
        assert_eq!(format::tick_temperature(100_000.0), "100,000 K");
        assert_eq!(format::tick_number(1e-6), "1e-6");
        assert_eq!(format::tick_number(1e-3), "0.001");
        assert_eq!(format::tick_number(1.0), "1");
        assert_eq!(format::tick_number(1e4), "10,000");
    }

    fn at_glow(s: f64, cursor: Option<&Cell>) -> Legend {
        let display = Display { glow_s: s, ..Display::default() };
        legend(View::Glow, &display, &elements(), cursor, glow_bar(s), 1200.0, &width)
    }

    #[test]
    fn the_title_is_the_views_and_the_strings_are_the_tables() {
        let d = Display::default();
        for (view, key) in [(View::Glow, "legend.glow"), (View::Heat, "legend.heat"), (View::Density, "legend.density"), (View::Element, "legend.element")] {
            let l = legend(view, &d, &elements(), None, Vec::new(), 1200.0, &width);
            assert_eq!(l.title, strings::get(key));
        }
        assert_eq!(strings::get("legend.glow"), "Temperature — colour and brightness");
        assert_eq!(strings::get("legend.density"), "Density (sandbox units)");
    }

    #[test]
    fn the_glow_setting_is_marked_in_glow_only_at_its_place_on_the_bar() {
        let l = at_glow(1e8, None);
        let white = l.white.expect("Glow marks S");
        assert!((white.position - 7.0 / 9.0).abs() < 1e-12);
        assert_eq!(white.label, "Full brightness at 100 million K");
        assert_eq!(at_glow(3_000.0, None).white.unwrap().label, "Full brightness at 3,000 K");
        let d = Display::default();
        for view in [View::Heat, View::Density, View::Element] {
            assert_eq!(legend(view, &d, &elements(), None, Vec::new(), 1200.0, &width).white, None);
        }
    }

    #[test]
    fn the_cursor_mark_stands_at_the_cells_value_on_the_shown_range() {
        let d = Display { heat: [4.0, 8.0], density: [-2.0, 2.0], ..Display::default() };
        let cell = gas(1e6, 1.0);
        let heat = legend(View::Heat, &d, &elements(), Some(&cell), Vec::new(), 1200.0, &width).cursor.unwrap();
        assert!((heat.position - 0.5).abs() < 1e-12);
        assert_eq!(heat.label, "Cursor: 1.00 million K");
        let density = legend(View::Density, &d, &elements(), Some(&cell), Vec::new(), 1200.0, &width).cursor.unwrap();
        assert!((density.position - 0.5).abs() < 1e-12);
        assert_eq!(density.label, "Cursor: 1.00");
        let glow = at_glow(3_000.0, Some(&cell)).cursor.unwrap();
        assert!((glow.position - 5.0 / 9.0).abs() < 1e-12);
        // A value past the range stands at its end.
        let hot = gas(1e9, 1.0);
        assert_eq!(legend(View::Heat, &d, &elements(), Some(&hot), Vec::new(), 1200.0, &width).cursor.unwrap().position, 1.0);
    }

    #[test]
    fn no_cursor_mark_over_vacuum_off_the_world_or_in_element() {
        let d = Display::default();
        for view in [View::Glow, View::Heat, View::Density, View::Element] {
            assert_eq!(legend(view, &d, &elements(), Some(&Cell::Vacuum), Vec::new(), 1200.0, &width).cursor, None, "{view:?} over vacuum");
            assert_eq!(legend(view, &d, &elements(), None, Vec::new(), 1200.0, &width).cursor, None, "{view:?} off the world");
        }
        let cell = gas(1e6, 1.0);
        assert_eq!(legend(View::Element, &d, &elements(), Some(&cell), Vec::new(), 1200.0, &width).cursor, None);
    }

    /// The Glow bar by the formula written out directly — the exact dimming and the blackbody itself, not their lookup tables.
    fn formula(t: f64, s: f64) -> [f64; 3] {
        let b = (1.0 - (-(t / s.min(3000.0)).powi(4)).exp()) * dimming(t, s);
        // The colour table spans 1,000 – 40,000 K and holds its end colours outside (§1.11).
        let bb = blackbody_srgb(t.clamp(1_000.0, 40_000.0));
        let haze = hex(HAZE);
        std::array::from_fn(|k| bb[k] * b + haze[k] * 0.6 * (1.0 - b))
    }

    #[test]
    fn the_glow_bar_is_the_cpu_formula_at_three_settings() {
        for s in [3_000.0, 1e6, 1e9] {
            let bar = glow_bar(s);
            assert_eq!(bar.len(), BAR_SAMPLES);
            let mut worst = 0.0f64;
            for (i, c) in bar.iter().enumerate() {
                let t = 10f64.powf(1.0 + 9.0 * (i as f64 + 0.5) / BAR_SAMPLES as f64);
                for (k, v) in formula(t, s).iter().enumerate() {
                    worst = worst.max((f64::from(c[k]) - v * 255.0).abs());
                }
            }
            assert!(worst <= 2.0, "S = {s}: the bar is {worst} levels from the formula");
        }
    }

    #[test]
    fn the_glow_bar_dims_with_the_setting() {
        let (a, b) = (glow_bar(3_000.0), glow_bar(1e9));
        assert_ne!(a, b);
        let sum = |bar: &[[u8; 3]]| bar.iter().map(|c| c.iter().map(|&v| u32::from(v)).sum::<u32>()).sum::<u32>();
        assert!(sum(&b) < sum(&a), "raising S dims the bar");
    }

    #[test]
    fn the_heat_and_density_bars_are_the_maps_and_the_ranges_move_the_ticks_only() {
        let heat = map_bar(&HEAT_STOPS);
        assert_eq!(heat.len(), BAR_SAMPLES);
        assert!(heat[0][0] < 8 && heat[0][2] < 12, "the bottom is #000004's, {:?}", heat[0]);
        let end = heat[BAR_SAMPLES - 1];
        assert!(end[0] > 240 && end[1] > 240, "the top is #FCFDBF's, {end:?}");
        let density = map_bar(&DENSITY_STOPS);
        assert!(density[0][0] > 60 && density[0][2] > 80, "the bottom is #440154's, {:?}", density[0]);
        let narrow = Display { heat: [4.0, 9.0], ..Display::default() };
        let l = legend(View::Heat, &narrow, &elements(), None, heat.clone(), 1200.0, &width);
        assert_eq!(l.bar, Bar::Gradient(heat));
        assert_eq!(range(View::Heat, &narrow), Some([4.0, 9.0]));
        assert_eq!(range(View::Heat, &Display::default()), Some(HEAT_SPAN));
    }

    #[test]
    fn element_is_the_ten_swatches_with_their_names() {
        let l = legend(View::Element, &Display::default(), &elements(), None, Vec::new(), 1200.0, &width);
        let Bar::Swatches(swatches) = &l.bar else { panic!("a gradient") };
        assert_eq!(swatches.len(), 10);
        assert_eq!(swatches[0], [0x7F, 0xB2, 0xFF]);
        assert_eq!(l.ticks.len(), 10);
        assert_eq!(labels(&l.ticks)[0], "Hydrogen");
        assert_eq!(labels(&l.ticks)[9], "Neutron matter");
        assert_eq!(l.ticks[0].position, 0.05);
    }

    #[test]
    fn the_bars_cache_follows_the_setting() {
        let mut bars = Bars::default();
        let d = Display::default();
        assert_eq!(bars.of(View::Glow, &d), glow_bar(3_000.0));
        let hot = Display { glow_s: 1e8, ..d };
        assert_eq!(bars.of(View::Glow, &hot), glow_bar(1e8));
        assert_eq!(bars.of(View::Element, &d), Vec::<[u8; 3]>::new());
    }

    #[test]
    fn the_world_the_legend_and_the_hint_fit_the_centre_of_the_default_window() {
        // 1760 × 940 less the bars (44, 24) and the panels (220, 300): 1240 × 872.
        let area = egui::Rect::from_min_size(egui::pos2(220.0, 44.0), egui::vec2(1240.0, 872.0));
        let l = layout(area, (600, 400), 1.0);
        assert_eq!(l.world.size(), egui::vec2(1200.0, 800.0), "s stays 2");
        assert_eq!(l.legend.top(), l.world.bottom() + 8.0);
        assert_eq!((l.legend.left(), l.legend.width(), l.legend.height()), (l.world.left(), 1200.0, 44.0));
        assert_eq!(l.hint.top(), l.legend.bottom());
        assert!(l.hint.bottom() <= area.bottom() + 1e-3 && l.world.top() >= area.top() - 1e-3, "{l:?}");
        assert!(l.world.left() >= area.left() && l.world.right() <= area.right());
    }

    #[test]
    fn the_scale_gives_way_when_the_strip_would_not_fit() {
        let area = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1240.0, 850.0));
        let l = layout(area, (600, 400), 1.0);
        assert_eq!(l.world.size(), egui::vec2(600.0, 400.0));
        assert!(l.hint.bottom() <= area.bottom());
    }
}
