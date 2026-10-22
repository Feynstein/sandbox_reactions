//! State dumps (m0_contrat.md §2.12.3, §3.1's `load_dump` and `dump_state`, §3.3's `--dump-every` and `--load-dump`;
//! M0-T26) — tests and headless runs only, never a save format (§2.13).
//!
//! A dump is the eight bytes `SRDUMP01`, a little-endian u32 header length, a UTF-8 JSON header, then §2.2's fourteen
//! channels, each a little-endian f32 plane in §2.2's order, row-major — the state's canonical form ([`crate::state`]).
//! The header: `version` (1), `width`, `height`, `step` (the steps run since the scene was written — the next step's
//! index), `sim_time`, `channels` (§2.2's names, in order), `sinks` (`x`, `y`, `vx`, `vy`, `m`; P3's sinks, none when the step runs no P3, M0-T52b),
//! and `resume` (contract §0.5 [M0-T26]): the step's own records, which no cell holds and a resumed run needs to go on
//! bit for bit (§1.3.4) — Δt_n and Δt_{n+1} (`dt`, `dt_next`), the latch's sim time since its last set (`latch_since`,
//! its arming, §1.8.5), the active box's record (`box`: x0, y0, w, h, fft_w, fft_h, from_step, fits) and its indirect
//! sizes (`box_args`, §1.3.3), whether F follows the channels (`flux`, §0.5 [M0-D33]) and whether P5's rate follows F
//! (`rate`, §0.5 [M0-D85]; a header without the key reads false). f32 values are written as JSON numbers, which carry them
//! exactly.
//!
//! When the run runs P5, F's two f32 planes (`side.flux`: F_x, then F_y, row-major) follow the fourteen channels and
//! `resume.flux` is true (contract §0.5 [M0-D33]): the next step's P4 pushes the gas with them (§2.4.4), and P5 took them
//! from the temperatures after P4, not from the dumped cells, so no resume can rebuild them. After F, P5's rate in the
//! step (one f32 plane, (T − T⁰)/Δt a cell) and `resume.rate` true (contract §0.5 [M0-D85]): the next step's first Newton
//! iterate is built from it; a dump without it resumes with the rate zero.
//!
//! [`Dump::from_bytes`] refuses, naming the cause: a wrong magic, a file cut anywhere, bytes past the last plane, a
//! header that is not UTF-8 JSON, an unknown or missing key, another version, a world §2.8 refuses, a channel list other
//! than §2.2's, more than 16 sinks, a `flux` or `rate` flag the planes disagree with, a rate without F.

use std::path::Path;

use serde_json::{json, Map, Value};

use crate::state::{read_buffers, SideFields, State, WorldConfig, CHANNELS, N_CHANNELS};
use crate::step::Step;

pub const MAGIC: &[u8; 8] = b"SRDUMP01";
pub const VERSION: u64 = 1;
/// §2.6: up to 16 sinks.
pub const MAX_SINKS: usize = 16;
/// F's planes after the channels when the run runs P5 (§0.5 [M0-D33]).
const FLUX_PLANES: usize = SideFields::FLUX_PLANES;

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
    /// F's two planes (F_x, then F_y) when the run runs P5 (§0.5 [M0-D33]); the header's `resume.flux` says which.
    pub flux: Option<Vec<f32>>,
    /// P5's rate in the step, one plane after F (§0.5 [M0-D85]); the header's `resume.rate` says which.
    pub rate: Option<Vec<f32>>,
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
                "flux": self.flux.is_some(),
                "rate": self.rate.is_some(),
            },
        })
    }

    /// The file's bytes (§2.12.3).
    pub fn to_bytes(&self) -> Vec<u8> {
        let header = self.header().to_string();
        let cells = self.world.cells();
        assert_eq!(self.planes.len(), N_CHANNELS * cells, "a dump holds 14 planes of its world");
        let flux = self.flux.as_deref().unwrap_or_default();
        assert!(self.flux.is_none() || flux.len() == FLUX_PLANES * cells, "F is 2 planes of its world");
        let rate = self.rate.as_deref().unwrap_or_default();
        assert!(self.rate.is_none() || (self.flux.is_some() && rate.len() == cells), "the rate is 1 plane of its world, after F");
        let mut out = Vec::with_capacity(12 + header.len() + 4 * (self.planes.len() + flux.len() + rate.len()));
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&(header.len() as u32).to_le_bytes());
        out.extend_from_slice(header.as_bytes());
        for c in 0..N_CHANNELS {
            for v in &self.planes[c * cells..(c + 1) * cells] {
                out.extend_from_slice(&v.to_le_bytes());
            }
        }
        for v in flux.iter().chain(rate) {
            out.extend_from_slice(&v.to_le_bytes());
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
        let (mut dump, flux, rate) = parse_header(&header)?;
        dump.sim_time = exact_number(text, "sim_time").unwrap_or(dump.sim_time);
        dump.sinks = exact_sinks(text).unwrap_or(dump.sinks);

        // §2.2's 14 planes, then F's 2 and the rate's 1 when the flags say so: a body the other flag would fit is the flag's
        // refusal.
        let body = &bytes[12 + len..];
        let plane = dump.world.cells() * 4;
        let channels = N_CHANNELS * plane;
        let with_flux = channels + usize::from(flux) * FLUX_PLANES * plane;
        let want = with_flux + usize::from(rate) * plane;
        if flux && body.len() == channels {
            return Err("header.resume.flux: true, but no F planes follow §2.2's 14".to_string());
        }
        if !flux && body.len() == channels + FLUX_PLANES * plane {
            return Err("header.resume.flux: false, but F's 2 planes follow §2.2's 14".to_string());
        }
        if rate && body.len() == with_flux {
            return Err("header.resume.rate: true, but no rate plane follows F's 2".to_string());
        }
        if !rate && body.len() == with_flux + plane {
            return Err("header.resume.rate: false, but a plane follows F's 2".to_string());
        }
        if body.len() < want {
            return Err(format!("truncated in the planes: {} of {want} bytes", body.len()));
        }
        if body.len() > want {
            return Err(format!("{} bytes past the last plane", body.len() - want));
        }
        let mut values: Vec<f32> = body.as_chunks::<4>().0.iter().map(|b| f32::from_le_bytes(*b)).collect();
        dump.rate = rate.then(|| values.split_off(with_flux / 4));
        dump.flux = flux.then(|| values.split_off(channels / 4));
        dump.planes = values;
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

/// The header's sinks, each of its five numbers read as [`exact_number`] reads sim_time: a sink's mass is an f32 pair's
/// sum (M0-T51), so its 17-digit f64 must come back bit for bit. The header holds no list inside a sink, so the list ends
/// at the first `]`.
fn exact_sinks(text: &str) -> Option<Vec<Sink>> {
    let from = &text[text.find("\"sinks\"")?..];
    let list = &from[from.find('[')? + 1..from.find(']')?];
    list.split('}')
        .filter(|object| object.contains('{'))
        .map(|o| {
            Some(Sink {
                x: exact_number(o, "x")?,
                y: exact_number(o, "y")?,
                vx: exact_number(o, "vx")?,
                vy: exact_number(o, "vy")?,
                m: exact_number(o, "m")?,
            })
        })
        .collect()
}

/// The header's object at `at`, holding exactly `keys`.
fn object<'a>(v: &'a Value, at: &str, keys: &[&str]) -> Result<&'a Map<String, Value>, String> {
    object_with(v, at, keys, &[])
}

/// The header's object at `at`, holding `keys` and any of `optional`, nothing else.
fn object_with<'a>(v: &'a Value, at: &str, keys: &[&str], optional: &[&str]) -> Result<&'a Map<String, Value>, String> {
    let m = v.as_object().ok_or_else(|| format!("{at}: not an object"))?;
    if let Some(k) = m.keys().find(|k| !keys.contains(&k.as_str()) && !optional.contains(&k.as_str())) {
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

/// The header's values and its `resume.flux` and `resume.rate` flags; the planes are left empty for the caller.
fn parse_header(v: &Value) -> Result<(Dump, bool, bool), String> {
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
    let r = object_with(&m["resume"], at_r, &["dt", "dt_next", "latch_since", "box", "box_args", "flux"], &["rate"])?;
    let flux = r["flux"].as_bool().ok_or_else(|| format!("{at_r}.flux: not true or false: {}", r["flux"]))?;
    let rate = match r.get("rate") {
        None => false,
        Some(v) => v.as_bool().ok_or_else(|| format!("{at_r}.rate: not true or false: {v}"))?,
    };
    if rate && !flux {
        return Err(format!("{at_r}.rate: true without F (flux false): the rate is P5's"));
    }
    let f32_of = |k: &str| number(r, at_r, k).map(|v| v as f32);
    let resume = Resume {
        dt: f32_of("dt")?,
        dt_next: f32_of("dt_next")?,
        latch_since: f32_of("latch_since")?,
        box_record: words(r, at_r, "box")?,
        box_args: words(r, at_r, "box_args")?,
    };
    let dump = Dump {
        world,
        step: whole(m, at, "step")?,
        sim_time: number(m, at, "sim_time")?,
        sinks,
        resume,
        planes: Vec::new(),
        flux: None,
        rate: None,
    };
    Ok((dump, flux, rate))
}

/// §3.1 `dump_state()`: the state, the step index, the frames' sim time, the step's records and, when the step runs P5,
/// F (§0.5 [M0-D33]) and P5's rate (§0.5 [M0-D85]), read back after every submitted step. Blocks on the device (tests and headless runs only, §6.3).
pub fn dump_state(device: &wgpu::Device, queue: &wgpu::Queue, state: &State, step: &Step) -> Dump {
    let (index, sim_time, resume) = step.resume_records(device, queue);
    let flux = step.side_flux().map(|f| read_buffers(device, queue, "side.flux.readback", std::iter::once(f)));
    let rate = step.heat_rate(device, queue);
    let sinks = step.read_sinks(device, queue).map(|r| r.sinks).unwrap_or_default();
    Dump { world: state.world(), step: index, sim_time, sinks, resume, planes: state.readback(device, queue), flux, rate }
}

/// §3.1 `load_dump(dump)`: writes the dump's cells into `state` and its records into `step`, whose next step is then
/// the dump's `step`; its sinks, when the step runs P3, go into P3's state (M0-T52b); its F, when it holds one, goes into
/// the step's `side.flux` (§0.5 [M0-D33]); its P5 rate, or zeros when it holds none, into the step's (§0.5 [M0-D85]). Call
/// it before `step` records anything. Refused: another world than the state's, a step index past the GPU's u32 count,
/// sinks into a step that runs no P3, F without P5 to read it, P5 without F (its first step would push with F = 0), a rate
/// of another length than the world's cells.
pub fn load_dump(queue: &wgpu::Queue, state: &State, step: &Step, dump: &Dump) -> Result<(), String> {
    if dump.world != state.world() {
        let (d, s) = (dump.world, state.world());
        return Err(format!("dump world {} × {}, the state's {} × {}", d.width, d.height, s.width, s.height));
    }
    if dump.step > u64::from(u32::MAX) {
        return Err(format!("dump step {}: past the GPU's u32 step count", dump.step));
    }
    if !dump.sinks.is_empty() && !step.runs_sinks() {
        return Err(format!("dump holds {} sink(s), and this step runs no P3 to keep them", dump.sinks.len()));
    }
    if step.runs_sinks() {
        step.set_sinks(queue, &dump.sinks)?;
    }
    let flux = match (&dump.flux, step.side_flux()) {
        (Some(f), Some(side)) if f.len() == FLUX_PLANES * dump.world.cells() => Some((f, side)),
        (Some(f), Some(_)) => return Err(format!("dump F: {} values, 2 planes of its world hold {}", f.len(), FLUX_PLANES * dump.world.cells())),
        (Some(_), None) => return Err("dump holds F, and this step runs no P5 to keep it (heat off)".to_string()),
        (None, Some(_)) => return Err("dump holds no F, and this step runs P5: its first step would push with F = 0".to_string()),
        (None, None) => None,
    };
    if let Some(r) = dump.rate.as_ref().filter(|r| r.len() != dump.world.cells()) {
        return Err(format!("dump rate: {} values, 1 plane of its world holds {}", r.len(), dump.world.cells()));
    }
    state.upload(queue, &dump.planes).map_err(|e| e.to_string())?;
    if let Some((f, side)) = flux {
        let bytes: Vec<u8> = f.iter().flat_map(|v| v.to_le_bytes()).collect();
        queue.write_buffer(side, 0, &bytes);
    }
    step.set_heat_rate(queue, dump.rate.as_deref());
    step.restore(queue, dump.step, dump.sim_time, &dump.resume);
    Ok(())
}
