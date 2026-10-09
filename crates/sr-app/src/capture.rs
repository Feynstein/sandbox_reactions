//! `--capture <script.json> --out <dir>` (m0_contrat.md §3.3): a scripted desktop session for the TV passes. The script
//! is a list of `{"id", "caption", "actions", "frames"}`; per step the driver applies the actions, lets `frames` frames
//! run, takes one screenshot through eframe's viewport command, writes `<dir>/<id>.png` with `png`, and at the end
//! `<dir>/captions.json` in review_page.py's shape (PLAYBOOK §A.4), then closes the window — exit 0.
//!
//! Only the actions that exist today are built (`pause`, `steps`). Every later block that builds a feature with a §3.3
//! action adds the action to `Action`, `parse_action` and the app's `apply`, and takes it off `NOT_BUILT`. A script that
//! names an action not built yet is refused before the window opens — `SR-ERROR` naming it, exit 4.
//!
//! Determinism: the app does not advance until the script's first step begins, and stands still from the pass that
//! asked for a screenshot until the PNG is written, so a capture shows the state after exactly `frames` passes of its
//! step — never a later one, whatever the machine's speed (a frame the surface could not acquire drops the request, and
//! the request is re-asked each pass until the reply arrives).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use eframe::egui;
use serde_json::{json, Value};

/// A §3.3 action built so far.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// `{"pause": true|false}`
    Pause(bool),
    /// `{"steps": n}` — n simulation steps, paused or not.
    Steps(u64),
}

/// §3.3's actions whose blocks have not built them: refused by name, not as unknown.
const NOT_BUILT: &[&str] =
    &["preset", "tool", "element", "paint", "view", "rung", "hover", "select", "edge", "slowdown"];

/// Most frames one step may wait: a typo of a few zeros would hang a run for hours.
const MAX_FRAMES: u64 = 100_000;
/// Passes the driver waits for a screenshot's reply before it gives up (a GPU that never answers).
const MAX_WAIT_PASSES: u32 = 1200;

pub struct Step {
    pub id: String,
    pub caption: String,
    pub actions: Vec<Action>,
    pub frames: u32,
}

/// `[A-Za-z0-9][A-Za-z0-9.-]*` (§3.3).
fn valid_id(id: &str) -> bool {
    let mut chars = id.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphanumeric())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
}

fn parse_action(step: &str, value: &Value) -> Result<Action, String> {
    let obj = value.as_object().filter(|o| o.len() == 1).ok_or_else(|| {
        format!("step {step:?}: an action is an object with exactly one key, got {value}")
    })?;
    let (name, arg) = obj.iter().next().expect("one key");
    match name.as_str() {
        "pause" => arg
            .as_bool()
            .map(Action::Pause)
            .ok_or_else(|| format!("step {step:?}: action \"pause\" takes true or false, got {arg}")),
        "steps" => arg
            .as_u64()
            .map(Action::Steps)
            .ok_or_else(|| format!("step {step:?}: action \"steps\" takes a whole number, got {arg}")),
        other if NOT_BUILT.contains(&other) => Err(format!(
            "step {step:?}: action {other:?} is not built yet (the block that builds its feature adds it to capture.rs)"
        )),
        other => Err(format!("step {step:?}: unknown action {other:?}")),
    }
}

/// The script's text → its steps, or the first thing wrong with it.
pub fn parse(text: &str) -> Result<Vec<Step>, String> {
    let value: Value = serde_json::from_str(text).map_err(|e| format!("not JSON ({e})"))?;
    let list = value.as_array().ok_or("not a list of steps")?;
    if list.is_empty() {
        return Err("the script has no step".into());
    }
    let mut steps: Vec<Step> = Vec::new();
    for (i, item) in list.iter().enumerate() {
        let obj = item.as_object().ok_or_else(|| format!("step {i}: not an object"))?;
        if let Some(key) = obj.keys().find(|k| !["id", "caption", "actions", "frames"].contains(&k.as_str())) {
            return Err(format!("step {i}: unknown key {key:?}"));
        }
        let id = obj.get("id").and_then(Value::as_str).ok_or_else(|| format!("step {i}: \"id\" is missing"))?;
        if !valid_id(id) {
            return Err(format!("step {i}: id {id:?} does not match [A-Za-z0-9][A-Za-z0-9.-]*"));
        }
        if steps.iter().any(|s| s.id == id) {
            return Err(format!("step {i}: id {id:?} is used twice"));
        }
        let caption = obj
            .get("caption")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("step {id:?}: \"caption\" is missing"))?;
        let actions = match obj.get("actions") {
            None => Vec::new(),
            Some(v) => v
                .as_array()
                .ok_or_else(|| format!("step {id:?}: \"actions\" is not a list"))?
                .iter()
                .map(|a| parse_action(id, a))
                .collect::<Result<_, _>>()?,
        };
        let frames = match obj.get("frames") {
            None => 1,
            Some(v) => v
                .as_u64()
                .filter(|n| (1..=MAX_FRAMES).contains(n))
                .ok_or_else(|| format!("step {id:?}: \"frames\" is a whole number from 1 to {MAX_FRAMES}, got {v}"))?,
        };
        steps.push(Step { id: id.into(), caption: caption.into(), actions, frames: frames as u32 });
    }
    Ok(steps)
}

/// How a run ended, shared with `run` once the window has closed.
#[derive(Default)]
pub struct Outcome {
    pub done: bool,
    pub error: Option<String>,
}

/// The tag a screenshot request carries, so its reply is told from the presented-frame probe's.
struct Tag(usize);

enum Phase {
    /// The step's actions are not applied yet.
    Pending,
    /// Frames still to run, this pass included.
    Counting(u32),
    /// The screenshot is asked for; the app stands still until it arrives.
    Waiting(u32),
    Finished,
}

pub struct Driver {
    steps: Vec<Step>,
    out: PathBuf,
    script_name: String,
    index: usize,
    phase: Phase,
    captions: Vec<Value>,
    outcome: Arc<Mutex<Outcome>>,
}

/// What the app does at the top of a pass.
pub struct Begin {
    pub actions: Vec<Action>,
    /// The simulation does not advance this pass.
    pub frozen: bool,
}

impl Driver {
    pub fn new(steps: Vec<Step>, script: &Path, out: PathBuf, outcome: Arc<Mutex<Outcome>>) -> Self {
        let script_name = script.file_name().map_or_else(|| "script.json".into(), |n| n.to_string_lossy().into_owned());
        Self { steps, out, script_name, index: 0, phase: Phase::Pending, captions: Vec::new(), outcome, }
    }

    fn fail(&mut self, ctx: &egui::Context, message: String) {
        self.outcome.lock().unwrap_or_else(PoisonError::into_inner).error = Some(message);
        self.phase = Phase::Finished;
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    fn request(&self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(Tag(self.index))));
    }

    /// The top of a pass. `presented`: the app's first frame has reached the screen (nothing starts before it).
    pub fn begin(&mut self, ctx: &egui::Context, presented: bool) -> Begin {
        let mut begin = Begin { actions: Vec::new(), frozen: true };
        if !presented {
            return begin;
        }
        if let Phase::Waiting(waited) = self.phase {
            let reply = ctx.input(|i| {
                i.raw.events.iter().find_map(|e| match e {
                    egui::Event::Screenshot { user_data, image, .. }
                        if user_data.data.as_ref().and_then(|d| d.downcast_ref::<Tag>()).is_some_and(|t| t.0 == self.index) =>
                    {
                        Some(image.clone())
                    }
                    _ => None,
                })
            });
            match reply {
                Some(image) => {
                    if let Err(e) = self.save(ctx, &image) {
                        self.fail(ctx, e);
                        return begin;
                    }
                }
                None if waited >= MAX_WAIT_PASSES => {
                    let id = self.steps[self.index].id.clone();
                    self.fail(ctx, format!("capture {id:?}: no screenshot came back in {MAX_WAIT_PASSES} passes"));
                    return begin;
                }
                None => {
                    // Still asking: a frame that could not acquire its texture dropped the request.
                    self.phase = Phase::Waiting(waited + 1);
                    self.request(ctx);
                    return begin;
                }
            }
        }
        match self.phase {
            Phase::Pending => {
                let step = &self.steps[self.index];
                begin.actions = step.actions.clone();
                self.phase = Phase::Counting(step.frames);
                begin.frozen = false;
            }
            Phase::Counting(_) => begin.frozen = false,
            Phase::Waiting(_) | Phase::Finished => {}
        }
        begin
    }

    /// The bottom of a pass: count it, and ask for the screenshot after the step's last frame.
    pub fn end(&mut self, ctx: &egui::Context) {
        if let Phase::Counting(left) = self.phase {
            if left <= 1 {
                self.phase = Phase::Waiting(0);
                self.request(ctx);
            } else {
                self.phase = Phase::Counting(left - 1);
            }
        }
    }

    fn save(&mut self, ctx: &egui::Context, image: &egui::ColorImage) -> Result<(), String> {
        let step = &self.steps[self.index];
        let [w, h] = image.size;
        let file = format!("{}.png", step.id);
        let rgb: Vec<u8> = image.pixels.iter().flat_map(|c| [c.r(), c.g(), c.b()]).collect();
        write_png(&self.out.join(&file), w as u32, h as u32, &rgb)
            .map_err(|e| format!("capture {:?}: cannot write {file} ({e})", step.id))?;
        println!("SR-CAPTURE {} {file} {w}x{h}", step.id);
        self.captions.push(json!({
            "id": step.id,
            "file": file,
            "route": "desktop",
            "path": self.script_name,
            "state": step.id,
            "viewport": format!("{w}x{h}"),
            "overflow": false,
            "caption": step.caption,
            "flag": false,
        }));
        self.index += 1;
        if self.index < self.steps.len() {
            self.phase = Phase::Pending;
            return Ok(());
        }
        let text = serde_json::to_string_pretty(&self.captions).expect("captions serialise") + "\n";
        write_atomic(&self.out.join("captions.json"), text.as_bytes())
            .map_err(|e| format!("cannot write captions.json ({e})"))?;
        println!("SR-CAPTURE DONE {} {}", self.captions.len(), self.out.display());
        self.outcome.lock().unwrap_or_else(PoisonError::into_inner).done = true;
        self.phase = Phase::Finished;
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        Ok(())
    }
}

fn write_png(path: &Path, w: u32, h: u32, rgb: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::io::BufWriter::new(std::fs::File::create(path)?);
    let mut encoder = png::Encoder::new(file, w, h);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(rgb)?;
    Ok(())
}

fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)
}

/// Reads and checks the script, and makes `out` (a probe file proves it can be written) — before any window opens.
pub fn prepare(script: &Path, out: &Path) -> Result<Vec<Step>, String> {
    let text = std::fs::read_to_string(script).map_err(|e| format!("--capture {}: cannot read ({e})", script.display()))?;
    let steps = parse(&text).map_err(|e| format!("--capture {}: {e}", script.display()))?;
    std::fs::create_dir_all(out).map_err(|e| format!("--out {}: cannot create ({e})", out.display()))?;
    let probe = out.join(".capture-probe");
    std::fs::write(&probe, b"").map_err(|e| format!("--out {}: cannot write ({e})", out.display()))?;
    let _ = std::fs::remove_file(probe);
    Ok(steps)
}
