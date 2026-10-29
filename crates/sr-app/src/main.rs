//! `sandbox-reactions` — the game's binary (m0_contrat.md §3.3). No subcommand is the desktop app (M0-T5); `headless`
//! runs the engine with no window. An argument nobody knows is exit 4, with a line starting `SR-ERROR`.

#[cfg(not(target_arch = "wasm32"))]
mod capture;
mod desktop;
// Number formats (§4.12, M0-T75): the readout and inspector lots call them; until then only the `formats` scope does.
#[allow(dead_code)]
mod format;
#[cfg(not(target_arch = "wasm32"))]
mod status;
mod strings;
mod ui;
#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(not(target_arch = "wasm32"))]
const USAGE: &str = "usage: sandbox-reactions [--adapter S] [--status-port P] [--offscreen-window] [--capture SCRIPT --out DIR] | \
sandbox-reactions headless --scene <FILE|preset:<sun|massive|giant>> --steps N [--world WxH] [--edge leave|bounce] \
[--max-steps N] [--out DIR] [--timing] [--dump-every N] [--load-dump FILE [--continue]] [--edits FILE] [--adapter S | --cpu-reference] | \
sandbox-reactions calibrate --out FILE [--adapter S] [--only ITEM] [--resume DIR] [--progress-every N] [--checkpoint-every M]";

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
        Some("calibrate") => sr_engine::calibrate::run(&args[1..]),
        Some("-h" | "--help") => {
            println!("{USAGE}");
            0
        }
        _ => desktop::run(&args),
    };
    std::process::exit(code);
}
