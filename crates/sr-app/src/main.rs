//! `sandbox-reactions` — the game's binary (m0_contrat.md §3.3). No subcommand is the desktop app (M0-T5); `headless`
//! runs the engine with no window. An argument nobody knows is exit 4, with a line starting `SR-ERROR`.

#[cfg(not(target_arch = "wasm32"))]
mod capture;
mod desktop;
#[cfg(not(target_arch = "wasm32"))]
mod status;
mod strings;
#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(not(target_arch = "wasm32"))]
const USAGE: &str = "usage: sandbox-reactions [--adapter S] [--status-port P] [--offscreen-window] [--capture SCRIPT --out DIR] | \
sandbox-reactions headless --scene preset:<sun|massive|giant> --steps N [--world WxH] [--max-steps N] [--out DIR] \
[--adapter S]";

/// The web build (§3.5): trunk's wasm-bindgen start runs `main`, which hands the canvas to the app and returns.
#[cfg(target_arch = "wasm32")]
fn main() {
    web::start();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.first().map(String::as_str) {
        Some("headless") => sr_engine::headless::run(&args[1..]),
        Some("-h" | "--help") => {
            println!("{USAGE}");
            0
        }
        _ => desktop::run(&args),
    };
    std::process::exit(code);
}
