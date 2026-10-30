//! The design tokens (m0_contrat.md §4.11) as egui colours, and the `Visuals` built from them (M0-D119). Every colour the
//! window draws is one of these constants or comes out of `visuals()` — egui's own themes (selection teal, grey chips,
//! #1B1B1B windows; the light theme's pale fills) are replaced once, at start-up (`install`), and no other file writes a
//! colour literal.

use eframe::egui::{self, Color32, CornerRadius, Shadow, Stroke};

/// `bg.space` (§4.11): the world's background and vacuum, #05070D.
pub const BG_SPACE: Color32 = Color32::from_rgb(0x05, 0x07, 0x0D);
/// `bg.panel` and `border.panel` (§4.11): #10141E and #232A3A.
pub const BG_PANEL: Color32 = Color32::from_rgb(0x10, 0x14, 0x1E);
pub const BORDER_PANEL: Color32 = Color32::from_rgb(0x23, 0x2A, 0x3A);
/// `text.primary` and `text.secondary` (§4.11): #E6EAF2 and #9AA4B8.
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xE6, 0xEA, 0xF2);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0x9A, 0xA4, 0xB8);
/// `accent` (§4.11): #F2B33D.
pub const ACCENT: Color32 = Color32::from_rgb(0xF2, 0xB3, 0x3D);
/// `banner.bg` and `banner.text` (§4.11): #3A2A0B and #FFD27A.
pub const BANNER_BG: Color32 = Color32::from_rgb(0x3A, 0x2A, 0x0B);
pub const BANNER_TEXT: Color32 = Color32::from_rgb(0xFF, 0xD2, 0x7A);
/// `error.text` (§4.11): #FF6B6B.
pub const ERROR_TEXT: Color32 = Color32::from_rgb(0xFF, 0x6B, 0x6B);
/// The banner's font size in px (§4.11).
pub const BANNER_FONT: f32 = 20.0;

/// The modal's backdrop: `bg.space` at 60 %, in place of egui's black at 100/255.
pub fn backdrop() -> Color32 {
    BG_SPACE.gamma_multiply(0.6)
}

/// egui's `Visuals` from the tokens (§4.2: «Selected tool, rung and view use `accent`»). A selected rung is `accent` text
/// (`selection.stroke`, which egui paints a selected button's text with) on `border.panel` (`selection.bg_fill`); an
/// unselected chip is `bg.panel` outlined by `border.panel`, `border.panel` filled under the pointer; every window,
/// panel and separator takes `bg.panel` and `border.panel`. The shadows are off — none of §4.11's colours is a shadow.
pub fn visuals() -> egui::Visuals {
    let mut v = egui::Visuals::dark();
    let outline = Stroke::new(1.0, BORDER_PANEL);
    let ink = Stroke::new(1.0, TEXT_PRIMARY);

    v.widgets.noninteractive.bg_fill = BG_PANEL;
    v.widgets.noninteractive.weak_bg_fill = BG_PANEL;
    v.widgets.noninteractive.bg_stroke = outline;
    v.widgets.noninteractive.fg_stroke = ink;

    v.widgets.inactive.bg_fill = BG_PANEL;
    v.widgets.inactive.weak_bg_fill = BG_PANEL;
    v.widgets.inactive.bg_stroke = outline;
    v.widgets.inactive.fg_stroke = ink;

    for hot in [&mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
        hot.bg_fill = BORDER_PANEL;
        hot.weak_bg_fill = BORDER_PANEL;
        hot.bg_stroke = Stroke::new(1.0, TEXT_SECONDARY);
        hot.fg_stroke = ink;
    }

    v.selection.bg_fill = BORDER_PANEL;
    v.selection.stroke = Stroke::new(1.0, ACCENT);

    v.window_fill = BG_PANEL;
    v.window_stroke = outline;
    v.panel_fill = BG_PANEL;
    v.extreme_bg_color = BG_SPACE;
    v.text_edit_bg_color = Some(BG_SPACE);
    v.faint_bg_color = BG_PANEL;
    v.code_bg_color = BG_PANEL;
    v.hyperlink_color = ACCENT;
    v.warn_fg_color = ACCENT;
    v.error_fg_color = ERROR_TEXT;
    v.window_shadow = Shadow::NONE;
    v.popup_shadow = Shadow::NONE;
    v.window_corner_radius = CornerRadius::same(6);
    v
}

/// Puts the tokens' `Visuals` on the context for good (M0-V12). egui follows the system's light or dark setting by default
/// (`ThemePreference::System`) and `set_visuals` dresses only the theme active at the call, so a light browser or desktop
/// drew egui's light fills under the tokens' light text — the bar's buttons blank. The window is dark whatever the system
/// prefers: the preference pinned to dark, and both of egui's themes given the tokens.
pub fn install(ctx: &egui::Context) {
    ctx.set_theme(egui::ThemePreference::Dark);
    ctx.set_visuals_of(egui::Theme::Dark, visuals());
    ctx.set_visuals_of(egui::Theme::Light, visuals());
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOKENS: [Color32; 9] =
        [BG_SPACE, BG_PANEL, BORDER_PANEL, TEXT_PRIMARY, TEXT_SECONDARY, ACCENT, BANNER_BG, BANNER_TEXT, ERROR_TEXT];

    /// Every colour `visuals()` hands egui that the bar, the dialog or the status bar can reach.
    fn painted(v: &egui::Visuals) -> Vec<(&'static str, Color32)> {
        let mut all = Vec::new();
        for (name, w) in [
            ("noninteractive", &v.widgets.noninteractive),
            ("inactive", &v.widgets.inactive),
            ("hovered", &v.widgets.hovered),
            ("active", &v.widgets.active),
            ("open", &v.widgets.open),
        ] {
            all.extend([(name, w.bg_fill), (name, w.weak_bg_fill), (name, w.bg_stroke.color), (name, w.fg_stroke.color)]);
        }
        all.extend([
            ("selection.bg_fill", v.selection.bg_fill),
            ("selection.stroke", v.selection.stroke.color),
            ("window_fill", v.window_fill),
            ("window_stroke", v.window_stroke.color),
            ("panel_fill", v.panel_fill),
            ("extreme_bg_color", v.extreme_bg_color),
            ("text_edit_bg_color", v.text_edit_bg_color.unwrap_or(BG_SPACE)),
            ("faint_bg_color", v.faint_bg_color),
            ("code_bg_color", v.code_bg_color),
            ("hyperlink_color", v.hyperlink_color),
            ("warn_fg_color", v.warn_fg_color),
            ("error_fg_color", v.error_fg_color),
        ]);
        all
    }

    #[test]
    fn every_colour_of_the_visuals_is_a_token() {
        for (name, colour) in painted(&visuals()) {
            assert!(TOKENS.contains(&colour), "{name} is {colour:?}, not a §4.11 token");
        }
    }

    #[test]
    fn a_selected_rung_is_accent_text_on_a_token_fill() {
        let v = visuals();
        assert_eq!(v.selection.stroke.color, ACCENT);
        assert_eq!(v.selection.bg_fill, BORDER_PANEL);
        assert_ne!(v.selection.bg_fill, v.selection.stroke.color);
    }

    #[test]
    fn the_windows_and_the_chips_read_the_panel_tokens() {
        let v = visuals();
        assert_eq!((v.window_fill, v.window_stroke.color), (BG_PANEL, BORDER_PANEL));
        assert_eq!((v.widgets.inactive.weak_bg_fill, v.widgets.noninteractive.bg_stroke.color), (BG_PANEL, BORDER_PANEL));
        assert_eq!(v.window_shadow, Shadow::NONE);
        assert_eq!(v.popup_shadow, Shadow::NONE);
    }

    /// A light browser or desktop still gets the tokens (M0-V12): egui reads the system's theme from each pass's input, and
    /// with the tokens set on the active theme alone a light system switched egui to its light fills under the light text.
    #[test]
    fn a_light_system_still_draws_the_tokens() {
        let ctx = egui::Context::default();
        install(&ctx);
        let _ = ctx.run_ui(egui::RawInput { system_theme: Some(egui::Theme::Light), ..Default::default() }, |_| {});
        let v = &ctx.global_style().visuals;
        assert_eq!(ctx.theme(), egui::Theme::Dark);
        assert_eq!((v.widgets.inactive.weak_bg_fill, v.selection.bg_fill, v.window_fill), (BG_PANEL, BORDER_PANEL, BG_PANEL));
    }

    /// Both entries install the tokens through `install` — the window and the web canvas alike — and neither sets egui's
    /// visuals itself, which would dress the start-up theme alone.
    #[test]
    fn the_desktop_and_the_web_install_the_tokens() {
        for (file, source) in [("desktop.rs", include_str!("../desktop.rs")), ("web.rs", include_str!("../web.rs"))] {
            assert!(source.contains("theme::install(&cc.egui_ctx)"), "{file} does not install the tokens");
            assert!(!source.contains("set_visuals("), "{file} sets egui's visuals itself");
        }
    }

    #[test]
    fn the_modal_backdrop_is_bg_space_dimmed() {
        let b = backdrop();
        assert!(b.a() > 0 && b.a() < 255);
        assert_eq!(b.to_opaque(), BG_SPACE.gamma_multiply(0.6).to_opaque());
    }

    /// No other file of the window writes a colour: desktop.rs and the bar read the tokens by name (egui's `Color32::WHITE`
    /// is the image's no-op tint, not a theme colour).
    #[test]
    fn the_window_files_write_no_colour_literal() {
        for (file, source) in [("desktop.rs", include_str!("../desktop.rs")), ("ui/topbar.rs", include_str!("topbar.rs"))] {
            for call in ["from_rgb", "from_gray", "from_black_alpha", "from_white_alpha", "from_rgba", "from_hex", "Color32::BLACK"] {
                assert!(!source.contains(call), "{file} writes a colour literal ({call})");
            }
        }
    }
}
