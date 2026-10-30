//! The web entry (m0_contrat.md §3.5, §4.8, §4.9): runs `desktop::SandboxApp` — the one app type of both targets — in
//! web/index.html's `<canvas id="sr-canvas">` through eframe's web runner. The page tells the world where the app
//! stands through `document.body.dataset`: `srState` is "loading" while the device comes up, "ready" after the first
//! presented frame, "error" on a fatal one (with §4.8's message on the page and in `srError`); `srAdapter` is the
//! adapter's description. index.html has already checked WebGPU — without it this code never loads (Q4).

use std::cell::RefCell;
use std::rc::Rc;

use eframe::{egui, egui_wgpu};
use sr_engine::gpu::AdapterDesc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::js_sys::{Array, Function, Object, Promise, Reflect};
use wasm_bindgen_futures::JsFuture;

use crate::desktop::{wgpu_setup, SandboxApp};
use crate::ui::theme::{self, BG_SPACE};

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
                theme::install(&cc.egui_ctx);
                // The callback runs on a lost device and on a destroyed one (the page closing): only the first is fatal.
                render.device.set_device_lost_callback(|reason, detail| {
                    if reason != wgpu::DeviceLostReason::Destroyed {
                        fail(ERROR_DEVICE_LOST, &detail);
                    }
                });
                Ok(Box::new(Warming::start(cc.egui_ctx.clone(), render.clone())))
            }),
        )
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| format!("{e:?}")))
}

/// The app while its pipelines compile: it draws `bg.space` until `warm` has run and `SandboxApp::new` built the world,
/// then is that app — "ready" stays the app's own first presented frame.
struct Warming {
    built: Rc<RefCell<Option<Result<SandboxApp, String>>>>,
    app: Option<SandboxApp>,
}

impl Warming {
    fn start(ctx: egui::Context, render: egui_wgpu::RenderState) -> Self {
        let built = Rc::new(RefCell::new(None));
        let slot = built.clone();
        let warmed = warm(&render.device, &render.queue);
        wasm_bindgen_futures::spawn_local(async move {
            // A rejected compile is the real build's to report: allSettled never rejects.
            let _ = JsFuture::from(warmed).await;
            *slot.borrow_mut() = Some(SandboxApp::new(&render));
            ctx.request_repaint();
        });
        Self { built, app: None }
    }
}

impl eframe::App for Warming {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        if self.app.is_none() {
            match self.built.borrow_mut().take() {
                Some(Ok(app)) => self.app = Some(app),
                Some(Err(detail)) => fail(ERROR_NO_ADAPTER, &detail),
                None => {}
            }
        }
        if let Some(app) = &mut self.app {
            app.ui(ui, frame);
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        BG_SPACE.to_normalized_gamma_f32()
    }
}

/// Chrome's D3D12 backend compiles each sync `createComputePipeline` on the spot, one after another: the app's 73 compute
/// pipelines held its first frame 11–31 s on win-laptop (M0-D118). The warm-up builds the app's device work once
/// (`SandboxApp::dry_run`) with `createComputePipeline` shadowed on the device: each descriptor goes to
/// `createComputePipelineAsync` — compiled in parallel, the result cached by content, so the real build's identical sync
/// creations find it done (≈ 5 s for all 73, then 0.1 s) — and the dry build gets an invalid pipeline back (an entry point
/// no module has: nothing compiles twice). Its errors, and those of the bind groups made from those pipelines, stay in a
/// validation error scope. Resolves once every compile has settled.
fn warm(device: &wgpu::Device, queue: &wgpu::Queue) -> Promise {
    let done = Promise::resolve(&JsValue::UNDEFINED);
    let Some(raw) = device.as_webgpu() else {
        return done;
    };
    let raw = JsValue::from(raw.clone());
    let key = JsValue::from_str("createComputePipeline");
    let (Ok(create), Ok(create_async)) = (Reflect::get(&raw, &key), Reflect::get(&raw, &"createComputePipelineAsync".into()))
    else {
        return done;
    };
    let (create, create_async) = (create.unchecked_into::<Function>(), create_async.unchecked_into::<Function>());
    let pending = Array::new();
    let shadow = {
        let (raw, pending) = (raw.clone(), pending.clone());
        Closure::<dyn FnMut(JsValue) -> JsValue>::new(move |desc: JsValue| {
            if let Ok(compiling) = create_async.call1(&raw, &desc) {
                pending.push(&compiling);
            }
            let module = Reflect::get(&desc, &"compute".into())
                .and_then(|c| Reflect::get(&c, &"module".into()))
                .unwrap_or(JsValue::UNDEFINED);
            let (invalid, compute) = (Object::new(), Object::new());
            let _ = Reflect::set(&compute, &"module".into(), &module);
            let _ = Reflect::set(&compute, &"entryPoint".into(), &"sr_warm_no_entry".into());
            let _ = Reflect::set(&invalid, &"layout".into(), &"auto".into());
            let _ = Reflect::set(&invalid, &"compute".into(), &compute);
            create.call1(&raw, &JsValue::from(invalid)).unwrap_or(JsValue::UNDEFINED)
        })
    };
    let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let _ = Reflect::set(&raw, &key, shadow.as_ref());
    let dry = SandboxApp::dry_run(device, queue);
    let _ = Reflect::delete_property(raw.unchecked_ref::<Object>(), &key);
    drop(scope);
    if let Err(detail) = dry {
        // The real build meets the same error and reports it.
        log::warn!("warm-up: {detail}");
    }
    Promise::all_settled(&pending)
}
