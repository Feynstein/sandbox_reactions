//! The desktop app (m0_contrat.md §3.3, §4.2, §6.1): eframe on the wgpu backend, a 1760 × 940 window titled
//! `app.title`, the world drawn in the centre at the integer scale (§1.11) on `bg.space`, the status bar under it.
//! `SandboxApp` is one type for the desktop and the web entry (web.rs runs the same app in the canvas); the native-only
//! parts — `--adapter`, `--offscreen-window`, the status endpoint — sit behind `cfg(not(target_arch = "wasm32"))`.
//!
//! The frame loop (§1.8.3, §3.1; M0-T84) is `sim::Sim`: each frame `TimeControl` plans the steps, the engine's
//! `encode_frame` runs them, `render` draws the active view into the texture the centre shows, and `poll()` — which never
//! blocks — brings the frames read back (≤ 2 frames late) to `on_report`.

use std::collections::VecDeque;
use std::sync::Arc;

use eframe::{egui, egui_wgpu};
use sr_engine::render::View;

use crate::format;
use crate::ui::banner::{self, Banner};
use crate::ui::display::{self, DisplayCommand, DisplayPanel};
use crate::ui::left::{self, LeftPanel};
use crate::ui::legend::{self, Bar, Legend};
use crate::ui::remnant;
use crate::ui::zoom::{self, Zoom};
use crate::ui::right::{CellPanel, StarPanel};
use crate::ui::theme::{
    self, ACCENT, BANNER_BG, BANNER_FONT, BANNER_TEXT, BG_PANEL, BG_SPACE, BORDER_PANEL, ERROR_TEXT, HEADING_FONT, STATUS_FONT, TEXT_PRIMARY,
    TEXT_SECONDARY,
};
use crate::ui::topbar::{self, Command, Dialog, TopBar};

/// The default window (§4.2), in logical px.
#[cfg(not(target_arch = "wasm32"))]
pub const WINDOW_SIZE: [f32; 2] = [1760.0, 940.0];

/// The device every frame runs on, desktop and web alike (§6.1, §6.2.2): WebGPU's default limits and no feature — bar the
/// native build's pass-boundary timestamps where the adapter offers them (the headless device's rule, `Gpu::new_headless`),
/// which the frame's cost per step (§1.8.3's c_step) is read from; without them the cost is the wall clock's.
fn device_descriptor(adapter: &wgpu::Adapter) -> wgpu::DeviceDescriptor<'static> {
    wgpu::DeviceDescriptor {
        label: Some("sr-app"),
        required_features: if cfg!(target_arch = "wasm32") {
            wgpu::Features::empty()
        } else {
            adapter.features() & wgpu::Features::TIMESTAMP_QUERY
        },
        required_limits: wgpu::Limits::default(),
        ..Default::default()
    }
}

/// The wgpu setup both targets start from: the engine's instance flags, the high-performance adapter,
/// `device_descriptor`'s device.
pub fn wgpu_setup() -> egui_wgpu::WgpuSetupCreateNew {
    let mut setup = egui_wgpu::WgpuSetupCreateNew::without_display_handle();
    setup.instance_descriptor.flags = sr_engine::gpu::instance_flags();
    setup.power_preference = wgpu::PowerPreference::HighPerformance;
    setup.device_descriptor = Arc::new(device_descriptor);
    setup
}

pub struct SandboxApp {
    frames: u64,
    /// The times (egui's clock, s) of the frames of the last second, for `fps`.
    recent: VecDeque<f64>,
    /// Raised by the first frame known to be presented (see `ui`), never by the window's creation.
    presented: bool,
    /// The adapter's name, for `status.line`.
    adapter: String,
    /// The world and its frame loop (§1.8.3).
    sim: sim::Sim,
    /// The legend's bars, kept while the Glow setting holds (ui::legend).
    legend_bars: legend::Bars,
    /// The zoom (§1.11 [M0-R12], ui::zoom): kept by Clear, as the four display settings are.
    zoom: Zoom,
    /// The wheel's notches until they make a step of z.
    wheel: zoom::Wheel,
    #[cfg(not(target_arch = "wasm32"))]
    status: Option<std::sync::Arc<crate::status::Status>>,
    /// `--capture`'s driver (capture.rs): the app waits on it before it advances.
    #[cfg(not(target_arch = "wasm32"))]
    capture: Option<crate::capture::Driver>,
    /// What a script's `hover` action holds the pointer on: it stands in for the pointer, which a capture has none of (`Some(None)`:
    /// the pointer held off the world).
    #[cfg(not(target_arch = "wasm32"))]
    scripted_hover: Option<Option<[u32; 2]>>,
    /// `--measure-ui`'s meter (measure.rs): the frames' cost is read at the start of each pass.
    #[cfg(not(target_arch = "wasm32"))]
    measure: Option<crate::measure::Meter>,
    /// What the pass has spent on the world itself (the read-back and the frame): not the UI's, so `--measure-ui` takes it off.
    #[cfg(not(target_arch = "wasm32"))]
    sim_cost: std::time::Duration,
}

impl SandboxApp {
    /// A new world — empty, at ×1, running (§1.8.2) — on the render state's device; an error is the `SR-ERROR` line that
    /// says why the engine could not start (a stale calibration is exit 6 on the desktop, §2.11).
    pub fn new(render: &egui_wgpu::RenderState) -> Result<Self, String> {
        let sim = sim::Sim::new(render)?;
        Ok(Self {
            frames: 0,
            recent: VecDeque::new(),
            presented: false,
            adapter: render.adapter.get_info().name,
            zoom: Zoom::home(sim.world()),
            wheel: zoom::Wheel::default(),
            sim,
            legend_bars: legend::Bars::default(),
            #[cfg(not(target_arch = "wasm32"))]
            status: None,
            #[cfg(not(target_arch = "wasm32"))]
            capture: None,
            #[cfg(not(target_arch = "wasm32"))]
            scripted_hover: None,
            #[cfg(not(target_arch = "wasm32"))]
            measure: None,
            #[cfg(not(target_arch = "wasm32"))]
            sim_cost: std::time::Duration::ZERO,
        })
    }

    /// The device work of `new`, built and dropped (web.rs's warm-up, M0-D118).
    #[cfg(target_arch = "wasm32")]
    pub fn dry_run(device: &wgpu::Device, queue: &wgpu::Queue) -> Result<(), String> {
        sim::Sim::dry_run(device, queue)
    }

    /// The active view when it is one of `views`, else the error naming `action` (the display controls exist under these views only).
    #[cfg(not(target_arch = "wasm32"))]
    fn expect_view(&self, views: &[View], action: &str) -> Result<View, String> {
        let active = self.sim.views().active();
        if views.contains(&active) {
            Ok(active)
        } else {
            Err(format!("{action}: the {active:?} view has no such control (it acts on {views:?})"))
        }
    }

    /// The web page's URL parameters (§3.5) through the same doors a script's actions use: the view and the rung, the
    /// preset dropped at the world's centre by the panel's path (§4.5), then `steps` forced through the loop and the clock
    /// paused. An error is the line the page fails with.
    #[cfg(target_arch = "wasm32")]
    pub fn open_with(&mut self, query: &crate::web::Query, now: f64) -> Result<(), String> {
        if let Some(view) = query.view {
            self.sim.view_command(left::ViewCommand::Select(view));
        }
        if let Some(k) = query.rung {
            self.sim.command(if k == 0 { Command::Max } else { Command::Numbered(k) });
        }
        if let Some(preset) = query.preset {
            self.sim.preset_command(left::PresetCommand::Arm(preset));
            let world = self.sim.world();
            self.sim.place([world.width / 2, world.height / 2], now).map_err(|e| format!("preset: {e}"))?;
        }
        if let Some(n) = query.steps {
            self.sim.queue_steps(n);
            if !self.sim.paused() {
                self.sim.command(Command::TogglePause);
            }
        }
        Ok(())
    }

    /// A scripted action (§3.3) — the ones built so far. An error is the line the capture fails with.
    #[cfg(not(target_arch = "wasm32"))]
    fn apply(&mut self, action: &crate::capture::Action, now: f64) -> Result<(), String> {
        use crate::capture::Action;
        match action {
            // The clock's actions go through the bar's own door (ui::topbar::apply), as a click or a key does.
            Action::Pause(p) => {
                if self.sim.paused() != *p {
                    self.sim.command(Command::TogglePause);
                }
            }
            Action::Rung(k) => self.sim.command(if *k == 0 { Command::Max } else { Command::Numbered(*k) }),
            Action::SlowDown(on) => self.sim.command(Command::SetSlowDown(*on)),
            Action::Edge(mode) => self.sim.command(Command::SetEdge(*mode)),
            Action::Steps(n) => self.sim.queue_steps(*n),
            // The panel's path (§4.5): the button arms the preset and a click on cell `at` (the world's centre when none) drops
            // it. A drop that does not fit is the window's own: the preset stays armed, `presets.no_room` is shown, the capture goes on.
            Action::Preset { name, at } => {
                let preset = left::Preset::parse(name).ok_or_else(|| format!("unknown preset {name:?}"))?;
                self.sim.preset_command(left::PresetCommand::Arm(preset));
                let world = self.sim.world();
                let cell = at.unwrap_or([world.width / 2, world.height / 2]);
                match self.sim.place(cell, now) {
                    Ok(()) | Err(sr_engine::step::edit::EditError::NoRoom) => {}
                    Err(e) => return Err(format!("preset {name}: {e}")),
                }
            }
            // The view buttons' door (ui::left::Views::apply), as a click or F1–F4 go through.
            Action::View(view) => self.sim.view_command(left::ViewCommand::Select(*view)),
            // The right click's door (ui::right::Stars::select_at), on the cell as the pointer would name it.
            Action::Select(cell) => self.sim.select_star(*cell),
            // The pointer's place, as the central panel reads it each frame (ui::right::Asker asks the engine for the cell).
            // A cell off the world is the pointer off the world (§3.3), as `cell_at` names it.
            Action::Hover(cell) => {
                let world = self.sim.world();
                self.scripted_hover = Some((cell[0] < world.width && cell[1] < world.height).then_some(*cell));
            }
            // The panel's own door too (ui::left::Tools::apply), as a click or a key goes through.
            Action::Tool(tool) => self.sim.tool_command(left::Command::SelectTool(*tool)),
            Action::Element(key) => {
                let id = self.sim.species_of(key).ok_or_else(|| format!("element {key:?} is not in the list"))?;
                self.sim.tool_command(left::Command::SelectElement(id));
            }
            // The display section's door (Sim::display_command), on the active view as the controls act: a view the control
            // is not shown under is an error, never a silent no-op.
            Action::GlowWhite(kelvin) => {
                self.expect_view(&[View::Glow], "glow_white")?;
                let x = display::parse(View::Glow, &kelvin.to_string()).ok_or("glow_white: not a number the control takes")?;
                self.sim.display_command(DisplayCommand::SetGlow(10f64.powf(x)));
            }
            Action::Range([from, to]) => {
                let view = self.expect_view(&[View::Heat, View::Density], "range")?;
                let (from, to) = (display::parse(view, &from.to_string()), display::parse(view, &to.to_string()));
                let (Some(from), Some(to)) = (from, to) else { return Err("range: not numbers the controls take".into()) };
                self.sim.display_command(DisplayCommand::SetRange([from, to]));
            }
            Action::Fit => {
                self.expect_view(&[View::Heat, View::Density], "fit")?;
                self.sim.display_command(DisplayCommand::Fit);
            }
            Action::DisplayReset => {
                self.expect_view(&[View::Glow, View::Heat, View::Density], "display_reset")?;
                self.sim.display_command(DisplayCommand::Reset);
            }
            // The view's own door (ui::zoom::Zoom), as the wheel and the middle drag go through.
            Action::Zoom { z, at } => {
                let world = self.sim.world();
                if at[0] >= world.width || at[1] >= world.height {
                    return Err(format!("zoom: cell {at:?} is outside the {} x {} world", world.width, world.height));
                }
                self.zoom.centre_on(*z, *at, world);
            }
            // One frame of the held button per cell, with the tool and the element the panel holds.
            Action::Paint(cells) => {
                for cell in cells {
                    self.sim.edit(Some(*cell)).map_err(|e| format!("paint {cell:?}: {e}"))?;
                }
            }
        }
        Ok(())
    }
}

impl eframe::App for SandboxApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // The presented-frame callback: egui-wgpu delivers a screenshot only for a frame whose surface texture it
        // acquired, and presents that frame right after starting the copy — so the event, read here a pass later,
        // proves a frame reached the screen. A frame that could not acquire its texture drops the request, so it is
        // re-asked every pass until one arrives.
        if !self.presented {
            if ui.input(|i| i.raw.events.iter().any(|e| matches!(e, egui::Event::Screenshot { .. }))) {
                self.presented = true;
                #[cfg(target_arch = "wasm32")]
                crate::web::signal_ready();
            } else {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
            }
        }

        // `--measure-ui` (measure.rs): the Sun-like preset at ×1 on the first pass; every pass reads what the pass before cost.
        #[cfg(not(target_arch = "wasm32"))]
        {
            let sim_s = std::mem::take(&mut self.sim_cost).as_secs_f64();
            if let Some(mut meter) = self.measure.take() {
                let mut closed = false;
                if meter.first_pass() {
                    let now = ui.input(|i| i.time);
                    let drop = [crate::capture::Action::Rung(1), crate::capture::Action::Preset { name: "sun".into(), at: None }];
                    if let Some(e) = drop.iter().find_map(|a| self.apply(a, now).err()) {
                        meter.fail(e);
                        closed = true;
                    }
                }
                if closed || meter.pass(frame.info().cpu_usage, sim_s) {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
                self.measure = Some(meter);
            }
        }

        // A capture's step begins here: its actions apply before this pass advances, and the app stands still from the
        // pass that asked for a screenshot until the reply is written.
        #[cfg(not(target_arch = "wasm32"))]
        let frozen = match self.capture.take() {
            Some(mut driver) => {
                let begin = driver.begin(ui.ctx(), self.presented);
                let now = ui.input(|i| i.time);
                if let Some(e) = begin.actions.iter().find_map(|a| self.apply(a, now).err()) {
                    driver.abort(ui.ctx(), e);
                }
                self.capture = Some(driver);
                begin.frozen
            }
            None => false,
        };
        #[cfg(target_arch = "wasm32")]
        let frozen = false;

        // The frames read back since the last pass book their events into the clock first, then the bar and the banner are
        // built from that one clock state (M0-D120); with no slow-down banner, the ignition notice takes its slot (M0-D139). The
        // keys of the bar (§4.10) and its buttons reach the clock before this pass plans its frame.
        #[cfg(not(target_arch = "wasm32"))]
        let polled = std::time::Instant::now();
        let (bar, banner) = banner::views_at(&mut self.sim, ui.input(|i| i.time));
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.sim_cost += polled.elapsed();
        }
        let Keys { bar_commands: mut commands, tool_commands, preset_commands, view_commands, zoom_home } =
            key_commands(ui.ctx(), self.sim.presets().armed().is_some());
        if zoom_home {
            self.zoom.reset(self.sim.world());
        }
        egui::Panel::top("topbar")
            .exact_size(TOP_BAR_HEIGHT)
            .frame(
                egui::Frame::NONE
                    .fill(BG_PANEL)
                    .stroke(egui::Stroke::new(1.0, BORDER_PANEL))
                    .inner_margin(egui::Margin::symmetric(8, 0)),
            )
            .show(ui, |ui| commands.extend(draw_top_bar(ui, &bar)));
        if let Some(dialog) = &bar.dialog {
            commands.extend(draw_dialog(ui.ctx(), dialog));
        }
        for command in commands {
            self.sim.command(command);
        }
        for command in tool_commands {
            self.sim.tool_command(command);
        }
        for command in preset_commands {
            self.sim.preset_command(command);
        }
        for command in view_commands {
            self.sim.view_command(command);
        }

        let now = ui.input(|i| i.time);
        // The frame (§1.8.3): the plan, the steps, the cell image — all submitted here, so they run before the pass egui
        // draws the image in.
        #[cfg(not(target_arch = "wasm32"))]
        let planned = std::time::Instant::now();
        self.sim.frame(now, frozen);
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.sim_cost += planned.elapsed();
        }

        self.frames += 1;
        self.recent.push_back(now);
        while self.recent.front().is_some_and(|t| now - t > 1.0) {
            self.recent.pop_front();
        }
        let (fps, world) = (self.recent.len() as f64, self.sim.world());
        let status_line = format::status_line(&self.adapter, fps, self.sim.steps_per_frame(), world.width, world.height);

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(status) = &self.status {
            let (frames, presented, step, sim_time) = (self.frames, self.presented, self.sim.submitted(), self.sim.sim_time());
            status.update(|s| {
                s.frame = frames;
                s.fps = fps;
                s.step = step;
                s.sim_time = sim_time;
                if presented {
                    s.state = crate::status::State::Ready;
                }
            });
        }

        // The status bar (§4.2, 24 px), then the centre: the world at the integer scale s (§1.11), centred on bg.space.
        egui::Panel::bottom("status")
            .exact_size(STATUS_BAR_HEIGHT)
            .frame(egui::Frame::NONE.fill(BG_SPACE).inner_margin(egui::Margin::symmetric(8, 4)))
            .show(ui, |ui| {
                ui.label(egui::RichText::new(&status_line).size(STATUS_FONT).color(TEXT_SECONDARY));
            });
        // The left panel (§4.2, 220 px) between the two bars; its clicks reach the tools before the centre reads the pointer.
        let panel = left::view(self.sim.tools(), self.sim.presets(), self.sim.views(), self.sim.elements());
        let mut tool_commands = Vec::new();
        let mut preset_commands = Vec::new();
        let mut view_commands = Vec::new();
        let mut display_commands = Vec::new();
        let active = self.sim.views().active();
        let display_panel = display::panel(active, self.sim.display());
        egui::Panel::left("left")
            .exact_size(LEFT_PANEL_WIDTH)
            .frame(
                egui::Frame::NONE
                    .fill(BG_PANEL)
                    .stroke(egui::Stroke::new(1.0, BORDER_PANEL))
                    .inner_margin(egui::Margin::same(8)),
            )
            .show(ui, |ui| {
                let (tools, presets, views, displays) = draw_left_panel(ui, &panel, active, display_panel.as_ref());
                tool_commands.extend(tools);
                preset_commands.extend(presets);
                view_commands.extend(views);
                display_commands.extend(displays);
            });
        for command in tool_commands {
            self.sim.tool_command(command);
        }
        for command in preset_commands {
            self.sim.preset_command(command);
        }
        for command in view_commands {
            self.sim.view_command(command);
        }
        for command in display_commands {
            self.sim.display_command(command);
        }

        // The right panel (§4.2, 300 px): the star readout (§4.6), then the centre.
        let star = self.sim.star_panel(now);
        let cell = self.sim.cell_panel();
        egui::Panel::right("right")
            .exact_size(RIGHT_PANEL_WIDTH)
            .frame(
                egui::Frame::NONE
                    .fill(BG_PANEL)
                    .stroke(egui::Stroke::new(1.0, BORDER_PANEL))
                    .inner_margin(egui::Margin::same(8)),
            )
            .show(ui, |ui| {
                draw_star_panel(ui, &star);
                ui.add_space(8.0);
                draw_cell_panel(ui, &cell);
            });

        // The tools act while the left button is held on the world, once per frame (§1.12) — never under the Clear dialog, and
        // never while a preset is armed: the next left click is the preset's alone (§4.5), a click reaches the brush or the
        // preset, never both.
        let modal = bar.dialog.is_some();
        let armed = self.sim.presets().armed();
        let mut placed = None;
        let mut selected = None;
        let mut hovered = None;
        let held = egui::CentralPanel::no_frame()
            .frame(egui::Frame::NONE.fill(BG_SPACE))
            .show(ui, |ui| {
                let area = ui.available_rect_before_wrap();
                let ppp = ui.ctx().pixels_per_point();
                let placed_at = legend::layout(area, (world.width, world.height), ppp);
                // The world's place at z = 1 is the viewport the zoomed image is clipped to (§4.2, ui::zoom): the wheel and the
                // middle drag move the view before anything reads the pointer, so every reader sees the same one.
                let rect = placed_at.world;
                #[cfg(target_arch = "wasm32")]
                crate::web::report_world(rect);
                let response = ui.interact(rect, egui::Id::new("world"), egui::Sense::click_and_drag());
                if !modal {
                    if let Some(pos) = response.hover_pos() {
                        let notches: Vec<(f32, bool)> = ui.input(|i| {
                            i.events
                                .iter()
                                .filter_map(|e| match e {
                                    egui::Event::MouseWheel { unit, delta, modifiers, .. } if !modifiers.command => {
                                        Some((delta.y, *unit != egui::MouseWheelUnit::Point))
                                    }
                                    _ => None,
                                })
                                .collect()
                        });
                        for (delta, lines) in notches {
                            let steps = self.wheel.steps(delta, lines);
                            self.zoom.step(steps, pos, rect, world);
                        }
                    }
                    if response.dragged_by(egui::PointerButton::Middle) {
                        self.zoom.pan(response.drag_delta(), rect, world);
                    }
                }
                let image = self.zoom.image(rect, world, ppp);
                let cell_at = |p: egui::Pos2| self.zoom.cell_at(p, rect, world, ppp);
                let hover = response.hover_pos().filter(|_| !modal).and_then(cell_at);
                hovered = hover;
                let painter = ui.painter();
                painter.image(self.sim.texture(), rect, self.zoom.uv(rect, image), egui::Color32::WHITE);
                // The remnant's ring from core collapse on (§0.5 [M0-D138], ui::remnant), in every view and at every zoom: `accent`
                // over a `bg.space` halo, so it reads on Glow's white ejecta and on the dark sky alike; the banner stays above it.
                for ring in remnant::rings(&self.sim.remnant_centres(), &self.zoom, rect, world, ppp) {
                    let clipped = painter.with_clip_rect(rect);
                    clipped.circle_stroke(ring.centre, ring.radius, egui::Stroke::new(4.0, BG_SPACE));
                    clipped.circle_stroke(ring.centre, ring.radius, egui::Stroke::new(2.0, ACCENT));
                }
                if let Some(banner) = &banner {
                    draw_banner(painter, rect, banner);
                }
                if let Some(cell) = hover {
                    // The armed preset's cloud at the radius the engine lays it out with, or the brush's disk — the zoomed image's
                    // scale, clipped to the viewport.
                    let (centre, radius) = match armed {
                        Some(preset) => left::cloud_outline(image, world, cell, left::cloud_radius(self.sim.preset_mass(preset))),
                        None => left::outline(self.sim.tools(), image, world, cell),
                    };
                    painter.with_clip_rect(rect).circle_stroke(centre, radius, egui::Stroke::new(1.0, TEXT_PRIMARY));
                }
                // The legend of the active view under the world (§4.2, M0-T128), then the lines under it: the hint while armed, the
                // refusal for 3 s in its place (§4.5, M0-D136: the slot holds one line).
                let view = self.sim.views().active();
                let bar = self.legend_bars.of(view, self.sim.display());
                let text_width = |text: &str| {
                    ui.ctx().fonts_mut(|f| f.layout_no_wrap(text.to_string(), egui::FontId::proportional(LEGEND_FONT), TEXT_PRIMARY).size().x)
                };
                let strip = legend::legend(
                    view,
                    self.sim.display(),
                    self.sim.elements(),
                    self.sim.cursor_cell().as_ref(),
                    bar,
                    placed_at.legend.width(),
                    &text_width,
                );
                draw_legend(painter, placed_at.legend, &strip, &text_width);
                let notices = self.sim.presets().notices(now);
                for (top, i) in legend::notice_slots(placed_at.hint, &notices) {
                    let at = egui::pos2(rect.center().x, top);
                    let colour = if i == 0 && armed.is_some() { TEXT_SECONDARY } else { ERROR_TEXT };
                    painter.text(at, egui::Align2::CENTER_TOP, &notices[i], egui::FontId::proportional(BAR_FONT), colour);
                }
                // A right click selects the star under the cursor (§4.10), whatever is armed.
                if !modal && response.clicked_by(egui::PointerButton::Secondary) {
                    selected = response.interact_pointer_pos().and_then(cell_at);
                }
                if armed.is_some() {
                    if !modal && response.clicked_by(egui::PointerButton::Primary) {
                        placed = response.interact_pointer_pos().and_then(cell_at);
                    }
                    return None;
                }
                let down = !modal && response.is_pointer_button_down_on() && ui.input(|i| i.pointer.primary_down());
                down.then(|| response.interact_pointer_pos().and_then(cell_at)).flatten()
            })
            .inner;
        // The cell under the pointer is the inspector's (§4.6); a capture's scripted hover stands in for a pointer it has none of.
        #[cfg(not(target_arch = "wasm32"))]
        let hovered = self.scripted_hover.unwrap_or(hovered);
        self.sim.set_hover(hovered);
        if let Some(cell) = selected {
            self.sim.select_star(cell);
        }
        if let Some(cell) = placed {
            // A refused drop is shown by the presets themselves; any other refusal (the step's queue full) asks for another click.
            let _ = self.sim.place(cell, now);
        }
        // A refused edit (the step's queue holds MAX_EDITS already) is asked again next frame, as the button is still held.
        let _ = self.sim.edit(held);
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(driver) = &mut self.capture {
            driver.end(ui.ctx(), &status_line);
        }
        ui.ctx().request_repaint();
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        BG_SPACE.to_normalized_gamma_f32()
    }
}

/// The status bar's height in logical px (§4.2).
const STATUS_BAR_HEIGHT: f32 = 24.0;
/// The top bar's height in logical px (§4.2).
pub const TOP_BAR_HEIGHT: f32 = 44.0;

/// The commands of this pass's key presses (§4.10): the bar's (ui::topbar) and the panel's (ui::left).
#[derive(Default)]
struct Keys {
    bar_commands: Vec<Command>,
    tool_commands: Vec<left::Command>,
    preset_commands: Vec<left::PresetCommand>,
    view_commands: Vec<left::ViewCommand>,
    /// Home (§4.10): back to the whole world.
    zoom_home: bool,
}

/// The commands of this pass's key presses (§4.10). The presses it takes leave the pass's input, so a button that holds the
/// keyboard focus (egui focuses what the mouse clicked) is not pressed a second time by the Space that paused the clock.
/// A key with Ctrl, Alt or Command held is not the app's, and neither is any key while a text field is being typed in.
///
/// Esc is taken only while a preset is `armed`: otherwise it stays in the input for whoever reads it.
fn key_commands(ctx: &egui::Context, armed: bool) -> Keys {
    if ctx.egui_wants_keyboard_input() {
        return Keys::default();
    }
    let mut keys = Keys::default();
    ctx.input_mut(|input| {
        input.events.retain(|event| match event {
            egui::Event::Key { key, pressed: true, repeat: false, modifiers, .. } if !modifiers.command && !modifiers.alt => {
                if let Some(command) = topbar::key_command(*key) {
                    keys.bar_commands.push(command);
                    false
                } else if let Some(command) = left::key_command(*key) {
                    keys.tool_commands.push(command);
                    false
                } else if let Some(command) = left::view_key_command(*key) {
                    keys.view_commands.push(command);
                    false
                } else if let Some(command) = left::preset_key_command(*key).filter(|_| armed) {
                    keys.preset_commands.push(command);
                    false
                } else if *key == egui::Key::Home {
                    keys.zoom_home = true;
                    false
                } else {
                    true
                }
            }
            _ => true,
        });
    });
    keys
}

/// The bar's text size (§4.11: body 14 px).
const BAR_FONT: f32 = 14.0;

/// The width `label` takes at the bar's text size.
fn text_width(ui: &egui::Ui, label: &str) -> f32 {
    ui.ctx().fonts_mut(|f| f.layout_no_wrap(label.to_string(), egui::FontId::proportional(BAR_FONT), TEXT_PRIMARY).size().x)
}

/// The top bar as the view model gives it (§4.2): drawn as given, and the commands of what was clicked handed back. Nothing in
/// it moves when the clock changes (the lead at M0-V12: «I dont like when an interaction in a bar moves the whole bar»): the
/// Pause/Play button is as wide as its wider label, and the speed readout sits in a slot as wide as its longest usual text — a
/// capped ×1000 or Max's six-digit speed. While paused the readout is `accent`, so a still world says so (§0.5 [M0-V12]).
pub(crate) fn draw_top_bar(ui: &mut egui::Ui, bar: &TopBar) -> Vec<Command> {
    let mut commands = Vec::new();
    let text = |label: &str, selected: bool| {
        egui::RichText::new(label).size(BAR_FONT).color(if selected { ACCENT } else { TEXT_PRIMARY })
    };
    let get = crate::strings::get;
    let play_width =
        text_width(ui, get("time.pause")).max(text_width(ui, get("time.play"))) + 2.0 * ui.spacing().button_padding.x;
    let readout_width = [
        get("time.speed_capped").replace("{rung}", "1000").replace("{actual}", "0.0083"),
        get("time.speed_max").replace("{actual}", "120000"),
    ]
    .iter()
    .map(|t| text_width(ui, t))
    .fold(0.0, f32::max);
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        if ui.add(egui::Button::new(text(&bar.play.label, false)).min_size(egui::vec2(play_width, 0.0))).clicked() {
            commands.push(bar.play.command);
        }
        if ui.button(text(&bar.step.label, false)).clicked() {
            commands.push(bar.step.command);
        }
        ui.separator();
        for rung in &bar.rungs {
            if ui.add(egui::Button::new(text(&rung.button.label, rung.selected)).selected(rung.selected)).clicked() {
                commands.push(rung.button.command);
            }
        }
        ui.separator();
        let slot = egui::vec2(readout_width, ui.spacing().interact_size.y);
        ui.allocate_ui_with_layout(slot, egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.set_min_width(readout_width);
            ui.label(text(&bar.speed, bar.paused));
        });
        ui.separator();
        let mut checked = bar.slow_down.checked;
        if ui.checkbox(&mut checked, text(&bar.slow_down.label, false)).changed() {
            commands.push(Command::SetSlowDown(checked));
        }
        ui.separator();
        ui.label(text(&bar.edge.label, false));
        for choice in [&bar.edge.leave, &bar.edge.bounce] {
            if ui.radio(choice.selected, text(&choice.button.label, false)).clicked() {
                commands.push(choice.button.command);
            }
        }
        ui.separator();
        if ui.button(text(&bar.clear.label, false)).clicked() {
            commands.push(bar.clear.command);
        }
    });
    commands
}

/// The left panel's width in logical px (§4.2).
const LEFT_PANEL_WIDTH: f32 = 220.0;

/// The right panel's width in logical px (§4.2).
const RIGHT_PANEL_WIDTH: f32 = 300.0;

/// The cell inspector as the view model gives it (§4.6): the heading, one line each.
fn draw_cell_panel(ui: &mut egui::Ui, panel: &CellPanel) {
    ui.spacing_mut().item_spacing = egui::vec2(8.0, 6.0);
    ui.label(egui::RichText::new(&panel.heading).size(HEADING_FONT).color(TEXT_PRIMARY));
    for line in &panel.lines {
        ui.label(egui::RichText::new(line).size(BAR_FONT).color(TEXT_PRIMARY));
    }
}

/// The star readout as the view model gives it (§4.6): the heading, one line each, the note under them.
fn draw_star_panel(ui: &mut egui::Ui, panel: &StarPanel) {
    ui.spacing_mut().item_spacing = egui::vec2(8.0, 6.0);
    ui.label(egui::RichText::new(&panel.heading).size(HEADING_FONT).color(TEXT_PRIMARY));
    for line in &panel.lines {
        ui.label(egui::RichText::new(line).size(BAR_FONT).color(TEXT_PRIMARY));
    }
    if let Some(note) = &panel.note {
        ui.label(egui::RichText::new(note).size(BAR_FONT).color(TEXT_SECONDARY));
    }
}

/// The left panel as the view model gives it (§4.2): the selected tool and element in `accent`, each element's swatch
/// beside its name, the armed preset in `accent`; the commands of what was clicked or dragged handed back.
fn draw_left_panel(
    ui: &mut egui::Ui,
    panel: &LeftPanel,
    active: View,
    display_panel: Option<&DisplayPanel>,
) -> (Vec<left::Command>, Vec<left::PresetCommand>, Vec<left::ViewCommand>, Vec<DisplayCommand>) {
    let mut commands = Vec::new();
    let mut display_commands = Vec::new();
    let mut preset_commands = Vec::new();
    let mut view_commands = Vec::new();
    let text = |label: &str, selected: bool| {
        egui::RichText::new(label).size(BAR_FONT).color(if selected { ACCENT } else { TEXT_PRIMARY })
    };
    let heading = |label: &str| egui::RichText::new(label).size(HEADING_FONT).color(TEXT_PRIMARY);
    ui.spacing_mut().item_spacing = egui::vec2(8.0, 6.0);

    ui.label(heading(&panel.tools.heading));
    let width = (ui.available_width() - ui.spacing().item_spacing.x) / 2.0;
    for pair in panel.tools.buttons.chunks(2) {
        ui.horizontal(|ui| {
            for button in pair {
                let widget = egui::Button::new(text(&button.label, button.selected)).selected(button.selected);
                if ui.add_sized([width, 24.0], widget).clicked() {
                    commands.push(button.command);
                }
            }
        });
    }
    let size = &panel.tools.size;
    ui.label(text(&size.label, false));
    let mut radius = size.value;
    // The slider's rail is `widgets.inactive.bg_fill`, `bg.panel` — the panel's own colour — so it is drawn in `border.panel`.
    ui.visuals_mut().widgets.inactive.bg_fill = BORDER_PANEL;
    ui.spacing_mut().slider_width = ui.available_width() - 56.0;
    if ui.add(egui::Slider::new(&mut radius, size.min..=size.max)).changed() {
        commands.push(left::Command::SetRadius(radius));
    }

    ui.add_space(8.0);
    ui.label(heading(&panel.elements.heading));
    for row in &panel.elements.rows {
        ui.horizontal(|ui| {
            let (slot, _) = ui.allocate_exact_size(egui::vec2(16.0, 22.0), egui::Sense::hover());
            ui.painter().rect_filled(egui::Rect::from_center_size(slot.center(), egui::vec2(16.0, 16.0)), 2.0, theme::swatch(row.swatch));
            let widget = egui::Button::new(text(&row.label, row.selected)).selected(row.selected);
            if ui.add_sized([ui.available_width(), 22.0], widget).clicked() {
                commands.push(row.command);
            }
        });
    }

    ui.add_space(8.0);
    ui.label(heading(&panel.presets.heading));
    for button in &panel.presets.buttons {
        let widget = egui::Button::new(text(&button.label, button.armed)).selected(button.armed);
        if ui.add_sized([ui.available_width(), 24.0], widget).clicked() {
            preset_commands.push(button.command);
        }
        ui.label(egui::RichText::new(&button.note).size(STATUS_FONT).color(TEXT_SECONDARY));
    }

    ui.add_space(8.0);
    ui.label(heading(&panel.views.heading));
    for pair in panel.views.buttons.chunks(2) {
        ui.horizontal(|ui| {
            for button in pair {
                let widget = egui::Button::new(text(&button.label, button.selected)).selected(button.selected);
                if ui.add_sized([width, 24.0], widget).clicked() {
                    view_commands.push(button.command);
                }
            }
        });
    }
    // The active view's display controls (§4.2): each a slider over its span and its value typed in place, then Fit and Reset.
    if let Some(display_panel) = display_panel {
        ui.add_space(4.0);
        for control in &display_panel.controls {
            ui.label(text(&control.label, false));
            ui.horizontal(|ui| {
                let mut x = control.value;
                ui.spacing_mut().slider_width = (ui.available_width() - 96.0).max(40.0);
                let slid = ui.add(egui::Slider::new(&mut x, control.min..=control.max).show_value(false)).changed();
                let value = egui::DragValue::new(&mut x)
                    .speed(0.0)
                    .range(control.min..=control.max)
                    .custom_formatter(move |x, _| display::show(active, x))
                    .custom_parser(move |typed| display::parse(active, typed));
                let typed = ui.add_sized([ui.available_width(), 20.0], value).changed();
                if slid || typed {
                    display_commands.push(display::command(control.control, x));
                }
            });
        }
        ui.horizontal(|ui| {
            if let Some(fit) = &display_panel.fit {
                if ui.add_sized([width, 24.0], egui::Button::new(text(fit, false))).clicked() {
                    display_commands.push(DisplayCommand::Fit);
                }
            }
            if ui.add_sized([width, 24.0], egui::Button::new(text(&display_panel.reset, false))).clicked() {
                display_commands.push(DisplayCommand::Reset);
            }
        });
    }
    (commands, preset_commands, view_commands, display_commands)
}

/// The Clear dialog (§4.7), modal over the window: the question, then `world.clear_yes` and `world.clear_no`.
fn draw_dialog(ctx: &egui::Context, dialog: &Dialog) -> Vec<Command> {
    let mut commands = Vec::new();
    egui::Modal::new(egui::Id::new("clear-world")).backdrop_color(theme::backdrop()).show(ctx, |ui| {
        ui.label(egui::RichText::new(&dialog.text).size(14.0).color(TEXT_PRIMARY));
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            for button in [&dialog.yes, &dialog.no] {
                if ui.button(egui::RichText::new(&button.label).size(14.0).color(TEXT_PRIMARY)).clicked() {
                    commands.push(button.command);
                }
            }
        });
    });
    commands
}

/// The banner over the world's top centre (§4.2): `banner.text` on `banner.bg`, 20 px.
fn draw_banner(painter: &egui::Painter, world: egui::Rect, banner: &Banner) {
    let galley = painter.layout_no_wrap(banner.text.clone(), egui::FontId::proportional(BANNER_FONT), BANNER_TEXT);
    let pad = egui::vec2(16.0, 8.0);
    let size = galley.size() + pad * 2.0;
    let min = egui::pos2(world.center().x - size.x / 2.0, world.top() + 8.0);
    painter.rect_filled(egui::Rect::from_min_size(min, size), 6.0, BANNER_BG);
    painter.galley(min + pad, galley, BANNER_TEXT);
}

/// The legend's text size (§4.11: small, 12 px).
const LEGEND_FONT: f32 = 12.0;

/// The legend strip in `strip` (§4.2, §4.11): the title and the marks' labels on the top row, the 12 px bar under them, the ticks and
/// their labels under the bar; a mark is a 10 px triangle over a 2 px line through the bar. The Glow setting is `accent`, the cursor
/// `text.primary`.
fn draw_legend(painter: &egui::Painter, strip: egui::Rect, legend: &Legend, text_width: &dyn Fn(&str) -> f32) {
    let font = egui::FontId::proportional(LEGEND_FONT);
    painter.text(strip.left_top(), egui::Align2::LEFT_TOP, &legend.title, font.clone(), TEXT_SECONDARY);
    let bar_top = strip.top() + legend::MARK_TRIANGLE + 6.0;
    let bar = egui::Rect::from_min_size(egui::pos2(strip.left(), bar_top), egui::vec2(strip.width(), legend::BAR_HEIGHT));
    let slices = match &legend.bar {
        Bar::Gradient(colours) | Bar::Swatches(colours) => colours,
    };
    let x_at = |f: f32| (bar.left() + bar.width() * f).round();
    for (i, colour) in slices.iter().enumerate() {
        let (a, b) = (x_at(i as f32 / slices.len() as f32), x_at((i + 1) as f32 / slices.len() as f32));
        let slice = egui::Rect::from_min_max(egui::pos2(a, bar.top()), egui::pos2(b, bar.bottom()));
        painter.rect_filled(slice, 0.0, theme::swatch(*colour));
    }
    for tick in &legend.ticks {
        let x = bar.left() + bar.width() * tick.position as f32;
        painter.line_segment(
            [egui::pos2(x, bar.bottom()), egui::pos2(x, bar.bottom() + legend::TICK_LENGTH)],
            egui::Stroke::new(1.0, TEXT_SECONDARY),
        );
        if let Some(label) = &tick.label {
            let left = bar.left() + legend::label_left(tick.position, text_width(label), bar.width());
            painter.text(egui::pos2(left, bar.bottom() + legend::TICK_LENGTH), egui::Align2::LEFT_TOP, label, font.clone(), TEXT_SECONDARY);
        }
    }
    // The marks' labels sit on the top row to the right of their triangle, to its left at the strip's right edge or where the
    // cursor's would cover the Glow setting's, and right of the title where either would meet it (`legend::mark_label_lefts`).
    let marks: Vec<(&legend::Mark, egui::Color32)> =
        legend.white.iter().map(|m| (m, ACCENT)).chain(legend.cursor.iter().map(|m| (m, TEXT_PRIMARY))).collect();
    let xs: Vec<(f32, f32)> = marks.iter().map(|(m, _)| (bar.width() * m.position as f32, text_width(&m.label))).collect();
    let lefts = legend::mark_label_lefts(text_width(&legend.title), &xs, strip.width());
    for ((mark, colour), left) in marks.into_iter().zip(lefts) {
        let x = bar.left() + bar.width() * mark.position as f32;
        let tip = bar.top();
        let top = tip - legend::MARK_TRIANGLE;
        painter.add(egui::Shape::convex_polygon(
            vec![egui::pos2(x - 5.0, top), egui::pos2(x + 5.0, top), egui::pos2(x, tip)],
            colour,
            egui::Stroke::NONE,
        ));
        painter.rect_filled(
            egui::Rect::from_min_max(egui::pos2(x - legend::MARK_LINE / 2.0, tip), egui::pos2(x + legend::MARK_LINE / 2.0, bar.bottom())),
            0.0,
            colour,
        );
        painter.text(egui::pos2(strip.left() + left, strip.top()), egui::Align2::LEFT_TOP, &mark.label, font.clone(), colour);
    }
}

/// The world and its frame loop on the app's device (§1.8.3, §3.1): `TimeControl` plans each frame, the engine's
/// `encode_frame` runs the plan, `render` draws the active view into the texture the centre shows, `poll()` — never
/// blocking — brings back the frames read back, which feed `on_report` in order.
mod sim {
    use std::collections::VecDeque;

    use eframe::{egui, egui_wgpu};
    use sr_engine::render::{Display, Renderer, IMAGE_FORMAT};
    use sr_engine::scene::{Scene, Switches, TestSettings};
    use sr_engine::state::{Booking, Inspector, SideFields, State, WorldConfig};
    use sr_engine::step::boxfit::BoxMode;
    use sr_engine::step::edit::{Edit, EditError};
    use sr_engine::step::heat::HeatSetup;
    use sr_engine::step::latch::FramePlan;
    use sr_engine::step::nuheat::NuHeatSetup;
    use sr_engine::step::react::ReactSetup;
    use sr_engine::step::sinks::SinkSetup;
    use sr_engine::step::{EosGpu, Passes, Step, Timing};
    use sr_physics::calibration::{self, PresetMasses};
    use sr_physics::registry::{Elements, Physics, Reactions};
    use sr_physics::time::{Estimates, TimeControl};

    use crate::ui::banner::{Banner, Frames, Pass};
    use crate::ui::display::{self, DisplayCommand};
    use crate::ui::left::{self, Presets, Tools, Views};
    use crate::ui::right::{inspect_view, Asker, Cell, CellPanel, Reader, StarPanel};
    use crate::ui::topbar::{self, Command, Effect, WorldBar};

    /// Most steps one scripted `steps` action puts into a single frame: a capture asking for 10⁶ would otherwise build one
    /// enormous command buffer.
    /// On the web, Chrome's GPU process died (the browser closed) when `?preset=sun&steps=n` put several steps into one
    /// submit on linux-pc's Quadro (M0-T94: 8 to a frame died at n = 24, 10 in one chunk was fine, 1 to a frame ran 200),
    /// so the page's `steps` goes one to a frame.
    #[cfg(not(target_arch = "wasm32"))]
    const FORCED_CHUNK: u64 = 256;
    #[cfg(target_arch = "wasm32")]
    const FORCED_CHUNK: u64 = 1;

    /// A Δt that is a CFL step: finite, above zero and below the empty world's `f32::MAX` (see `sr_physics::time`).
    fn real_dt(dt: f64) -> Option<f64> {
        (dt.is_finite() && dt > 0.0 && dt < f64::from(f32::MAX)).then_some(dt)
    }

    pub struct Sim {
        device: wgpu::Device,
        queue: wgpu::Queue,
        world: WorldConfig,
        /// What a preset's drop reads.
        physics: Physics,
        elements: Elements,
        masses: PresetMasses,
        /// The selected tool, element and brush size (ui::left).
        tools: Tools,
        /// The armed preset, and the refusal shown under the world (ui::left).
        presets: Presets,
        state: State,
        /// The state before the last frame's steps (the renderer blends with it by α, §1.8.3).
        previous: State,
        step: Step,
        renderer: Renderer,
        /// The cell image: the renderer writes it, egui draws it (nearest filtering).
        target: wgpu::TextureView,
        texture: egui::TextureId,
        time: TimeControl,
        /// The edge choice and the Clear dialog (ui::topbar).
        world_bar: WorldBar,
        /// The frames in flight and the observers they feed (ui::banner): their events slow the clock.
        frames: Frames,
        /// The view the renderer draws (ui::left).
        views: Views,
        /// The Glow setting and the Heat and Density ranges (ui::display): kept by Clear and by a switch of view.
        display: Display,
        /// The cell inspector (§4.6): the engine's one-cell readback, the one request it may have in flight, what its reply
        /// reads as, and the cell under the pointer.
        inspector: Inspector,
        asker: Asker,
        reader: Reader,
        hover: Option<[u32; 2]>,
        reading: Option<Cell>,
        /// Steps a capture asked of the real loop and the loop has not yet submitted.
        forced: u64,
        /// The last Δt read back; none before the first report and while the world is empty.
        dt_est: Option<f64>,
        /// Sim time owed, as `TimeControl` counts it (§1.8.3), for α.
        owed: f64,
        alpha: f32,
        last_now: Option<f64>,
        /// (time, steps) of the frames of the last second.
        recent: VecDeque<(f64, u32)>,
        submitted: u64,
        sim_time: f64,
        /// Held so the image outlives its view.
        _image: wgpu::Texture,
    }

    /// What `Sim::new` builds on the device before egui sees it: the step, the renderer and their world.
    struct Parts {
        cal: calibration::Calibration,
        physics: Physics,
        elements: Elements,
        world: WorldConfig,
        world_bar: WorldBar,
        state: State,
        previous: State,
        step: Step,
        renderer: Renderer,
        image: wgpu::Texture,
        target: wgpu::TextureView,
    }

    impl Pass for Sim {
        fn poll(&mut self) {
            self.poll_frames();
        }
        fn time(&self) -> &TimeControl {
            &self.time
        }
        fn world_bar(&self) -> &WorldBar {
            &self.world_bar
        }
        fn notice(&self, now: f64) -> Option<Banner> {
            self.frames.notice(now)
        }
    }

    impl Sim {
        pub fn new(render: &egui_wgpu::RenderState) -> Result<Sim, String> {
            let (device, queue) = (render.device.clone(), render.queue.clone());
            let Parts { cal, physics, elements, world, world_bar, state, previous, step, renderer, image, target } =
                Self::parts(&device, &queue)?;
            // IMAGE_FORMAT holds sRGB-encoded bytes in a UNORM texture, "the display draws it as is" (render.rs): egui-wgpu
            // reads its textures as gamma-encoded bytes in Rgba8Unorm too, so the quad needs no view of its own.
            let texture = render.renderer.write().register_native_texture(&device, &target, wgpu::FilterMode::Nearest);
            let time = TimeControl::new(Some(cal.top_rung), cal.render_reserve_ms)?;
            let frames = Frames::new(&physics, &cal, world.width)?;
            let inspector = Inspector::new(world, &physics, &elements);
            let reader = Reader::new(&physics, &cal, &elements)?;
            let tools = Tools::new(&elements);
            Ok(Sim {
                device,
                queue,
                world,
                physics,
                elements,
                masses: cal.preset_mass_sb,
                tools,
                presets: Presets::default(),
                state,
                previous,
                step,
                renderer,
                target,
                texture,
                time,
                world_bar,
                frames,
                views: Views::default(),
                display: Display::default(),
                inspector,
                asker: Asker::default(),
                reader,
                hover: None,
                reading: None,
                forced: 0,
                dt_est: None,
                owed: 0.0,
                alpha: 1.0,
                last_now: None,
                recent: VecDeque::new(),
                submitted: 0,
                sim_time: 0.0,
                _image: image,
            })
        }

        /// `new`'s device work alone, built and dropped: web.rs's dry run, whose pipeline creations it turns into
        /// parallel compiles the real build then finds done (M0-D118).
        #[cfg(target_arch = "wasm32")]
        pub fn dry_run(device: &wgpu::Device, queue: &wgpu::Queue) -> Result<(), String> {
            Self::parts(device, queue).map(drop)
        }

        fn parts(device: &wgpu::Device, queue: &wgpu::Queue) -> Result<Parts, String> {
            let cal = calibration::resolve(calibration::EMBEDDED, true)
                .map_err(|e| e.line())?
                .ok_or("calibration missing: the game needs assets/calibration.json (sandbox-reactions calibrate writes it)")?;
            let physics = Physics::shipped().map_err(|e| e.to_string())?;
            let elements = Elements::shipped().map_err(|e| e.to_string())?;
            let reactions = Reactions::shipped(&elements, &physics).map_err(|e| e.to_string())?;
            let world = WorldConfig::default();
            let world_bar = WorldBar::default();
            let (edge, switches, test) = (world_bar.edge(), Switches::default(), TestSettings::default());

            // A new world is empty: every cell at the floor state (§2.8).
            let scene = Scene { world, edge, slowdown: true, switches, test, objects: Vec::new(), overrides: Vec::new() };
            let (planes, _) = scene.planes(world, &physics, &elements)?;
            let state = State::new(device, world).map_err(|e| e.to_string())?;
            let previous = State::new(device, world).map_err(|e| e.to_string())?;
            let booking = Booking::new(device, world).map_err(|e| e.to_string())?;
            let eos = EosGpu::new(device, &physics, &elements);
            let side = SideFields::new(device, world).map_err(|e| e.to_string())?;
            let passes = Passes { mode: BoxMode::Fit, gravity: Some(&side), hydro: Some(edge) };
            let step = Step::with_neutrino_heating(
                device,
                &state,
                &booking,
                &eos,
                &physics,
                &passes,
                Some(&HeatSetup { side: &side, test }),
                Some(&ReactSetup { reactions: &reactions, test, neutrinos: true, side: &side }),
                Some(&SinkSetup { side: &side }),
                Some(&NuHeatSetup { side: &side }),
            )
            .map_err(|e| format!("gravity over a {}x{} world: {e}", world.width, world.height))?
            .with_edits(device, &state, &booking, &eos, &physics);
            state.upload(queue, &planes).map_err(|e| e.to_string())?;
            previous.upload(queue, &planes).map_err(|e| e.to_string())?;
            // §1.8.3: c_step is the cost per step the frames read back; GPU timestamps where the device has them.
            step.set_timing(device, queue, Timing::Auto);

            let anchor = |k: &str| physics.get(k).ok_or_else(|| format!("physics.json holds no {k}"));
            let anchors = [
                anchor("t_floor")?,
                cal.t_eff_sun_sb,
                anchor("t_h")?,
                anchor("t_he")?,
                anchor("t_c")?,
                anchor("t_ne")?,
                anchor("t_o")?,
                anchor("t_si")?,
            ];
            let renderer = Renderer::new(device, &state, &previous, &eos, anchors, &elements);
            let image = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("world.image"),
                size: wgpu::Extent3d { width: world.width, height: world.height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: IMAGE_FORMAT,
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let target = image.create_view(&wgpu::TextureViewDescriptor::default());
            Ok(Parts { cal, physics, elements, world, world_bar, state, previous, step, renderer, image, target })
        }

        pub fn world(&self) -> WorldConfig {
            self.world
        }

        pub fn texture(&self) -> egui::TextureId {
            self.texture
        }

        pub fn paused(&self) -> bool {
            self.time.paused()
        }

        /// A command of the top bar, a key or a script (ui::topbar): the clock's, then the world's.
        pub fn command(&mut self, command: Command) {
            topbar::apply(&mut self.time, command);
            match self.world_bar.apply(command) {
                Some(Effect::Edge(mode)) => self.step.set_edge_mode(&self.queue, mode),
                Some(Effect::Clear) => self.clear(),
                None => {}
            }
        }

        pub fn tools(&self) -> &Tools {
            &self.tools
        }

        pub fn elements(&self) -> &Elements {
            &self.elements
        }

        pub fn views(&self) -> &Views {
            &self.views
        }

        /// The star readout at egui time `now` (ui::right): the selected object's lines, the rate hidden while paused.
        pub fn star_panel(&self, now: f64) -> StarPanel {
            self.frames.stars().panel(now, self.time.paused())
        }

        /// The centres of the remnants to ring over the world (ui::remnant).
        pub fn remnant_centres(&self) -> Vec<[f64; 2]> {
            self.frames.stars().remnant_centres()
        }

        /// The cell inspector's panel (§4.6): the last reply for the cell under the pointer, or `inspect.empty` — no cell, or
        /// no reply yet.
        pub fn cell_panel(&self) -> CellPanel {
            inspect_view(self.hover.and(self.reading.as_ref()))
        }

        /// The cell under the pointer this frame (none: the pointer is off the world, §4.6): the next `frame` asks the engine
        /// for it. Leaving the world drops the last reply, so coming back never shows a cell the pointer left.
        pub fn set_hover(&mut self, cell: Option<[u32; 2]>) {
            self.hover = cell;
            if cell.is_none() {
                self.reading = None;
            }
        }

        /// The replies of the inspector's readback (never blocks), then the next request: one in flight at a time, for the cell
        /// hovered now (ui::right::Asker).
        fn inspect_frame(&mut self) {
            for reply in self.inspector.poll(&self.device) {
                self.asker.answered();
                if self.hover.is_some() {
                    self.reading = Some(self.reader.read(&reply));
                }
            }
            if let Some([x, y]) = self.asker.next(self.hover) {
                if self.inspector.inspect(&self.device, &self.queue, &self.state, (x, y)).is_err() {
                    self.asker.answered();
                }
            }
        }

        /// A right click on world cell `cell` (§4.10): selects the star under it.
        pub fn select_star(&mut self, cell: [u32; 2]) {
            self.frames.stars_mut().select_at(cell);
        }

        /// A view button or F1–F4 (ui::left): the next frame draws it.
        pub fn view_command(&mut self, command: left::ViewCommand) {
            self.views.apply(command);
        }

        pub fn display(&self) -> &Display {
            &self.display
        }

        /// The hovered cell as the inspector last read it, for the legend's cursor mark: none off the world or before a reply.
        pub fn cursor_cell(&self) -> Option<Cell> {
            self.hover.and(self.reading.clone())
        }

        /// A control of the display section (ui::display): the renderer is handed the settings only when they changed.
        /// «Fit to view» asks the renderer once, here, on the click (a blocking read-back; the web build has no way to
        /// wait for it, so there the button leaves the range as it is).
        pub fn display_command(&mut self, command: DisplayCommand) {
            let view = self.views.active();
            let command = match command {
                DisplayCommand::Fit => match self.fit(view) {
                    Some(range) => DisplayCommand::SetRange(range),
                    None => return,
                },
                other => other,
            };
            if display::apply(&mut self.display, view, command) {
                self.renderer.set_display(self.display);
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        fn fit(&self, view: sr_engine::render::View) -> Option<[f64; 2]> {
            self.renderer.fit_range(&self.queue, view)
        }

        #[cfg(target_arch = "wasm32")]
        fn fit(&self, _view: sr_engine::render::View) -> Option<[f64; 2]> {
            None
        }

        /// A command of the left panel or a key (ui::left).
        pub fn tool_command(&mut self, command: left::Command) {
            self.tools.apply(command);
        }

        /// The species id of the paintable element `key` names (a script's `element`).
        #[cfg(not(target_arch = "wasm32"))]
        pub fn species_of(&self, key: &str) -> Option<usize> {
            self.tools.species_of(&self.elements, key)
        }

        /// One frame of the left button held on `cell` (none: not held), as the tool and the brush size stand (§1.12): the
        /// edit for the next step's P0. A full queue (MAX_EDITS in one step) refuses it, and the held button asks again.
        pub fn edit(&mut self, cell: Option<[u32; 2]>) -> Result<(), sr_engine::step::edit::EditError> {
            match left::frame_edit(&self.tools, cell) {
                Some(edit) => self.step.queue_edit(&self.queue, &edit),
                None => Ok(()),
            }
        }

        /// Clear world (§1.12): the engine's `Clear` edit for the next step's P0 — every cell to the floor state, the sinks
        /// removed — and the observers of the old objects start over with the new world's first frame, the frames still in
        /// flight booking nothing (M0-D122). The rung, the view, the edge mode and the slow-down are not touched.
        fn clear(&mut self) {
            // A full queue (MAX_EDITS in one step) refuses the edit and the player asks again.
            if self.step.queue_edit(&self.queue, &Edit::Clear).is_ok() {
                self.frames.clear();
            }
        }

        /// Steps submitted to the engine since the world began.
        #[cfg(not(target_arch = "wasm32"))]
        pub fn submitted(&self) -> u64 {
            self.submitted
        }

        #[cfg(not(target_arch = "wasm32"))]
        /// The sim time after the last frame read back.
        pub fn sim_time(&self) -> f64 {
            self.sim_time
        }

        /// `status.line`'s steps per frame: the mean steps of the frames that ran any in the last second — at ×1 a Sun-like
        /// star is owed less than one step a frame (§1.8.3), and a mean over every frame would read 0 under it. Paused: 0.
        pub fn steps_per_frame(&self) -> f64 {
            let ran = self.recent.iter().filter(|&&(_, n)| n > 0).count();
            if ran == 0 {
                return 0.0;
            }
            self.recent.iter().map(|&(_, n)| f64::from(n)).sum::<f64>() / ran as f64
        }

        /// `steps` of a script (§3.3) or of the web's URL (§3.5): `n` steps through the real loop, paused or not, however the clock stands.
        pub fn queue_steps(&mut self, n: u64) {
            self.forced += n;
        }

        /// A preset's mass in sandbox units (calibration's `preset_mass_sb`, §2.11).
        pub fn preset_mass(&self, preset: left::Preset) -> f64 {
            match preset {
                left::Preset::Sun => self.masses.sun,
                left::Preset::Massive => self.masses.massive,
                left::Preset::Giant => self.masses.giant,
            }
        }

        /// The left click on the world at `cell` while a preset is armed (§4.5): the engine's `DropPreset` (§3.1) centred
        /// there, for the next step's P0. Dropped, the click is spent; `NoRoom` is shown as `presets.no_room` at egui time
        /// `now` and the preset stays armed. Another refusal (the step's queue full) is the caller's, and the preset stays
        /// armed for another click. Nothing armed: nothing happens.
        ///
        /// An edit lands at a step's P0, and a paused clock takes none: a drop while paused asks the clock for one step
        /// (the Step button's own), so the star is on screen when it is dropped (M0-T84's flag).
        pub fn place(&mut self, cell: [u32; 2], now: f64) -> Result<(), EditError> {
            let Some(preset) = self.presets.armed() else { return Ok(()) };
            let dropped = Edit::drop_preset(preset.name(), self.preset_mass(preset), cell, self.world, &self.physics, &self.elements)
                .and_then(|edit| self.step.queue_edit(&self.queue, &edit));
            match dropped {
                Ok(()) => {
                    self.presets.placed();
                    if self.time.paused() {
                        self.command(Command::Step);
                    }
                    Ok(())
                }
                Err(EditError::NoRoom) => {
                    self.presets.refused(now);
                    Err(EditError::NoRoom)
                }
                Err(e) => Err(e),
            }
        }

        pub fn presets(&self) -> &Presets {
            &self.presets
        }

        /// A preset button or Esc (ui::left).
        pub fn preset_command(&mut self, command: left::PresetCommand) {
            self.presets.apply(command);
        }

        /// The frames read back since the last call, in order: each one's report to `on_report`, its latch cut and the
        /// events its block summary names to the slow-down (§1.8.5), through `Frames`. Never blocks.
        fn poll_frames(&mut self) {
            for frame in self.step.poll_frames(&self.device) {
                self.sim_time = frame.report.sim_time;
                self.dt_est = real_dt(f64::from(frame.report.dt_next));
                self.frames.read_back(&mut self.time, &frame, self.last_now.unwrap_or(0.0));
            }
        }

        /// One frame at egui time `now` (s), the pass's read-back (`Pass::poll`) done. `frozen`: a capture is waiting for its screenshot, so nothing advances and the
        /// image on screen stays the one drawn last.
        pub fn frame(&mut self, now: f64, frozen: bool) {
            if frozen {
                return;
            }
            self.inspect_frame();
            let wall_s = self.last_now.replace(now).map_or(1.0 / 60.0, |t| (now - t).max(0.0));
            let (steps, slow_down, forced) = if self.forced > 0 {
                let n = self.forced.min(FORCED_CHUNK);
                self.forced -= n;
                (n as u32, false, true)
            } else {
                let plan = self.time.plan_frame(Estimates { wall_s });
                (plan.steps, plan.slow_down, false)
            };
            self.recent.push_back((now, steps));
            while self.recent.front().is_some_and(|&(t, _)| now - t > 1.0) {
                self.recent.pop_front();
            }
            // A scripted frame is not paced by the clock: it shows the state it reached.
            if forced {
                self.alpha = 1.0;
            } else {
                self.interpolate(steps);
            }

            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
            if steps > 0 {
                // The previous state the renderer blends with is the state before this frame's steps (§1.8.3).
                for g in 0..3 {
                    let buffer = self.state.group(g);
                    encoder.copy_buffer_to_buffer(buffer, 0, self.previous.group(g), 0, buffer.size());
                }
                self.step.encode_frame(&FramePlan { steps, slow_down }, &mut encoder);
                self.frames.submitted(forced);
                self.submitted += u64::from(steps);
            }
            self.renderer.render(self.views.active(), self.alpha, &self.target, &mut encoder);
            self.queue.submit([encoder.finish()]);
        }

        /// α (§1.8.3, TW-E4): when less than one step per frame is owed, the cell image is the blend of the last two states —
        /// the owed time runs from −Δt_est (a step just taken, the previous state shown) up to 0 (the next step due, the
        /// current one); in every other frame α = 1.
        fn interpolate(&mut self, steps: u32) {
            let rung = &self.time.rungs()[self.time.rung()];
            let per_frame = rung.value / f64::from(self.time.f_target());
            let owed_now = per_frame.is_finite() && !self.time.paused();
            if owed_now {
                self.owed += per_frame;
            }
            let Some(dt) = self.dt_est else {
                self.owed = 0.0;
                self.alpha = 1.0;
                return;
            };
            self.owed = (self.owed - f64::from(steps) * dt).clamp(-dt, if owed_now { per_frame } else { 0.0 });
            self.alpha = if owed_now && per_frame < dt { (1.0 + self.owed / dt).clamp(0.0, 1.0) as f32 } else { 1.0 };
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::run;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex, PoisonError};

    use eframe::{egui, egui_wgpu};
    use sr_engine::gpu::{match_adapter, AdapterDesc};
    use sr_engine::headless::{EXIT_ARGS, EXIT_DONE, EXIT_GPU};

    use super::{SandboxApp, WINDOW_SIZE};
    use crate::strings;

    const USAGE: &str = "usage: sandbox-reactions [--adapter <name substring>] [--status-port <port>] \
[--offscreen-window] [--capture <script.json> --out <dir>] [--measure-ui <frames> [--calibration <file>]]";

    /// `--offscreen-window` (§3.3 [M0-TJ3]): far outside any monitor's area — monitors sit within a few screens of
    /// the origin, in either direction — never activated, out of the taskbar. Wayland ignores a position; the route
    /// is win-laptop's (§6.6), where Win32 honours it.
    const OFFSCREEN_POS: [f32; 2] = [-20_000.0, -20_000.0];

    struct Args {
        adapter: Option<String>,
        status_port: Option<u16>,
        offscreen: bool,
        capture: Option<PathBuf>,
        out: Option<PathBuf>,
        /// `--measure-ui`'s frame count (measure.rs) and the file its key goes into.
        measure: Option<usize>,
        calibration: Option<PathBuf>,
        help: bool,
    }

    fn parse(args: &[String]) -> Result<Args, String> {
        let mut out = Args { adapter: None, status_port: None, offscreen: false, capture: None, out: None, measure: None, calibration: None, help: false };
        let mut it = args.iter();
        while let Some(arg) = it.next() {
            let mut value = || it.next().cloned().ok_or_else(|| format!("{arg} needs a value"));
            match arg.as_str() {
                "--adapter" => out.adapter = Some(value()?),
                "--status-port" => {
                    let v = value()?;
                    out.status_port =
                        Some(v.parse().ok().filter(|p| *p != 0).ok_or_else(|| format!("--status-port {v:?}: not a port"))?);
                }
                "-h" | "--help" => out.help = true,
                "--offscreen-window" => out.offscreen = true,
                "--capture" => out.capture = Some(value()?.into()),
                "--out" => out.out = Some(value()?.into()),
                "--measure-ui" => {
                    let v = value()?;
                    out.measure = Some(v.parse().ok().filter(|n| *n > 0).ok_or_else(|| format!("--measure-ui {v:?}: not a frame count"))?);
                }
                "--calibration" => out.calibration = Some(value()?.into()),
                "--world" => return Err(format!("{arg} is not built yet (its block comes later)")),
                other => return Err(format!("unknown argument {other:?}")),
            }
        }
        if out.help {
            return Ok(out);
        }
        match (&out.capture, &out.out) {
            (Some(_), None) => return Err("--capture needs --out <dir>".into()),
            (None, Some(_)) => return Err("--out is for --capture".into()),
            _ => {}
        }
        if out.measure.is_some() && out.capture.is_some() {
            return Err("--measure-ui and --capture are two different runs".into());
        }
        if out.calibration.is_some() && out.measure.is_none() {
            return Err("--calibration is for --measure-ui".into());
        }
        Ok(out)
    }

    fn native_wgpu_setup(query: Option<String>) -> egui_wgpu::WgpuSetup {
        let mut setup = super::wgpu_setup();
        // `--adapter`: M0-T2's matching rule (case-insensitive substring, Vulkan over another backend, then wgpu's
        // order) over every adapter listed; the pick must also present to this window. No flag: egui-wgpu asks wgpu
        // for the high-performance adapter compatible with the window's surface (§6.1).
        if let Some(query) = query {
            setup.native_adapter_selector = Some(Arc::new(move |adapters, surface| {
                let seen: Vec<AdapterDesc> = adapters.iter().map(|a| (&a.get_info()).into()).collect();
                let i = match_adapter(&seen, &query).map_err(|e| e.to_string())?;
                let adapter = &adapters[i];
                if surface.is_some_and(|s| !adapter.is_surface_supported(s)) {
                    return Err(format!("adapter {} cannot present to this window", seen[i]));
                }
                Ok(adapter.clone())
            }));
        }
        egui_wgpu::WgpuSetup::CreateNew(setup)
    }

    /// The desktop app (no subcommand). Returns the process's exit code (§3.3).
    pub fn run(args: &[String]) -> i32 {
        let args = match parse(args) {
            Ok(a) => a,
            Err(e) => {
                println!("SR-ERROR {e}; {USAGE}");
                return EXIT_ARGS;
            }
        };
        if args.help {
            println!("{USAGE}");
            return 0;
        }
        // A capture's script is read and checked, and its folder made, before any window opens: a refusal costs no boot.
        let capture = match (&args.capture, &args.out) {
            (Some(script), Some(out)) => match crate::capture::prepare(script, out) {
                Ok(steps) => Some((steps, script.clone(), out.clone(), Arc::new(Mutex::new(crate::capture::Outcome::default())))),
                Err(e) => {
                    println!("SR-ERROR {e}");
                    return EXIT_ARGS;
                }
            },
            _ => None,
        };
        let outcome = capture.as_ref().map(|c| c.3.clone());
        let mut capture = capture.map(|(steps, script, out, outcome)| crate::capture::Driver::new(steps, &script, out, outcome));

        let measure = args.measure.map(|frames| (frames, args.calibration.clone(), Arc::new(Mutex::new(crate::measure::Outcome::default()))));
        let measured = measure.as_ref().map(|m| m.2.clone());
        let mut measure = measure;

        // §2.11 (G-CAL): the game starts new worlds that drop presets, so a stale or missing calibration is exit 6 before any
        // window opens, as the headless run's gate is.
        if let Err(e) = sr_physics::calibration::resolve(sr_physics::calibration::EMBEDDED, true) {
            println!("{}", e.line());
            return e.exit_code();
        }

        // Bound before the window, so a poller reads "booting" while the window and the device come up (§3.4).
        // Its port taken, the server moves to the first free one of status::FALLBACK and says where (§0.5 [M0-D99]).
        let status = match args.status_port.map(crate::status::serve).transpose() {
            Ok(Some((s, port))) => {
                if Some(port) != args.status_port {
                    println!("SR-STATUS-PORT {port}");
                }
                Some(s)
            }
            Ok(None) => None,
            Err(e) => {
                println!(
                    "SR-ERROR --status-port {}: cannot listen on 127.0.0.1, nor on {}–{} ({e})",
                    args.status_port.unwrap_or(0),
                    crate::status::FALLBACK.start(),
                    crate::status::FALLBACK.end()
                );
                return EXIT_ARGS;
            }
        };

        let mut viewport = egui::ViewportBuilder::default().with_title(strings::get("app.title")).with_inner_size(WINDOW_SIZE);
        if args.offscreen {
            viewport = viewport.with_position(OFFSCREEN_POS).with_active(false).with_taskbar(false);
        }
        let options = eframe::NativeOptions {
            viewport,
            wgpu_options: egui_wgpu::WgpuConfiguration {
                wgpu_setup: native_wgpu_setup(args.adapter),
                ..Default::default()
            },
            ..Default::default()
        };

        let shared = status.clone();
        let result = eframe::run_native(
            strings::get("app.title"),
            options,
            Box::new(move |cc| {
                let render = cc.wgpu_render_state.as_ref().ok_or("eframe gave no wgpu render state")?;
                crate::ui::theme::install(&cc.egui_ctx);
                let mut app = SandboxApp::new(render)?;
                let info: AdapterDesc = (&render.adapter.get_info()).into();
                println!("{}", info.sr_adapter_line());
                if let Some(status) = &status {
                    status.update(|s| s.adapter = info.name.clone());
                }
                app.status = status;
                app.capture = capture.take();
                if let Some((frames, file, outcome)) = measure.take() {
                    let software = render.adapter.get_info().device_type == wgpu::DeviceType::Cpu;
                    app.measure = Some(crate::measure::Meter::new(frames, file, &info.name, software, outcome)?);
                }
                Ok(Box::new(app))
            }),
        );
        match result {
            Ok(()) if measured.is_some() => {
                let outcome = measured.as_ref().map(|o| o.lock().unwrap_or_else(PoisonError::into_inner));
                match outcome.as_deref() {
                    Some(crate::measure::Outcome { error: Some(e), .. }) => {
                        println!("SR-ERROR {e}");
                        EXIT_GPU
                    }
                    Some(crate::measure::Outcome { done: true, .. }) => EXIT_DONE,
                    _ => {
                        println!("SR-ERROR measure-ui: the window closed before the frames were measured");
                        EXIT_GPU
                    }
                }
            }
            Ok(()) => match outcome {
                None => EXIT_DONE,
                Some(outcome) => {
                    let outcome = outcome.lock().unwrap_or_else(PoisonError::into_inner);
                    match (&outcome.error, outcome.done) {
                        (Some(e), _) => {
                            println!("SR-ERROR {e}");
                            EXIT_GPU
                        }
                        (None, false) => {
                            println!("SR-ERROR capture: the window closed before the script finished");
                            EXIT_GPU
                        }
                        (None, true) => EXIT_DONE,
                    }
                }
            },
            Err(e) => {
                if let Some(status) = &shared {
                    status.update(|s| s.state = crate::status::State::Error);
                }
                println!("SR-ERROR {e}");
                EXIT_GPU
            }
        }
    }
}
