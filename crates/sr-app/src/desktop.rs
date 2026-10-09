//! The desktop app (m0_contrat.md §3.3, §4.2, §6.1): eframe on the wgpu backend, a 1760 × 940 window titled
//! `app.title`, the world area filled with `bg.space`. `SandboxApp` is one type for the desktop and the web entry
//! (web.rs runs the same app in the canvas); the native-only parts — `--adapter`, `--offscreen-window`, the status
//! endpoint — sit behind `cfg(not(target_arch = "wasm32"))`.

use std::collections::VecDeque;
use std::sync::Arc;

use eframe::{egui, egui_wgpu};

/// `app.title` (§4.1); the string table itself, crates/sr-app/src/strings.rs, comes with M0-T13.
#[cfg(not(target_arch = "wasm32"))]
pub const APP_TITLE: &str = "Sandbox Reactions";
/// `bg.space` (§4.11): the world's background and vacuum, #05070D.
pub const BG_SPACE: egui::Color32 = egui::Color32::from_rgb(0x05, 0x07, 0x0D);
/// The default window (§4.2), in logical px.
#[cfg(not(target_arch = "wasm32"))]
pub const WINDOW_SIZE: [f32; 2] = [1760.0, 940.0];

/// The device every frame runs on, desktop and web alike (§6.1, §6.2.2): WebGPU's default limits, no feature.
fn device_descriptor(_adapter: &wgpu::Adapter) -> wgpu::DeviceDescriptor<'static> {
    wgpu::DeviceDescriptor {
        label: Some("sr-app"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        ..Default::default()
    }
}

/// The wgpu setup both targets start from: the high-performance adapter, `device_descriptor`'s device.
pub fn wgpu_setup() -> egui_wgpu::WgpuSetupCreateNew {
    let mut setup = egui_wgpu::WgpuSetupCreateNew::without_display_handle();
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
    /// Simulation steps taken. Until the engine is wired into the app (the lot after M0-T8) the app counts them itself:
    /// one per frame while running, so `pause` and `steps` already mean something on screen and in a capture.
    step: u64,
    paused: bool,
    #[cfg(not(target_arch = "wasm32"))]
    status: Option<std::sync::Arc<crate::status::Status>>,
    /// `--capture`'s driver (capture.rs): the app waits on it before it advances.
    #[cfg(not(target_arch = "wasm32"))]
    capture: Option<crate::capture::Driver>,
}

impl SandboxApp {
    pub fn new() -> Self {
        Self {
            frames: 0,
            recent: VecDeque::new(),
            presented: false,
            step: 0,
            paused: false,
            #[cfg(not(target_arch = "wasm32"))]
            status: None,
            #[cfg(not(target_arch = "wasm32"))]
            capture: None,
        }
    }

    /// A scripted action (§3.3) — the ones built so far.
    #[cfg(not(target_arch = "wasm32"))]
    fn apply(&mut self, action: &crate::capture::Action) {
        use crate::capture::Action;
        match *action {
            Action::Pause(p) => self.paused = p,
            Action::Steps(n) => self.step += n,
        }
    }
}

impl Default for SandboxApp {
    fn default() -> Self {
        Self::new()
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
                begin.actions.iter().for_each(|a| self.apply(a));
                self.capture = Some(driver);
                begin.frozen
            }
            None => false,
        };
        #[cfg(target_arch = "wasm32")]
        let frozen = false;
        if !self.paused && !frozen {
            self.step += 1;
        }

        self.frames += 1;
        let now = ui.input(|i| i.time);
        self.recent.push_back(now);
        while self.recent.front().is_some_and(|t| now - t > 1.0) {
            self.recent.pop_front();
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(status) = &self.status {
            let (frames, fps, presented) = (self.frames, self.recent.len() as f64, self.presented);
            status.update(|s| {
                s.frame = frames;
                s.fps = fps;
                if presented {
                    s.state = crate::status::State::Ready;
                }
            });
        }

        // The world area: until the panels (§4.2) and the world's view (§1.11) arrive, the whole window is the
        // centre, filled with bg.space, with the step counter as a stand-in readout (no string-table entry: it goes
        // when the readouts arrive).
        let readout = format!("step {}{}", self.step, if self.paused { " · paused" } else { "" });
        egui::CentralPanel::no_frame().frame(egui::Frame::NONE.fill(BG_SPACE)).show(ui, |ui| {
            ui.painter().text(
                egui::pos2(16.0, 12.0),
                egui::Align2::LEFT_TOP,
                readout,
                egui::FontId::monospace(18.0),
                egui::Color32::from_gray(0xc8),
            );
        });
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(driver) = &mut self.capture {
            driver.end(ui.ctx());
        }
        ui.ctx().request_repaint();
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        BG_SPACE.to_normalized_gamma_f32()
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

    use super::{SandboxApp, APP_TITLE, WINDOW_SIZE};

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

        // Bound before the window, so a poller reads "booting" while the window and the device come up (§3.4).
        let status = match args.status_port.map(crate::status::serve).transpose() {
            Ok(s) => s,
            Err(e) => {
                println!("SR-ERROR --status-port {}: cannot listen on 127.0.0.1 ({e})", args.status_port.unwrap_or(0));
                return EXIT_ARGS;
            }
        };

        let mut viewport = egui::ViewportBuilder::default().with_title(APP_TITLE).with_inner_size(WINDOW_SIZE);
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
            APP_TITLE,
            options,
            Box::new(move |cc| {
                let mut app = SandboxApp::new();
                let render = cc.wgpu_render_state.as_ref().ok_or("eframe gave no wgpu render state")?;
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
