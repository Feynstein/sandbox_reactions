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

use crate::format;
use crate::ui::banner::{self, Banner};
use crate::ui::topbar::{self, Command, Dialog, TopBar};

/// `bg.space` (§4.11): the world's background and vacuum, #05070D.
pub const BG_SPACE: egui::Color32 = egui::Color32::from_rgb(0x05, 0x07, 0x0D);
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
    #[cfg(not(target_arch = "wasm32"))]
    status: Option<std::sync::Arc<crate::status::Status>>,
    /// `--capture`'s driver (capture.rs): the app waits on it before it advances.
    #[cfg(not(target_arch = "wasm32"))]
    capture: Option<crate::capture::Driver>,
}

impl SandboxApp {
    /// A new world — empty, at ×1, running (§1.8.2) — on the render state's device; an error is the `SR-ERROR` line that
    /// says why the engine could not start (a stale calibration is exit 6 on the desktop, §2.11).
    pub fn new(render: &egui_wgpu::RenderState) -> Result<Self, String> {
        Ok(Self {
            frames: 0,
            recent: VecDeque::new(),
            presented: false,
            adapter: render.adapter.get_info().name,
            sim: sim::Sim::new(render)?,
            #[cfg(not(target_arch = "wasm32"))]
            status: None,
            #[cfg(not(target_arch = "wasm32"))]
            capture: None,
        })
    }

    /// The device work of `new`, built and dropped (web.rs's warm-up, M0-D118).
    #[cfg(target_arch = "wasm32")]
    pub fn dry_run(device: &wgpu::Device, queue: &wgpu::Queue) -> Result<(), String> {
        sim::Sim::dry_run(device, queue)
    }

    /// A scripted action (§3.3) — the ones built so far. An error is the line the capture fails with.
    #[cfg(not(target_arch = "wasm32"))]
    fn apply(&mut self, action: &crate::capture::Action) -> Result<(), String> {
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
            Action::Preset(name) => self.sim.drop_preset(name)?,
        }
        Ok(())
    }
}

impl eframe::App for SandboxApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
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

        // A capture's step begins here: its actions apply before this pass advances, and the app stands still from the
        // pass that asked for a screenshot until the reply is written.
        #[cfg(not(target_arch = "wasm32"))]
        let frozen = match self.capture.take() {
            Some(mut driver) => {
                let begin = driver.begin(ui.ctx(), self.presented);
                if let Some(e) = begin.actions.iter().find_map(|a| self.apply(a).err()) {
                    driver.abort(ui.ctx(), e);
                }
                self.capture = Some(driver);
                begin.frozen
            }
            None => false,
        };
        #[cfg(target_arch = "wasm32")]
        let frozen = false;

        // The keys of the bar (§4.10) and its buttons; the commands reach the clock before this pass plans its frame.
        let mut commands = key_commands(ui.ctx());
        let bar = topbar::view(self.sim.time(), self.sim.world_bar());
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

        let now = ui.input(|i| i.time);
        // The frame (§1.8.3): the plan, the steps, the cell image — all submitted here, so they run before the pass egui
        // draws the image in.
        self.sim.frame(now, frozen);

        // The banner as the clock stands after this pass's frame: an event read back in it has slowed the clock already.
        let banner = banner::view(self.sim.time());

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
                ui.label(egui::RichText::new(&status_line).color(egui::Color32::from_gray(0xc8)));
            });
        egui::CentralPanel::no_frame().frame(egui::Frame::NONE.fill(BG_SPACE)).show(ui, |ui| {
            let painter = ui.painter();
            let area = ui.available_rect_before_wrap();
            let rect = world_rect(area, world, ui.ctx().pixels_per_point());
            painter.image(
                self.sim.texture(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
            if let Some(banner) = &banner {
                draw_banner(painter, rect, banner);
            }
        });
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
/// `bg.panel` and `border.panel` (§4.11): #10141E and #232A3A.
const BG_PANEL: egui::Color32 = egui::Color32::from_rgb(0x10, 0x14, 0x1E);
const BORDER_PANEL: egui::Color32 = egui::Color32::from_rgb(0x23, 0x2A, 0x3A);
/// `text.primary` and `accent` (§4.11): #E6EAF2 and #F2B33D.
const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(0xE6, 0xEA, 0xF2);
const ACCENT: egui::Color32 = egui::Color32::from_rgb(0xF2, 0xB3, 0x3D);
/// `banner.bg` and `banner.text` (§4.11): #3A2A0B and #FFD27A; the banner's font is 20 px.
const BANNER_BG: egui::Color32 = egui::Color32::from_rgb(0x3A, 0x2A, 0x0B);
const BANNER_TEXT: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xD2, 0x7A);
const BANNER_FONT: f32 = 20.0;

/// The commands of this pass's key presses (§4.10). The presses it takes leave the pass's input, so a button that holds the
/// keyboard focus (egui focuses what the mouse clicked) is not pressed a second time by the Space that paused the clock.
/// A key with Ctrl, Alt or Command held is not the bar's, and neither is any key while a text field is being typed in.
fn key_commands(ctx: &egui::Context) -> Vec<Command> {
    if ctx.egui_wants_keyboard_input() {
        return Vec::new();
    }
    let mut commands = Vec::new();
    ctx.input_mut(|input| {
        input.events.retain(|event| match event {
            egui::Event::Key { key, pressed: true, repeat: false, modifiers, .. } if !modifiers.command && !modifiers.alt => {
                match topbar::key_command(*key) {
                    Some(command) => {
                        commands.push(command);
                        false
                    }
                    None => true,
                }
            }
            _ => true,
        });
    });
    commands
}

/// The top bar as the view model gives it (§4.2): drawn as given, and the commands of what was clicked handed back.
fn draw_top_bar(ui: &mut egui::Ui, bar: &TopBar) -> Vec<Command> {
    let mut commands = Vec::new();
    let text = |label: &str, selected: bool| {
        egui::RichText::new(label).size(14.0).color(if selected { ACCENT } else { TEXT_PRIMARY })
    };
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        for button in [&bar.play, &bar.step] {
            if ui.button(text(&button.label, false)).clicked() {
                commands.push(button.command);
            }
        }
        ui.separator();
        for rung in &bar.rungs {
            if ui.add(egui::Button::new(text(&rung.button.label, rung.selected)).selected(rung.selected)).clicked() {
                commands.push(rung.button.command);
            }
        }
        ui.separator();
        ui.label(text(&bar.speed, false));
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

/// The Clear dialog (§4.7), modal over the window: the question, then `world.clear_yes` and `world.clear_no`.
fn draw_dialog(ctx: &egui::Context, dialog: &Dialog) -> Vec<Command> {
    let mut commands = Vec::new();
    egui::Modal::new(egui::Id::new("clear-world")).show(ctx, |ui| {
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

/// Where the world's image sits in `area` (§1.11): at the integer scale s = max(1, ⌊min(width / W, height / H)⌋) of the
/// screen's pixels — visible square cells — centred, its corner on a pixel so nearest filtering shows every cell whole.
fn world_rect(area: egui::Rect, world: sr_engine::state::WorldConfig, pixels_per_point: f32) -> egui::Rect {
    let (w, h) = (world.width as f32, world.height as f32);
    let scale = ((area.width() * pixels_per_point / w).min(area.height() * pixels_per_point / h)).floor().max(1.0);
    let size = egui::vec2(w, h) * scale / pixels_per_point;
    let min = ((area.center() - size / 2.0) * pixels_per_point).round() / pixels_per_point;
    egui::Rect::from_min_size(min, size)
}

/// The world and its frame loop on the app's device (§1.8.3, §3.1): `TimeControl` plans each frame, the engine's
/// `encode_frame` runs the plan, `render` draws the active view into the texture the centre shows, `poll()` — never
/// blocking — brings back the frames read back, which feed `on_report` in order.
mod sim {
    use std::collections::VecDeque;

    use eframe::{egui, egui_wgpu};
    use sr_engine::render::{Renderer, View, IMAGE_FORMAT};
    use sr_engine::scene::{Scene, Switches, TestSettings};
    use sr_engine::state::{Booking, SideFields, State, WorldConfig};
    use sr_engine::step::boxfit::BoxMode;
    use sr_engine::step::edit::Edit;
    use sr_engine::step::heat::HeatSetup;
    use sr_engine::step::latch::FramePlan;
    use sr_engine::step::nuheat::NuHeatSetup;
    use sr_engine::step::react::ReactSetup;
    use sr_engine::step::sinks::SinkSetup;
    use sr_engine::ledger::At;
    use sr_engine::step::{EosGpu, Passes, Step, Timing};
    use sr_engine::watch::Watch;
    use sr_physics::calibration::{self, PresetMasses};
    use sr_physics::registry::{Elements, Physics, Reactions};
    use sr_physics::time::{Estimates, Report, TimeControl};

    use crate::ui::banner::Feed;
    use crate::ui::topbar::{self, Command, Effect, WorldBar};

    /// Most steps one scripted `steps` action puts into a single frame: a capture asking for 10⁶ would otherwise build one
    /// enormous command buffer.
    const FORCED_CHUNK: u64 = 256;

    /// A Δt that is a CFL step: finite, above zero and below the empty world's `f32::MAX` (see `sr_physics::time`).
    fn real_dt(dt: f64) -> Option<f64> {
        (dt.is_finite() && dt > 0.0 && dt < f64::from(f32::MAX)).then_some(dt)
    }

    pub struct Sim {
        device: wgpu::Device,
        queue: wgpu::Queue,
        world: WorldConfig,
        // What a preset's drop reads: only a script drops one until the panel's (M0-T90), so the web build leaves them.
        #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
        physics: Physics,
        #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
        elements: Elements,
        #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
        masses: PresetMasses,
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
        /// The observers of the world, fed each frame's block summary (§1.9): their events slow the clock.
        watch: Watch,
        /// The slow-down's two sources, booked once per event (ui::banner).
        events: Feed,
        view: View,
        /// Steps a capture asked of the real loop and the loop has not yet submitted.
        forced: u64,
        /// Per submitted frame not yet read back, whether a capture forced it (its report is not the plan's to book).
        in_flight: VecDeque<bool>,
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

    impl Sim {
        pub fn new(render: &egui_wgpu::RenderState) -> Result<Sim, String> {
            let (device, queue) = (render.device.clone(), render.queue.clone());
            let Parts { cal, physics, elements, world, world_bar, state, previous, step, renderer, image, target } =
                Self::parts(&device, &queue)?;
            // IMAGE_FORMAT holds sRGB-encoded bytes in a UNORM texture, "the display draws it as is" (render.rs): egui-wgpu
            // reads its textures as gamma-encoded bytes in Rgba8Unorm too, so the quad needs no view of its own.
            let texture = render.renderer.write().register_native_texture(&device, &target, wgpu::FilterMode::Nearest);
            let time = TimeControl::new(Some(cal.top_rung), cal.render_reserve_ms)?;
            let watch = Watch::new(&physics, world.width, 0.0);
            Ok(Sim {
                device,
                queue,
                world,
                physics,
                elements,
                masses: cal.preset_mass_sb,
                state,
                previous,
                step,
                renderer,
                target,
                texture,
                time,
                world_bar,
                watch,
                events: Feed::new(),
                view: View::Glow,
                forced: 0,
                in_flight: VecDeque::new(),
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

        #[cfg(not(target_arch = "wasm32"))]
        pub fn paused(&self) -> bool {
            self.time.paused()
        }

        /// The clock, for the top bar's view (§4.2).
        pub fn time(&self) -> &TimeControl {
            &self.time
        }

        /// The edge choice and the dialog, for the top bar's view (§4.2, §4.7).
        pub fn world_bar(&self) -> &WorldBar {
            &self.world_bar
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

        /// Clear world (§1.12): the engine's `Clear` edit for the next step's P0 — every cell to the floor state, the sinks
        /// removed — and the observers of the old objects start over. The rung, the view, the edge mode and the slow-down
        /// are not touched.
        fn clear(&mut self) {
            // A full queue (MAX_EDITS in one step) refuses the edit and the player asks again.
            if self.step.queue_edit(&self.queue, &Edit::Clear).is_ok() {
                self.watch = Watch::new(&self.physics, self.world.width, self.sim_time);
                self.events = Feed::new();
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

        #[cfg(not(target_arch = "wasm32"))]
        /// `steps` of a script (§3.3): `n` steps through the real loop, paused or not, however the clock stands.
        pub fn queue_steps(&mut self, n: u64) {
            self.forced += n;
        }

        #[cfg(not(target_arch = "wasm32"))]
        /// A preset dropped at the world's centre, as the engine's `DropPreset` (§3.1) — the panel's own comes with M0-T90.
        pub fn drop_preset(&mut self, name: &str) -> Result<(), String> {
            let mass = match name {
                "sun" => self.masses.sun,
                "massive" => self.masses.massive,
                "giant" => self.masses.giant,
                _ => return Err(format!("unknown preset {name:?}")),
            };
            let cell = [self.world.width / 2, self.world.height / 2];
            let edit = Edit::drop_preset(name, mass, cell, self.world, &self.physics, &self.elements).map_err(|e| e.to_string())?;
            self.step.queue_edit(&self.queue, &edit).map_err(|e| e.to_string())
        }

        /// The frames read back since the last call, in order: each one's report to `on_report`, its latch cut and the
        /// events its block summary names to the slow-down (§1.8.5). Never blocks.
        fn poll(&mut self) {
            for frame in self.step.poll_frames(&self.device) {
                let r = frame.report;
                let forced = self.in_flight.pop_front().unwrap_or(false);
                self.sim_time = r.sim_time;
                self.dt_est = real_dt(f64::from(r.dt_next));
                // A forced frame was never planned, so it owes nothing: its report carries the next Δt and no steps.
                self.time.on_report(&Report {
                    steps_run: if forced { 0 } else { r.steps_run },
                    advanced: f64::from(r.advanced),
                    dt_next: f64::from(r.dt_next),
                    step_ms: r.cost.map(|c| c.step_ms),
                    latch_fired: r.latch_fired.is_some(),
                });
                // A forced frame runs with the slow-down off (`frame`), so the latch never cut it.
                if !forced && r.latch_fired.is_some() {
                    self.events.latch(&mut self.time);
                }
                for event in self.watch.update(&frame.summary, At { step: r.steps, sim_time: r.sim_time }) {
                    self.events.event(&mut self.time, event.object, event.kind);
                }
            }
        }

        /// One frame at egui time `now` (s). `frozen`: a capture is waiting for its screenshot, so nothing advances and the
        /// image on screen stays the one drawn last.
        pub fn frame(&mut self, now: f64, frozen: bool) {
            self.poll();
            if frozen {
                return;
            }
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
                self.in_flight.push_back(forced);
                self.submitted += u64::from(steps);
            }
            self.renderer.render(self.view, self.alpha, &self.target, &mut encoder);
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
[--offscreen-window] [--capture <script.json> --out <dir>]";

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
        help: bool,
    }

    fn parse(args: &[String]) -> Result<Args, String> {
        let mut out = Args { adapter: None, status_port: None, offscreen: false, capture: None, out: None, help: false };
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
                "--world" | "--measure-ui" => {
                    return Err(format!("{arg} is not built yet (its block comes later)"))
                }
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
                let mut app = SandboxApp::new(render)?;
                let info: AdapterDesc = (&render.adapter.get_info()).into();
                println!("{}", info.sr_adapter_line());
                if let Some(status) = &status {
                    status.update(|s| s.adapter = info.name.clone());
                }
                app.status = status;
                app.capture = capture.take();
                Ok(Box::new(app))
            }),
        );
        match result {
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
