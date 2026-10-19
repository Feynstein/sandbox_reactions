//! The calibration file and its physics hash (m0_contrat.md §2.11, §3.1, §3.3, §5.4's G-CAL; M0-T65).
//!
//! `assets/calibration.json` holds what the physics measured about itself (the ceilings, the thresholds, the presets'
//! masses, the top rung). It is true only of the constants it was measured on, so it carries `physics_hash`: the
//! SHA-256 of the canonical JSON of `elements.json`, `reactions.json` and `physics.json`. The engine refuses a
//! calibration whose hash differs from its embedded assets — `SR-ERROR calibration out of date`, exit 6: change a
//! constant, calibrate again.
//!
//! **The hash.** Each file is parsed and written back canonically — keys sorted, no whitespace, every number as the
//! shortest f64 text (so `2`, `2.0` and `2e0` are one value) — and the three go into one object
//! `{"elements":…,"physics":…,"reactions":…}` (the framing is this module's choice: the contract names the three
//! files and not how they join), whose canonical text is hashed. A reformatted or re-ordered file keeps its hash; a
//! changed value, key or record does not. SHA-256 is written here against FIPS 180-4's test vectors: §1.2's table
//! holds no hash crate. Nothing under `sr_engine::step` reads calibration (G-WATCH); only the observers, the time
//! control and the app do, so a run whose scene names no preset needs none (and `calibrate` itself must run on a
//! stale one).

use std::collections::BTreeMap;
use std::fmt;

use serde_json::{json, Map, Value};

use crate::registry::Physics;

const ELEMENTS_JSON: &str = include_str!("../../../assets/elements.json");
const REACTIONS_JSON: &str = include_str!("../../../assets/reactions.json");
const PHYSICS_JSON: &str = include_str!("../../../assets/physics.json");

/// §3.3's exit code for a calibration that failed: out of date, invalid, or missing where a preset needs it.
pub const EXIT_CALIBRATION: i32 = 6;
/// §2.11's schema version.
pub const VERSION: u64 = 1;

/// The calibration compiled into the binary. `None` until M0-V9's run writes `assets/calibration.json`; the line
/// becomes `Some(include_str!("../../../assets/calibration.json"))` in that task, and every engine start then checks it
/// against the assets it embeds ([`resolve`]).
pub const EMBEDDED: Option<&str> = None;

// ---------------------------------------------------------------------------------------------------------------
// SHA-256 (FIPS 180-4 §6.2)

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98,
    0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786,
    0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8,
    0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819,
    0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a,
    0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
    0xc67178f2,
];

/// The SHA-256 digest of `data`.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] =
        [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
    // Padding: a 1 bit, zeros up to 56 mod 64, then the message's length in bits as a big-endian u64.
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&(data.len() as u64).wrapping_mul(8).to_be_bytes());
    for block in msg.as_chunks::<64>().0 {
        let mut w = [0u32; 64];
        for (i, word) in block.as_chunks::<4>().0.iter().enumerate() {
            w[i] = u32::from_be_bytes(*word);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (slot, x) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *slot = slot.wrapping_add(x);
        }
    }
    let mut out = [0u8; 32];
    for (bytes, word) in out.as_chunks_mut::<4>().0.iter_mut().zip(h) {
        *bytes = word.to_be_bytes();
    }
    out
}

/// The digest as 64 lowercase hex digits.
pub fn sha256_hex(data: &[u8]) -> String {
    sha256(data).iter().map(|b| format!("{b:02x}")).collect()
}

// ---------------------------------------------------------------------------------------------------------------
// The physics hash

/// `value` as canonical JSON: object keys sorted, no whitespace, arrays in order, strings as serde_json writes them,
/// every number as the shortest text of its f64 (`-0` as `0.0`).
pub fn canonical_json(value: &Value) -> String {
    let mut out = String::new();
    write_canonical(value, &mut out);
    out
}

fn write_canonical(value: &Value, out: &mut String) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => {
            let x = n.as_f64().unwrap_or(f64::NAN);
            out.push_str(&format!("{:?}", if x == 0.0 { 0.0 } else { x }));
        }
        Value::String(s) => out.push_str(&Value::String(s.clone()).to_string()),
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_canonical(item, out);
            }
            out.push(']');
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            out.push('{');
            for (i, key) in keys.into_iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&Value::String(key.clone()).to_string());
                out.push(':');
                write_canonical(&map[key], out);
            }
            out.push('}');
        }
    }
}

/// The physics hash of three files' texts: the SHA-256 of the canonical text of `{"elements","physics","reactions"}`.
pub fn physics_hash_of(elements: &str, reactions: &str, physics: &str) -> Result<String, CalibrationError> {
    let parse = |file: &str, text: &str| {
        serde_json::from_str::<Value>(text).map_err(|e| CalibrationError::invalid(file, format!("not JSON: {e}")))
    };
    let all = json!({
        "elements": parse("elements.json", elements)?,
        "reactions": parse("reactions.json", reactions)?,
        "physics": parse("physics.json", physics)?,
    });
    Ok(sha256_hex(canonical_json(&all).as_bytes()))
}

/// The physics hash of the assets this build embeds.
pub fn physics_hash() -> Result<String, CalibrationError> {
    physics_hash_of(ELEMENTS_JSON, REACTIONS_JSON, PHYSICS_JSON)
}

// ---------------------------------------------------------------------------------------------------------------
// Errors

/// Why a calibration is refused. Every one is exit [`EXIT_CALIBRATION`]; [`CalibrationError::line`] is its stdout line.
#[derive(Debug, Clone, PartialEq)]
pub enum CalibrationError {
    /// The file's `physics_hash` is not the embedded assets' (G-CAL).
    OutOfDate { file: String, assets: String },
    /// The file is not §2.11's, or breaks one of its bounds; names the key.
    Invalid { key: String, message: String },
    /// A run that needs a calibration (it names a preset) has none.
    Missing,
}

impl CalibrationError {
    fn invalid(key: &str, message: impl Into<String>) -> CalibrationError {
        CalibrationError::Invalid { key: key.to_string(), message: message.into() }
    }

    pub fn exit_code(&self) -> i32 {
        EXIT_CALIBRATION
    }

    /// The `SR-ERROR` line (§3.3). A stale hash starts with §5.4's exact words.
    pub fn line(&self) -> String {
        match self {
            CalibrationError::OutOfDate { file, assets } => format!(
                "SR-ERROR calibration out of date: its physics_hash is {file}, the assets' is {assets} \
                 (change a constant, calibrate again: sandbox-reactions calibrate)"
            ),
            CalibrationError::Invalid { key, message } => format!("SR-ERROR calibration invalid: {key}: {message}"),
            CalibrationError::Missing => "SR-ERROR calibration missing: a preset needs assets/calibration.json \
                 (sandbox-reactions calibrate writes it)"
                .to_string(),
        }
    }
}

impl fmt::Display for CalibrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.line().trim_start_matches("SR-ERROR "))
    }
}

impl std::error::Error for CalibrationError {}

// ---------------------------------------------------------------------------------------------------------------
// The file (§2.11)

#[derive(Debug, Clone, PartialEq)]
pub struct Measured {
    pub date: String,
    pub git: String,
    pub adapter: String,
    /// The `box` key: the machine's hostname.
    pub machine: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PresetMasses {
    pub sun: f64,
    pub massive: f64,
    pub giant: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Calibration {
    pub physics_hash: String,
    pub measured: Measured,
    pub m_ch_sb: f64,
    pub m_tov_sb: f64,
    pub ns_sigma_max_sb: f64,
    pub m_ign_sb: f64,
    pub m_up_sb: f64,
    pub m_bh_sb: f64,
    pub preset_mass_sb: PresetMasses,
    pub t_eff_sun_sb: f64,
    pub life_sun_sb: f64,
    /// A speed (§1.8.2): the numbered rungs below it, then Max.
    pub top_rung: f64,
    /// Stage id → (mass_sb, sim time) pairs.
    pub stage_durations_sb: BTreeMap<String, Vec<(f64, f64)>>,
    pub render_reserve_ms: f64,
    pub max_gas_speed: f64,
}

const KEYS: [&str; 16] = [
    "version",
    "physics_hash",
    "measured",
    "m_ch_sb",
    "m_tov_sb",
    "ns_sigma_max_sb",
    "m_ign_sb",
    "m_up_sb",
    "m_bh_sb",
    "preset_mass_sb",
    "t_eff_sun_sb",
    "life_sun_sb",
    "top_rung",
    "stage_durations_sb",
    "render_reserve_ms",
    "max_gas_speed",
];

/// One JSON object being read: its path for messages, its keys checked against the ones it may hold.
struct Fields<'a> {
    map: &'a Map<String, Value>,
    at: &'a str,
}

impl<'a> Fields<'a> {
    fn new(value: &'a Value, at: &'a str, known: &[&str]) -> Result<Fields<'a>, CalibrationError> {
        let Some(map) = value.as_object() else {
            return Err(CalibrationError::invalid(at, "must be an object"));
        };
        if let Some(k) = map.keys().find(|k| !known.contains(&k.as_str())) {
            return Err(CalibrationError::invalid(&join(at, k), "unknown key"));
        }
        Ok(Fields { map, at })
    }

    fn get(&self, key: &str) -> Result<&'a Value, CalibrationError> {
        self.map.get(key).ok_or_else(|| CalibrationError::invalid(&join(self.at, key), "missing"))
    }

    /// A finite number, > 0, or ≥ 0 where `zero_ok`.
    fn number(&self, key: &str, zero_ok: bool) -> Result<f64, CalibrationError> {
        let path = join(self.at, key);
        let Some(x) = self.get(key)?.as_f64().filter(|x| x.is_finite()) else {
            return Err(CalibrationError::invalid(&path, "must be a finite number"));
        };
        match (zero_ok, x) {
            (false, x) if x <= 0.0 => Err(CalibrationError::invalid(&path, format!("must be > 0, got {x}"))),
            (true, x) if x < 0.0 => Err(CalibrationError::invalid(&path, format!("must be ≥ 0, got {x}"))),
            _ => Ok(x),
        }
    }

    fn text(&self, key: &str) -> Result<String, CalibrationError> {
        match self.get(key)?.as_str() {
            Some(s) if !s.is_empty() => Ok(s.to_string()),
            _ => Err(CalibrationError::invalid(&join(self.at, key), "must be a non-empty string")),
        }
    }
}

fn join(at: &str, key: &str) -> String {
    if at.is_empty() {
        key.to_string()
    } else {
        format!("{at}.{key}")
    }
}

impl Calibration {
    /// §2.11's file, parsed and validated: every key present and no other, each number finite and > 0 (the render
    /// reserve ≥ 0), the hash 64 lowercase hex digits, and the order the procedure's item 5 demands (K9).
    pub fn parse(text: &str) -> Result<Calibration, CalibrationError> {
        let root: Value = serde_json::from_str(text).map_err(|e| CalibrationError::invalid("<json>", e.to_string()))?;
        let top = Fields::new(&root, "", &KEYS)?;
        match top.get("version")?.as_u64() {
            Some(VERSION) => {}
            _ => return Err(CalibrationError::invalid("version", format!("must be {VERSION}"))),
        }
        let physics_hash = top.text("physics_hash")?;
        if physics_hash.len() != 64 || !physics_hash.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
            return Err(CalibrationError::invalid("physics_hash", "must be 64 lowercase hex digits"));
        }
        let m = Fields::new(top.get("measured")?, "measured", &["date", "git", "adapter", "box"])?;
        let measured = Measured { date: m.text("date")?, git: m.text("git")?, adapter: m.text("adapter")?, machine: m.text("box")? };
        let p = Fields::new(top.get("preset_mass_sb")?, "preset_mass_sb", &["sun", "massive", "giant"])?;
        let preset_mass_sb =
            PresetMasses { sun: p.number("sun", false)?, massive: p.number("massive", false)?, giant: p.number("giant", false)? };

        let stages = top.get("stage_durations_sb")?;
        let Some(stages) = stages.as_object() else {
            return Err(CalibrationError::invalid("stage_durations_sb", "must be an object of stage id → [[mass_sb, t.u.], …]"));
        };
        let mut stage_durations_sb = BTreeMap::new();
        for (id, list) in stages {
            let path = format!("stage_durations_sb.{id}");
            let pairs = list.as_array().filter(|l| !l.is_empty());
            let Some(pairs) = pairs else {
                return Err(CalibrationError::invalid(&path, "must be a non-empty list of [mass_sb, t.u.] pairs"));
            };
            let mut out = Vec::with_capacity(pairs.len());
            for pair in pairs {
                let nums = pair.as_array().filter(|p| p.len() == 2).map(|p| (p[0].as_f64(), p[1].as_f64()));
                match nums {
                    Some((Some(a), Some(b))) if a.is_finite() && b.is_finite() && a > 0.0 && b > 0.0 => out.push((a, b)),
                    _ => return Err(CalibrationError::invalid(&path, "each pair must be two finite numbers > 0")),
                }
            }
            stage_durations_sb.insert(id.clone(), out);
        }

        let cal = Calibration {
            physics_hash,
            measured,
            m_ch_sb: top.number("m_ch_sb", false)?,
            m_tov_sb: top.number("m_tov_sb", false)?,
            ns_sigma_max_sb: top.number("ns_sigma_max_sb", false)?,
            m_ign_sb: top.number("m_ign_sb", false)?,
            m_up_sb: top.number("m_up_sb", false)?,
            m_bh_sb: top.number("m_bh_sb", false)?,
            preset_mass_sb,
            t_eff_sun_sb: top.number("t_eff_sun_sb", false)?,
            life_sun_sb: top.number("life_sun_sb", false)?,
            top_rung: top.number("top_rung", false)?,
            stage_durations_sb,
            render_reserve_ms: top.number("render_reserve_ms", true)?,
            max_gas_speed: top.number("max_gas_speed", false)?,
        };
        cal.check_order()?;
        Ok(cal)
    }

    /// Item 5 (K9): m_ign < m_ch < m_up < m_bh, m_ch < m_tov and m_tov/m_ch in [1.4, 2.0]; the presets read
    /// sun < massive < giant (the mass translation increases strictly, §1.10.1).
    fn check_order(&self) -> Result<(), CalibrationError> {
        let rule = |ok: bool, key: &str, message: String| if ok { Ok(()) } else { Err(CalibrationError::invalid(key, message)) };
        rule(self.m_ign_sb < self.m_ch_sb, "m_ign_sb", format!("must be < m_ch_sb = {} (K9), got {}", self.m_ch_sb, self.m_ign_sb))?;
        rule(self.m_ch_sb < self.m_up_sb, "m_up_sb", format!("must be > m_ch_sb = {} (K9), got {}", self.m_ch_sb, self.m_up_sb))?;
        rule(self.m_up_sb < self.m_bh_sb, "m_bh_sb", format!("must be > m_up_sb = {} (K9), got {}", self.m_up_sb, self.m_bh_sb))?;
        rule(self.m_ch_sb < self.m_tov_sb, "m_tov_sb", format!("must be > m_ch_sb = {} (K9), got {}", self.m_ch_sb, self.m_tov_sb))?;
        let ratio = self.m_tov_sb / self.m_ch_sb;
        rule((1.4..=2.0).contains(&ratio), "m_tov_sb", format!("m_tov_sb / m_ch_sb must be in [1.4, 2.0] (K9), got {ratio}"))?;
        let p = &self.preset_mass_sb;
        rule(p.sun < p.massive, "preset_mass_sb.massive", format!("must be > sun = {}, got {}", p.sun, p.massive))?;
        rule(p.massive < p.giant, "preset_mass_sb.giant", format!("must be > massive = {}, got {}", p.massive, p.giant))
    }

    /// G-CAL, and item 6's speed bound: the hash equals the assets' and `c_sb` exceeds twice `max_gas_speed`.
    pub fn check_against(&self, assets_hash: &str, c_sb: f64) -> Result<(), CalibrationError> {
        if self.physics_hash != assets_hash {
            return Err(CalibrationError::OutOfDate { file: self.physics_hash.clone(), assets: assets_hash.to_string() });
        }
        if c_sb <= 2.0 * self.max_gas_speed {
            return Err(CalibrationError::invalid(
                "max_gas_speed",
                format!("c_sb = {c_sb} must exceed twice it, {} (§2.11 item 6)", 2.0 * self.max_gas_speed),
            ));
        }
        Ok(())
    }

    /// The file's JSON, keys sorted, with a trailing newline — what `calibrate` writes (M0-T66).
    pub fn to_json(&self) -> String {
        let stages: Map<String, Value> = self
            .stage_durations_sb
            .iter()
            .map(|(id, pairs)| (id.clone(), Value::Array(pairs.iter().map(|&(m, t)| json!([m, t])).collect())))
            .collect();
        let value = json!({
            "version": VERSION,
            "physics_hash": self.physics_hash,
            "measured": {
                "date": self.measured.date, "git": self.measured.git,
                "adapter": self.measured.adapter, "box": self.measured.machine,
            },
            "m_ch_sb": self.m_ch_sb,
            "m_tov_sb": self.m_tov_sb,
            "ns_sigma_max_sb": self.ns_sigma_max_sb,
            "m_ign_sb": self.m_ign_sb,
            "m_up_sb": self.m_up_sb,
            "m_bh_sb": self.m_bh_sb,
            "preset_mass_sb": {
                "sun": self.preset_mass_sb.sun, "massive": self.preset_mass_sb.massive, "giant": self.preset_mass_sb.giant,
            },
            "t_eff_sun_sb": self.t_eff_sun_sb,
            "life_sun_sb": self.life_sun_sb,
            "top_rung": self.top_rung,
            "stage_durations_sb": Value::Object(stages),
            "render_reserve_ms": self.render_reserve_ms,
            "max_gas_speed": self.max_gas_speed,
        });
        let mut text = serde_json::to_string_pretty(&value).expect("a Value always serialises");
        text.push('\n');
        text
    }

    /// A valid calibration for a physics hash, with round numbers near §2.7's arithmetic — the "test calibration" that
    /// scopes use before M0-V9 measures the real one (M0-T69). Never a measurement: `measured.git` reads `test`.
    pub fn test_calibration(physics_hash: &str) -> Calibration {
        Calibration {
            physics_hash: physics_hash.to_string(),
            measured: Measured { date: "1970-01-01T00:00:00Z".into(), git: "test".into(), adapter: "test".into(), machine: "test".into() },
            m_ch_sb: 3970.0,
            m_tov_sb: 6000.0,
            ns_sigma_max_sb: 3200.0,
            m_ign_sb: 270.0,
            m_up_sb: 26800.0,
            m_bh_sb: 83800.0,
            preset_mass_sb: PresetMasses { sun: 3351.0, massive: 50265.0, giant: 134040.0 },
            t_eff_sun_sb: 0.3,
            life_sun_sb: 1200.0,
            top_rung: 1000.0,
            stage_durations_sb: BTreeMap::from([("main_sequence".to_string(), vec![(3351.0, 900.0)])]),
            render_reserve_ms: 3.0,
            max_gas_speed: 20.0,
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------
// The engine's gate (§3.1's `Engine::new` errors)

/// The calibration a run starts with, given the file's text (`None`: there is none) and whether the run needs one —
/// it names a preset, whose mass, the time control's top rung and the stage durations come from calibration.
///
/// A run that needs none (headless, test and `calibrate` runs whose scene names no preset: the step never reads
/// calibration, G-WATCH) gets `Ok(None)` whatever the file says — `calibrate` must run on a stale one. A run that needs
/// one gets it parsed, validated and checked against the embedded assets ([`Calibration::check_against`]), or the
/// [`CalibrationError`] that is exit 6.
pub fn resolve(text: Option<&str>, needs_calibration: bool) -> Result<Option<Calibration>, CalibrationError> {
    if !needs_calibration {
        return Ok(None);
    }
    let physics = Physics::shipped().map_err(|e| CalibrationError::invalid("physics.json", e.to_string()))?;
    let c_sb = physics.get("c_sb").ok_or_else(|| CalibrationError::invalid("physics.json", "c_sb is missing"))?;
    resolve_against(text, true, &physics_hash()?, c_sb)
}

/// [`resolve`] against a given hash and c_sb — the assets' identity, passed in so a test can stand in for the assets.
pub fn resolve_against(
    text: Option<&str>,
    needs_calibration: bool,
    assets_hash: &str,
    c_sb: f64,
) -> Result<Option<Calibration>, CalibrationError> {
    if !needs_calibration {
        return Ok(None);
    }
    let cal = Calibration::parse(text.ok_or(CalibrationError::Missing)?)?;
    cal.check_against(assets_hash, c_sb)?;
    Ok(Some(cal))
}
