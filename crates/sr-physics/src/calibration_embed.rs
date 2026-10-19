//! Which calibration the binary embeds (m0_contrat.md §2.11, §3.1; M0-D62): `assets/calibration.json` whenever the file
//! exists, else none — so a calibration is in the next build with no hand edit after `calibrate` writes it, and a tree
//! without the file still builds. `build.rs` compiles this file alone (no dependency) and writes [`embedded_source`]'s
//! text into `OUT_DIR`, which [`super::EMBEDDED`] includes; the `calibration` scope checks the choice on a folder with and
//! without the file.

use std::path::Path;

/// The calibration's file name inside the assets folder.
pub const FILE: &str = "calibration.json";

/// The Rust expression `EMBEDDED` is for an assets folder: `Some(include_str!("<assets>/calibration.json"))` when the
/// file exists, else `None`.
pub fn embedded_source(assets: &Path) -> String {
    let file = assets.join(FILE);
    if file.is_file() {
        format!("Some(include_str!({:?}))", file.display().to_string())
    } else {
        "None".to_string()
    }
}
