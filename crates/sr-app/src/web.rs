//! The web entry (m0_contrat.md §3.5, §4.8, §4.9): runs `desktop::SandboxApp` — the one app type of both targets — in
//! web/index.html's `<canvas id="sr-canvas">` through eframe's web runner. The page tells the world where the app
//! stands through `document.body.dataset`: `srState` is "loading" while the device comes up, "ready" after the first
//! presented frame, "error" on a fatal one (with §4.8's message on the page and in `srError`); `srAdapter` is the
//! adapter's description. index.html has already checked WebGPU — without it this code never loads (Q4).

use eframe::egui_wgpu;
use sr_engine::gpu::AdapterDesc;
use wasm_bindgen::JsCast;

use crate::desktop::{wgpu_setup, SandboxApp};

/// `error.no_adapter` and `error.device_lost` (§4.1, §4.8), verbatim; the string table (strings.rs) comes with M0-T13.
const ERROR_NO_ADAPTER: &str =
    "No compatible graphics card found. Sandbox Reactions needs a graphics card with Vulkan, Metal or DirectX 12 support.";
const ERROR_DEVICE_LOST: &str = "The graphics card stopped responding. Please restart Sandbox Reactions.";

const CANVAS_ID: &str = "sr-canvas";
const LOADING_ID: &str = "sr-loading";

fn body() -> Option<web_sys::HtmlElement> {
    web_sys::window()?.document()?.body()
}

fn set_data(key: &str, value: &str) {
    if let Some(body) = body() {
        let _ = body.dataset().set(key, value);
    }
}

/// `srState` "ready": raised by the app's first presented frame (desktop.rs's flag), never by the page loading.
pub fn signal_ready() {
    // A fatal error already on the page stays the last word.
    if body().is_some_and(|b| b.dataset().get("srState").as_deref() == Some("error")) {
        return;
    }
    set_data("srState", "ready");
    if let Some(line) = web_sys::window().and_then(|w| w.document()).and_then(|d| d.get_element_by_id(LOADING_ID)) {
        let _ = line.set_attribute("hidden", "");
    }
}

/// A fatal error: `srState` "error", the message in `srError` and on the page where the loading line stood.
fn fail(message: &str, detail: &str) {
    log::error!("{message} ({detail})");
    set_data("srState", "error");
    set_data("srError", message);
    set_data("srErrorDetail", detail);
    if let Some(line) = web_sys::window().and_then(|w| w.document()).and_then(|d| d.get_element_by_id(LOADING_ID)) {
        line.set_text_content(Some(message));
        let _ = line.remove_attribute("hidden");
    }
}

/// The wasm binary's `main`: sets the state and starts the runner on the page's canvas; returns at once, the
/// browser's event loop carries on.
pub fn start() {
    console_error_panic_hook::set_once();
    set_data("srState", "loading");
    wasm_bindgen_futures::spawn_local(async {
        if let Err(detail) = run().await {
            fail(ERROR_NO_ADAPTER, &detail);
        }
    });
}

async fn run() -> Result<(), String> {
    let canvas = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(CANVAS_ID))
        .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        .ok_or_else(|| format!("no <canvas id=\"{CANVAS_ID}\"> on the page"))?;

    let options = eframe::WebOptions {
        wgpu_options: egui_wgpu::WgpuConfiguration {
            wgpu_setup: egui_wgpu::WgpuSetup::CreateNew(wgpu_setup()),
            ..Default::default()
        },
        ..Default::default()
    };

    eframe::WebRunner::new()
        .start(
            canvas,
            options,
            Box::new(|cc| {
                let render = cc.wgpu_render_state.as_ref().ok_or("eframe gave no wgpu render state")?;
                let info: AdapterDesc = (&render.adapter.get_info()).into();
                set_data("srAdapter", &info.to_string());
                // The callback runs on a lost device and on a destroyed one (the page closing): only the first is fatal.
                render.device.set_device_lost_callback(|reason, detail| {
                    if reason != wgpu::DeviceLostReason::Destroyed {
                        fail(ERROR_DEVICE_LOST, &detail);
                    }
                });
                Ok(Box::new(SandboxApp::new()))
            }),
        )
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| format!("{e:?}")))
}
