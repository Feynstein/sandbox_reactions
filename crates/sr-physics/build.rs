//! Embeds `assets/calibration.json` whenever it exists (m0_contrat.md §2.11, §3.1; M0-D62): src/calibration_embed.rs
//! chooses, this writes the choice where `calibration::EMBEDDED` includes it.

#[path = "src/calibration_embed.rs"]
mod calibration_embed;

use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR"));
    // The folder, not the file: cargo reruns a script on every build while a watched file is missing, and it scans a
    // watched folder, so the file's arrival, change or removal reruns this.
    println!("cargo:rerun-if-changed=../../assets");
    println!("cargo:rerun-if-changed=src/calibration_embed.rs");
    let source = calibration_embed::embedded_source(&manifest.join("../../assets"));
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR")).join("embedded_calibration.rs");
    std::fs::write(out, source).expect("OUT_DIR is writable");
}
