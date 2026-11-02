//! `--measure-ui <frames>` (§3.3, §1.8.3's R_reserve): the desktop app with the Sun-like preset at ×1, the panels' and the
//! presentation's CPU cost per frame measured — eframe's own figure for the frame (the egui pass, the tessellation, the paint,
//! the vsync wait taken out) less what the app spent on the world itself (its read-back and its frame, `SandboxApp::sim_cost`) —
//! `SR-UI p95_ms=<x>` printed, and `render_reserve_ms` = p95 + 0.5 written into the calibration file. The key is outside
//! `physics_hash`, so G-CAL holds. A software adapter never writes the default file: a reserve measured on lavapipe is not the
//! Quadro's, and every frame budget would carry it.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};

use sr_physics::calibration::Calibration;

/// The file the key goes into when `--calibration` names none (the repo root's, where the game is run from).
pub const DEFAULT_FILE: &str = "assets/calibration.json";
/// §3.3: the reserve is the p95 and this much more.
const MARGIN_MS: f64 = 0.5;
/// Frames before the first sample: the preset's drop, the first font layouts, the pipelines' first use.
const WARM_UP: u32 = 30;

/// What the run reports to `desktop::run`, as the capture's driver does.
#[derive(Default)]
pub struct Outcome {
    pub done: bool,
    pub error: Option<String>,
}

pub struct Meter {
    frames: usize,
    file: PathBuf,
    adapter: String,
    samples: Vec<f64>,
    /// Passes seen, the first of which drops the preset.
    passes: u32,
    /// The report is made; the passes before the window closes measure nothing.
    reported: bool,
    outcome: Arc<Mutex<Outcome>>,
}

impl Meter {
    /// `file`: where the key goes (`--calibration`), the default file when none. A software adapter measures only into a file
    /// that was named; the file must already be a valid calibration (the run adds one key, never makes the file).
    pub fn new(frames: usize, file: Option<PathBuf>, adapter: &str, software: bool, outcome: Arc<Mutex<Outcome>>) -> Result<Meter, String> {
        if software && file.is_none() {
            return Err(format!(
                "--measure-ui on {adapter} (a software adapter) would write its cost as the real one's: name a file with --calibration, \
                 or run on the GPU the game is for"
            ));
        }
        let file = file.unwrap_or_else(|| DEFAULT_FILE.into());
        let text = std::fs::read_to_string(&file).map_err(|e| format!("--measure-ui: cannot read {}: {e}", file.display()))?;
        Calibration::parse(&text).map_err(|e| format!("--measure-ui: {} is not a calibration: {}", file.display(), e.line()))?;
        Ok(Meter { frames, file, adapter: adapter.to_string(), samples: Vec::with_capacity(frames), passes: 0, reported: false, outcome })
    }

    /// True on the first pass only: the caller drops the Sun-like preset at ×1.
    pub fn first_pass(&self) -> bool {
        self.passes == 0
    }

    /// The run cannot go on: the caller closes the window, and `desktop::run` exits with the line.
    pub fn fail(&mut self, error: String) {
        self.outcome.lock().unwrap_or_else(PoisonError::into_inner).error = Some(error);
    }

    /// One pass begins. `cpu_s`: eframe's CPU time of the frame before, `sim_s` what that frame spent on the world. True once the
    /// samples are in and the report is made (the caller closes the window).
    pub fn pass(&mut self, cpu_s: Option<f32>, sim_s: f64) -> bool {
        self.passes += 1;
        if self.reported {
            return true;
        }
        if let (Some(cpu), true) = (cpu_s, self.passes > WARM_UP) {
            self.samples.push((f64::from(cpu) - sim_s).max(0.0) * 1000.0);
        }
        if self.samples.len() < self.frames {
            return false;
        }
        self.reported = true;
        let result = self.report();
        let mut outcome = self.outcome.lock().unwrap_or_else(PoisonError::into_inner);
        match result {
            Ok(()) => outcome.done = true,
            Err(e) => outcome.error = Some(e),
        }
        true
    }

    fn report(&mut self) -> Result<(), String> {
        let p95 = (p95(&mut self.samples) * 1000.0).round() / 1000.0;
        println!("SR-UI p95_ms={p95} frames={} adapter={}", self.samples.len(), self.adapter);
        let reserve = p95 + MARGIN_MS;
        let text = std::fs::read_to_string(&self.file).map_err(|e| format!("cannot read {}: {e}", self.file.display()))?;
        let mut cal = Calibration::parse(&text).map_err(|e| e.line())?;
        cal.render_reserve_ms = Some(reserve);
        std::fs::write(&self.file, cal.to_json()).map_err(|e| format!("cannot write {}: {e}", self.file.display()))?;
        println!("SR-UI render_reserve_ms={reserve} file={} (the next build embeds it)", self.file.display());
        Ok(())
    }
}

/// The nearest-rank 95th percentile (sorts `samples`).
fn p95(samples: &mut [f64]) -> f64 {
    samples.sort_by(f64::total_cmp);
    let rank = (samples.len() as f64 * 0.95).ceil() as usize;
    samples[rank.clamp(1, samples.len()) - 1]
}

#[cfg(test)]
mod tests {
    use super::p95;

    #[test]
    fn p95_is_the_nearest_rank() {
        let mut v: Vec<f64> = (1..=100).rev().map(f64::from).collect();
        assert_eq!(p95(&mut v), 95.0);
        assert_eq!(p95(&mut [7.0]), 7.0);
        assert_eq!(p95(&mut [3.0, 1.0, 2.0]), 3.0);
    }
}
