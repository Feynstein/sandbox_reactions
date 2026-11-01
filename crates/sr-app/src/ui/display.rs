//! The display controls under the four views (m0_contrat.md §1.11 [M0-R12], §1.12, §4.2, §4.12; M0-T127b), as a view model:
//! the Glow setting S (`views.glow_white`, kelvins), and the Heat and Density views' ranges (`views.range_from`,
//! `views.range_to`, log₁₀ ends), each a slider over its span with its value typed in place, then `views.fit` and
//! `views.reset`. The state is the engine's own [`Display`] — the window hands it to `Renderer::set_display` when
//! [`apply`] says it changed, never otherwise. It lives apart from `Views`, `Tools` and the world, so switching views and
//! Clear keep it. `panel` says what the left panel shows for the active view; `parse` and `command` are what the egui layer's
//! slider and typed value ask; the Fit query itself (`Renderer::fit_range`, a blocking read-back) is the window's.

use sr_engine::render::{Display, View, DENSITY_SPAN, GLOW_S_DEFAULT, GLOW_S_MAX, GLOW_S_MIN, HEAT_SPAN};

use crate::format;
use crate::strings;

/// Two ends of a range stay at least this far apart, in decades (§4.2).
pub const MIN_GAP: f64 = 0.1;

/// The control a slider or a typed value belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Control {
    Glow,
    From,
    To,
}

/// What a control of the display section asks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DisplayCommand {
    /// The Glow setting, in kelvins.
    SetGlow(f64),
    /// The active view's range start, as log₁₀ (T_K or Σ).
    SetFrom(f64),
    /// The active view's range end.
    SetTo(f64),
    /// `views.fit` was clicked: the window asks the renderer, then sends `SetRange`. `apply` ignores it.
    Fit,
    /// What «Fit to view» found, `[from, to]` in the active view's log₁₀ units.
    SetRange([f64; 2]),
    /// `views.reset`: the active view's setting back to its default.
    Reset,
}

/// One slider with its typed value.
#[derive(Clone, Debug, PartialEq)]
pub struct DisplayControl {
    pub control: Control,
    pub label: String,
    /// The slider's position: log₁₀ of S in kelvins, of T_K, or of Σ.
    pub value: f64,
    pub min: f64,
    pub max: f64,
    /// The value as typed in place shows it (§4.12).
    pub text: String,
}

/// The active view's controls: the sliders, `views.fit` (Heat and Density) and `views.reset`.
#[derive(Clone, Debug, PartialEq)]
pub struct DisplayPanel {
    pub controls: Vec<DisplayControl>,
    pub fit: Option<String>,
    pub reset: String,
}

/// The Glow slider's span, log₁₀ K: 10 K – 10 billion K.
fn glow_span() -> [f64; 2] {
    [GLOW_S_MIN.log10(), GLOW_S_MAX.log10()]
}

/// The span of the log₁₀ quantity a view's controls move: none for Element.
fn span(view: View) -> Option<[f64; 2]> {
    match view {
        View::Glow => Some(glow_span()),
        View::Heat => Some(HEAT_SPAN),
        View::Density => Some(DENSITY_SPAN),
        View::Element => None,
    }
}

/// `x` on a 10⁻⁹ grid, so a slider's `10^x` round trip never moves a value by an ulp.
fn snap(x: f64) -> f64 {
    (x * 1e9).round() / 1e9
}

/// `kelvin` at 9 significant digits: 10^log₁₀ 3000 is 3000 again, not 3000.0000000000005 (S ≤ 3,000 K is the default look
/// byte for byte, §1.11).
fn snap_kelvin(kelvin: f64) -> f64 {
    format!("{kelvin:.8e}").parse().unwrap_or(kelvin)
}

/// The value of a control at log₁₀ position `x`, as typed in place shows it (§4.12): a temperature for Glow and Heat,
/// an inspector number for Density.
pub fn show(view: View, x: f64) -> String {
    let value = 10f64.powf(x);
    if view == View::Density {
        format::number(value)
    } else {
        format::temperature(value)
    }
}

/// What the display section shows for `view` under `display`: none for Element.
pub fn panel(view: View, display: &Display) -> Option<DisplayPanel> {
    let [min, max] = span(view)?;
    let control = |control: Control, key: &str, value: f64| DisplayControl {
        control,
        label: strings::get(key).to_string(),
        value,
        min,
        max,
        text: show(view, value),
    };
    let reset = strings::get("views.reset").to_string();
    Some(match view {
        View::Glow => DisplayPanel {
            controls: vec![control(Control::Glow, "views.glow_white", display.glow_s.log10())],
            fit: None,
            reset,
        },
        View::Heat | View::Density => {
            let [from, to] = if view == View::Heat { display.heat } else { display.density };
            DisplayPanel {
                controls: vec![control(Control::From, "views.range_from", from), control(Control::To, "views.range_to", to)],
                fit: Some(strings::get("views.fit").to_string()),
                reset,
            }
        }
        View::Element => return None,
    })
}

/// A typed number (§4.2): digits with optional "," groups (a group of three after a first of one to three — "1,5" is not a
/// number), a "." and e-notation, and, when `kelvin` allows it, a trailing "K". Anything else is `None`. Never negative.
pub fn parse_number(text: &str, kelvin: bool) -> Option<f64> {
    let mut s = text.trim();
    if kelvin {
        if let Some(rest) = s.strip_suffix(['K', 'k']) {
            s = rest.trim_end();
        }
    }
    let (mantissa, exponent) = match s.split_once(['e', 'E']) {
        Some((m, e)) => (m, Some(e)),
        None => (s, None),
    };
    let exponent = match exponent {
        Some(e) => {
            let digits = e.strip_prefix(['+', '-']).unwrap_or(e);
            if digits.is_empty() || digits.len() > 4 || !digits.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            e
        }
        None => "0",
    };
    let (int, frac) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let digits = |p: &str| p.bytes().all(|b| b.is_ascii_digit());
    if !digits(frac) || (int.is_empty() && frac.is_empty()) {
        return None;
    }
    let int = if int.contains(',') {
        let groups: Vec<&str> = int.split(',').collect();
        let (first, rest) = (groups[0], &groups[1..]);
        let first_ok = (1..=3).contains(&first.len()) && digits(first);
        if !first_ok || !rest.iter().all(|g| g.len() == 3 && digits(g)) {
            return None;
        }
        groups.concat()
    } else if digits(int) {
        int.to_string()
    } else {
        return None;
    };
    let value: f64 = format!("{}.{}e{exponent}", if int.is_empty() { "0" } else { &int }, if frac.is_empty() { "0" } else { frac })
        .parse()
        .ok()?;
    Some(if value.is_finite() { value } else { f64::MAX })
}

/// A typed value of `view`'s controls as the log₁₀ position it stands for, clamped to the control's span ("out of range
/// clamped"); `None` — anything else — and the shown value stays.
pub fn parse(view: View, text: &str) -> Option<f64> {
    let [lo, hi] = span(view)?;
    let value = parse_number(text, view != View::Density)?;
    Some(if value <= 0.0 { lo } else { snap(value.log10()).clamp(lo, hi) })
}

/// The command a slider or a typed value sends: `x` is the log₁₀ position (`parse`'s, or the slider's).
pub fn command(control: Control, x: f64) -> DisplayCommand {
    match control {
        Control::Glow => DisplayCommand::SetGlow(snap_kelvin(10f64.powf(x))),
        Control::From => DisplayCommand::SetFrom(snap(x)),
        Control::To => DisplayCommand::SetTo(snap(x)),
    }
}

/// The range of `view` in `display`, none for the views with none.
fn range_mut(display: &mut Display, view: View) -> Option<&mut [f64; 2]> {
    match view {
        View::Heat => Some(&mut display.heat),
        View::Density => Some(&mut display.density),
        View::Glow | View::Element => None,
    }
}

/// Does what `command` asks of `display` for the active `view`; true when `display` changed — the window hands it to the
/// renderer then, and only then (a slider that repeats its value every frame costs nothing). The ends stay inside the span
/// and [`MIN_GAP`] apart: the end being moved stops at the other. Element has no controls, so nothing applies to it.
pub fn apply(display: &mut Display, view: View, command: DisplayCommand) -> bool {
    let before = *display;
    let Some([lo, hi]) = span(view) else { return false };
    let finite = match command {
        DisplayCommand::SetGlow(x) | DisplayCommand::SetFrom(x) | DisplayCommand::SetTo(x) => x.is_finite(),
        DisplayCommand::SetRange([a, b]) => a.is_finite() && b.is_finite(),
        DisplayCommand::Fit | DisplayCommand::Reset => true,
    };
    if !finite {
        return false;
    }
    match command {
        DisplayCommand::SetGlow(kelvin) if view == View::Glow => {
            display.glow_s = kelvin.clamp(GLOW_S_MIN, GLOW_S_MAX);
        }
        DisplayCommand::SetFrom(x) => {
            if let Some(range) = range_mut(display, view) {
                range[0] = snap(x.clamp(lo, range[1] - MIN_GAP));
            }
        }
        DisplayCommand::SetTo(x) => {
            if let Some(range) = range_mut(display, view) {
                range[1] = snap(x.clamp(range[0] + MIN_GAP, hi));
            }
        }
        DisplayCommand::SetRange([from, to]) => {
            if let Some(range) = range_mut(display, view) {
                let from = from.clamp(lo, hi - MIN_GAP);
                *range = [from, to.clamp(from + MIN_GAP, hi)];
            }
        }
        DisplayCommand::Reset => match view {
            View::Glow => display.glow_s = GLOW_S_DEFAULT,
            View::Heat => display.heat = HEAT_SPAN,
            View::Density => display.density = DENSITY_SPAN,
            View::Element => {}
        },
        DisplayCommand::Fit | DisplayCommand::SetGlow(_) => {}
    }
    *display != before
}

#[cfg(test)]
mod display_tests {
    use super::*;
    use crate::ui::left::{ViewCommand, Views, VIEWS};

    fn texts(view: View) -> Vec<String> {
        let p = panel(view, &Display::default()).expect("the view has controls");
        let mut out: Vec<String> = p.controls.iter().map(|c| c.label.clone()).collect();
        out.extend(p.fit);
        out.push(p.reset);
        out
    }

    #[test]
    fn the_texts_are_byte_exact_and_element_has_none() {
        assert_eq!(texts(View::Glow), ["Full brightness at", "Reset"]);
        assert_eq!(texts(View::Heat), ["From", "To", "Fit to view", "Reset"]);
        assert_eq!(texts(View::Density), ["From", "To", "Fit to view", "Reset"]);
        assert_eq!(panel(View::Element, &Display::default()), None);
    }

    #[test]
    fn the_values_read_in_the_inspectors_formats() {
        let d = Display::default();
        let glow = panel(View::Glow, &d).unwrap().controls;
        assert_eq!(glow[0].text, "3,000 K");
        let heat = panel(View::Heat, &d).unwrap().controls;
        assert_eq!((heat[0].text.as_str(), heat[1].text.as_str()), ("10.0 K", "10.0 billion K"));
        let density = panel(View::Density, &d).unwrap().controls;
        assert_eq!((density[0].text.as_str(), density[1].text.as_str()), (format::number(1e-6).as_str(), format::number(1e4).as_str()));
        assert_eq!(show(View::Glow, 6.0), "1.00 million K");
    }

    #[test]
    fn good_numbers_parse_grouped_e_notation_and_k_alike() {
        for (text, want) in [
            ("5000", 5000.0),
            (" 5000 ", 5000.0),
            ("5000 K", 5000.0),
            ("5000K", 5000.0),
            ("5,000", 5000.0),
            ("5,000 k", 5000.0),
            ("1,234,567", 1_234_567.0),
            ("12,345.5", 12_345.5),
            ("3e3", 3000.0),
            ("3e3K", 3000.0),
            ("3E+3 K", 3000.0),
            ("1.5e4", 15_000.0),
            ("1e-3", 0.001),
            (".5", 0.5),
            ("2.", 2.0),
            ("0", 0.0),
        ] {
            assert_eq!(parse_number(text, true), Some(want), "{text:?}");
        }
        // The group never reads "1,5" as 15, and a comma never reads as a decimal point.
        assert_eq!(parse_number("1,500", true), Some(1500.0));
        assert_ne!(parse_number("1,5", true), Some(15.0));
    }

    #[test]
    fn junk_restores_the_shown_value() {
        for text in [
            "", " ", "K", "abc", "1,5", "1,50", "12,34", "1,2345", ",500", "1,,000", "1,000,", "1,000.5,0", "-5", "+5", "5 6", "1e", "e3",
            "1e+", "1.2.3", "inf", "NaN", "5KK", "K5", "5K5", "1 000", "0x10", "1e99999", "5 million K",
        ] {
            let got = parse_number(text, true);
            assert!(got.is_none(), "{text:?} read as {got:?}");
        }
        assert_eq!(parse(View::Glow, "junk"), None);
        assert_eq!(parse(View::Density, "5 K"), None, "Density is no temperature");
        assert_eq!(parse(View::Element, "5"), None);
    }

    #[test]
    fn out_of_range_is_clamped_to_the_span() {
        assert_eq!(parse(View::Glow, "5"), Some(1.0));
        assert_eq!(parse(View::Glow, "0"), Some(1.0));
        assert_eq!(parse(View::Glow, "1e12"), Some(10.0));
        assert_eq!(parse(View::Glow, "1e9999"), Some(10.0), "past the float range is the top, not junk");
        assert_eq!(parse(View::Glow, "3e3K"), Some(snap(3000f64.log10())));
        assert_eq!(parse(View::Heat, "0.5 K"), Some(1.0));
        assert_eq!(parse(View::Heat, "1e11"), Some(10.0));
        assert_eq!(parse(View::Density, "1e-9"), Some(-6.0));
        assert_eq!(parse(View::Density, "1e6"), Some(4.0));
        assert_eq!(parse(View::Density, "0.001"), Some(-3.0));
    }

    #[test]
    fn the_slider_maps_logarithmically_and_lands_on_exact_kelvins() {
        let (lo, hi) = (glow_span()[0], glow_span()[1]);
        assert_eq!((lo, hi), (1.0, 10.0));
        assert_eq!(command(Control::Glow, lo), DisplayCommand::SetGlow(10.0));
        assert_eq!(command(Control::Glow, hi), DisplayCommand::SetGlow(1e10));
        assert_eq!(command(Control::Glow, 5.5), DisplayCommand::SetGlow(snap_kelvin(10f64.powf(5.5))));
        // The slider's midpoint is the geometric middle of 10 K and 10 billion K, not the arithmetic one.
        let DisplayCommand::SetGlow(mid) = command(Control::Glow, 5.5) else { unreachable!() };
        assert!((mid / 316_227.766 - 1.0).abs() < 1e-6, "{mid}");
        // 3,000 K typed, or slid to its own position, is 3000.0 exactly — the default look byte for byte.
        let x = parse(View::Glow, "3000").unwrap();
        assert_eq!(command(Control::Glow, x), DisplayCommand::SetGlow(3000.0));
        assert_eq!(command(Control::Glow, 3000f64.log10()), DisplayCommand::SetGlow(3000.0));
        // The panel's slider position is log₁₀ S.
        let mut d = Display::default();
        for kelvin in [10.0, 300.0, 3000.0, 1e5, 1e10] {
            assert!(apply(&mut d, View::Glow, DisplayCommand::SetGlow(kelvin)) || kelvin == 3000.0);
            let c = &panel(View::Glow, &d).unwrap().controls[0];
            assert!((c.value - kelvin.log10()).abs() < 1e-12);
            assert_eq!(command(Control::Glow, c.value), DisplayCommand::SetGlow(kelvin), "{kelvin}");
        }
    }

    #[test]
    fn from_and_to_are_kept_a_tenth_of_a_decade_apart() {
        for view in [View::Heat, View::Density] {
            let [lo, hi] = span(view).unwrap();
            let range = |d: &Display| if view == View::Heat { d.heat } else { d.density };
            let mut d = Display::default();
            // From pushed past To stops under it; To pushed under From stops over it.
            apply(&mut d, view, DisplayCommand::SetFrom(hi));
            assert!((range(&d)[1] - range(&d)[0] - MIN_GAP).abs() < 1e-9, "{view:?} {:?}", range(&d));
            apply(&mut d, view, DisplayCommand::SetTo(lo));
            assert!((range(&d)[1] - range(&d)[0] - MIN_GAP).abs() < 1e-9, "{view:?} {:?}", range(&d));
            assert_eq!(range(&d)[0], hi - MIN_GAP, "To stopped over From, From stayed");
            // Never outside the span, never closer than the gap, whatever the order of a sweep of values.
            let mut d = Display::default();
            for i in 0..200 {
                let x = lo - 3.0 + (hi - lo + 6.0) * ((i * 37) % 200) as f64 / 199.0;
                apply(&mut d, view, if i % 2 == 0 { DisplayCommand::SetFrom(x) } else { DisplayCommand::SetTo(x) });
                let [a, b] = range(&d);
                assert!(lo <= a && b <= hi && b - a >= MIN_GAP - 1e-9, "{view:?} {a} {b} after {x}");
            }
        }
        // Typed ends at the extremes of the span: From "1e10" under the default To is 9.9, not 10.
        let mut d = Display::default();
        let x = parse(View::Heat, "1e10").unwrap();
        apply(&mut d, View::Heat, command(Control::From, x));
        assert_eq!(d.heat, [9.9, 10.0]);
    }

    #[test]
    fn reset_goes_to_the_defaults_of_the_active_view_alone() {
        let mut d = Display::default();
        apply(&mut d, View::Glow, DisplayCommand::SetGlow(1e6));
        apply(&mut d, View::Heat, DisplayCommand::SetFrom(3.0));
        apply(&mut d, View::Heat, DisplayCommand::SetTo(5.0));
        apply(&mut d, View::Density, DisplayCommand::SetFrom(-2.0));
        assert!(apply(&mut d, View::Glow, DisplayCommand::Reset));
        assert_eq!(d.glow_s, 3000.0);
        assert_eq!((d.heat, d.density), ([3.0, 5.0], [-2.0, DENSITY_SPAN[1]]));
        assert!(apply(&mut d, View::Heat, DisplayCommand::Reset));
        assert_eq!(d.heat, HEAT_SPAN);
        assert_eq!(d.density, [-2.0, DENSITY_SPAN[1]]);
        assert!(apply(&mut d, View::Density, DisplayCommand::Reset));
        assert_eq!(d, Display::default());
        assert!(!apply(&mut d, View::Density, DisplayCommand::Reset), "nothing to reset is no change");
    }

    #[test]
    fn fit_sets_the_active_views_range_and_element_ignores_everything() {
        let mut d = Display::default();
        assert!(apply(&mut d, View::Heat, DisplayCommand::SetRange([3.2, 4.4])));
        assert_eq!((d.heat, d.density, d.glow_s), ([3.2, 4.4], DENSITY_SPAN, 3000.0));
        assert!(!apply(&mut d, View::Heat, DisplayCommand::Fit), "the window answers Fit with SetRange");
        // Glow has no range; a range sent to it, or to Element, changes nothing.
        let before = d;
        for view in [View::Glow, View::Element] {
            for c in [DisplayCommand::SetRange([2.0, 3.0]), DisplayCommand::SetFrom(2.0), DisplayCommand::SetTo(9.0), DisplayCommand::Reset] {
                assert!(!apply(&mut d, view, c), "{view:?} {c:?}");
            }
        }
        assert!(!apply(&mut d, View::Element, DisplayCommand::SetGlow(1e6)));
        assert!(!apply(&mut d, View::Heat, DisplayCommand::SetGlow(1e6)), "S is the Glow view's");
        assert_eq!(d, before);
        // A fit past the span is cut to it, 0.1 apart.
        apply(&mut d, View::Density, DisplayCommand::SetRange([-3.0, 1.0]));
        assert!(apply(&mut d, View::Density, DisplayCommand::SetRange([-9.0, 12.0])));
        assert_eq!(d.density, DENSITY_SPAN);
        assert!(apply(&mut d, View::Density, DisplayCommand::SetRange([4.0, 4.0])));
        assert!((d.density[1] - d.density[0] - MIN_GAP).abs() < 1e-9);
    }

    #[test]
    fn switching_views_keeps_every_setting_and_shows_the_active_ones() {
        let mut d = Display::default();
        apply(&mut d, View::Glow, DisplayCommand::SetGlow(2e5));
        apply(&mut d, View::Heat, DisplayCommand::SetRange([2.0, 6.0]));
        apply(&mut d, View::Density, DisplayCommand::SetRange([-3.0, 1.0]));
        let kept = d;
        let mut views = Views::default();
        for _ in 0..2 {
            for view in VIEWS {
                views.apply(ViewCommand::Select(view));
                assert_eq!(d, kept, "{view:?}");
                let shown = panel(views.active(), &d).map(|p| p.controls.iter().map(|c| c.value).collect::<Vec<_>>());
                let want: Option<Vec<f64>> = match view {
                    View::Glow => Some(vec![2e5f64.log10()]),
                    View::Heat => Some(vec![2.0, 6.0]),
                    View::Density => Some(vec![-3.0, 1.0]),
                    View::Element => None,
                };
                assert_eq!(shown, want, "{view:?}");
            }
        }
    }

    #[test]
    fn the_renderer_is_asked_only_when_a_setting_changed() {
        // The window calls `Renderer::set_display` for each `true`: a slider that writes S every frame while it rests, or
        // while it is dragged to the same place, must be one change; each new place is one.
        let mut d = Display::default();
        let mut asked = 0;
        let x = 5.0;
        for _ in 0..120 {
            if apply(&mut d, View::Glow, command(Control::Glow, x)) {
                asked += 1;
            }
        }
        assert_eq!((asked, d.glow_s), (1, 1e5));
        let mut asked = 0;
        for i in 0..60 {
            // 60 frames of a drag over 20 distinct places, each held three frames.
            let x = 2.0 + 0.1 * (i / 3) as f64;
            if apply(&mut d, View::Glow, command(Control::Glow, x)) {
                asked += 1;
            }
        }
        assert_eq!(asked, 20);
        // The ranges too: the same From written every frame is one change; so is a Reset at the default, none.
        let mut asked = 0;
        for _ in 0..60 {
            asked += u32::from(apply(&mut d, View::Heat, command(Control::From, 3.3)));
        }
        assert_eq!(asked, 1);
        let s = d.glow_s;
        assert!(!apply(&mut d, View::Glow, DisplayCommand::SetGlow(s)));
    }
}
