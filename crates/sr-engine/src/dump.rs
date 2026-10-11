//! State dumps (m0_contrat.md §2.12.3, §3.1's `load_dump` and `dump_state`, §3.3's `--dump-every` and `--load-dump`;
//! M0-T26) — tests and headless runs only, never a save format (§2.13).
//!
//! A dump is the eight bytes `SRDUMP01`, a little-endian u32 header length, a UTF-8 JSON header, then §2.2's fourteen
//! channels, each a little-endian f32 plane in §2.2's order, row-major — the state's canonical form ([`crate::state`]).
//! The header: `version` (1), `width`, `height`, `step` (the steps run since the scene was written — the next step's
//! index), `sim_time`, `channels` (§2.2's names, in order), `sinks` (`x`, `y`, `vx`, `vy`, `m`; none until P3's block),
//! and `resume` (contract §0.5 [M0-T26]): the step's own records, which no cell holds and a resumed run needs to go on
//! bit for bit (§1.3.4) — Δt_n and Δt_{n+1} (`dt`, `dt_next`), the latch's sim time since its last set (`latch_since`,
//! its arming, §1.8.5), the active box's record (`box`: x0, y0, w, h, fft_w, fft_h, from_step, fits) and its indirect
//! sizes (`box_args`, §1.3.3). f32 values are written as JSON numbers, which carry them exactly.
//!
//! [`Dump::from_bytes`] refuses, naming the cause: a wrong magic, a file cut anywhere, bytes past the last plane, a
//! header that is not UTF-8 JSON, an unknown or missing key, another version, a world §2.8 refuses, a channel list other
//! than §2.2's, more than 16 sinks.

use std::path::Path;

use serde_json::{json, Map, Value};

use crate::state::{State, WorldConfig, CHANNELS, N_CHANNELS};
use crate::step::Step;

pub const MAGIC: &[u8; 8] = b"SRDUMP01";
pub const VERSION: u64 = 1;
/// §2.6: up to 16 sinks.
pub const MAX_SINKS: usize = 16;

/// A sink as the header holds it (§2.6): position and velocity in cells and cells per t.u., mass in sandbox units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sink {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub m: f64,
}

/// The header's `resume`: the step's records at a step boundary, beyond the cells and the step index.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Resume {
    pub dt: f32,
    pub dt_next: f32,
    pub latch_since: f32,
    pub box_record: [u32; 8],
    pub box_args: [u32; 6],
}

/// A state dump in memory: the header's values and the planes in canonical order.
#[derive(Clone, Debug, PartialEq)]
pub struct Dump {
    pub world: WorldConfig,
    pub step: u64,
    pub sim_time: f64,
    pub sinks: Vec<Sink>,
    pub resume: Resume,
    pub planes: Vec<f32>,
}

impl Dump {
    /// The header as written: `version`, `width`, `height`, `step`, `sim_time`, `channels`, `sinks`, `resume`.
    fn header(&self) -> Value {
        let r = &self.resume;
        let sinks: Vec<Value> =
            self.sinks.iter().map(|s| json!({"x": s.x, "y": s.y, "vx": s.vx, "vy": s.vy, "m": s.m})).collect();
        json!({
            "version": VERSION,
            "width": self.world.width,
            "height": self.world.height,
            "step": self.step,
            "sim_time": self.sim_time,
            "channels": CHANNELS,
            "sinks": sinks,
            "resume": {
                "dt": f64::from(r.dt),
                "dt_next": f64::from(r.dt_next),
                "latch_since": f64::from(r.latch_since),
                "box": r.box_record,
                "box_args": r.box_args,
            },
        })
    }

    /// The file's bytes (§2.12.3).
    pub fn to_bytes(&self) -> Vec<u8> {
        let header = self.header().to_string();
        let cells = self.world.cells();
        assert_eq!(self.planes.len(), N_CHANNELS * cells, "a dump holds 14 planes of its world");
        let mut out = Vec::with_capacity(12 + header.len() + 4 * self.planes.len());
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&(header.len() as u32).to_le_bytes());
        out.extend_from_slice(header.as_bytes());
        for c in 0..N_CHANNELS {
            for v in &self.planes[c * cells..(c + 1) * cells] {
                out.extend_from_slice(&v.to_le_bytes());
            }
        }
        out
    }

    /// Reads a dump from its bytes; any departure from §2.12.3 is refused, naming it.
    pub fn from_bytes(bytes: &[u8]) -> Result<Dump, String> {
        let magic = bytes.get(..8).ok_or_else(|| format!("truncated: {} bytes, the magic alone is 8", bytes.len()))?;
        if magic != MAGIC {
            return Err(format!("not a state dump: magic {:?}, expected \"SRDUMP01\"", String::from_utf8_lossy(magic)));
        }
        let len = bytes.get(8..12).ok_or("truncated in the header length")?;
        let len = u32::from_le_bytes(len.try_into().expect("4 bytes")) as usize;
        let text = bytes.get(12..12 + len).ok_or_else(|| {
            format!("truncated in the header: {len} bytes announced, {} present", bytes.len().saturating_sub(12))
        })?;
        let text = std::str::from_utf8(text).map_err(|e| format!("header: not UTF-8: {e}"))?;
        let header: Value = serde_json::from_str(text).map_err(|e| format!("header: not JSON: {e}"))?;
        let mut dump = parse_header(&header)?;
        dump.sim_time = exact_number(text, "sim_time").unwrap_or(dump.sim_time);

        let body = &bytes[12 + len..];
        let want = N_CHANNELS * dump.world.cells() * 4;
        if body.len() < want {
            return Err(format!("truncated in the planes: {} of {want} bytes", body.len()));
        }
        if body.len() > want {
            return Err(format!("{} bytes past the last plane", body.len() - want));
        }
        dump.planes = body.as_chunks::<4>().0.iter().map(|b| f32::from_le_bytes(*b)).collect();
        Ok(dump)
    }

    /// Writes the dump to `path`.
    pub fn write(&self, path: &Path) -> std::io::Result<()> {
        std::fs::write(path, self.to_bytes())
    }

    /// Reads and checks the dump at `path`.
    pub fn read(path: &Path) -> Result<Dump, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Dump::from_bytes(&bytes)
    }
}

/// serde_json parses floats best-effort (its `float_roundtrip` feature is off, §1.2's table unchanged): a 17-digit f64
/// may come back one ulp off. The f32 values survive the cast back to f32; `sim_time`, an f64, is re-read from its token
/// with Rust's correctly rounded parser, which brings back what serde_json's shortest form wrote. Called after the header
/// is checked, so the key's first occurrence is the top-level key (every string value is one of §2.2's names).
fn exact_number(text: &str, key: &str) -> Option<f64> {
    let quoted = format!("\"{key}\"");
    let rest = text[text.find(&quoted)? + quoted.len()..].trim_start().strip_prefix(':')?.trim_start();
    let end = rest.find(|c: char| !(c.is_ascii_digit() || "+-.eE".contains(c))).unwrap_or(rest.len());
    rest[..end].parse().ok().filter(|v: &f64| v.is_finite())
}

/// The header's object at `at`, holding exactly `keys`.
fn object<'a>(v: &'a Value, at: &str, keys: &[&str]) -> Result<&'a Map<String, Value>, String> {
    let m = v.as_object().ok_or_else(|| format!("{at}: not an object"))?;
    if let Some(k) = m.keys().find(|k| !keys.contains(&k.as_str())) {
        return Err(format!("{at}.{k}: unknown key"));
    }
    match keys.iter().find(|k| !m.contains_key(**k)) {
        Some(k) => Err(format!("{at}.{k}: missing")),
        None => Ok(m),
    }
}

fn whole(m: &Map<String, Value>, at: &str, k: &str) -> Result<u64, String> {
    m[k].as_u64().ok_or_else(|| format!("{at}.{k}: not a whole number: {}", m[k]))
}

fn number(m: &Map<String, Value>, at: &str, k: &str) -> Result<f64, String> {
    m[k].as_f64().filter(|v| v.is_finite()).ok_or_else(|| format!("{at}.{k}: not a finite number: {}", m[k]))
}

fn words<const N: usize>(m: &Map<String, Value>, at: &str, k: &str) -> Result<[u32; N], String> {
    let bad = || format!("{at}.{k}: not {N} whole numbers below 2^32: {}", m[k]);
    let list = m[k].as_array().filter(|a| a.len() == N).ok_or_else(bad)?;
    let mut out = [0u32; N];
    for (o, v) in out.iter_mut().zip(list) {
        *o = v.as_u64().and_then(|w| u32::try_from(w).ok()).ok_or_else(bad)?;
    }
    Ok(out)
}

/// The header's values; the planes are left empty for the caller.
fn parse_header(v: &Value) -> Result<Dump, String> {
    let at = "header";
    let keys = ["version", "width", "height", "step", "sim_time", "channels", "sinks", "resume"];
    let m = object(v, at, &keys)?;
    let version = whole(m, at, "version")?;
    if version != VERSION {
        return Err(format!("header.version: {version}, this build reads {VERSION}"));
    }
    let size = |k: &str| whole(m, at, k).and_then(|n| u32::try_from(n).map_err(|_| format!("header.{k}: {n}")));
    let world = WorldConfig::new(size("width")?, size("height")?).map_err(|e| format!("header: {e}"))?;
    let channels: Option<Vec<&str>> = m["channels"].as_array().and_then(|a| a.iter().map(Value::as_str).collect());
    if channels.as_deref() != Some(&CHANNELS[..]) {
        return Err(format!("header.channels: {}, expected §2.2's {CHANNELS:?} in that order", m["channels"]));
    }
    let sinks = m["sinks"].as_array().ok_or("header.sinks: not a list")?;
    if sinks.len() > MAX_SINKS {
        return Err(format!("header.sinks: {} sinks, at most {MAX_SINKS}", sinks.len()));
    }
    let sinks = sinks
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let at = format!("header.sinks[{i}]");
            let s = object(s, &at, &["x", "y", "vx", "vy", "m"])?;
            let n = |k: &str| number(s, &at, k);
            Ok(Sink { x: n("x")?, y: n("y")?, vx: n("vx")?, vy: n("vy")?, m: n("m")? })
        })
        .collect::<Result<Vec<Sink>, String>>()?;
    let at_r = "header.resume";
    let r = object(&m["resume"], at_r, &["dt", "dt_next", "latch_since", "box", "box_args"])?;
    let f32_of = |k: &str| number(r, at_r, k).map(|v| v as f32);
    let resume = Resume {
        dt: f32_of("dt")?,
        dt_next: f32_of("dt_next")?,
        latch_since: f32_of("latch_since")?,
        box_record: words(r, at_r, "box")?,
        box_args: words(r, at_r, "box_args")?,
    };
    Ok(Dump {
        world,
        step: whole(m, at, "step")?,
        sim_time: number(m, at, "sim_time")?,
        sinks,
        resume,
        planes: Vec::new(),
    })
}

/// §3.1 `dump_state()`: the state, the step index, the frames' sim time and the step's records, read back after every
/// submitted step. Blocks on the device (tests and headless runs only, §6.3).
pub fn dump_state(device: &wgpu::Device, queue: &wgpu::Queue, state: &State, step: &Step) -> Dump {
    let (index, sim_time, resume) = step.resume_records(device, queue);
    Dump { world: state.world(), step: index, sim_time, sinks: Vec::new(), resume, planes: state.readback(device, queue) }
}

/// §3.1 `load_dump(dump)`: writes the dump's cells into `state` and its records into `step`, whose next step is then
/// the dump's `step`. Call it before `step` records anything. Refused: another world than the state's, a step index past
/// the GPU's u32 count, any sink (P3 holds none until its block).
pub fn load_dump(queue: &wgpu::Queue, state: &State, step: &Step, dump: &Dump) -> Result<(), String> {
    if dump.world != state.world() {
        let (d, s) = (dump.world, state.world());
        return Err(format!("dump world {} × {}, the state's {} × {}", d.width, d.height, s.width, s.height));
    }
    if dump.step > u64::from(u32::MAX) {
        return Err(format!("dump step {}: past the GPU's u32 step count", dump.step));
    }
    if !dump.sinks.is_empty() {
        return Err(format!("dump holds {} sink(s): no sink state before P3's block", dump.sinks.len()));
    }
    state.upload(queue, &dump.planes).map_err(|e| e.to_string())?;
    step.restore(queue, dump.step, dump.sim_time, &dump.resume);
    Ok(())
}
