//! The element registry (`assets/elements.json`, m0_contrat.md §1.5) and the physics constants
//! (`assets/physics.json`, §2.5.1 and §2.7), both embedded, parsed and validated. A refusal names the key it is about
//! (`RegistryError::key`), so a bad scene override or a bad edit is told where it went wrong.
//!
//! physics.json is one flat object of numbers; its keys are the ones a scene's `overrides` name (§2.12.1). The bounds
//! are the contract's own: nothing here is loosened to make a value pass (§0.4).

use std::collections::BTreeMap;
use std::fmt;

use serde_json::{Map, Value};

const ELEMENTS_JSON: &str = include_str!("../../../assets/elements.json");
const PHYSICS_JSON: &str = include_str!("../../../assets/physics.json");

/// M0's species count: the state's mass-fraction channels are this many (sr-engine `state::SPECIES`).
pub const N_SPECIES: usize = 10;
const ID_H: usize = 0;
const ID_HE: usize = 1;
const ID_N: usize = 9;

/// A refusal: the file, the key it is about and why.
#[derive(Debug, Clone, PartialEq)]
pub struct RegistryError {
    pub file: &'static str,
    pub key: String,
    pub message: String,
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}: {}", self.file, self.key, self.message)
    }
}

impl std::error::Error for RegistryError {}

fn refuse<T>(file: &'static str, key: &str, message: String) -> Result<T, RegistryError> {
    Err(RegistryError { file, key: key.to_string(), message })
}

// ---------------------------------------------------------------------------------------------------------------
// elements.json (§1.5)

#[derive(Debug, Clone, PartialEq)]
pub struct Species {
    pub key: String,
    pub name: String,
    pub symbol: String,
    pub a: u32,
    pub z: u32,
    pub paintable: bool,
    pub color: [u8; 3],
}

/// §1.5's table: key, A, Z, paintable. The list order is the species id.
const CORE: [(&str, u32, u32, bool); N_SPECIES] = [
    ("H", 1, 1, true),
    ("He", 4, 2, true),
    ("C", 12, 6, true),
    ("O", 16, 8, true),
    ("Ne", 20, 10, true),
    ("Mg", 24, 12, true),
    ("Si", 28, 14, true),
    ("S", 32, 16, true),
    ("Fe", 56, 26, true),
    ("n", 1, 0, false),
];

const SPECIES_FIELDS: [&str; 7] = ["key", "name", "symbol", "a", "z", "paintable", "color"];

/// Per-composition values derived from mass fractions (§1.5; gas fully ionized at every temperature).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Composition {
    /// 1/μ = Σ X_i (Z_i + 1)/A_i
    pub inv_mu: f64,
    /// electrons per nucleon, Y_e = Σ X_i Z_i/A_i
    pub y_e: f64,
    /// metals, Z_met = 1 − X_H − X_He − X_n
    pub z_met: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Elements {
    species: Vec<Species>,
}

fn element_err<T>(key: String, message: &str) -> Result<T, RegistryError> {
    refuse("elements.json", &key, message.to_string())
}

fn parse_color(s: &str) -> Option<[u8; 3]> {
    let hex = s.strip_prefix('#')?;
    if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let channel = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    Some([channel(0)?, channel(2)?, channel(4)?])
}

impl Elements {
    /// The registry shipped in `assets/elements.json`.
    pub fn shipped() -> Result<Elements, RegistryError> {
        Elements::parse(ELEMENTS_JSON)
    }

    pub fn parse(text: &str) -> Result<Elements, RegistryError> {
        let root: Value = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(e) => return element_err("<json>".into(), &e.to_string()),
        };
        let Some(root) = root.as_object() else {
            return element_err("<root>".into(), "must be an object {\"version\", \"species\"}");
        };
        if let Some(k) = root.keys().find(|k| *k != "version" && *k != "species") {
            return element_err(k.clone(), "unknown key");
        }
        if root.get("version") != Some(&Value::from(1)) {
            return element_err("version".into(), "must be 1");
        }
        let Some(list) = root.get("species").and_then(Value::as_array) else {
            return element_err("species".into(), "missing, or not a list");
        };
        if list.len() != N_SPECIES {
            return element_err("species".into(), &format!("M0 has {N_SPECIES} species (§1.5), got {}", list.len()));
        }
        let species = list
            .iter()
            .enumerate()
            .map(|(id, v)| parse_species(id, v))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Elements { species })
    }

    /// The species in id order (the element menu's order).
    pub fn species(&self) -> &[Species] {
        &self.species
    }

    /// Species id of a key (`"He"` → 1).
    pub fn index_of(&self, key: &str) -> Option<usize> {
        self.species.iter().position(|s| s.key == key)
    }

    /// 1/μ, Y_e and Z_met of mass fractions `x` (in species-id order, summing to 1).
    pub fn composition(&self, x: &[f64; N_SPECIES]) -> Composition {
        let (mut inv_mu, mut y_e) = (0.0, 0.0);
        for (s, &xi) in self.species.iter().zip(x) {
            let (a, z) = (f64::from(s.a), f64::from(s.z));
            inv_mu += xi * (z + 1.0) / a;
            y_e += xi * z / a;
        }
        Composition { inv_mu, y_e, z_met: 1.0 - x[ID_H] - x[ID_HE] - x[ID_N] }
    }
}

fn parse_species(id: usize, v: &Value) -> Result<Species, RegistryError> {
    let at = |field: &str| format!("species[{id}].{field}");
    let Some(obj) = v.as_object() else {
        return element_err(format!("species[{id}]"), "must be an object");
    };
    if let Some(k) = obj.keys().find(|k| !SPECIES_FIELDS.contains(&k.as_str())) {
        return element_err(at(k), "unknown field");
    }
    let text = |field: &str| match obj.get(field).and_then(Value::as_str) {
        Some(s) if !s.is_empty() => Ok(s.to_string()),
        _ => element_err(at(field), "must be a non-empty string"),
    };
    let int = |field: &str| match obj.get(field).and_then(Value::as_u64).and_then(|n| u32::try_from(n).ok()) {
        Some(n) => Ok(n),
        None => element_err(at(field), "must be a whole number ≥ 0"),
    };
    let key = text("key")?;
    let (a, z) = (int("a")?, int("z")?);
    let Some(paintable) = obj.get("paintable").and_then(Value::as_bool) else {
        return element_err(at("paintable"), "must be true or false");
    };
    let color = match parse_color(&text("color")?) {
        Some(c) => c,
        None => return element_err(at("color"), "must be \"#RRGGBB\""),
    };
    let (core_key, core_a, core_z, core_paintable) = CORE[id];
    if key != core_key {
        return element_err(at("key"), &format!("species {id} is \"{core_key}\" (§1.5), got \"{key}\""));
    }
    if a != core_a {
        return element_err(at("a"), &format!("{core_key} has A = {core_a} (§1.5), got {a}"));
    }
    if z != core_z {
        return element_err(at("z"), &format!("{core_key} has Z = {core_z} (§1.5), got {z}"));
    }
    if paintable != core_paintable {
        return element_err(at("paintable"), &format!("{core_key} is paintable = {core_paintable} (§1.5)"));
    }
    Ok(Species { key, name: text("name")?, symbol: text("symbol")?, a, z, paintable, color })
}

// ---------------------------------------------------------------------------------------------------------------
// physics.json (§2.5.1, §2.7)

/// The stand-in laws (§1.6.4, FROZEN): both ship off and only a PLAN amendment citing a measured failure turns one on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Standins {
    /// S1, the dust-driven wind: `kappa_dust` may be above 0.
    pub s1: bool,
    /// S2, the thermal bomb: `f_dep` may rise to 1.0.
    pub s2: bool,
}

impl Standins {
    pub const SHIPPED: Standins = Standins { s1: false, s2: false };
}

/// The bound every key meets on its own; relations between keys are in `validate`.
#[derive(Clone, Copy)]
enum Class {
    /// > 0
    Pos,
    /// ≥ 0 (a switch that may be off)
    NonNeg,
    /// a whole number ≥ 1
    Whole,
    /// ν of a late record, ≥ 4 (K7)
    Nu,
}

/// Every key of physics.json, in file order, with its own bound.
const SCHEMA: [(&str, Class); 52] = [
    ("t_h", Class::Pos),
    ("t_he", Class::Pos),
    ("t_c", Class::Pos),
    ("t_ne", Class::Pos),
    ("t_o", Class::Pos),
    ("t_si", Class::Pos),
    ("q_h", Class::Pos),
    ("a_h_burn", Class::Pos),
    ("a_he_burn", Class::Pos),
    ("a_c_alpha", Class::Pos),
    ("a_c_burn", Class::Pos),
    ("a_ne_burn", Class::Pos),
    ("a_o_burn", Class::Pos),
    ("a_mg_burn", Class::Pos),
    ("a_si_burn", Class::Pos),
    ("a_s_burn", Class::Pos),
    ("a_n_fe", Class::Pos),
    ("nu_c_burn", Class::Nu),
    ("nu_ne_burn", Class::Nu),
    ("nu_o_burn", Class::Nu),
    ("nu_mg_burn", Class::Nu),
    ("nu_si_burn", Class::Nu),
    ("nu_s_burn", Class::Nu),
    ("sigma_n", Class::Pos),
    ("burn_dx", Class::Pos),
    ("burn_dt", Class::Pos),
    ("sigma_floor", Class::Pos),
    ("sigma_vac", Class::Pos),
    ("t_floor", Class::Pos),
    ("sigma_obj", Class::Pos),
    ("paint_sigma", Class::Pos),
    ("paint_t", Class::Pos),
    ("heat_factor", Class::Pos),
    ("cool_factor", Class::Pos),
    ("k1e", Class::NonNeg),
    ("k2e", Class::NonNeg),
    ("k1n", Class::NonNeg),
    ("k2n", Class::NonNeg),
    ("a_r", Class::Pos),
    ("c_sb", Class::Pos),
    ("kappa0", Class::NonNeg),
    ("k_cond", Class::NonNeg),
    ("a_nu", Class::NonNeg),
    ("t_nu", Class::Pos),
    ("m_nu", Class::Pos),
    ("f_dep", Class::NonNeg),
    ("sigma_bh", Class::Pos),
    ("cfl", Class::Pos),
    ("eta_g", Class::Pos),
    ("kappa_dust", Class::NonNeg),
    ("t_dust", Class::NonNeg),
    ("block", Class::Whole),
];

/// The ν keys of the late records (m must stay below each — §2.5.3).
const NU_KEYS: [&str; 6] = ["nu_c_burn", "nu_ne_burn", "nu_o_burn", "nu_mg_burn", "nu_si_burn", "nu_s_burn"];

#[derive(Debug, Clone, PartialEq)]
pub struct Physics {
    values: BTreeMap<&'static str, f64>,
    standins: Standins,
}

fn physics_err<T>(key: &str, message: String) -> Result<T, RegistryError> {
    refuse("physics.json", key, message)
}

impl Physics {
    /// The constants shipped in `assets/physics.json`, the stand-ins off.
    pub fn shipped() -> Result<Physics, RegistryError> {
        Physics::parse(PHYSICS_JSON, Standins::SHIPPED)
    }

    pub fn parse(text: &str, standins: Standins) -> Result<Physics, RegistryError> {
        let root: Value = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(e) => return physics_err("<json>", e.to_string()),
        };
        let Some(obj) = root.as_object() else {
            return physics_err("<root>", "must be one object of numbers".into());
        };
        if let Some(k) = obj.keys().find(|k| !SCHEMA.iter().any(|(s, _)| s == k)) {
            return physics_err(k, "unknown key".into());
        }
        Physics::from_object(obj, standins)
    }

    fn from_object(obj: &Map<String, Value>, standins: Standins) -> Result<Physics, RegistryError> {
        let mut values = BTreeMap::new();
        for (key, _) in SCHEMA {
            match obj.get(key).map(Value::as_f64) {
                Some(Some(x)) => values.insert(key, x),
                Some(None) => return physics_err(key, "must be a number".into()),
                None => return physics_err(key, "missing".into()),
            };
        }
        let physics = Physics { values, standins };
        physics.validate()?;
        Ok(physics)
    }

    /// A key's value; `None` for a name that is not a physics.json key.
    pub fn get(&self, key: &str) -> Option<f64> {
        self.values.get(key).copied()
    }

    /// Every key, in file order.
    pub fn keys() -> impl Iterator<Item = &'static str> {
        SCHEMA.iter().map(|(k, _)| *k)
    }

    /// A copy with `overrides` applied (a scene's `overrides`, §2.12.1), validated as a whole under the same bounds.
    pub fn with(&self, overrides: &[(&str, f64)]) -> Result<Physics, RegistryError> {
        let mut next = self.clone();
        for &(key, x) in overrides {
            match next.values.get_mut(key) {
                Some(slot) => *slot = x,
                None => return physics_err(key, "unknown key".into()),
            }
        }
        next.validate()?;
        Ok(next)
    }

    fn validate(&self) -> Result<(), RegistryError> {
        for (key, class) in SCHEMA {
            let x = self.values[key];
            let bad = match class {
                _ if !x.is_finite() => Some("must be a finite number"),
                Class::Pos if x <= 0.0 => Some("must be > 0"),
                Class::NonNeg if x < 0.0 => Some("must be ≥ 0"),
                Class::Whole if x < 1.0 || x.fract() != 0.0 => Some("must be a whole number ≥ 1"),
                Class::Nu if x < 4.0 => Some("must be ≥ 4 (K7: steep burning)"),
                _ => None,
            };
            if let Some(why) = bad {
                return physics_err(key, format!("{why}, got {x}"));
            }
        }
        let v = |key: &str| self.values[key];

        // The ignition ladder (K6, §1.6.3): T_H < T_He < T_C < T_Ne ≤ T_O < T_Si, T_Si/T_H ≤ 10.
        let (t_h, t_he, t_c, t_ne, t_o, t_si) = (v("t_h"), v("t_he"), v("t_c"), v("t_ne"), v("t_o"), v("t_si"));
        if t_he <= t_h {
            return physics_err("t_he", format!("must be > t_h = {t_h} (K6), got {t_he}"));
        }
        if t_c <= t_he {
            return physics_err("t_c", format!("must be > t_he = {t_he} (K6), got {t_c}"));
        }
        if t_ne <= t_c {
            return physics_err("t_ne", format!("must be > t_c = {t_c} (K6), got {t_ne}"));
        }
        if t_o < t_ne {
            return physics_err("t_o", format!("must be ≥ t_ne = {t_ne} (K6; Ne and O may tie), got {t_o}"));
        }
        if t_si <= t_o {
            return physics_err("t_si", format!("must be > t_o = {t_o} (K6), got {t_si}"));
        }
        if t_si / t_h > 10.0 {
            return physics_err("t_si", format!("t_si / t_h must be ≤ 10 (§1.6.3), got {}", t_si / t_h));
        }

        // The neutrino cooling law stays gentler than every late record (§2.5.3).
        for nu in NU_KEYS {
            if v("m_nu") >= v(nu) {
                return physics_err("m_nu", format!("must be < {nu} = {} (§2.5.3), got {}", v(nu), v("m_nu")));
            }
        }

        // Cold pressure: each pair of K's is on (both > 0) or off (both 0, a test scene's override), never half set.
        let on = |k1: &'static str, k2: &'static str| -> Result<bool, RegistryError> {
            match (v(k1) > 0.0, v(k2) > 0.0) {
                (true, true) => Ok(true),
                (false, false) => Ok(false),
                (false, true) => physics_err(k1, format!("must be > 0 while {k2} is (a pair is on or off)")),
                (true, false) => physics_err(k2, format!("must be > 0 while {k1} is (a pair is on or off)")),
            }
        };
        let electrons = on("k1e", "k2e")?;
        let neutrons = on("k1n", "k2n")?;
        if electrons {
            // Iron neutronizes only once its electrons are relativistic: Σ_N ≥ 2 (K₂ₑ/K₁ₑ)².
            let floor = 2.0 * (v("k2e") / v("k1e")).powi(2);
            if v("sigma_n") < floor {
                return physics_err("sigma_n", format!("must be ≥ 2 (k2e/k1e)² = {floor} (§2.5.1), got {}", v("sigma_n")));
            }
        }
        if electrons && neutrons {
            // M_tov / M_ch = (K₂ₙ² · 1) / (K₂ₑ² · Y_e³) with Y_e = 0.5 (§2.7's own figures) ∈ [1.4, 2.0] (K9).
            let ratio = 8.0 * (v("k2n") / v("k2e")).powi(2);
            if !(1.4..=2.0).contains(&ratio) {
                return physics_err("k2n", format!("m_tov / m_ch must be in [1.4, 2.0] (§2.7, K9), got {ratio}"));
            }
        }

        // The stand-ins ship off (§1.6.4): S2 lifts f_dep's cap from 0.1 to 1.0, S1 lets κ_dust leave 0.
        let cap = if self.standins.s2 { 1.0 } else { 0.1 };
        if v("f_dep") > cap {
            let why = if self.standins.s2 { "S2 is on" } else { "S2 is off" };
            return physics_err("f_dep", format!("must be ≤ {cap} while {why} (§2.5.4, §1.6.4), got {}", v("f_dep")));
        }
        if !self.standins.s1 && v("kappa_dust") != 0.0 {
            return physics_err("kappa_dust", format!("must be 0 while S1 is off (§1.6.4), got {}", v("kappa_dust")));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------------------------
// reactions.json (§1.6)

const REACTIONS_JSON: &str = include_str!("../../../assets/reactions.json");

/// The groups a record books its nuclear energy into (§1.9.1's accumulators).
pub const GROUPS: [&str; 7] = ["H", "He", "C", "Ne", "O", "Si", "N"];
/// Mass conserved per record: input shares and output shares each sum to 1 within this (§1.6.1).
pub const SHARE_TOLERANCE: f64 = 1e-12;
/// The physics.json temperatures a rate term's `t_key` may name (§1.6.3's ladder).
const TEMPERATURE_KEYS: [&str; 6] = ["t_h", "t_he", "t_c", "t_ne", "t_o", "t_si"];
const RECORD_FIELDS: [&str; 8] = ["id", "group", "inputs", "outputs", "q_ratio", "rate", "enabled", "standin"];
const RATE_FIELDS: [&str; 6] = ["a", "orders", "terms", "t_thr_factor", "gate", "a_coef"];
/// K7: every steepness exponent is at least this.
const NU_MIN: f64 = 4.0;

/// One species' share of a record's inputs or outputs.
#[derive(Debug, Clone, PartialEq)]
pub struct Share {
    /// The species id (`Elements::index_of`).
    pub species: usize,
    pub share: f64,
}

/// One term of f(T) = Σ_k w_k (T / T_k)^ν_k, with T_k = t_factor · physics.json's `t_key`.
#[derive(Debug, Clone, PartialEq)]
pub struct RateTerm {
    pub t_key: String,
    pub t_factor: f64,
    /// T_k, resolved against the physics the registry was parsed with.
    pub t_k: f64,
    /// ν_k, resolved (a record may name a physics.json key for it, the tuned ones do).
    pub nu: f64,
    pub w: f64,
}

/// A gated record's g(Σ) = ((Y_e Σ)/Σ_g − 1)^p when Y_e Σ > Σ_g, else 0.
#[derive(Debug, Clone, PartialEq)]
pub struct Gate {
    pub sigma_key: String,
    /// Σ_g, resolved.
    pub sigma_g: f64,
    pub p: f64,
}

/// ω = A · Σ^a · Π X_i^{n_i} · f(T) · g(Σ).
#[derive(Debug, Clone, PartialEq)]
pub struct RateLaw {
    /// The exponent of Σ.
    pub a: f64,
    /// n_i, one per input, in the input list's order.
    pub orders: Vec<f64>,
    /// Empty means f ≡ 1 (no temperature dependence and no threshold).
    pub terms: Vec<RateTerm>,
    pub t_thr_factor: Option<f64>,
    /// T_thr = t_thr_factor · (the first term's T_k): below it f = 0. `None` while there are no terms.
    pub t_thr: Option<f64>,
    pub gate: Option<Gate>,
    /// The physics.json key of A, and A resolved.
    pub a_key: String,
    pub a_coef: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub id: String,
    pub group: String,
    pub inputs: Vec<Share>,
    pub outputs: Vec<Share>,
    pub q_ratio: f64,
    pub rate: RateLaw,
    pub enabled: bool,
    pub standin: bool,
}

impl Record {
    /// The thermal energy this record releases per unit mass converted: q = q_ratio · Q_H (q < 0 absorbs).
    pub fn q(&self, physics: &Physics) -> f64 {
        self.q_ratio * physics.get("q_h").expect("q_h is a physics.json key")
    }
}

/// The records in file order (the GPU table's order).
#[derive(Debug, Clone, PartialEq)]
pub struct Reactions {
    records: Vec<Record>,
}

fn reaction_err<T>(key: String, message: &str) -> Result<T, RegistryError> {
    refuse("reactions.json", &key, message.to_string())
}

fn finite(v: Option<&Value>) -> Option<f64> {
    v.and_then(Value::as_f64).filter(|x| x.is_finite())
}

impl Reactions {
    /// The records shipped in `assets/reactions.json`, resolved against `physics`.
    pub fn shipped(elements: &Elements, physics: &Physics) -> Result<Reactions, RegistryError> {
        Reactions::parse(REACTIONS_JSON, elements, physics)
    }

    /// Parse and validate. Keys resolve against `elements` (species) and `physics` (temperatures, A, ν, Σ_g); a refusal's
    /// key is `records[<id>].<field>`.
    pub fn parse(text: &str, elements: &Elements, physics: &Physics) -> Result<Reactions, RegistryError> {
        let root: Value = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(e) => return reaction_err("<json>".into(), &e.to_string()),
        };
        let Some(root) = root.as_object() else {
            return reaction_err("<root>".into(), "must be an object {\"version\", \"records\"}");
        };
        if let Some(k) = root.keys().find(|k| *k != "version" && *k != "records") {
            return reaction_err(k.clone(), "unknown key");
        }
        if root.get("version") != Some(&Value::from(1)) {
            return reaction_err("version".into(), "must be 1");
        }
        let Some(list) = root.get("records").and_then(Value::as_array).filter(|l| !l.is_empty()) else {
            return reaction_err("records".into(), "missing, empty, or not a list");
        };
        let mut records: Vec<Record> = Vec::with_capacity(list.len());
        for (i, v) in list.iter().enumerate() {
            let record = parse_record(i, v, elements, physics)?;
            if records.iter().any(|r| r.id == record.id) {
                return reaction_err(format!("records[{}].id", record.id), "duplicate id");
            }
            records.push(record);
        }
        Ok(Reactions { records })
    }

    pub fn records(&self) -> &[Record] {
        &self.records
    }

    pub fn get(&self, id: &str) -> Option<&Record> {
        self.records.iter().find(|r| r.id == id)
    }
}

/// `[[key, share], …]`: species resolve, shares are in (0, 1], no species twice, and they sum to 1 within the tolerance.
fn parse_shares(at: &dyn Fn(&str) -> String, field: &str, v: Option<&Value>, elements: &Elements) -> Result<Vec<Share>, RegistryError> {
    let Some(list) = v.and_then(Value::as_array).filter(|l| !l.is_empty()) else {
        return reaction_err(at(field), "must be a non-empty list of [species, share]");
    };
    let mut shares: Vec<Share> = Vec::with_capacity(list.len());
    for pair in list {
        let Some(pair) = pair.as_array().filter(|p| p.len() == 2) else {
            return reaction_err(at(field), "each entry must be [species key, share]");
        };
        let (Some(key), Some(share)) = (pair[0].as_str(), finite(Some(&pair[1]))) else {
            return reaction_err(at(field), "each entry must be [species key, share]");
        };
        let Some(species) = elements.index_of(key) else {
            return reaction_err(at(field), &format!("\"{key}\" is not a species"));
        };
        if !(share > 0.0 && share <= 1.0) {
            return reaction_err(at(field), &format!("share of {key} must be in (0, 1], got {share}"));
        }
        if shares.iter().any(|s| s.species == species) {
            return reaction_err(at(field), &format!("{key} appears twice"));
        }
        shares.push(Share { species, share });
    }
    let sum: f64 = shares.iter().map(|s| s.share).sum();
    if (sum - 1.0).abs() > SHARE_TOLERANCE {
        let why = format!("shares must sum to 1 within {SHARE_TOLERANCE:e} (mass conserved per record, §1.6.1), got {sum:.17}");
        return reaction_err(at(field), &why);
    }
    Ok(shares)
}

/// A physics.json key's value, or a refusal at `at`.
fn physics_value(at: &str, physics: &Physics, key: &str) -> Result<f64, RegistryError> {
    match physics.get(key) {
        Some(x) => Ok(x),
        None => reaction_err(at.to_string(), &format!("\"{key}\" is not a physics.json key")),
    }
}

fn parse_record(i: usize, v: &Value, elements: &Elements, physics: &Physics) -> Result<Record, RegistryError> {
    let Some(obj) = v.as_object() else {
        return reaction_err(format!("records[{i}]"), "must be an object");
    };
    let id = match obj.get("id").and_then(Value::as_str) {
        Some(s) if !s.is_empty() && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') => s.to_string(),
        _ => return reaction_err(format!("records[{i}].id"), "must be a non-empty string of letters, digits and _"),
    };
    let at = |field: &str| format!("records[{id}].{field}");
    if let Some(k) = obj.keys().find(|k| !RECORD_FIELDS.contains(&k.as_str())) {
        return reaction_err(at(k), "unknown field");
    }
    let group = match obj.get("group").and_then(Value::as_str) {
        Some(g) if GROUPS.contains(&g) => g.to_string(),
        _ => return reaction_err(at("group"), &format!("must be one of {GROUPS:?} (§1.9.1)")),
    };
    let inputs = parse_shares(&at, "inputs", obj.get("inputs"), elements)?;
    let outputs = parse_shares(&at, "outputs", obj.get("outputs"), elements)?;
    let Some(q_ratio) = finite(obj.get("q_ratio")) else {
        return reaction_err(at("q_ratio"), "must be a finite number");
    };
    let (Some(enabled), Some(standin)) = (obj.get("enabled").and_then(Value::as_bool), obj.get("standin").and_then(Value::as_bool)) else {
        return reaction_err(at("enabled"), "`enabled` and `standin` must each be true or false");
    };
    if standin && enabled {
        return reaction_err(at("enabled"), "a stand-in record ships disabled (§1.6.4); only a PLAN amendment enables it");
    }
    let rate = parse_rate(&at, obj.get("rate"), inputs.len(), physics)?;
    Ok(Record { id: id.clone(), group, inputs, outputs, q_ratio, rate, enabled, standin })
}

fn parse_rate(at: &dyn Fn(&str) -> String, v: Option<&Value>, n_inputs: usize, physics: &Physics) -> Result<RateLaw, RegistryError> {
    let Some(obj) = v.and_then(Value::as_object) else {
        return reaction_err(at("rate"), "must be an object");
    };
    if let Some(k) = obj.keys().find(|k| !RATE_FIELDS.contains(&k.as_str())) {
        return reaction_err(at(&format!("rate.{k}")), "unknown field");
    }
    let a = match finite(obj.get("a")) {
        Some(a) if a >= 0.0 => a,
        _ => return reaction_err(at("rate.a"), "must be a finite number ≥ 0"),
    };
    let orders: Vec<f64> = obj
        .get("orders")
        .and_then(Value::as_array)
        .map(|l| l.iter().map(|o| finite(Some(o)).filter(|&n| n > 0.0)).collect::<Option<Vec<_>>>())
        .unwrap_or(None)
        .unwrap_or_default();
    if orders.is_empty() || orders.len() != n_inputs {
        return reaction_err(at("rate.orders"), &format!("must be {n_inputs} numbers > 0, one per input"));
    }

    let Some(term_list) = obj.get("terms").and_then(Value::as_array) else {
        return reaction_err(at("rate.terms"), "must be a list (empty means f ≡ 1)");
    };
    let mut terms = Vec::with_capacity(term_list.len());
    for (j, t) in term_list.iter().enumerate() {
        let here = at(&format!("rate.terms[{j}]"));
        let Some(parts) = t.as_array().filter(|p| p.len() == 4) else {
            return reaction_err(here, "must be [t_key, t_factor, nu, w]");
        };
        let (t_key, t_factor, nu, w) = (&parts[0], &parts[1], &parts[2], &parts[3]);
        let Some(t_key) = t_key.as_str().filter(|k| TEMPERATURE_KEYS.contains(k)) else {
            return reaction_err(here, &format!("t_key must be one of {TEMPERATURE_KEYS:?}"));
        };
        let (Some(t_factor), Some(w)) = (finite(Some(t_factor)).filter(|&f| f > 0.0), finite(Some(w)).filter(|&w| w > 0.0)) else {
            return reaction_err(here, "t_factor and w must be numbers > 0");
        };
        let nu = match nu {
            Value::String(key) => physics_value(&here, physics, key)?,
            other => match finite(Some(other)) {
                Some(x) => x,
                None => return reaction_err(here, "nu must be a number or a physics.json key"),
            },
        };
        if nu < NU_MIN {
            return reaction_err(here, &format!("nu must be ≥ {NU_MIN} (K7: steep burning), got {nu}"));
        }
        let t_k = t_factor * physics_value(&here, physics, t_key)?;
        terms.push(RateTerm { t_key: t_key.to_string(), t_factor, t_k, nu, w });
    }

    let t_thr_factor = match (obj.get("t_thr_factor"), terms.is_empty()) {
        (Some(Value::Null), true) => None,
        (Some(Value::Null), false) => return reaction_err(at("rate.t_thr_factor"), "a record with terms needs a threshold in (0, 1]"),
        (other, true) => {
            let why = if other.is_some() { "f ≡ 1 has no threshold: must be null" } else { "missing" };
            return reaction_err(at("rate.t_thr_factor"), why);
        }
        (other, false) => match finite(other).filter(|f| *f > 0.0 && *f <= 1.0) {
            Some(f) => Some(f),
            None => return reaction_err(at("rate.t_thr_factor"), "must be a number in (0, 1]"),
        },
    };
    let t_thr = t_thr_factor.map(|f| f * terms[0].t_k);

    let gate = match obj.get("gate") {
        Some(Value::Null) => None,
        Some(Value::Object(g)) => {
            if let Some(k) = g.keys().find(|k| *k != "sigma_key" && *k != "p") {
                return reaction_err(at(&format!("rate.gate.{k}")), "unknown field");
            }
            let Some(sigma_key) = g.get("sigma_key").and_then(Value::as_str) else {
                return reaction_err(at("rate.gate.sigma_key"), "must be a physics.json key");
            };
            let sigma_g = physics_value(&at("rate.gate.sigma_key"), physics, sigma_key)?;
            let Some(p) = finite(g.get("p")).filter(|&p| p > 0.0) else {
                return reaction_err(at("rate.gate.p"), "must be a number > 0");
            };
            Some(Gate { sigma_key: sigma_key.to_string(), sigma_g, p })
        }
        _ => return reaction_err(at("rate.gate"), "must be null or {\"sigma_key\", \"p\"}"),
    };

    let Some(a_key) = obj.get("a_coef").and_then(Value::as_str) else {
        return reaction_err(at("rate.a_coef"), "must be the physics.json key of A");
    };
    let a_coef = physics_value(&at("rate.a_coef"), physics, a_key)?;
    Ok(RateLaw { a, orders, terms, t_thr_factor, t_thr, gate, a_key: a_key.to_string(), a_coef })
}

// ---------------------------------------------------------------------------------------------------------------

/// The registries as shipped.
#[derive(Debug, Clone, PartialEq)]
pub struct Registry {
    pub elements: Elements,
    pub physics: Physics,
    pub reactions: Reactions,
}

impl Registry {
    pub fn load() -> Result<Registry, RegistryError> {
        let (elements, physics) = (Elements::shipped()?, Physics::shipped()?);
        let reactions = Reactions::shipped(&elements, &physics)?;
        Ok(Registry { elements, physics, reactions })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units;

    fn phys() -> Physics {
        Physics::shipped().expect("shipped physics.json loads")
    }

    /// The key a refusal names; panics if the value was accepted.
    fn refused_key(r: Result<Physics, RegistryError>) -> String {
        r.expect_err("must be refused").key
    }

    fn accepted(r: Result<Physics, RegistryError>) {
        if let Err(e) = r {
            panic!("must be accepted: {e}");
        }
    }

    /// elements.json with `from` (found exactly once) replaced by `to`.
    fn elements_with(from: &str, to: &str) -> Result<Elements, RegistryError> {
        assert_eq!(ELEMENTS_JSON.matches(from).count(), 1, "{from} must appear once");
        Elements::parse(&ELEMENTS_JSON.replace(from, to))
    }

    fn elements_key(from: &str, to: &str) -> String {
        elements_with(from, to).expect_err("must be refused").key
    }

    #[test]
    fn shipped_files_load() {
        let r = Registry::load().expect("both embedded files load");
        assert_eq!(r.elements.species().len(), N_SPECIES);
        assert_eq!(Physics::keys().count(), 52);
    }

    #[test]
    fn elements_in_contract_order_verbatim() {
        let table = [
            ("H", "Hydrogen", 1, 1, true, [0x7F, 0xB2, 0xFF]),
            ("He", "Helium", 4, 2, true, [0xFF, 0xE3, 0x8A]),
            ("C", "Carbon", 12, 6, true, [0x9E, 0x9E, 0x9E]),
            ("O", "Oxygen", 16, 8, true, [0x6B, 0xE3, 0xC9]),
            ("Ne", "Neon", 20, 10, true, [0xFF, 0x6F, 0xAE]),
            ("Mg", "Magnesium", 24, 12, true, [0xB5, 0x8C, 0xFF]),
            ("Si", "Silicon", 28, 14, true, [0xD8, 0xB2, 0x7A]),
            ("S", "Sulfur", 32, 16, true, [0xF2, 0xE1, 0x4C]),
            ("Fe", "Iron", 56, 26, true, [0xD9, 0x57, 0x2B]),
            ("n", "Neutron matter", 1, 0, false, [0xFF, 0xFF, 0xFF]),
        ];
        let e = Elements::shipped().unwrap();
        for (id, (key, name, a, z, paintable, color)) in table.into_iter().enumerate() {
            let s = &e.species()[id];
            assert_eq!((s.key.as_str(), s.name.as_str(), s.symbol.as_str()), (key, name, key), "id {id}");
            assert_eq!((s.a, s.z, s.paintable, s.color), (a, z, paintable, color), "id {id}");
            assert_eq!(e.index_of(key), Some(id));
        }
    }

    #[test]
    fn elements_refusals_name_their_key() {
        assert_eq!(elements_key("\"key\": \"C\",", "\"key\": \"O\","), "species[2].key"); // order
        assert_eq!(elements_key("\"a\": 4,", "\"a\": 5,"), "species[1].a");
        assert_eq!(elements_key("\"z\": 26", "\"z\": 25"), "species[8].z");
        assert_eq!(elements_key("\"paintable\": false", "\"paintable\": true"), "species[9].paintable");
        assert_eq!(elements_key("\"#7FB2FF\"", "\"7FB2FF\""), "species[0].color");
        assert_eq!(elements_key("\"#7FB2FF\"", "\"#7FB2GG\""), "species[0].color");
        assert_eq!(elements_key("\"name\": \"Neon\"", "\"name\": \"\""), "species[4].name");
        assert_eq!(elements_key("\"symbol\": \"H\",", "\"symbol\": \"H\", \"extra\": 1,"), "species[0].extra");
        assert_eq!(elements_key("\"version\": 1", "\"version\": 2"), "version");
        let mut root: Value = serde_json::from_str(ELEMENTS_JSON).unwrap();
        root["species"].as_array_mut().unwrap().pop();
        assert_eq!(Elements::parse(&root.to_string()).unwrap_err().key, "species");
    }

    #[test]
    fn physics_initial_values_verbatim() {
        let expected: [(&str, f64); 52] = [
            ("t_h", 100.0), ("t_he", 140.0), ("t_c", 196.0), ("t_ne", 274.0), ("t_o", 300.0), ("t_si", 420.0),
            ("q_h", 8700.0), ("a_h_burn", 5e-4), ("a_he_burn", 1e-3), ("a_c_alpha", 1e-3), ("a_c_burn", 1e-3),
            ("a_ne_burn", 1e-3), ("a_o_burn", 1e-3), ("a_mg_burn", 1e-3), ("a_si_burn", 1e-3), ("a_s_burn", 1e-3),
            ("a_n_fe", 1e-3), ("nu_c_burn", 30.0), ("nu_ne_burn", 30.0), ("nu_o_burn", 30.0), ("nu_mg_burn", 30.0),
            ("nu_si_burn", 30.0), ("nu_s_burn", 30.0), ("sigma_n", 160.0), ("burn_dx", 0.1), ("burn_dt", 0.05),
            ("sigma_floor", 1e-8), ("sigma_vac", 1e-6), ("t_floor", 0.05), ("sigma_obj", 1e-3), ("paint_sigma", 0.02),
            ("paint_t", 2.0), ("heat_factor", 1.05), ("cool_factor", 0.95), ("k1e", 21.3), ("k2e", 134.0),
            ("k1n", 2.67), ("k2n", 58.2), ("a_r", 4e-7), ("c_sb", 200.0), ("kappa0", 1.0), ("k_cond", 0.0),
            ("a_nu", 1e-3), ("t_nu", 166.0), ("m_nu", 8.0), ("f_dep", 0.05), ("sigma_bh", 4000.0), ("cfl", 0.4),
            ("eta_g", 0.3), ("kappa_dust", 0.0), ("t_dust", 0.0), ("block", 4.0),
        ];
        let p = phys();
        for (key, x) in expected {
            assert_eq!(p.get(key), Some(x), "{key}");
        }
        assert_eq!(Physics::keys().collect::<Vec<_>>(), expected.map(|(k, _)| k));
        assert_eq!(p.get("not_a_key"), None);
    }

    #[test]
    fn physics_unknown_missing_and_not_a_number() {
        let mut root: Value = serde_json::from_str(PHYSICS_JSON).unwrap();
        let parse = |v: &Value| Physics::parse(&v.to_string(), Standins::SHIPPED).unwrap_err();
        root["extra_key"] = Value::from(1);
        assert_eq!(parse(&root).key, "extra_key");
        root.as_object_mut().unwrap().remove("extra_key");
        root["cfl"] = Value::from("0.4");
        assert_eq!(parse(&root).key, "cfl");
        root.as_object_mut().unwrap().remove("cfl");
        let e = parse(&root);
        assert_eq!((e.key.as_str(), e.message.as_str()), ("cfl", "missing"));
        assert_eq!(refused_key(phys().with(&[("not_a_key", 1.0)])), "not_a_key");
        assert!(Physics::parse("[1]", Standins::SHIPPED).is_err());
        assert!(e.to_string().starts_with("physics.json: cfl: "));
    }

    #[test]
    fn class_bounds_at_their_edge() {
        let eps = 1e-9;
        for (key, lo) in [("t_h", 0.0), ("q_h", 0.0), ("a_h_burn", 0.0), ("a_n_fe", 0.0), ("sigma_n", 0.0), ("c_sb", 0.0)] {
            assert_eq!(refused_key(phys().with(&[(key, lo)])), key, "{key} = {lo}");
            // t_h at a hair above 0 breaks T_Si/T_H ≤ 10, and σ_N has its own floor (below).
            if key != "t_h" && key != "sigma_n" {
                accepted(phys().with(&[(key, lo + eps)]));
            }
        }
        assert_eq!(refused_key(phys().with(&[("k_cond", -eps)])), "k_cond");
        accepted(phys().with(&[("k_cond", 0.0)]));
        for bad in [0.0, 0.5, 4.5, -1.0] {
            assert_eq!(refused_key(phys().with(&[("block", bad)])), "block", "block = {bad}");
        }
        accepted(phys().with(&[("block", 1.0)]));
        assert_eq!(refused_key(phys().with(&[("cfl", f64::NAN)])), "cfl");
    }

    #[test]
    fn ladder_is_strict_where_the_contract_says_less_than() {
        let eps = 1e-9;
        // T_H < T_He < T_C < T_Ne and T_O < T_Si: equal to the one below is refused, a hair above is not.
        for (key, below, value) in [("t_he", "t_h", 100.0), ("t_c", "t_he", 140.0), ("t_ne", "t_c", 196.0), ("t_si", "t_o", 300.0)] {
            assert_eq!(refused_key(phys().with(&[(key, value)])), key, "{key} == {below}");
            accepted(phys().with(&[(key, value + eps)]));
        }
        // Raising T_H onto T_He breaks the first step.
        assert_eq!(refused_key(phys().with(&[("t_h", 140.0)])), "t_he");
        // T_He > T_C (swapped) is refused.
        assert_eq!(refused_key(phys().with(&[("t_he", 200.0)])), "t_c");
    }

    #[test]
    fn neon_and_oxygen_may_tie_but_not_invert() {
        accepted(phys().with(&[("t_o", 274.0)]));
        assert_eq!(refused_key(phys().with(&[("t_o", 274.0 - 1e-9)])), "t_o");
    }

    #[test]
    fn silicon_over_hydrogen_at_most_ten() {
        accepted(phys().with(&[("t_si", 1000.0)]));
        assert_eq!(refused_key(phys().with(&[("t_si", 1000.0 + 1e-9)])), "t_si");
    }

    #[test]
    fn sigma_n_at_least_twice_the_squared_k_ratio() {
        let floor = 2.0 * (134.0f64 / 21.3).powi(2); // ≈ 79.1
        accepted(phys().with(&[("sigma_n", floor)]));
        assert_eq!(refused_key(phys().with(&[("sigma_n", floor - 1e-9)])), "sigma_n");
        // The floor follows the K's: halving K₁ₑ quadruples it.
        assert_eq!(refused_key(phys().with(&[("k1e", 21.3 / 2.0)])), "sigma_n");
    }

    #[test]
    fn nu_at_least_four_and_m_below_every_nu() {
        for key in NU_KEYS {
            accepted(phys().with(&[(key, 4.0), ("m_nu", 3.0)]));
            assert_eq!(refused_key(phys().with(&[(key, 4.0 - 1e-9), ("m_nu", 3.0)])), key);
            assert_eq!(refused_key(phys().with(&[(key, 8.0)])), "m_nu", "m = nu"); // m_nu is 8
            accepted(phys().with(&[(key, 8.0 + 1e-9)]));
        }
    }

    #[test]
    fn f_dep_cap_follows_the_s2_switch() {
        let eps = 1e-9;
        accepted(phys().with(&[("f_dep", 0.0)]));
        accepted(phys().with(&[("f_dep", 0.1)]));
        assert_eq!(refused_key(phys().with(&[("f_dep", 0.1 + eps)])), "f_dep");
        assert_eq!(refused_key(phys().with(&[("f_dep", -eps)])), "f_dep");
        let s2 = Physics::parse(PHYSICS_JSON, Standins { s1: false, s2: true }).unwrap();
        accepted(s2.with(&[("f_dep", 1.0)]));
        assert_eq!(refused_key(s2.with(&[("f_dep", 1.0 + eps)])), "f_dep");
    }

    #[test]
    fn kappa_dust_is_zero_while_s1_is_off() {
        assert_eq!(refused_key(phys().with(&[("kappa_dust", 1e-9)])), "kappa_dust");
        let s1 = Physics::parse(PHYSICS_JSON, Standins { s1: true, s2: false }).unwrap();
        accepted(s1.with(&[("kappa_dust", 1e-9), ("t_dust", 1.0)]));
        assert_eq!(refused_key(s1.with(&[("kappa_dust", -1e-9)])), "kappa_dust");
    }

    #[test]
    fn cold_pressure_pairs_and_the_tov_to_chandrasekhar_ratio() {
        // Off (a Sod or cold-collapse scene's overrides): all four zero is a legitimate state.
        accepted(phys().with(&[("k1e", 0.0), ("k2e", 0.0), ("k1n", 0.0), ("k2n", 0.0)]));
        // Half a pair is refused, naming the zero one.
        assert_eq!(refused_key(phys().with(&[("k1e", 0.0)])), "k1e");
        assert_eq!(refused_key(phys().with(&[("k2n", 0.0)])), "k2n");
        // m_tov / m_ch = 8 (K₂ₙ/K₂ₑ)² ∈ [1.4, 2.0]: shipped ≈ 1.51; the edges sit at K₂ₙ = K₂ₑ √(r/8).
        accepted(phys().with(&[("k2n", 67.0)])); // exactly 2.0
        assert_eq!(refused_key(phys().with(&[("k2n", 67.0 + 1e-6)])), "k2n");
        let lo = 134.0 * (1.4f64 / 8.0).sqrt();
        accepted(phys().with(&[("k2n", lo + 1e-9)]));
        assert_eq!(refused_key(phys().with(&[("k2n", lo - 1e-9)])), "k2n");
    }

    #[test]
    fn composition_of_the_solar_mix_against_hand_values() {
        // §2.10: H 0.7346, He 0.2485, O 0.0077, C 0.0092. By hand:
        //   1/μ = 0.7346·2/1 + 0.2485·3/4 + 0.0077·9/16 + 0.0092·7/12 = 1.4692 + 0.186375 + 0.00433125 + 0.00536667
        //       = 1.66527292 (rounded)
        //   Y_e = 0.7346 + 0.2485/2 + 0.0077/2 + 0.0092/2 = 0.8673
        //   Z_met = 1 − 0.7346 − 0.2485 = 0.0169 (= O + C)
        let mut x = [0.0; N_SPECIES];
        (x[0], x[1], x[2], x[3]) = (0.7346, 0.2485, 0.0092, 0.0077);
        let c = Elements::shipped().unwrap().composition(&x);
        assert!((c.inv_mu - 1.66527292).abs() < 1e-8, "1/mu {}", c.inv_mu);
        assert!((c.y_e - 0.8673).abs() < 1e-12, "Y_e {}", c.y_e);
        assert!((c.z_met - 0.0169).abs() < 1e-12, "Z_met {}", c.z_met);
        // Pure species: hydrogen 1/μ = 2, Y_e = 1; neutron matter 1/μ = 1, Y_e = 0, no metals; iron is all metal.
        let e = Elements::shipped().unwrap();
        let pure = |id: usize| {
            let mut x = [0.0; N_SPECIES];
            x[id] = 1.0;
            e.composition(&x)
        };
        assert_eq!((pure(0).inv_mu, pure(0).y_e, pure(0).z_met), (2.0, 1.0, 0.0));
        assert_eq!((pure(9).inv_mu, pure(9).y_e, pure(9).z_met), (1.0, 0.0, 0.0));
        assert_eq!((pure(8).inv_mu, pure(8).y_e, pure(8).z_met), (27.0 / 56.0, 26.0 / 56.0, 1.0));
    }

    #[test]
    fn units_nominal_sun_like_mass() {
        // M₀ = 2π · 1 · 40² / 3 = 3,200π/3 = 3351.0321638… (§2.1 writes ≈ 3,351).
        assert!((units::M0 - 3351.0321638291).abs() < 1e-9, "{}", units::M0);
        assert_eq!(units::disk_mass(1.0, 40.0), units::M0);
        assert_eq!((units::G, units::DX, units::SIGMA_C, units::A_SUN), (1.0, 1.0, 1.0, 40.0));
        // The disk mass scales as Σc a²: doubling a quadruples it.
        assert!((units::disk_mass(1.0, 80.0) / units::M0 - 4.0).abs() < 1e-12);
    }

    // ---- reactions.json (M0-T11) ------------------------------------------------------------------------------

    fn reactions() -> Reactions {
        Reactions::shipped(&Elements::shipped().unwrap(), &phys()).expect("shipped reactions.json loads")
    }

    /// reactions.json edited through its JSON tree, parsed against the shipped species and constants.
    fn reactions_edit(edit: impl FnOnce(&mut Value)) -> Result<Reactions, RegistryError> {
        let mut root: Value = serde_json::from_str(REACTIONS_JSON).unwrap();
        edit(&mut root);
        Reactions::parse(&root.to_string(), &Elements::shipped().unwrap(), &phys())
    }

    fn record_mut<'a>(root: &'a mut Value, id: &str) -> &'a mut Value {
        root["records"].as_array_mut().unwrap().iter_mut().find(|r| r["id"] == id).expect("record exists")
    }

    /// The key a refusal names; panics if the edit was accepted.
    fn reaction_key(edit: impl FnOnce(&mut Value)) -> String {
        reactions_edit(edit).expect_err("must be refused").key
    }

    fn accepted_reactions(edit: impl FnOnce(&mut Value)) {
        if let Err(e) = reactions_edit(edit) {
            panic!("must be accepted: {e}");
        }
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() <= 1e-15 * b.abs().max(1.0)
    }

    #[test]
    fn reactions_ten_records_verbatim() {
        // §1.6.2: id, group, inputs → outputs, q_ratio, Σ exponent a, orders, terms (t_key, T_k factor, ν), gate.
        type Shares = &'static [(&'static str, f64)];
        type Row = (&'static str, &'static str, Shares, Shares, f64, f64, &'static [f64], &'static [(&'static str, f64, f64)]);
        let table: [Row; 10] = [
            ("H_burn", "H", &[("H", 1.0)], &[("He", 1.0)], 1.0, 1.0, &[2.0], &[("t_h", 1.0, 4.0), ("t_h", 1.15, 18.0)]),
            ("He_burn", "He", &[("He", 1.0)], &[("C", 1.0)], 1.0 / 10.8, 2.0, &[3.0], &[("t_he", 1.0, 40.0)]),
            ("C_alpha", "He", &[("C", 0.75), ("He", 0.25)], &[("O", 1.0)], 1.0 / 14.6, 1.0, &[1.0, 1.0], &[("t_he", 1.0, 40.0)]),
            ("C_burn", "C", &[("C", 1.0)], &[("Ne", 0.8), ("Mg", 0.2)], 1.0 / 15.7, 1.0, &[2.0], &[("t_c", 1.0, 30.0)]),
            ("Ne_burn", "Ne", &[("Ne", 1.0)], &[("O", 0.4), ("Mg", 0.6)], 1.0 / 57.0, 1.0, &[1.0], &[("t_ne", 1.0, 30.0)]),
            ("O_burn", "O", &[("O", 1.0)], &[("Si", 0.7), ("S", 0.3)], 1.0 / 12.6, 1.0, &[2.0], &[("t_o", 1.0, 30.0)]),
            ("Mg_burn", "O", &[("Mg", 1.0)], &[("Si", 1.0)], 1.0 / 12.6, 1.0, &[1.0], &[("t_o", 1.0, 30.0)]),
            ("Si_burn", "Si", &[("Si", 1.0)], &[("Fe", 1.0)], 1.0 / 33.0, 1.0, &[1.0], &[("t_si", 1.0, 30.0)]),
            ("S_burn", "Si", &[("S", 1.0)], &[("Fe", 1.0)], 1.0 / 33.0, 1.0, &[1.0], &[("t_si", 1.0, 30.0)]),
            ("N_Fe", "N", &[("Fe", 1.0)], &[("n", 1.0)], -1.14, 0.0, &[1.0], &[]),
        ];
        let (e, r) = (Elements::shipped().unwrap(), reactions());
        assert_eq!(r.records().len(), 10);
        for (rec, (id, group, inputs, outputs, q_ratio, a, orders, terms)) in r.records().iter().zip(table) {
            assert_eq!((rec.id.as_str(), rec.group.as_str()), (id, group));
            let shares = |list: &[Share]| list.iter().map(|s| (e.species()[s.species].key.clone(), s.share)).collect::<Vec<_>>();
            let want = |list: Shares| list.iter().map(|&(k, s)| (k.to_string(), s)).collect::<Vec<_>>();
            assert_eq!((shares(&rec.inputs), shares(&rec.outputs)), (want(inputs), want(outputs)), "{id}");
            assert!(close(rec.q_ratio, q_ratio), "{id} q_ratio {}", rec.q_ratio);
            assert_eq!((rec.rate.a, rec.rate.orders.as_slice()), (a, orders), "{id}");
            let got: Vec<_> = rec.rate.terms.iter().map(|t| (t.t_key.as_str(), t.t_factor, t.nu, t.w)).collect();
            let want: Vec<_> = terms.iter().map(|&(k, f, nu)| (k, f, nu, 1.0)).collect();
            assert_eq!(got, want, "{id}");
            assert_eq!((rec.enabled, rec.standin), (true, false), "{id}");
            // T_thr_factor 0.5 for every burning record; N_Fe is f ≡ 1 with no threshold.
            assert_eq!(rec.rate.t_thr_factor, if id == "N_Fe" { None } else { Some(0.5) }, "{id}");
        }
        let gate = r.get("N_Fe").unwrap().rate.gate.as_ref().expect("N_Fe is gated");
        assert_eq!((gate.sigma_key.as_str(), gate.sigma_g, gate.p), ("sigma_n", 160.0, 1.0));
        assert!(r.records().iter().filter(|rec| rec.id != "N_Fe").all(|rec| rec.rate.gate.is_none()));
        assert_eq!(r.get("nope"), None);
    }

    #[test]
    fn reactions_groups_are_the_accumulators() {
        let groups: Vec<_> = reactions().records().iter().map(|r| r.group.clone()).collect();
        assert_eq!(groups, ["H", "He", "He", "C", "Ne", "O", "O", "Si", "Si", "N"]);
        assert!(groups.iter().all(|g| GROUPS.contains(&g.as_str())));
    }

    #[test]
    fn reactions_resolve_against_physics() {
        let p = phys();
        let r = reactions();
        let h = r.get("H_burn").unwrap();
        // T_k = t_factor · T_H; T_thr = 0.5 · the first term's T_k; A from a_h_burn.
        assert_eq!((h.rate.terms[0].t_k, h.rate.terms[1].t_k), (100.0, 1.15 * 100.0));
        assert_eq!((h.rate.t_thr, h.rate.a_key.as_str(), h.rate.a_coef), (Some(50.0), "a_h_burn", 5e-4));
        let si = r.get("Si_burn").unwrap();
        assert_eq!((si.rate.terms[0].t_k, si.rate.t_thr, si.rate.a_coef, si.rate.terms[0].nu), (420.0, Some(210.0), 1e-3, 30.0));
        assert_eq!(r.get("N_Fe").unwrap().rate.t_thr, None);
        // A scene's overrides reach the records: a tuned ν and A and a moved temperature.
        let tuned = p.with(&[("nu_c_burn", 35.0), ("a_c_burn", 2e-3), ("t_c", 200.0)]).unwrap();
        let r = Reactions::shipped(&Elements::shipped().unwrap(), &tuned).unwrap();
        let c = r.get("C_burn").unwrap();
        assert_eq!((c.rate.terms[0].nu, c.rate.a_coef, c.rate.terms[0].t_k, c.rate.t_thr), (35.0, 2e-3, 200.0, Some(100.0)));
        // q = q_ratio · Q_H: hydrogen's own, helium's tenth-ish, and N_Fe absorbing.
        let q = |id: &str| reactions().get(id).unwrap().q(&p);
        assert_eq!(q("H_burn"), 8700.0);
        assert!(close(q("He_burn"), 8700.0 / 10.8));
        assert!(q("N_Fe") < 0.0 && close(q("N_Fe"), -1.14 * 8700.0));
    }

    #[test]
    fn reactions_conserve_mass_per_record() {
        for rec in reactions().records() {
            for list in [&rec.inputs, &rec.outputs] {
                let sum: f64 = list.iter().map(|s| s.share).sum();
                assert!((sum - 1.0).abs() <= SHARE_TOLERANCE, "{} sums to {sum}", rec.id);
            }
        }
        // C_alpha's 3:1 by mass is ¹²C + ⁴He → ¹⁶O.
        let c_alpha = reactions().get("C_alpha").unwrap().clone();
        assert_eq!((c_alpha.inputs[0].share, c_alpha.inputs[1].share), (12.0 / 16.0, 4.0 / 16.0));
    }

    #[test]
    fn shares_off_by_a_millionth_are_refused_naming_the_record() {
        // 0.75 + 0.25 is 1 in decimal; 0.750001 + 0.25 is not, and the sum's own rounding (1e-16) is not the reason.
        let key = reaction_key(|r| record_mut(r, "C_alpha")["inputs"][0][1] = Value::from(0.750001));
        assert_eq!(key, "records[C_alpha].inputs");
        let key = reaction_key(|r| record_mut(r, "O_burn")["outputs"][1][1] = Value::from(0.300001));
        assert_eq!(key, "records[O_burn].outputs");
        let key = reaction_key(|r| record_mut(r, "H_burn")["outputs"][0][1] = Value::from(0.999999));
        assert_eq!(key, "records[H_burn].outputs");
        let err = reactions_edit(|r| record_mut(r, "O_burn")["outputs"][1][1] = Value::from(0.300001)).unwrap_err();
        assert_eq!(err.file, "reactions.json");
        assert!(err.message.contains("1e-12") && err.message.contains("sum to 1"), "{}", err.message);
    }

    #[test]
    fn share_tolerance_is_a_trillionth() {
        // Rounding noise passes (0.7 + 0.1 + 0.2 is within an ulp of 1); 5e-13 passes; 2e-12 does not.
        accepted_reactions(|r| record_mut(r, "O_burn")["outputs"] = serde_json::json!([["Si", 0.7], ["S", 0.1], ["Mg", 0.2]]));
        accepted_reactions(|r| record_mut(r, "C_alpha")["inputs"][0][1] = Value::from(0.75 + 5e-13));
        let key = reaction_key(|r| record_mut(r, "C_alpha")["inputs"][0][1] = Value::from(0.75 + 2e-12));
        assert_eq!(key, "records[C_alpha].inputs");
        let key = reaction_key(|r| record_mut(r, "C_alpha")["inputs"][0][1] = Value::from(0.75 - 2e-12));
        assert_eq!(key, "records[C_alpha].inputs");
    }

    #[test]
    fn share_entries_are_refused() {
        let k = |edit: fn(&mut Value)| reaction_key(edit);
        assert_eq!(k(|r| record_mut(r, "C_burn")["outputs"][0][0] = Value::from("Xx")), "records[C_burn].outputs"); // not a species
        assert_eq!(k(|r| record_mut(r, "C_burn")["outputs"] = serde_json::json!([["Ne", 0.5], ["Ne", 0.5]])), "records[C_burn].outputs"); // twice
        assert_eq!(k(|r| record_mut(r, "C_burn")["outputs"] = serde_json::json!([["Ne", 1.0], ["Mg", 0.0]])), "records[C_burn].outputs"); // zero share
        assert_eq!(k(|r| record_mut(r, "C_burn")["outputs"] = serde_json::json!([["Ne", 1.5], ["Mg", -0.5]])), "records[C_burn].outputs"); // outside (0, 1]
        assert_eq!(k(|r| record_mut(r, "C_burn")["outputs"] = serde_json::json!([])), "records[C_burn].outputs");
        assert_eq!(k(|r| record_mut(r, "C_burn")["inputs"] = serde_json::json!([["C"]])), "records[C_burn].inputs");
        assert_eq!(k(|r| record_mut(r, "C_burn")["inputs"] = serde_json::json!("C")), "records[C_burn].inputs");
    }

    #[test]
    fn rate_keys_must_resolve() {
        let k = |edit: fn(&mut Value)| reaction_key(edit);
        // t_key: a physics.json temperature, not any key and not a made-up one.
        assert_eq!(k(|r| record_mut(r, "C_burn")["rate"]["terms"][0][0] = Value::from("t_zz")), "records[C_burn].rate.terms[0]");
        assert_eq!(k(|r| record_mut(r, "C_burn")["rate"]["terms"][0][0] = Value::from("q_h")), "records[C_burn].rate.terms[0]");
        // ν naming a key that is not in physics.json; A naming one; the gate's Σ_g naming one.
        assert_eq!(k(|r| record_mut(r, "C_burn")["rate"]["terms"][0][2] = Value::from("nu_zz")), "records[C_burn].rate.terms[0]");
        assert_eq!(k(|r| record_mut(r, "C_burn")["rate"]["a_coef"] = Value::from("a_zz")), "records[C_burn].rate.a_coef");
        assert_eq!(k(|r| record_mut(r, "N_Fe")["rate"]["gate"]["sigma_key"] = Value::from("sigma_zz")), "records[N_Fe].rate.gate.sigma_key");
        assert_eq!(k(|r| record_mut(r, "C_burn")["rate"]["terms"][0] = serde_json::json!(["t_c", 1.0, 30])), "records[C_burn].rate.terms[0]");
        assert_eq!(k(|r| record_mut(r, "C_burn")["rate"]["terms"][0][1] = Value::from(0.0)), "records[C_burn].rate.terms[0]"); // t_factor
        assert_eq!(k(|r| record_mut(r, "C_burn")["rate"]["terms"][0][3] = Value::from(-1.0)), "records[C_burn].rate.terms[0]"); // w
    }

    #[test]
    fn steepness_is_at_least_four() {
        // K7: ν ≥ 4 whether the record writes the number or names the physics.json key.
        let key = reaction_key(|r| record_mut(r, "H_burn")["rate"]["terms"][0][2] = Value::from(3.9));
        assert_eq!(key, "records[H_burn].rate.terms[0]");
        accepted_reactions(|r| record_mut(r, "H_burn")["rate"]["terms"][0][2] = Value::from(4.0));
        let key = reaction_key(|r| record_mut(r, "S_burn")["rate"]["terms"][0][2] = Value::from(3.99));
        assert_eq!(key, "records[S_burn].rate.terms[0]");
        // The key form: physics.json itself refuses ν < 4, so a record can never resolve below the bound.
        assert_eq!(refused_key(phys().with(&[("nu_s_burn", 3.9)])), "nu_s_burn");
        for id in ["C_burn", "Ne_burn", "O_burn", "Mg_burn", "Si_burn", "S_burn"] {
            assert!(reactions().get(id).unwrap().rate.terms.iter().all(|t| t.nu >= 4.0), "{id}");
        }
    }

    #[test]
    fn a_stand_in_record_ships_disabled() {
        // §1.6.4: a stand-in that is a registry record carries "standin": true and ships "enabled": false.
        let key = reaction_key(|r| record_mut(r, "C_burn")["standin"] = Value::from(true));
        assert_eq!(key, "records[C_burn].enabled");
        let r = reactions_edit(|r| {
            let rec = record_mut(r, "Mg_burn");
            rec["standin"] = Value::from(true);
            rec["enabled"] = Value::from(false);
        })
        .unwrap();
        assert_eq!(r.get("Mg_burn").map(|m| (m.standin, m.enabled)), Some((true, false)));
        assert!(reactions().records().iter().all(|r| r.enabled && !r.standin), "M0's ten are primary records");
        assert_eq!(reaction_key(|r| record_mut(r, "C_burn")["enabled"] = Value::from("yes")), "records[C_burn].enabled");
    }

    #[test]
    fn the_threshold_the_gate_and_the_orders_have_their_shapes() {
        let k = |edit: fn(&mut Value)| reaction_key(edit);
        // Burning records need a threshold in (0, 1]; f ≡ 1 (no terms) has none.
        assert_eq!(k(|r| record_mut(r, "C_burn")["rate"]["t_thr_factor"] = Value::Null), "records[C_burn].rate.t_thr_factor");
        assert_eq!(k(|r| record_mut(r, "C_burn")["rate"]["t_thr_factor"] = Value::from(0.0)), "records[C_burn].rate.t_thr_factor");
        assert_eq!(k(|r| record_mut(r, "C_burn")["rate"]["t_thr_factor"] = Value::from(1.5)), "records[C_burn].rate.t_thr_factor");
        assert_eq!(k(|r| record_mut(r, "N_Fe")["rate"]["t_thr_factor"] = Value::from(0.5)), "records[N_Fe].rate.t_thr_factor");
        assert_eq!(k(|r| record_mut(r, "N_Fe")["rate"]["gate"]["p"] = Value::from(0.0)), "records[N_Fe].rate.gate.p");
        assert_eq!(k(|r| record_mut(r, "N_Fe")["rate"]["gate"]["x"] = Value::from(1)), "records[N_Fe].rate.gate.x");
        assert_eq!(k(|r| record_mut(r, "N_Fe")["rate"]["gate"] = Value::from(true)), "records[N_Fe].rate.gate");
        accepted_reactions(|r| record_mut(r, "C_burn")["rate"]["gate"] = serde_json::json!({"sigma_key": "sigma_n", "p": 2}));
        // Orders: one per input, each > 0; a ≥ 0.
        assert_eq!(k(|r| record_mut(r, "C_alpha")["rate"]["orders"] = serde_json::json!([1])), "records[C_alpha].rate.orders");
        assert_eq!(k(|r| record_mut(r, "C_burn")["rate"]["orders"] = serde_json::json!([0])), "records[C_burn].rate.orders");
        assert_eq!(k(|r| record_mut(r, "C_burn")["rate"]["a"] = Value::from(-1)), "records[C_burn].rate.a");
    }

    #[test]
    fn the_file_and_record_shape_refusals_name_their_place() {
        let k = |edit: fn(&mut Value)| reaction_key(edit);
        assert_eq!(k(|r| r["version"] = Value::from(2)), "version");
        assert_eq!(k(|r| r["extra"] = Value::from(1)), "extra");
        assert_eq!(k(|r| r["records"] = serde_json::json!([])), "records");
        assert_eq!(k(|r| record_mut(r, "He_burn")["extra"] = Value::from(1)), "records[He_burn].extra");
        assert_eq!(k(|r| record_mut(r, "He_burn")["rate"]["extra"] = Value::from(1)), "records[He_burn].rate.extra");
        assert_eq!(k(|r| record_mut(r, "He_burn")["group"] = Value::from("Fe")), "records[He_burn].group");
        assert_eq!(k(|r| record_mut(r, "He_burn")["q_ratio"] = Value::from("tenth")), "records[He_burn].q_ratio");
        assert_eq!(k(|r| record_mut(r, "He_burn")["id"] = Value::from("H_burn")), "records[H_burn].id"); // duplicate
        assert_eq!(k(|r| record_mut(r, "He_burn")["id"] = Value::from("no way")), "records[1].id");
        assert_eq!(k(|r| r["records"][2] = Value::from(7)), "records[2]");
        let e = Elements::shipped().unwrap();
        assert_eq!(Reactions::parse("{", &e, &phys()).unwrap_err().key, "<json>");
        assert_eq!(Reactions::parse("[]", &e, &phys()).unwrap_err().key, "<root>");
        // Everything above refused the same file: reactions.json.
        assert!(reactions_edit(|r| r["version"] = Value::from(2)).unwrap_err().to_string().starts_with("reactions.json: version"));
    }

    #[test]
    fn the_registry_loads_the_reactions_too() {
        let r = Registry::load().unwrap();
        assert_eq!(r.reactions.records().len(), 10);
        assert_eq!(r.reactions, Reactions::shipped(&r.elements, &r.physics).unwrap());
    }
}
