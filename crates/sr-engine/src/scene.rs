//! Scene files (m0_contrat.md §2.12.1): `scenes/*.json`, read by headless runs and tests — never a save format.
//!
//! Unknown keys are refused at every level (the top, `world`, `switches`, `test`, each object and its `composition`),
//! each refusal naming its path (`switches.hydor: unknown key`); `headless` exits 4 on any refusal. A `disk` or `rect`
//! takes `t` or `pressure`, exactly one. `overrides` are physics.json keys only, applied through [`Physics::with`] (the
//! same bounds as the shipped file); a scene with overrides never writes calibration ([`Scene::writes_calibration`]).
//!
//! Coordinates (§2.12.4): x to the right, y down, cell (0, 0) at the top-left. An object's `x`, `y` are points in cell
//! units — cell (i, j) covers [i, i + 1) × [j, j + 1) and is sampled at its centre (i + ½, j + ½) — so a disk at (300,
//! 200) sits on the 600 × 400 world's exact centre. A `rect` covers the cells [x0, x0 + w) × [y0, y0 + h), clipped to
//! the world. Objects add where they overlap (mass, momentum, energy and each species' mass); cells no object covers
//! are vacuum (Σ = 0, x_H = 1, as P8 leaves them). A world resized from the command line (`--world`) keeps the objects
//! where they stood relative to the world's centre.

use std::path::Path;

use serde_json::{Map, Value};
use sr_physics::calibration::{self, Calibration, PresetMasses};
use sr_physics::eos::Eos;
use sr_physics::reference::grav::Gravity;
use sr_physics::registry::{Elements, Physics, N_SPECIES};

use crate::state::{WorldConfig, N_CHANNELS, SPECIES};

/// The presets §2.10 names. Their masses come from calibration (`preset_mass_sb`, §2.11): a run reads them through
/// [`Scene::with_preset_masses`]; a scene with none to read (no calibration embedded) loads the nominal Sun-like disk, and
/// says so.
pub const PRESETS: [&str; 3] = ["sun", "massive", "giant"];

/// §2.10's composition by mass: H 0.7346, He 0.2485, O 0.0077 and the remaining 0.0092 as C, in §1.5's order.
pub const SUN_MIX: [f64; N_SPECIES] = [0.7346, 0.2485, 0.0092, 0.0077, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
/// §2.1: the nominal cloud has Σc = 1 at radius a = 40 cells.
const SUN_SIGMA_C: f64 = 1.0;
const SUN_RADIUS: f64 = 40.0;
/// §2.10: a preset cloud never grows past this radius; its Σc grows instead.
pub const MAX_RADIUS: f64 = 150.0;
/// §2.1's unit mass M₀ = 2πΣc a²/3 (≈ 3,351): the nominal cloud's, and what a preset loads until calibration.
pub const UNIT_MASS: f64 = 2.0 * std::f64::consts::PI * SUN_SIGMA_C * SUN_RADIUS * SUN_RADIUS / 3.0;
/// §2.10: the uniform temperature is set so that U_th = 0.1 |W|.
const THERMAL_OVER_W: f64 = 0.1;
/// A composition's fractions must sum to 1 within this.
const COMPOSITION_TOLERANCE: f64 = 1e-6;

/// §2.8: what the world's edge does with matter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeMode {
    Leave,
    Bounce,
}

impl EdgeMode {
    pub fn parse(text: &str) -> Result<EdgeMode, String> {
        match text {
            "leave" => Ok(EdgeMode::Leave),
            "bounce" => Ok(EdgeMode::Bounce),
            _ => Err(format!("edge must be \"leave\" or \"bounce\", got {text:?}")),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            EdgeMode::Leave => "leave",
            EdgeMode::Bounce => "bounce",
        }
    }
}

/// §2.12.1's `switches`: each physics on or off; a key left out is on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Switches {
    pub gravity: bool,
    pub hydro: bool,
    pub heat: bool,
    pub reactions: bool,
    pub neutrinos: bool,
    pub sinks: bool,
}

impl Default for Switches {
    fn default() -> Self {
        Self { gravity: true, hydro: true, heat: true, reactions: true, neutrinos: true, sinks: true }
    }
}

/// §2.12.1's `test`: the test-only settings; a key left out takes the game's value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TestSettings {
    pub pin_temperature: bool,
    pub constant_diffusivity: Option<f64>,
    pub flux_limiter: bool,
}

impl Default for TestSettings {
    fn default() -> Self {
        Self { pin_temperature: false, constant_diffusivity: None, flux_limiter: true }
    }
}

/// How a `disk` or `rect` sets its thermal energy: a temperature, or a total pressure Π (§2.3.3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Heat {
    T(f64),
    Pressure(f64),
}

/// The gas an object lays down: its thermal energy, mass fractions (§1.5's order) and velocity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gas {
    pub heat: Heat,
    pub composition: [f64; N_SPECIES],
    pub v: [f64; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub enum Object {
    /// The preset cloud (§2.10) of `mass` centred at (x, y); a scene file's is the nominal [`UNIT_MASS`], whatever the
    /// preset, until M0-T74.
    Preset { preset: String, x: f64, y: f64, mass: f64 },
    /// A Maclaurin disk Σ(r) = Σc √(1 − r²/a²) inside r < a.
    Disk { x: f64, y: f64, a: f64, sigma_c: f64, gas: Gas },
    /// A uniform Σ over w × h cells from (x0, y0).
    Rect { x0: i64, y0: i64, w: i64, h: i64, sigma: f64, gas: Gas },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    pub world: WorldConfig,
    pub edge: EdgeMode,
    pub slowdown: bool,
    pub switches: Switches,
    pub test: TestSettings,
    pub objects: Vec<Object>,
    pub overrides: Vec<(String, f64)>,
}

/// One JSON object being read: its path for messages, its keys checked against the ones it may hold.
struct Fields<'a> {
    map: &'a Map<String, Value>,
    at: String,
}

fn join(at: &str, key: &str) -> String {
    if at.is_empty() {
        key.to_string()
    } else {
        format!("{at}.{key}")
    }
}

/// The first key of `map` that is not in `known`, refused by its path.
fn check_keys(map: &Map<String, Value>, at: &str, known: &[&str]) -> Result<(), String> {
    match map.keys().find(|k| !known.contains(&k.as_str())) {
        Some(k) => Err(format!("{}: unknown key", join(at, k))),
        None => Ok(()),
    }
}

impl<'a> Fields<'a> {
    fn new(value: &'a Value, at: &str, known: &[&str]) -> Result<Fields<'a>, String> {
        let name = if at.is_empty() { "the scene" } else { at };
        let map = value.as_object().ok_or_else(|| format!("{name}: must be an object"))?;
        check_keys(map, at, known)?;
        Ok(Fields { map, at: at.to_string() })
    }

    fn path(&self, key: &str) -> String {
        join(&self.at, key)
    }

    fn get(&self, key: &str) -> Option<&'a Value> {
        self.map.get(key)
    }

    fn number(&self, key: &str) -> Result<Option<f64>, String> {
        match self.get(key) {
            None => Ok(None),
            Some(v) => match v.as_f64() {
                Some(x) if x.is_finite() => Ok(Some(x)),
                _ => Err(format!("{}: must be a finite number", self.path(key))),
            },
        }
    }

    fn required(&self, key: &str) -> Result<f64, String> {
        self.number(key)?.ok_or_else(|| format!("{}: missing", self.path(key)))
    }

    fn positive(&self, key: &str) -> Result<f64, String> {
        let x = self.required(key)?;
        if x > 0.0 {
            Ok(x)
        } else {
            Err(format!("{}: must be > 0, got {x}", self.path(key)))
        }
    }

    fn whole(&self, key: &str) -> Result<i64, String> {
        self.get(key)
            .and_then(Value::as_i64)
            .ok_or_else(|| format!("{}: must be a whole number", self.path(key)))
    }

    fn flag(&self, key: &str, default: bool) -> Result<bool, String> {
        match self.get(key) {
            None => Ok(default),
            Some(v) => v.as_bool().ok_or_else(|| format!("{}: must be true or false", self.path(key))),
        }
    }
}

fn parse_gas(f: &Fields, elements: &Elements) -> Result<Gas, String> {
    let heat = match (f.get("t").is_some(), f.get("pressure").is_some()) {
        (true, true) => return Err(format!("{}: takes t or pressure, not both", f.at)),
        (false, false) => return Err(format!("{}: needs t or pressure", f.at)),
        (true, false) => Heat::T(f.positive("t")?),
        (false, true) => Heat::Pressure(f.positive("pressure")?),
    };
    let at = f.path("composition");
    let map = f.get("composition").ok_or_else(|| format!("{at}: missing"))?;
    let map = map.as_object().ok_or_else(|| format!("{at}: must be an object of mass fractions"))?;
    let mut composition = [0.0; N_SPECIES];
    for (key, v) in map {
        let i = elements.index_of(key).ok_or_else(|| format!("{}: unknown key", join(&at, key)))?;
        composition[i] = v
            .as_f64()
            .filter(|x| x.is_finite() && *x >= 0.0)
            .ok_or_else(|| format!("{}: must be a number ≥ 0", join(&at, key)))?;
    }
    let sum: f64 = composition.iter().sum();
    if (sum - 1.0).abs() > COMPOSITION_TOLERANCE {
        return Err(format!("{at}: the fractions sum to {sum}, not 1"));
    }
    let v = match f.get("v") {
        None => [0.0, 0.0],
        Some(v) => match v.as_array().map(|a| a.iter().map(Value::as_f64).collect::<Vec<_>>()).as_deref() {
            Some(&[Some(vx), Some(vy)]) if vx.is_finite() && vy.is_finite() => [vx, vy],
            _ => return Err(format!("{}: must be [vx, vy], two finite numbers", f.path("v"))),
        },
    };
    Ok(Gas { heat, composition, v })
}

fn parse_object(value: &Value, at: &str, elements: &Elements) -> Result<Object, String> {
    let kind = value.get("kind").and_then(Value::as_str);
    let object = match kind {
        Some("preset") => {
            let f = Fields::new(value, at, &["kind", "preset", "x", "y"])?;
            let name = f.get("preset").and_then(Value::as_str).unwrap_or_default();
            if !PRESETS.contains(&name) {
                return Err(format!("{}: unknown preset {name:?} (known: {})", f.path("preset"), PRESETS.join(", ")));
            }
            Object::Preset { preset: name.to_string(), x: f.required("x")?, y: f.required("y")?, mass: UNIT_MASS }
        }
        Some("disk") => {
            let f = Fields::new(value, at, &["kind", "x", "y", "a", "sigma_c", "t", "pressure", "composition", "v"])?;
            let (x, y, a, sigma_c) = (f.required("x")?, f.required("y")?, f.positive("a")?, f.positive("sigma_c")?);
            Object::Disk { x, y, a, sigma_c, gas: parse_gas(&f, elements)? }
        }
        Some("rect") => {
            let f = Fields::new(value, at, &["kind", "x0", "y0", "w", "h", "sigma", "t", "pressure", "composition", "v"])?;
            let (w, h) = (f.whole("w")?, f.whole("h")?);
            if w < 1 || h < 1 {
                return Err(format!("{at}: w and h must be ≥ 1, got {w} × {h}"));
            }
            let (x0, y0, sigma) = (f.whole("x0")?, f.whole("y0")?, f.positive("sigma")?);
            Object::Rect { x0, y0, w, h, sigma, gas: parse_gas(&f, elements)? }
        }
        _ => return Err(format!("{at}.kind: must be \"preset\", \"disk\" or \"rect\"")),
    };
    Ok(object)
}

impl Scene {
    /// The scene `--scene preset:<name>` stands for: one preset at the world's centre, everything else as a new game.
    pub fn preset(name: &str, world: WorldConfig) -> Result<Scene, String> {
        Scene::preset_mass(name, UNIT_MASS, world)
    }

    /// [`Scene::preset`] with the cloud's mass given, in sandbox units (§2.10; the calibrated `preset_mass_sb`).
    pub fn preset_mass(name: &str, mass: f64, world: WorldConfig) -> Result<Scene, String> {
        if !(mass.is_finite() && mass > 0.0) {
            return Err(format!("preset {name}: the mass must be a finite number > 0, got {mass}"));
        }
        if !PRESETS.contains(&name) {
            return Err(format!("unknown preset {name:?} (known: {})", PRESETS.join(", ")));
        }
        let (x, y) = (f64::from(world.width) / 2.0, f64::from(world.height) / 2.0);
        Ok(Scene {
            world,
            edge: EdgeMode::Leave,
            slowdown: true,
            switches: Switches::default(),
            test: TestSettings::default(),
            objects: vec![Object::Preset { preset: name.to_string(), x, y, mass }],
            overrides: Vec::new(),
        })
    }

    /// The scene with every preset at its calibrated mass (§2.10, `preset_mass_sb`): what a scene file's or `preset:<name>`'s
    /// preset loads as, which the parser leaves the nominal [`UNIT_MASS`] because it reads no calibration. A preset built
    /// in code with a mass of its own ([`Scene::preset_mass`]) is not passed through here.
    pub fn with_preset_masses(mut self, masses: &PresetMasses) -> Scene {
        for object in &mut self.objects {
            if let Object::Preset { preset, mass, .. } = object {
                *mass = match preset.as_str() {
                    "sun" => masses.sun,
                    "massive" => masses.massive,
                    _ => masses.giant,
                };
            }
        }
        self
    }

    /// [`Scene::with_preset_masses`] at the masses of the calibration this binary embeds (§2.11), and the scene as it is where
    /// none is embedded or the file does not parse — what `headless` does after its gate (exit 6) has passed.
    pub fn with_embedded_masses(self) -> Scene {
        match calibration::EMBEDDED.and_then(|text| Calibration::parse(text).ok()) {
            Some(cal) => self.with_preset_masses(&cal.preset_mass_sb),
            None => self,
        }
    }

    pub fn load(path: &Path) -> Result<Scene, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let elements = Elements::shipped().map_err(|e| e.to_string())?;
        Scene::parse(&text, &elements).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn parse(text: &str, elements: &Elements) -> Result<Scene, String> {
        let root: Value = serde_json::from_str(text).map_err(|e| format!("not JSON: {e}"))?;
        let top = Fields::new(&root, "", &["version", "world", "slowdown", "switches", "test", "objects", "overrides"])?;
        match top.get("version").and_then(Value::as_i64) {
            Some(1) => {}
            _ => return Err("version: must be 1".to_string()),
        }

        let w = Fields::new(top.get("world").ok_or("world: missing")?, "world", &["width", "height", "edge"])?;
        let size = |key: &str| u32::try_from(w.whole(key)?).map_err(|_| format!("{}: out of range", w.path(key)));
        let world = WorldConfig::new(size("width")?, size("height")?).map_err(|e| format!("world: {e}"))?;
        let edge = match w.get("edge") {
            None => EdgeMode::Leave,
            Some(v) => EdgeMode::parse(v.as_str().unwrap_or_default()).map_err(|e| format!("world.{e}"))?,
        };

        let slowdown = top.flag("slowdown", true)?;
        let mut switches = Switches::default();
        if let Some(v) = top.get("switches") {
            let s = Fields::new(v, "switches", &["gravity", "hydro", "heat", "reactions", "neutrinos", "sinks"])?;
            switches = Switches {
                gravity: s.flag("gravity", true)?,
                hydro: s.flag("hydro", true)?,
                heat: s.flag("heat", true)?,
                reactions: s.flag("reactions", true)?,
                neutrinos: s.flag("neutrinos", true)?,
                sinks: s.flag("sinks", true)?,
            };
        }
        let mut test = TestSettings::default();
        if let Some(v) = top.get("test") {
            let t = Fields::new(v, "test", &["pin_temperature", "constant_diffusivity", "flux_limiter"])?;
            let constant_diffusivity = match t.get("constant_diffusivity") {
                None | Some(Value::Null) => None,
                Some(_) => Some(t.positive("constant_diffusivity")?),
            };
            test = TestSettings {
                pin_temperature: t.flag("pin_temperature", false)?,
                constant_diffusivity,
                flux_limiter: t.flag("flux_limiter", true)?,
            };
        }

        let list = top.get("objects").and_then(Value::as_array).ok_or("objects: must be a list")?;
        let objects = list
            .iter()
            .enumerate()
            .map(|(i, v)| parse_object(v, &format!("objects[{i}]"), elements))
            .collect::<Result<Vec<_>, _>>()?;

        let mut overrides = Vec::new();
        if let Some(v) = top.get("overrides") {
            let map = v.as_object().ok_or("overrides: must be an object of physics.json keys")?;
            for (key, x) in map {
                if !Physics::keys().any(|k| k == key) {
                    return Err(format!("overrides.{key}: unknown key (physics.json keys only)"));
                }
                let x = x.as_f64().ok_or_else(|| format!("overrides.{key}: must be a number"))?;
                overrides.push((key.clone(), x));
            }
            // The overridden constants must hold physics.json's bounds as a whole (§2.7).
            let shipped = Physics::shipped().map_err(|e| e.to_string())?;
            with_overrides(&shipped, &overrides).map_err(|e| format!("overrides: {e}"))?;
        }

        Ok(Scene { world, edge, slowdown, switches, test, objects, overrides })
    }

    /// §2.12.1: a run with overrides never writes calibration.
    pub fn writes_calibration(&self) -> bool {
        self.overrides.is_empty()
    }

    /// The shipped constants with this scene's overrides.
    pub fn physics(&self) -> Result<Physics, String> {
        with_overrides(&Physics::shipped().map_err(|e| e.to_string())?, &self.overrides)
    }

    /// What the run should say before it starts (`SR-WARN`): presets that are still the nominal disk.
    pub fn warnings(&self) -> Vec<String> {
        self.objects
            .iter()
            .filter_map(|o| match o {
                Object::Preset { preset, mass, .. } if preset != "sun" && *mass == UNIT_MASS => Some(format!(
                    "preset {preset}: the nominal Sun-like disk until calibration gives preset_mass_sb (§2.11)"
                )),
                _ => None,
            })
            .collect()
    }

    /// The state's canonical planes (§2.2) for `world` — the scene's own or `--world`'s. Refused: a `pressure` below
    /// the cold pressure at some cell (no thermal energy can make it). Also returns a warning per object that covers no
    /// cell of the world.
    pub fn planes(&self, world: WorldConfig, physics: &Physics, elements: &Elements) -> Result<(Vec<f32>, Vec<String>), String> {
        let eos = Eos::from_physics(physics).map_err(|e| e.to_string())?;
        let (w, h) = (world.width as usize, world.height as usize);
        // A resized world keeps the objects where they stood relative to its centre; both sizes are multiples of 8.
        let shift = [
            (i64::from(world.width) - i64::from(self.world.width)) / 2,
            (i64::from(world.height) - i64::from(self.world.height)) / 2,
        ];
        let mut sum = Sums::new(w * h);
        let mut warnings = Vec::new();
        for (n, object) in self.objects.iter().enumerate() {
            let cells = cells_of(object, shift, w, h);
            if cells.is_empty() {
                warnings.push(format!("objects[{n}] covers no cell of the {w}x{h} world"));
                continue;
            }
            let gas = match object {
                Object::Preset { .. } => sun_gas(&cells, physics, elements)?,
                Object::Disk { gas, .. } | Object::Rect { gas, .. } => *gas,
            };
            let comp = elements.composition(&gas.composition);
            let (mu, x_n) = (1.0 / comp.inv_mu, gas.composition[N_SPECIES - 1]);
            for &(x, y, sigma) in &cells {
                let eps_th = match gas.heat {
                    Heat::T(t) => Eos::eps_th_of_t(t, mu),
                    Heat::Pressure(p) => {
                        let thermal = p - eos.pi_e(sigma, comp.y_e) - eos.pi_n(sigma, x_n);
                        if thermal <= 0.0 {
                            return Err(format!("objects[{n}].pressure: {p} is below the cold pressure at cell ({x}, {y})"));
                        }
                        thermal / sigma
                    }
                };
                let kinetic = 0.5 * (gas.v[0] * gas.v[0] + gas.v[1] * gas.v[1]);
                sum.add(y * w + x, sigma, &gas, eps_th + eos.eps_cold(sigma, comp.y_e, x_n) + kinetic);
            }
        }
        Ok((sum.planes(), warnings))
    }
}

/// The scene's running sums per cell, in f64, before they become planes.
struct Sums {
    sigma: Vec<f64>,
    mom: [Vec<f64>; 2],
    energy: Vec<f64>,
    species: Vec<[f64; N_SPECIES]>,
}

impl Sums {
    fn new(cells: usize) -> Sums {
        Sums {
            sigma: vec![0.0; cells],
            mom: [vec![0.0; cells], vec![0.0; cells]],
            energy: vec![0.0; cells],
            species: vec![[0.0; N_SPECIES]; cells],
        }
    }

    /// Adds Σ of `gas` at cell `i`, with `eps` its specific energy (thermal + cold + kinetic).
    fn add(&mut self, i: usize, sigma: f64, gas: &Gas, eps: f64) {
        self.sigma[i] += sigma;
        self.mom[0][i] += sigma * gas.v[0];
        self.mom[1][i] += sigma * gas.v[1];
        self.energy[i] += sigma * eps;
        for (m, x) in self.species[i].iter_mut().zip(gas.composition) {
            *m += sigma * x;
        }
    }

    fn planes(&self) -> Vec<f32> {
        let cells = self.sigma.len();
        let mut planes = vec![0f32; N_CHANNELS * cells];
        for i in 0..cells {
            let sigma = self.sigma[i];
            if sigma <= 0.0 {
                planes[SPECIES.start * cells + i] = 1.0;
                continue;
            }
            for (c, value) in [sigma, self.mom[0][i], self.mom[1][i], self.energy[i]].into_iter().enumerate() {
                planes[c * cells + i] = value as f32;
            }
            for (s, m) in self.species[i].iter().enumerate() {
                planes[(SPECIES.start + s) * cells + i] = (m / sigma) as f32;
            }
        }
        planes
    }
}

/// The cells an object covers in a w × h world, shifted by `shift`, with its Σ there: (x, y, Σ).
fn cells_of(object: &Object, shift: [i64; 2], w: usize, h: usize) -> Vec<(usize, usize, f64)> {
    let shifted = |x: f64, y: f64| (x + shift[0] as f64, y + shift[1] as f64);
    match *object {
        Object::Preset { x, y, mass, .. } => {
            let CloudShape { a, sigma_c } = cloud_shape(mass);
            let (x, y) = shifted(x, y);
            // The cells sample the profile at their centres, so their sum is not exactly M (off by up to 2√2 a^-3/2 of
            // it): every cell scaled so the whole cloud, wherever the world clips it, holds the mass it is named for.
            let whole: f64 = disk_samples(x, y, a, sigma_c).map(|c| c.2).sum();
            let k = if whole > 0.0 { mass / whole } else { 1.0 };
            let mut cells = disk_cells(x, y, a, sigma_c, w, h);
            cells.iter_mut().for_each(|c| c.2 *= k);
            cells
        }
        Object::Disk { x, y, a, sigma_c, .. } => {
            let (x, y) = shifted(x, y);
            disk_cells(x, y, a, sigma_c, w, h)
        }
        Object::Rect { x0, y0, w: rw, h: rh, sigma, .. } => {
            let span = |start: i64, len: i64, n: usize| {
                (start.clamp(0, n as i64) as usize)..((start + len).clamp(0, n as i64) as usize)
            };
            let (xs, ys) = (span(x0 + shift[0], rw, w), span(y0 + shift[1], rh, h));
            ys.flat_map(|j| xs.clone().map(move |i| (i, j, sigma))).collect()
        }
    }
}

/// Σ of a Maclaurin disk Σ(r) = Σc √(1 − r²/a²) about (x, y) at the centre of each cell it covers: (i, j, Σ) over the
/// whole plane, indices signed.
fn disk_samples(x: f64, y: f64, a: f64, sigma_c: f64) -> impl Iterator<Item = (i64, i64, f64)> {
    let (j0, j1, i0, i1) = ((y - a).floor() as i64, (y + a).ceil() as i64, (x - a).floor() as i64, (x + a).ceil() as i64);
    (j0..j1).flat_map(move |j| {
        (i0..i1).filter_map(move |i| {
            let (dx, dy) = (i as f64 + 0.5 - x, j as f64 + 0.5 - y);
            let r2 = (dx * dx + dy * dy) / (a * a);
            (r2 < 1.0).then(|| (i, j, sigma_c * (1.0 - r2).sqrt()))
        })
    })
}

/// [`disk_samples`] clipped to a `w` × `h` world.
fn disk_cells(x: f64, y: f64, a: f64, sigma_c: f64, w: usize, h: usize) -> Vec<(usize, usize, f64)> {
    disk_samples(x, y, a, sigma_c)
        .filter(|&(i, j, _)| i >= 0 && j >= 0 && (i as usize) < w && (j as usize) < h)
        .map(|(i, j, m)| (i as usize, j as usize, m))
        .collect()
}

/// The preset cloud's Maclaurin profile (§2.10): radius a and central Σc.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CloudShape {
    pub a: f64,
    pub sigma_c: f64,
}

/// §2.10's cloud of mass `mass` (sandbox units): a = 40 √(M/M₀) cells, capped at [`MAX_RADIUS`], and Σc = 3M/(2π a²)
/// (§2.1) — 1 at M₀ and growing once the cap holds the radius.
pub fn cloud_shape(mass: f64) -> CloudShape {
    let a = (SUN_RADIUS * (mass / UNIT_MASS).sqrt()).min(MAX_RADIUS);
    CloudShape { a, sigma_c: 3.0 * mass / (2.0 * std::f64::consts::PI * a * a) }
}

/// The preset's gas (§2.10): the Sun's mix, at rest, at the uniform temperature that makes U_th = Σ m ε_th = 0.1 |W|
/// over its own cells. W = ½ Σ m φ is the grid's own: φ is one solve of §1.4.2's kernel over the cloud's cells, vacuum
/// (Σ < σ_vac) no source, as P2 solves it — never the Maclaurin disk's closed form, which the grid's W differs from.
fn sun_gas(cells: &[(usize, usize, f64)], physics: &Physics, elements: &Elements) -> Result<Gas, String> {
    let sigma_vac = physics.get("sigma_vac").ok_or("physics.json has no sigma_vac")?;
    let (x0, x1) = cells.iter().fold((usize::MAX, 0), |(lo, hi), c| (lo.min(c.0), hi.max(c.0)));
    let (y0, y1) = cells.iter().fold((usize::MAX, 0), |(lo, hi), c| (lo.min(c.1), hi.max(c.1)));
    let (bw, bh) = (x1 - x0 + 1, y1 - y0 + 1);
    let mut source = vec![0.0; bw * bh];
    for &(x, y, m) in cells {
        if m >= sigma_vac {
            source[(y - y0) * bw + (x - x0)] = m;
        }
    }
    let phi = Gravity::new(1.0).potential(&source, bw, bh).map_err(|e| e.to_string())?;
    let w_sum: f64 = 0.5 * source.iter().zip(&phi).map(|(m, p)| m * p).sum::<f64>();
    let mass: f64 = cells.iter().map(|c| c.2).sum();
    let eps_th = THERMAL_OVER_W * w_sum.abs() / mass;
    let mu = 1.0 / elements.composition(&SUN_MIX).inv_mu;
    Ok(Gas { heat: Heat::T(mu * eps_th), composition: SUN_MIX, v: [0.0, 0.0] })
}

/// A preset cloud ready to drop into a running world (§1.12; M0-T76): the Σ of its cells in the bounding rectangle
/// (x0, y0, w, h), row-major, and the uniform temperature and composition of its gas — what [`Scene::planes`] would
/// lay down for the same `Object::Preset`, the energy left to the edit pass, which adds it to the gas it lands on.
#[derive(Clone, Debug, PartialEq)]
pub struct Cloud {
    pub x0: u32,
    pub y0: u32,
    pub w: u32,
    pub h: u32,
    pub sigma: Vec<f32>,
    pub t: f32,
    pub composition: [f32; N_SPECIES],
}

/// Why a preset cloud could not be made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CloudError {
    /// A cell of the cloud falls outside the world (§1.12: refused with `presets.no_room`).
    NoRoom,
    /// An unknown preset or a mass that is not a finite number > 0, or the physics refusing the cloud.
    Invalid(String),
}

/// The preset `name` at `mass` (sandbox units) centred at the point `centre` (cell units, §2.12.4) in `world`: the same
/// cloud `Scene::preset_mass` loads, or [`CloudError::NoRoom`] when the world clips any cell of it.
pub fn preset_cloud(
    name: &str,
    mass: f64,
    centre: [f64; 2],
    world: WorldConfig,
    physics: &Physics,
    elements: &Elements,
) -> Result<Cloud, CloudError> {
    if !(mass.is_finite() && mass > 0.0) {
        return Err(CloudError::Invalid(format!("preset {name}: the mass must be a finite number > 0, got {mass}")));
    }
    if !PRESETS.contains(&name) {
        return Err(CloudError::Invalid(format!("unknown preset {name:?} (known: {})", PRESETS.join(", "))));
    }
    let CloudShape { a, sigma_c } = cloud_shape(mass);
    let (w, h) = (world.width as usize, world.height as usize);
    let object = Object::Preset { preset: name.to_string(), x: centre[0], y: centre[1], mass };
    let cells = cells_of(&object, [0, 0], w, h);
    if cells.is_empty() || cells.len() < disk_samples(centre[0], centre[1], a, sigma_c).count() {
        return Err(CloudError::NoRoom);
    }
    let gas = sun_gas(&cells, physics, elements).map_err(CloudError::Invalid)?;
    let Heat::T(t) = gas.heat else { unreachable!("a preset's gas is set by its temperature") };
    let (x0, x1) = cells.iter().fold((usize::MAX, 0), |(lo, hi), c| (lo.min(c.0), hi.max(c.0)));
    let (y0, y1) = cells.iter().fold((usize::MAX, 0), |(lo, hi), c| (lo.min(c.1), hi.max(c.1)));
    let (cw, ch) = (x1 - x0 + 1, y1 - y0 + 1);
    let mut sigma = vec![0f32; cw * ch];
    for &(x, y, m) in &cells {
        sigma[(y - y0) * cw + (x - x0)] = m as f32;
    }
    Ok(Cloud {
        x0: x0 as u32,
        y0: y0 as u32,
        w: cw as u32,
        h: ch as u32,
        sigma,
        t: t as f32,
        composition: gas.composition.map(|x| x as f32),
    })
}

/// The nominal Sun-like disk alone, centred in `world`, with the shipped constants — the `preset:sun` scene's planes.
pub fn sun_disk_planes(world: WorldConfig) -> Vec<f32> {
    let (physics, elements) = (Physics::shipped().expect("shipped physics"), Elements::shipped().expect("shipped elements"));
    let scene = Scene::preset("sun", world).expect("sun is a preset");
    scene.planes(world, &physics, &elements).expect("the preset is in the world").0
}

fn with_overrides(physics: &Physics, overrides: &[(String, f64)]) -> Result<Physics, String> {
    let pairs: Vec<(&str, f64)> = overrides.iter().map(|(k, x)| (k.as_str(), *x)).collect();
    physics.with(&pairs).map_err(|e| e.to_string())
}
