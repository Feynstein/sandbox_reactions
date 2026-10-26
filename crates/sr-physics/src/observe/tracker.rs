//! Objects (m0_contrat.md §1.9.2; M0-T58): 8-connected blocks around dense cells, their identity from frame to frame,
//! and every per-object quantity the stages and the readouts read.
//!
//! Choices where §1.9.2 is silent (declared in M0-T58's handoff):
//! - a block holding a sink joins an object even when no cell of it reaches Σ_obj (a sink at the centre of swallowed
//!   gas is still an object);
//! - R and R_core never read below half a block (the summary's resolution), so τ_dyn is finite for a point-like core;
//! - identity: a pair (new object, previous object) shares the weight both frames hold (below); the pair counts when that
//!   is ≥ 25 % of the previous object's weight; pairs are taken by the shared weight, largest first, ties by the lower id,
//!   each id once — so on a merge the heavier id survives and on a split the larger part keeps it;
//!
//! A sink alone, and identity by sink (contract §0.5 [M0-D70]; the lead at M0-D70, «Its own object (Recommended)», «Share by
//! sink (Recommended)»): a sink that no block of the summary holds — outside the box, which follows the gas alone — is an
//! object of its own: M its mass, c the sink, R half a block. A pair shares, per common block, the lesser of the two frames'
//! gas masses, and per sink both hold the lesser of its two masses: each previous sink, carried on by its velocity over the
//! frame's sim time, is the sink of this frame nearest that point within the larger of the two accretion radii,
//! max(2 cells, 2GM_s/c_sb²) (§2.6; two live sinks are never closer — they would have merged); a merge gathers several.
//! Declared in M0-D70's handoff: the match runs from each previous sink to this frame's (a merged sink holds both), ties
//! by the lower slot.
//! The id follows the centre (contract §0.5 [M0-D97]; the lead at M0-D97, «Id follows the centre (Recommended)»): a new
//! object holding a previous object's centre — the block of its max-Σ cell, or its heaviest sink's successor — keeps that
//! object's id, whatever its share, when that previous object is the one it shares the most with; such pairs are taken
//! first, then the shares as above. On a split the piece that holds the centre keeps the id and the history (a blast's core,
//! a planetary nebula's white dwarf), the rest are new; a merge still keeps the heavier id (it shares the most).
//! Declared in M0-D97's handoff: the centre is read from the last frame's object (no snapshot change); a centre held by a
//! new object that shares more with another previous object counts for nothing.
//! - K is the gas's kinetic energy (the sinks' ½Mv² is not in it), and v_r weights the sinks in with their mass.
//!
//! L, the nearest star (contract §0.5 [M0-D53]; the lead at M0-V18, «Nearest star (Recommended)»): the light a block outside
//! every listed object radiates — a halo under Σ_obj, a wisp's blocks — is credited to the listed object whose member block
//! is nearest (block centres, Euclidean); ties go to the heavier object, then the lower id. Declared in M0-D53's handoff:
//! only an object's edge blocks are searched (an inner block always has a nearer neighbour of its own object), and a box
//! with no listed object credits its light to none.

use std::collections::BTreeMap;

use serde_json::{json, Map, Value};

use super::{BlockData, Report, SinkData, GROUPS, GROUP_N};
use crate::registry::Physics;

/// An f64 as the observers' snapshot writes it (contract §0.5 [M0-D76]): Rust's shortest round-trip form, as a JSON string.
/// serde_json reads a number best-effort (its `float_roundtrip` feature is off, §1.2's table unchanged), so a 17-digit f64
/// may come back one ulp off; a string read by Rust's correctly rounded parser comes back bit for bit, ±inf and NaN too.
pub fn exact(x: f64) -> Value {
    Value::String(format!("{x:?}"))
}

/// An f64 [`exact`] wrote; `at` names it in the refusal.
pub fn read_exact(v: &Value, at: &str) -> Result<f64, String> {
    v.as_str().and_then(|s| s.parse().ok()).ok_or_else(|| format!("{at}: not an f64 written as a string: {v}"))
}

/// A whole number of the snapshot at `v[key]`.
pub fn read_whole(v: &Value, key: &str) -> Result<u64, String> {
    v[key].as_u64().ok_or_else(|| format!("{key}: not a whole number: {}", v[key]))
}

/// A list of the snapshot at `v[key]`.
pub fn read_list<'a>(v: &'a Value, key: &str) -> Result<&'a Vec<Value>, String> {
    v[key].as_array().ok_or_else(|| format!("{key}: not a list: {}", v[key]))
}

/// The Sun-like preset's unit mass, 2πΣc a²/3 with Σc = 1 and a = 40 (§2.1), before the calibration's exact figure.
pub const SUN_MASS_NOMINAL: f64 = 2.0 * std::f64::consts::PI * 40.0 * 40.0 / 3.0;
/// The fraction of the Sun-like mass below which an object is a wisp (§1.9.2).
const WISP_FRACTION: f64 = 0.01;
/// The share of an object's weight a successor must hold to inherit its id (§1.9.2).
const INHERIT: f64 = 0.25;
/// The mass fraction R encloses, and the power fraction R_core encloses (§1.9.2).
const ENCLOSED: f64 = 0.9;
/// How far (in blocks) a block reaches to join its neighbours: 1 is 8-connectivity.
const LINK: i64 = 1;
/// The accretion radius's floor in cells (§2.6).
const R_ACC_MIN: f64 = 2.0;

/// What the tracker needs of the constants: Σ_obj and the block edge (§2.7), G (§2.1), the light speed c_sb of the
/// accretion radius (§2.6), the wisp mass (§1.9.2).
#[derive(Clone, Debug, PartialEq)]
pub struct TrackerConfig {
    pub sigma_obj: f64,
    pub block: u32,
    pub g: f64,
    pub c_sb: f64,
    pub wisp_mass: f64,
}

impl TrackerConfig {
    /// `physics`' Σ_obj, block edge and c_sb, G = 1, wisps under 1 % of `sun_mass` (calibration's `preset_mass_sb.sun`).
    pub fn from_physics(physics: &Physics, sun_mass: f64) -> TrackerConfig {
        TrackerConfig {
            sigma_obj: physics.get("sigma_obj").expect("sigma_obj is a physics.json key"),
            block: physics.get("block").expect("block is a physics.json key") as u32,
            g: 1.0,
            c_sb: physics.get("c_sb").expect("c_sb is a physics.json key"),
            wisp_mass: WISP_FRACTION * sun_mass,
        }
    }

    /// A sink's accretion radius, max(2 cells, 2GM_s/c_sb²) (§2.6).
    fn r_acc(&self, m: f64) -> f64 {
        R_ACC_MIN.max(2.0 * self.g * m / (self.c_sb * self.c_sb))
    }
}

/// One object after a frame, in sandbox units (§2.1).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Object {
    pub id: u32,
    /// M: gas and sinks, and the two parts.
    pub mass: f64,
    pub gas_mass: f64,
    pub sink_mass: f64,
    /// The sinks inside.
    pub sinks: usize,
    /// c: the heaviest sink inside, else the max-Σ cell's centre.
    pub centre: [f64; 2],
    /// R: the radius about c holding 90 % of M, from block masses.
    pub radius: f64,
    /// τ_dyn = √(R³/GM).
    pub tau_dyn: f64,
    /// L, the frame's radiated energy ÷ its sim time: its own blocks' and the light credited to it as the nearest star.
    pub luminosity: f64,
    /// The part of L that left the object's own member blocks.
    pub luminosity_own: f64,
    /// L_g by group ([`GROUPS`]; L_N ≤ 0 is the neutronization's absorbed power).
    pub l_group: [f64; 7],
    /// L_nuc: the sum of the positive L_g.
    pub l_nuc: f64,
    /// L_ν: the neutrino power lost, and the part deposited back.
    pub l_nu: f64,
    pub l_nu_deposited: f64,
    /// Σ, T and X at c, and d_c = Π_cold / Π there.
    pub sigma_c: f64,
    pub t_c: f64,
    pub x_c: [f64; 10],
    pub d_c: f64,
    /// v_r: the mass-weighted radial velocity about c.
    pub v_r: f64,
    /// U (thermal), K (kinetic, gas), P = ∫Π dA, W = ½Σmφ, M_unb.
    pub thermal: f64,
    pub kinetic: f64,
    pub pressure: f64,
    pub potential: f64,
    pub unbound: f64,
    /// The group of the dominant burning stage (the largest positive L_g, N excluded), its R_core and the gas outside
    /// 2 R_core; none while no group burns.
    pub core_group: Option<usize>,
    pub r_core: Option<f64>,
    pub envelope_mass: Option<f64>,
}

/// A component's blocks as the next frame's identity reads them: absolute block coordinates and gas masses.
type Weights = BTreeMap<(i64, i64), f64>;

/// An object as the next frame's identity reads it: its blocks' gas (absolute block coordinates), its sinks, and the
/// weight of both.
struct Previous {
    id: u32,
    weight: f64,
    blocks: Weights,
    sinks: Vec<SinkData>,
}

/// An object found in a frame: its measures, its blocks' gas for the next frame, its member blocks (none for a sink alone)
/// and its sinks (indices into the report's).
struct Found {
    object: Object,
    weights: Weights,
    list: Vec<usize>,
    sinks: Vec<usize>,
}

/// The objects of a run: ids that last across frames, merges and splits (§1.9.2).
pub struct Tracker {
    config: TrackerConfig,
    next_id: u32,
    previous: Vec<Previous>,
    objects: Vec<Object>,
}

impl Tracker {
    pub fn new(config: TrackerConfig) -> Tracker {
        Tracker { config, next_id: 1, previous: Vec::new(), objects: Vec::new() }
    }

    /// The objects after the last [`update`](Tracker::update), by id.
    pub fn objects(&self) -> &[Object] {
        &self.objects
    }

    /// A frame's report in: its objects out, by id. Every id is new or inherited by §1.9.2's rule, never reused.
    pub fn update(&mut self, report: &Report) -> &[Object] {
        let mut found: Vec<Found> = components(&self.config, report)
            .into_iter()
            .map(|list| measure(&self.config, report, list))
            .chain(lone_sinks(report).into_iter().map(|k| measure_sink(&self.config, report, k)))
            .filter(|f| f.object.mass >= self.config.wisp_mass)
            .collect();
        let ids = self.inherit(report, &found);
        let mut previous = Vec::with_capacity(found.len());
        for (f, id) in found.iter_mut().zip(ids) {
            f.object.id = id;
            let sinks: Vec<SinkData> = f.sinks.iter().map(|&k| report.sinks[k].clone()).collect();
            let weight = f.weights.values().sum::<f64>() + sinks.iter().map(|s| s.m).sum::<f64>();
            previous.push(Previous { id, weight, blocks: std::mem::take(&mut f.weights), sinks });
        }
        credit_light(report, &mut found);
        self.objects = found.into_iter().map(|f| f.object).collect();
        self.objects.sort_by_key(|o| o.id);
        self.previous = previous;
        &self.objects
    }

    /// For each previous object, each of its sinks' successor in `report` (an index into its sinks): the sink nearest the
    /// previous one carried on by its velocity over the frame, within the larger of the two accretion radii.
    fn sink_successors(&self, report: &Report) -> Vec<Vec<Option<usize>>> {
        self.previous
            .iter()
            .map(|old| {
                old.sinks
                    .iter()
                    .map(|s| {
                        let at = [s.x + s.vx * report.sim_dt, s.y + s.vy * report.sim_dt];
                        let mut best: Option<(f64, usize)> = None;
                        for (k, n) in report.sinks.iter().enumerate() {
                            let d = (n.x - at[0]).hypot(n.y - at[1]);
                            if d <= self.config.r_acc(s.m).max(self.config.r_acc(n.m)) && best.is_none_or(|(bd, _)| d < bd) {
                                best = Some((d, k));
                            }
                        }
                        best.map(|(_, k)| k)
                    })
                    .collect()
            })
            .collect()
    }

    /// Whether `new` holds `old`'s centre (§0.5 [M0-D97]): its heaviest sink's successor when the centre was a sink, else the
    /// block of its max-Σ cell — read from the last frame's object of that id.
    fn holds_centre(&self, old: &Previous, next: &[Option<usize>], new: &Found) -> bool {
        let Some(o) = self.objects.iter().find(|o| o.id == old.id) else { return false };
        match old.sinks.iter().position(|s| s.x == o.centre[0] && s.y == o.centre[1]) {
            Some(j) => next[j].is_some_and(|k| new.sinks.contains(&k)),
            None => {
                let block = self.config.block as f64;
                new.weights.contains_key(&((o.centre[0] / block).floor() as i64, (o.centre[1] / block).floor() as i64))
            }
        }
    }

    /// The id of each new object: the previous object whose centre it holds, when that is also the previous object it
    /// shares the most weight with (§0.5 [M0-D97]); else the previous object it shares the most weight with — its common
    /// blocks' lesser gas and its sinks' lesser masses — when that is ≥ 25 % of the previous object's; each previous id to
    /// one successor, else a fresh id.
    fn inherit(&mut self, report: &Report, found: &[Found]) -> Vec<u32> {
        let successors = self.sink_successors(report);
        let mut pairs: Vec<(bool, f64, u32, usize)> = Vec::new();
        for (i, new) in found.iter().enumerate() {
            let shares: Vec<(f64, usize)> = self
                .previous
                .iter()
                .zip(&successors)
                .enumerate()
                .map(|(o, (old, next))| {
                    let gas: f64 = new.weights.iter().filter_map(|(k, w)| old.blocks.get(k).map(|o| w.min(*o))).sum();
                    let sinks: f64 = old
                        .sinks
                        .iter()
                        .zip(next)
                        .filter_map(|(s, k)| k.filter(|k| new.sinks.contains(k)).map(|k| s.m.min(report.sinks[k].m)))
                        .sum();
                    (gas + sinks, o)
                })
                .filter(|(shared, _)| *shared > 0.0)
                .collect();
            // The previous object this one came from most: the largest share, ties by the lower id.
            let main = shares
                .iter()
                .copied()
                .reduce(|a, b| match b.0.total_cmp(&a.0).then(self.previous[a.1].id.cmp(&self.previous[b.1].id)) {
                    std::cmp::Ordering::Greater => b,
                    _ => a,
                })
                .map(|(_, o)| o);
            for (shared, o) in shares {
                let old = &self.previous[o];
                let centre = main == Some(o) && self.holds_centre(old, &successors[o], new);
                if centre || shared >= INHERIT * old.weight {
                    pairs.push((centre, shared, old.id, i));
                }
            }
        }
        pairs.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.total_cmp(&a.1)).then(a.2.cmp(&b.2)).then(a.3.cmp(&b.3)));
        let mut ids: Vec<Option<u32>> = vec![None; found.len()];
        let mut taken: Vec<u32> = Vec::new();
        for (_, _, id, i) in pairs {
            if ids[i].is_none() && !taken.contains(&id) {
                ids[i] = Some(id);
                taken.push(id);
            }
        }
        ids.into_iter()
            .map(|id| {
                id.unwrap_or_else(|| {
                    self.next_id += 1;
                    self.next_id - 1
                })
            })
            .collect()
    }

    /// What the next frame reads, for the observers' snapshot (§0.5 [M0-D76]): the next id, the last frame's blocks and
    /// sinks by object and its objects.
    pub fn snapshot(&self) -> Value {
        let previous: Vec<Value> = self
            .previous
            .iter()
            .map(|p| {
                let blocks: Vec<Value> = p.blocks.iter().map(|(&(x, y), &w)| json!([x, y, exact(w)])).collect();
                let sinks: Vec<Value> = p.sinks.iter().map(|s| json!([s.x, s.y, s.vx, s.vy, s.m].map(exact))).collect();
                json!({"id": p.id, "weight": exact(p.weight), "blocks": blocks, "sinks": sinks})
            })
            .collect();
        json!({"next_id": self.next_id, "previous": previous, "objects": self.objects.iter().map(Object::snapshot).collect::<Vec<_>>()})
    }

    /// The tracker [`Tracker::snapshot`] wrote, under `config` (the run's constants).
    pub fn restore(config: TrackerConfig, v: &Value) -> Result<Tracker, String> {
        let id = |v: &Value, key: &str| read_whole(v, key).and_then(|n| u32::try_from(n).map_err(|_| format!("{key}: {n}")));
        let previous = read_list(v, "previous")?
            .iter()
            .map(|p| {
                let blocks = read_list(p, "blocks")?
                    .iter()
                    .map(|b| match (b[0].as_i64(), b[1].as_i64()) {
                        (Some(x), Some(y)) => Ok(((x, y), read_exact(&b[2], "previous.blocks")?)),
                        _ => Err(format!("previous.blocks: not [x, y, weight]: {b}")),
                    })
                    .collect::<Result<Weights, String>>()?;
                let sinks = read_list(p, "sinks")?
                    .iter()
                    .map(|s| {
                        let r = |i: usize| read_exact(&s[i], "previous.sinks");
                        match s.as_array().map(Vec::len) {
                            Some(5) => Ok(SinkData { x: r(0)?, y: r(1)?, vx: r(2)?, vy: r(3)?, m: r(4)? }),
                            _ => Err(format!("previous.sinks: not [x, y, vx, vy, m]: {s}")),
                        }
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                Ok(Previous { id: id(p, "id")?, weight: read_exact(&p["weight"], "previous.weight")?, blocks, sinks })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let objects = read_list(v, "objects")?.iter().map(Object::restore).collect::<Result<Vec<_>, String>>()?;
        Ok(Tracker { config, next_id: id(v, "next_id")?, previous, objects })
    }
}

impl Object {
    /// Every f64 field by name, the snapshot's table both ways.
    fn reals(&mut self) -> [(&'static str, &mut f64); 19] {
        [
            ("mass", &mut self.mass),
            ("gas_mass", &mut self.gas_mass),
            ("sink_mass", &mut self.sink_mass),
            ("radius", &mut self.radius),
            ("tau_dyn", &mut self.tau_dyn),
            ("luminosity", &mut self.luminosity),
            ("luminosity_own", &mut self.luminosity_own),
            ("l_nuc", &mut self.l_nuc),
            ("l_nu", &mut self.l_nu),
            ("l_nu_deposited", &mut self.l_nu_deposited),
            ("sigma_c", &mut self.sigma_c),
            ("t_c", &mut self.t_c),
            ("d_c", &mut self.d_c),
            ("v_r", &mut self.v_r),
            ("thermal", &mut self.thermal),
            ("kinetic", &mut self.kinetic),
            ("pressure", &mut self.pressure),
            ("potential", &mut self.potential),
            ("unbound", &mut self.unbound),
        ]
    }

    /// Every f64 list by name.
    fn lists(&mut self) -> [(&'static str, &mut [f64]); 3] {
        [("centre", &mut self.centre[..]), ("l_group", &mut self.l_group[..]), ("x_c", &mut self.x_c[..])]
    }

    /// The object as the observers' snapshot holds it, every field.
    pub fn snapshot(&self) -> Value {
        let mut o = self.clone();
        let mut m = Map::new();
        m.insert("id".into(), json!(o.id));
        m.insert("sinks".into(), json!(o.sinks));
        m.insert("core_group".into(), json!(o.core_group));
        m.insert("r_core".into(), o.r_core.map_or(Value::Null, exact));
        m.insert("envelope_mass".into(), o.envelope_mass.map_or(Value::Null, exact));
        for (k, x) in o.reals() {
            m.insert(k.into(), exact(*x));
        }
        for (k, xs) in o.lists() {
            m.insert(k.into(), xs.iter().map(|x| exact(*x)).collect());
        }
        Value::Object(m)
    }

    /// The object [`Object::snapshot`] wrote.
    pub fn restore(v: &Value) -> Result<Object, String> {
        let optional = |key: &str| if v[key].is_null() { Ok(None) } else { read_exact(&v[key], key).map(Some) };
        let mut o = Object {
            id: u32::try_from(read_whole(v, "id")?).map_err(|_| "objects.id: past u32".to_string())?,
            sinks: read_whole(v, "sinks")? as usize,
            core_group: if v["core_group"].is_null() { None } else { Some(read_whole(v, "core_group")? as usize) },
            r_core: optional("r_core")?,
            envelope_mass: optional("envelope_mass")?,
            ..Object::default()
        };
        for (k, x) in o.reals() {
            *x = read_exact(&v[k], k)?;
        }
        for (k, xs) in o.lists() {
            let list = read_list(v, k)?;
            if list.len() != xs.len() {
                return Err(format!("objects.{k}: {} values, expected {}", list.len(), xs.len()));
            }
            for (x, s) in xs.iter_mut().zip(list) {
                *x = read_exact(s, k)?;
            }
        }
        Ok(o)
    }
}

/// The connected components of member blocks (a cell at Σ ≥ Σ_obj, or a sink inside), each a row-major list of block
/// indices, the components in the order their first block is met.
fn components(config: &TrackerConfig, report: &Report) -> Vec<Vec<usize>> {
    let columns = report.columns as i64;
    if columns == 0 {
        return Vec::new();
    }
    let rows = report.blocks.len() as i64 / columns;
    let member: Vec<bool> = report.blocks.iter().map(|b| b.max_sigma >= config.sigma_obj || b.sinks != 0).collect();
    let mut seen = vec![false; member.len()];
    let mut out = Vec::new();
    for start in 0..member.len() {
        if !member[start] || seen[start] {
            continue;
        }
        let mut stack = vec![start];
        let mut list = Vec::new();
        seen[start] = true;
        while let Some(at) = stack.pop() {
            list.push(at);
            let (x, y) = (at as i64 % columns, at as i64 / columns);
            for dy in -LINK..=LINK {
                for dx in -LINK..=LINK {
                    let (nx, ny) = (x + dx, y + dy);
                    if nx < 0 || ny < 0 || nx >= columns || ny >= rows {
                        continue;
                    }
                    let n = (ny * columns + nx) as usize;
                    if member[n] && !seen[n] {
                        seen[n] = true;
                        stack.push(n);
                    }
                }
            }
        }
        list.sort_unstable();
        out.push(list);
    }
    out
}

/// The nearest star's light (§0.5 [M0-D53]): each block outside every listed object adds its radiated energy to the L of
/// the listed object whose member block is nearest; ties to the heavier object, then the lower id.
fn credit_light(report: &Report, objects: &mut [Found]) {
    let columns = report.columns as i64;
    if columns == 0 || objects.is_empty() || report.sim_dt <= 0.0 {
        return;
    }
    let rows = report.blocks.len() as i64 / columns;
    let mut owner: Vec<Option<usize>> = vec![None; report.blocks.len()];
    for (k, f) in objects.iter().enumerate() {
        for &i in &f.list {
            owner[i] = Some(k);
        }
    }
    // Each object's edge blocks: an 8-neighbour outside the box or not its own.
    let edge: Vec<(i64, i64, usize)> = objects
        .iter()
        .enumerate()
        .flat_map(|(k, f)| f.list.iter().map(move |&i| (k, i)))
        .filter(|&(k, i)| {
            let (x, y) = (i as i64 % columns, i as i64 / columns);
            (-1..=1).any(|dy| {
                (-1..=1).any(|dx| {
                    let (nx, ny) = (x + dx, y + dy);
                    nx < 0 || ny < 0 || nx >= columns || ny >= rows || owner[(ny * columns + nx) as usize] != Some(k)
                })
            })
        })
        .map(|(k, i)| (i as i64 % columns, i as i64 / columns, k))
        .collect();
    let mut credited = vec![0.0; objects.len()];
    for (i, b) in report.blocks.iter().enumerate() {
        if owner[i].is_some() || b.radiated == 0.0 {
            continue;
        }
        let (x, y) = (i as i64 % columns, i as i64 / columns);
        let mut best: Option<(i64, usize)> = None;
        for &(ex, ey, k) in &edge {
            let d = (ex - x).pow(2) + (ey - y).pow(2);
            let better = match best {
                None => true,
                Some((bd, bk)) => {
                    let (o, ob) = (&objects[k].object, &objects[bk].object);
                    d < bd || (d == bd && (o.mass > ob.mass || (o.mass == ob.mass && o.id < ob.id)))
                }
            };
            if better {
                best = Some((d, k));
            }
        }
        if let Some((_, k)) = best {
            credited[k] += b.radiated;
        }
    }
    for (f, extra) in objects.iter_mut().zip(credited) {
        f.object.luminosity = f.object.luminosity_own + extra / report.sim_dt;
    }
}

/// The radius about the centre holding `fraction` of the weight of `(distance, weight)` entries: the distance where the
/// running sum reaches it, interpolated across the entry that crosses it from the one before (0 before the first).
fn radius_holding(entries: &mut [(f64, f64)], fraction: f64) -> f64 {
    entries.sort_by(|a, b| a.0.total_cmp(&b.0));
    let target = fraction * entries.iter().map(|e| e.1).sum::<f64>();
    if target <= 0.0 {
        return 0.0;
    }
    let (mut sum, mut d_prev) = (0.0, 0.0);
    for &(d, w) in entries.iter() {
        if sum + w >= target {
            return d_prev + (d - d_prev) * (target - sum) / w;
        }
        sum += w;
        d_prev = d;
    }
    d_prev
}

/// The sinks that no block of the report holds (outside the box), by index.
fn lone_sinks(report: &Report) -> Vec<usize> {
    let held = report.blocks.iter().fold(0u32, |bits, b| bits | b.sinks);
    (0..report.sinks.len()).filter(|&k| k >= 32 || held >> k & 1 == 0).collect()
}

/// A sink no block holds, as an object of its own (§0.5 [M0-D70]): M its mass, c the sink, R half a block, no gas.
fn measure_sink(config: &TrackerConfig, report: &Report, k: usize) -> Found {
    let s = &report.sinks[k];
    let radius = config.block as f64 / 2.0;
    let object = Object {
        mass: s.m,
        sink_mass: s.m,
        sinks: 1,
        centre: [s.x, s.y],
        radius,
        tau_dyn: (radius.powi(3) / (config.g * s.m)).sqrt(),
        ..Object::default()
    };
    Found { object, weights: Weights::new(), list: Vec::new(), sinks: vec![k] }
}

/// One component's object (id 0 until the tracker names it), with its blocks' gas for the next frame's identity.
fn measure(config: &TrackerConfig, report: &Report, members: Vec<usize>) -> Found {
    let list: &[usize] = &members;
    let block = config.block as f64;
    let r_min = block / 2.0;
    let columns = report.columns as usize;
    let at = |i: usize| -> &BlockData { &report.blocks[i] };
    let key = |i: usize| -> (i64, i64) {
        ((report.origin[0] / config.block) as i64 + (i % columns) as i64, (report.origin[1] / config.block) as i64 + (i / columns) as i64)
    };
    let centroid = |i: usize| -> [f64; 2] {
        let b = at(i);
        if b.mass > 0.0 {
            [b.mx / b.mass, b.my / b.mass]
        } else {
            [report.origin[0] as f64 + ((i % columns) as f64 + 0.5) * block, report.origin[1] as f64 + ((i / columns) as f64 + 0.5) * block]
        }
    };
    // The sinks inside: (index in the report, block).
    let mut sinks: Vec<(usize, usize)> = Vec::new();
    for &i in list {
        for k in 0..report.sinks.len().min(32) {
            if at(i).sinks >> k & 1 == 1 {
                sinks.push((k, i));
            }
        }
    }
    let heaviest = sinks.iter().copied().fold(None::<(usize, usize)>, |best, s| match best {
        Some(b) if report.sinks[b.0].m >= report.sinks[s.0].m => Some(b),
        _ => Some(s),
    });
    // The centre: the heaviest sink, else the max-Σ cell (the lowest y then x among equals: `list` is row-major).
    let mut max_block = list[0];
    for &i in list {
        if at(i).max_sigma > at(max_block).max_sigma {
            max_block = i;
        }
    }
    let (centre, centre_block) = match heaviest {
        Some((k, i)) => ([report.sinks[k].x, report.sinks[k].y], i),
        None => {
            let cell = at(max_block).max_cell;
            ([(cell % report.world_width) as f64 + 0.5, (cell / report.world_width) as f64 + 0.5], max_block)
        }
    };
    let cb = at(centre_block);
    let gas_mass: f64 = list.iter().map(|&i| at(i).mass).sum();
    let sink_mass: f64 = sinks.iter().map(|&(k, _)| report.sinks[k].m).sum();
    let mass = gas_mass + sink_mass;
    let dist = |p: [f64; 2]| ((p[0] - centre[0]).powi(2) + (p[1] - centre[1]).powi(2)).sqrt();

    // R from block and sink masses.
    let mut entries: Vec<(f64, f64)> = list.iter().filter(|&&i| at(i).mass > 0.0).map(|&i| (dist(centroid(i)), at(i).mass)).collect();
    entries.extend(sinks.iter().map(|&(k, _)| (dist([report.sinks[k].x, report.sinks[k].y]), report.sinks[k].m)));
    let radius = radius_holding(&mut entries, ENCLOSED).max(r_min);
    let tau_dyn = (radius.powi(3) / (config.g * mass)).sqrt();

    // v_r: Σ (p · r̂) / M, a block's momentum at its centroid, a sink's at its position, a point at c adding none.
    let radial = |p: [f64; 2], at_: [f64; 2]| -> f64 {
        let (dx, dy) = (at_[0] - centre[0], at_[1] - centre[1]);
        let d = dx.hypot(dy);
        if d > 1e-12 {
            (p[0] * dx + p[1] * dy) / d
        } else {
            0.0
        }
    };
    let mut radial_sum: f64 = list.iter().map(|&i| radial(at(i).momentum, centroid(i))).sum();
    radial_sum += sinks.iter().map(|&(k, _)| report.sinks[k].m * radial([report.sinks[k].vx, report.sinks[k].vy], [report.sinks[k].x, report.sinks[k].y])).sum::<f64>();
    let v_r = if mass > 0.0 { radial_sum / mass } else { 0.0 };

    // Powers over the frame.
    let dt = report.sim_dt;
    let per_time = |energy: f64| if dt > 0.0 { energy / dt } else { 0.0 };
    let sum = |f: &dyn Fn(&BlockData) -> f64| -> f64 { list.iter().map(|&i| f(at(i))).sum() };
    let l_group: [f64; 7] = std::array::from_fn(|g| per_time(sum(&|b| b.nuclear[g])));
    let l_nuc: f64 = l_group.iter().filter(|l| **l > 0.0).sum();
    // The light credited from outside the object is added by `credit_light` once every object is known.
    let luminosity_own = per_time(sum(&|b| b.radiated));

    // The dominant burning group's core and the envelope outside twice its radius.
    let dominant = (0..GROUPS.len())
        .filter(|&g| g != GROUP_N && l_group[g] > 0.0)
        .fold(None::<usize>, |best, g| match best {
            Some(b) if l_group[b] >= l_group[g] => Some(b),
            _ => Some(g),
        });
    let (r_core, envelope_mass) = match dominant {
        Some(g) => {
            let mut power: Vec<(f64, f64)> = list.iter().filter(|&&i| at(i).nuclear[g] > 0.0).map(|&i| (dist(centroid(i)), at(i).nuclear[g])).collect();
            let r = radius_holding(&mut power, ENCLOSED).max(r_min);
            let outside: f64 = list.iter().filter(|&&i| dist(centroid(i)) > 2.0 * r).map(|&i| at(i).mass).sum();
            (Some(r), Some(outside))
        }
        None => (None, None),
    };

    let weights: Weights = list.iter().map(|&i| (key(i), at(i).mass)).collect();
    let object = Object {
        id: 0,
        mass,
        gas_mass,
        sink_mass,
        sinks: sinks.len(),
        centre,
        radius,
        tau_dyn,
        luminosity: luminosity_own,
        luminosity_own,
        l_group,
        l_nuc,
        l_nu: per_time(sum(&|b| b.neutrino_lost)),
        l_nu_deposited: per_time(sum(&|b| b.neutrino_deposited)),
        sigma_c: cb.max_sigma,
        t_c: cb.max_t,
        x_c: cb.max_x,
        d_c: cb.cold_ratio,
        v_r,
        thermal: sum(&|b| b.thermal),
        kinetic: sum(&|b| b.kinetic),
        pressure: sum(&|b| b.pressure),
        potential: 0.5 * sum(&|b| b.phi_m),
        unbound: sum(&|b| b.unbound),
        core_group: dominant,
        r_core,
        envelope_mass,
    };
    Found { object, weights, list: members, sinks: sinks.into_iter().map(|(k, _)| k).collect() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> TrackerConfig {
        TrackerConfig { sigma_obj: 1e-3, block: 4, g: 1.0, c_sb: 400.0, wisp_mass: WISP_FRACTION * SUN_MASS_NOMINAL }
    }

    /// An empty report of `cols` × `rows` blocks over a world as wide as the box, one sim time unit.
    fn report(cols: u32, rows: u32) -> Report {
        Report { sim_dt: 1.0, world_width: cols * 4, origin: [0, 0], columns: cols, blocks: vec![BlockData::default(); (cols * rows) as usize], sinks: vec![] }
    }

    /// A block of `mass` centred in its cell range, Σ = 1 at its cell (4col + 1, 4row + 1).
    fn put(r: &mut Report, col: u32, row: u32, mass: f64) {
        let centroid = [(col * 4 + 2) as f64, (row * 4 + 2) as f64];
        put_at(r, col, row, mass, centroid);
    }

    /// The same with the block's mass centred at `centroid`.
    fn put_at(r: &mut Report, col: u32, row: u32, mass: f64, centroid: [f64; 2]) {
        let i = (row * r.columns + col) as usize;
        let b = &mut r.blocks[i];
        b.mass = mass;
        b.mx = mass * centroid[0];
        b.my = mass * centroid[1];
        b.max_sigma = 1.0;
        b.max_cell = (row * 4 + 1) * r.world_width + col * 4 + 1;
    }

    /// A rectangle of blocks, `mass` each.
    fn rect(r: &mut Report, cols: std::ops::Range<u32>, rows: std::ops::Range<u32>, mass: f64) {
        for row in rows {
            for col in cols.clone() {
                put(r, col, row, mass);
            }
        }
    }

    fn ids(t: &Tracker) -> Vec<u32> {
        t.objects().iter().map(|o| o.id).collect()
    }

    #[test]
    fn one_blob() {
        let mut r = report(8, 8);
        rect(&mut r, 2..5, 2..5, 100.0);
        r.blocks[3 * 8 + 3].max_sigma = 2.0;
        let mut t = Tracker::new(cfg());
        let objects = t.update(&r);
        assert_eq!(objects.len(), 1);
        let o = &objects[0];
        assert_eq!((o.id, o.mass, o.gas_mass, o.sink_mass, o.sinks), (1, 900.0, 900.0, 0.0, 0));
        // The max-Σ cell is block (3, 3)'s (13, 13); its centre is (13.5, 13.5).
        assert_eq!((o.centre, o.sigma_c), ([13.5, 13.5], 2.0));
    }

    #[test]
    fn two_blobs_keep_their_ids() {
        let mut t = Tracker::new(cfg());
        for frame in 0..3 {
            let mut r = report(16, 8);
            rect(&mut r, 1..4, 1..4, 100.0 + frame as f64);
            rect(&mut r, 9..11, 2..4, 50.0);
            t.update(&r);
            assert_eq!(ids(&t), vec![1, 2], "frame {frame}");
            assert_eq!(t.objects()[1].mass, 200.0);
        }
    }

    #[test]
    fn eight_connected_joins_a_corner_but_not_an_empty_block() {
        let mut t = Tracker::new(cfg());
        let mut r = report(8, 8);
        put(&mut r, 1, 1, 100.0);
        put(&mut r, 2, 2, 100.0);
        assert_eq!(t.update(&r).len(), 1, "a diagonal neighbour joins");
        let mut r = report(8, 8);
        put(&mut r, 1, 1, 100.0);
        put(&mut r, 3, 1, 100.0);
        assert_eq!(Tracker::new(cfg()).update(&r).len(), 2, "one empty block between separates");
        let mut r = report(8, 8);
        put(&mut r, 1, 1, 100.0);
        put(&mut r, 3, 3, 100.0);
        assert_eq!(Tracker::new(cfg()).update(&r).len(), 2, "one empty block on the diagonal separates");
    }

    #[test]
    fn a_block_below_sigma_obj_does_not_join() {
        let mut r = report(8, 8);
        put(&mut r, 2, 2, 100.0);
        put(&mut r, 3, 2, 100.0);
        r.blocks[2 * 8 + 3].max_sigma = 9e-4;
        let o = Tracker::new(cfg()).update(&r).to_vec();
        assert_eq!((o.len(), o[0].mass), (1, 100.0));
        r.blocks[2 * 8 + 3].max_sigma = 1e-3;
        assert_eq!(Tracker::new(cfg()).update(&r)[0].mass, 200.0, "Σ = Σ_obj joins");
    }

    #[test]
    fn a_wisp_is_left_out() {
        let limit = WISP_FRACTION * SUN_MASS_NOMINAL;
        let mut r = report(16, 8);
        put(&mut r, 1, 1, 0.5 * limit);
        rect(&mut r, 8..10, 1..3, 0.3 * limit);
        let mut t = Tracker::new(cfg());
        let o = t.update(&r);
        assert_eq!(o.len(), 1, "the 0.5 limit blob is a wisp, the 1.2 limit one is not");
        assert!((o[0].mass - 1.2 * limit).abs() < 1e-9);
        // A wisp that grows takes a fresh id: it was never an object.
        put(&mut r, 1, 1, 2.0 * limit);
        assert_eq!(ids_after(&mut t, &r), vec![1, 2]);
    }

    fn ids_after(t: &mut Tracker, r: &Report) -> Vec<u32> {
        t.update(r);
        ids(t)
    }

    #[test]
    fn a_merge_keeps_the_heavier_id() {
        let mut t = Tracker::new(cfg());
        let mut r = report(16, 8);
        rect(&mut r, 1..4, 2..5, 200.0); // id 1, 1800
        rect(&mut r, 7..9, 2..5, 100.0); // id 2, 600
        assert_eq!(ids_after(&mut t, &r), vec![1, 2]);
        // The bridge joins them: one object, the heavier's id.
        rect(&mut r, 4..7, 3..4, 50.0);
        assert_eq!(ids_after(&mut t, &r), vec![1]);
        // Id 2 is gone for good: a new blob is 3.
        rect(&mut r, 12..14, 6..8, 100.0);
        assert_eq!(ids_after(&mut t, &r), vec![1, 3]);
    }

    #[test]
    fn a_split_keeps_the_id_with_the_larger_part() {
        let mut t = Tracker::new(cfg());
        let mut r = report(16, 4);
        rect(&mut r, 1..11, 1..3, 100.0); // 10 × 2 blocks, 2000
        assert_eq!(ids_after(&mut t, &r), vec![1]);
        // Cut the column 8 out: 7 columns on the left (70 %), 2 on the right (20 %, under 25 % of the old weight).
        for row in 1..3 {
            let i = (row * 16 + 8) as usize;
            r.blocks[i] = BlockData::default();
        }
        assert_eq!(ids_after(&mut t, &r), vec![1, 2]);
        assert_eq!(t.objects()[0].mass, 1400.0);
        // A 60/40 cut: both parts hold ≥ 25 %, the larger keeps the id, the other is new.
        let mut t = Tracker::new(cfg());
        let mut r = report(16, 4);
        rect(&mut r, 1..11, 1..3, 100.0);
        t.update(&r);
        for row in 1..3 {
            r.blocks[(row * 16 + 7) as usize] = BlockData::default();
        }
        // Columns 1..7 (6) left, 8..11 (3) right → 66/33.
        assert_eq!(ids_after(&mut t, &r), vec![1, 2]);
        let left = t.objects().iter().find(|o| o.id == 1).unwrap();
        assert_eq!(left.mass, 1200.0);
    }

    #[test]
    fn a_split_keeps_the_id_with_the_centre() {
        // The max-Σ cell in column 9: the 20 % part on the right holds it and keeps the id, under 25 % of the old weight
        // (§0.5 [M0-D97]); the 70 % part is new.
        let mut t = Tracker::new(cfg());
        let mut r = report(16, 4);
        rect(&mut r, 1..11, 1..3, 100.0);
        r.blocks[16 + 9].max_sigma = 5.0;
        assert_eq!(ids_after(&mut t, &r), vec![1]);
        for row in 1..3 {
            r.blocks[(row * 16 + 8) as usize] = BlockData::default();
        }
        assert_eq!(ids_after(&mut t, &r), vec![1, 2]);
        let masses: Vec<(u32, f64)> = t.objects().iter().map(|o| (o.id, o.mass)).collect();
        assert_eq!(masses, vec![(1, 400.0), (2, 1400.0)]);
        // A sink is the centre: the part holding it keeps the id.
        let mut t = Tracker::new(cfg());
        let mut r = report(16, 4);
        rect(&mut r, 1..11, 1..3, 100.0);
        let r = with_sinks(r, vec![sink(9.0, 9.0, 0.0, 0.0, 10.0)]);
        assert_eq!(ids_after(&mut t, &r), vec![1]);
        let mut r = r;
        for row in 1..3 {
            r.blocks[(row * 16 + 3) as usize] = BlockData::default();
        }
        t.update(&r);
        let masses: Vec<(u32, f64)> = t.objects().iter().map(|o| (o.id, o.mass)).collect();
        assert_eq!(masses, vec![(1, 410.0), (2, 1400.0)], "the sink's two columns keep the id");
    }

    #[test]
    fn a_blast_s_core_keeps_the_star_s_history() {
        // m_bh_sb's life 07 (11,200.210, RTX 4090, M0-D97): the blast splits the star, the ejecta outweigh the core. The core
        // holds the star's centre, keeps its id and its passed core collapse, and reads neutron_star; the ejecta are new.
        use super::super::{Stage, StageTracker};
        let (fe, n) = (8, 9); // §1.5's X order: Fe, then free neutrons.
        let mut t = Tracker::new(cfg());
        let mut stages = StageTracker::new(f64::INFINITY);
        let mut r = report(17, 17);
        rect(&mut r, 5..12, 5..12, 50.0);
        let centre = 8 * 17 + 8;
        r.blocks[centre].max_sigma = 5.0;
        r.blocks[centre].nuclear[GROUP_N] = -1.0;
        t.update(&r);
        assert_eq!(stages.update(t.objects(), 10.0)[0].kind, super::super::EventKind::CoreCollapse);
        // The next frame: the core alone in its block, a ring three blocks out (24 × 100) past two empty rings.
        let mut r = report(17, 17);
        for row in 5..12u32 {
            for col in 5..12u32 {
                if row.abs_diff(8).max(col.abs_diff(8)) == 3 {
                    put(&mut r, col, row, 100.0);
                }
            }
        }
        put(&mut r, 8, 8, 300.0);
        let core = &mut r.blocks[centre];
        core.max_sigma = 5.0;
        (core.max_x[fe], core.max_x[n]) = (0.54, 0.46);
        core.cold_ratio = 0.9;
        t.update(&r);
        stages.update(t.objects(), 11.0);
        let masses: Vec<(u32, f64)> = t.objects().iter().map(|o| (o.id, o.mass)).collect();
        assert_eq!(masses, vec![(1, 300.0), (2, 2400.0)], "the core keeps the id at 2 % of the star's weight");
        let s = stages.stage(1).unwrap();
        assert_eq!((s.raw, s.history.last_collapse), (Stage::NeutronStar, Some(10.0)));
        assert_eq!(stages.stage(2).unwrap().history.last_collapse, None, "the ejecta inherit no stage they never had");
    }

    #[test]
    fn a_blob_shifted_by_two_blocks_keeps_its_id() {
        let mut t = Tracker::new(cfg());
        let mut r = report(24, 8);
        rect(&mut r, 1..7, 1..7, 50.0);
        t.update(&r);
        for step in 1..=4 {
            let mut r = report(24, 8);
            rect(&mut r, 1 + 2 * step..7 + 2 * step, 1..7, 50.0);
            t.update(&r);
            assert_eq!(ids(&t), vec![1], "shift {step}");
        }
    }

    #[test]
    fn a_box_that_moves_does_not_change_an_id() {
        let mut t = Tracker::new(cfg());
        let mut r = report(8, 8);
        rect(&mut r, 2..5, 2..5, 100.0);
        r.origin = [32, 16];
        r.world_width = 128;
        t.update(&r);
        // The same blob in a box whose origin moved by one block column: its absolute blocks are unchanged.
        let mut r = report(8, 8);
        r.origin = [28, 16];
        r.world_width = 128;
        rect(&mut r, 3..6, 2..5, 100.0);
        t.update(&r);
        assert_eq!(ids(&t), vec![1]);
    }

    #[test]
    fn radius_ninety_percent_and_tau_dyn() {
        // The centre block holds no mass; Σ there puts c at cell (1, 1) → (1.5, 1.5).
        let field = |m1: f64, d1: f64, m2: f64, d2: f64| {
            let mut r = report(4, 1);
            put(&mut r, 0, 0, 0.0);
            put_at(&mut r, 1, 0, m1, [1.5 + d1, 1.5]);
            put_at(&mut r, 2, 0, m2, [1.5 + d2, 1.5]);
            r
        };
        let o = Tracker::new(cfg()).update(&field(900.0, 10.0, 100.0, 50.0)).to_vec();
        assert!((o[0].radius - 10.0).abs() < 1e-12, "90 % of 1000 sits at the first block: {}", o[0].radius);
        assert!((o[0].tau_dyn - 1.0).abs() < 1e-12, "√(10³ / 1000) = 1: {}", o[0].tau_dyn);
        // 500 at 10 and 500 at 20: the 900 crosses in the second, 4/5 of the way from 10 to 20.
        let o = Tracker::new(cfg()).update(&field(500.0, 10.0, 500.0, 20.0)).to_vec();
        assert!((o[0].radius - 18.0).abs() < 1e-12, "{}", o[0].radius);
        assert!((o[0].tau_dyn - (18.0f64.powi(3) / 1000.0).sqrt()).abs() < 1e-12);
        // A point-like object reads half a block, not 0.
        let mut r = report(4, 1);
        put_at(&mut r, 0, 0, 1000.0, [1.5, 1.5]);
        let o = Tracker::new(cfg()).update(&r).to_vec();
        assert_eq!(o[0].radius, 2.0);
    }

    #[test]
    fn powers_are_the_frame_sums_over_its_sim_time() {
        let mut r = report(4, 2);
        r.sim_dt = 2.0;
        rect(&mut r, 0..3, 0..1, 400.0);
        for b in r.blocks.iter_mut().filter(|b| b.mass > 0.0) {
            b.radiated = 4.0;
            b.nuclear = [10.0, 2.0, 0.0, 0.0, 0.0, 0.0, -8.0 / 3.0];
            b.neutrino_lost = 1.0;
            b.neutrino_deposited = 0.5;
        }
        let o = Tracker::new(cfg()).update(&r).to_vec();
        let o = &o[0];
        assert_eq!(o.luminosity, 6.0);
        assert_eq!(o.l_group[0], 15.0);
        assert_eq!(o.l_group[1], 3.0);
        assert!((o.l_group[6] + 4.0).abs() < 1e-12, "L_N is an absorbed power ≤ 0");
        assert_eq!(o.l_nuc, 18.0, "L_nuc leaves the negative L_N out");
        assert_eq!((o.l_nu, o.l_nu_deposited), (1.5, 0.75));
        // No sim time: no power, not a division by zero.
        r.sim_dt = 0.0;
        let o = Tracker::new(cfg()).update(&r).to_vec();
        assert_eq!((o[0].luminosity, o[0].l_nuc, o[0].l_nu), (0.0, 0.0, 0.0));
    }

    /// A ring of halo blocks around (`col`, `row`) at Chebyshev distance `ring`: under Σ_obj, a little mass, `radiated` each.
    fn halo(r: &mut Report, col: u32, row: u32, ring: u32, radiated: f64) -> f64 {
        let mut total = 0.0;
        for y in row - ring..=row + ring {
            for x in col - ring..=col + ring {
                if x.abs_diff(col).max(y.abs_diff(row)) == ring {
                    let b = &mut r.blocks[(y * r.columns + x) as usize];
                    b.mass = 1e-3;
                    b.max_sigma = 5e-4;
                    b.radiated = radiated;
                    total += radiated;
                }
            }
        }
        total
    }

    #[test]
    fn a_halo_s_light_is_the_nearest_object_s() {
        // M0-V8's star: the light leaves where a halo under Σ_obj meets vacuum, none from the object's own blocks.
        let mut r = report(12, 12);
        r.sim_dt = 2.0;
        rect(&mut r, 4..7, 4..7, 400.0);
        let lit = halo(&mut r, 5, 5, 3, 3.0) + halo(&mut r, 5, 5, 2, 1.0);
        let o = Tracker::new(cfg()).update(&r).to_vec();
        assert_eq!(o.len(), 1, "the halo is not an object of its own");
        assert_eq!(o[0].luminosity_own, 0.0);
        assert_eq!(o[0].luminosity, lit / 2.0, "the box's whole light, over the frame's sim time");
        // Its own blocks' light and the halo's add.
        r.blocks[5 * 12 + 5].radiated = 8.0;
        let o = Tracker::new(cfg()).update(&r).to_vec();
        assert_eq!((o[0].luminosity_own, o[0].luminosity), (4.0, 4.0 + lit / 2.0));
        // A wisp's own blocks are no star's: their light goes to the nearest listed object.
        put(&mut r, 11, 0, 10.0);
        r.blocks[11].radiated = 6.0;
        let o = Tracker::new(cfg()).update(&r).to_vec();
        assert_eq!(o.len(), 1, "the wisp is not listed");
        assert_eq!(o[0].luminosity, 4.0 + (lit + 6.0) / 2.0);
    }

    #[test]
    fn two_stars_each_keep_their_own_halo_s_light() {
        // G-MULTI's adversary: every block's light given to the heaviest object would hide the lighter star's.
        let mut r = report(24, 12);
        rect(&mut r, 4..7, 4..7, 400.0);
        rect(&mut r, 18..20, 5..7, 100.0);
        let heavy = halo(&mut r, 5, 5, 2, 1.0);
        let light = halo(&mut r, 18, 5, 2, 5.0);
        let o = Tracker::new(cfg()).update(&r).to_vec();
        assert_eq!(o.len(), 2);
        let (a, b) = if o[0].mass > o[1].mass { (&o[0], &o[1]) } else { (&o[1], &o[0]) };
        assert_eq!((a.luminosity, b.luminosity), (heavy, light), "each its own halo's, none lost");
        // A block exactly between the two stars' edges goes to the heavier one.
        r.blocks[5 * 24 + 12].radiated = 7.0;
        let o = Tracker::new(cfg()).update(&r).to_vec();
        let (a, b) = if o[0].mass > o[1].mass { (&o[0], &o[1]) } else { (&o[1], &o[0]) };
        assert_eq!((a.luminosity, b.luminosity), (heavy + 7.0, light), "a tie: the heavier");
    }

    #[test]
    fn energies_and_the_centre_values() {
        let mut r = report(4, 2);
        rect(&mut r, 0..3, 0..1, 400.0);
        for (k, b) in r.blocks.iter_mut().filter(|b| b.mass > 0.0).enumerate() {
            let k = k as f64 + 1.0;
            b.thermal = 10.0 * k;
            b.kinetic = 2.0 * k;
            b.pressure = 3.0 * k;
            b.phi_m = -8.0 * k;
            b.unbound = 5.0 * k;
            b.max_t = 100.0 * k;
            b.cold_ratio = 0.1 * k;
            b.max_x = [k; 10];
        }
        // The centre is the max-Σ cell: the second block.
        r.blocks[1].max_sigma = 7.0;
        let o = Tracker::new(cfg()).update(&r).to_vec();
        let o = &o[0];
        assert_eq!((o.thermal, o.kinetic, o.pressure, o.unbound), (60.0, 12.0, 18.0, 30.0));
        assert_eq!(o.potential, -24.0, "W = ½ Σ mφ");
        assert_eq!((o.sigma_c, o.t_c, o.x_c, o.d_c), (7.0, 200.0, [2.0; 10], 0.2));
    }

    #[test]
    fn the_radial_velocity_is_mass_weighted_about_the_centre() {
        // c = (1.5, 1.5) from the empty first block's Σ.
        let mut r = report(4, 1);
        put(&mut r, 0, 0, 0.0);
        put_at(&mut r, 1, 0, 400.0, [11.5, 1.5]);
        put_at(&mut r, 2, 0, 400.0, [-8.5, 1.5]);
        put_at(&mut r, 3, 0, 200.0, [1.5, 11.5]);
        r.blocks[1].momentum = [30.0, 0.0]; // outward
        r.blocks[2].momentum = [-30.0, 0.0]; // outward on the other side
        r.blocks[3].momentum = [0.0, -20.0]; // inward
        let o = Tracker::new(cfg()).update(&r).to_vec();
        assert!((o[0].v_r - (30.0 + 30.0 - 20.0) / 1000.0).abs() < 1e-12, "{}", o[0].v_r);
    }

    #[test]
    fn a_sink_is_the_centre_and_adds_its_mass() {
        let mut r = report(8, 8);
        rect(&mut r, 2..5, 2..5, 100.0);
        r.blocks[3 * 8 + 3].max_sigma = 5.0; // the gas's densest cell, away from the sink
        // A sink in block (2, 2) and a lighter one in (4, 4); a third sink alone in an empty block (6, 6).
        r.sinks = vec![
            SinkData { x: 9.0, y: 9.0, vx: 1.0, vy: 0.0, m: 700.0 },
            SinkData { x: 17.0, y: 17.0, vx: 0.0, vy: 0.0, m: 100.0 },
            SinkData { x: 25.0, y: 25.0, vx: 0.0, vy: 0.0, m: 400.0 },
        ];
        r.blocks[2 * 8 + 2].sinks = 1;
        r.blocks[4 * 8 + 4].sinks = 2;
        r.blocks[6 * 8 + 6].sinks = 4;
        let o = Tracker::new(cfg()).update(&r).to_vec();
        assert_eq!(o.len(), 2, "a sink's block joins even with no dense cell: the lone sink is an object");
        let o = &o[0];
        assert_eq!((o.sinks, o.sink_mass, o.gas_mass, o.mass), (2, 800.0, 900.0, 1700.0));
        assert_eq!(o.centre, [9.0, 9.0], "the heaviest sink");
        assert_eq!(o.sigma_c, 1.0, "Σ_c from the sink's block, not the gas's densest");
    }

    #[test]
    fn the_envelope_lies_outside_twice_the_core() {
        // Carbon burns in the three blocks nearest c: 90 % of its power within the third's distance, 10 at 40.
        let mut r = report(8, 1);
        put_at(&mut r, 0, 0, 100.0, [1.5, 1.5]);
        put_at(&mut r, 1, 0, 100.0, [6.5, 1.5]);
        put_at(&mut r, 2, 0, 100.0, [11.5, 1.5]);
        put_at(&mut r, 3, 0, 300.0, [31.5, 1.5]);
        put_at(&mut r, 4, 0, 500.0, [41.5, 1.5]);
        for (i, power) in [(0, 30.0), (1, 30.0), (2, 30.0), (4, 10.0)] {
            r.blocks[i].nuclear[2] = power;
        }
        r.blocks[3].nuclear[0] = 5.0; // a weaker hydrogen shell
        let o = Tracker::new(cfg()).update(&r).to_vec();
        let o = &o[0];
        assert_eq!(o.core_group, Some(2), "carbon is the largest L_g");
        // Entries (0, 30), (5, 30), (10, 30), (40, 10): 90 of 100 is reached at the third, d = 10.
        assert!((o.r_core.unwrap() - 10.0).abs() < 1e-12, "{:?}", o.r_core);
        assert_eq!(o.envelope_mass, Some(800.0), "the blocks past 2 R_core = 20");
        // Nothing burning: no core, no envelope.
        for b in r.blocks.iter_mut() {
            b.nuclear = [0.0; 7];
        }
        let o = Tracker::new(cfg()).update(&r).to_vec();
        assert_eq!((o[0].core_group, o[0].r_core, o[0].envelope_mass), (None, None, None));
    }

    /// A sink at (`x`, `y`) moving at (`vx`, `vy`).
    fn sink(x: f64, y: f64, vx: f64, vy: f64, m: f64) -> SinkData {
        SinkData { x, y, vx, vy, m }
    }

    /// `r` with its sinks, each block's bits set for the sinks whose position lies in its cells inside the box.
    fn with_sinks(mut r: Report, sinks: Vec<SinkData>) -> Report {
        let rows = r.blocks.len() as u32 / r.columns.max(1);
        for (k, s) in sinks.iter().enumerate() {
            let (col, row) = ((s.x as u32 - r.origin[0]) / 4, (s.y as u32 - r.origin[1]) / 4);
            if r.columns > 0 && s.x >= r.origin[0] as f64 && s.y >= r.origin[1] as f64 && col < r.columns && row < rows {
                r.blocks[(row * r.columns + col) as usize].sinks |= 1 << k;
            }
        }
        r.sinks = sinks;
        r
    }

    #[test]
    fn a_sink_no_block_holds_is_an_object_of_its_own() {
        // M0-D66's T 3 cloud on the RTX 4090: the sink has eaten the gas, the box is empty — no block at all.
        let empty = Report { sim_dt: 1.0, world_width: 64, ..Report::default() };
        let r = with_sinks(empty, vec![sink(34.429, 30.581, 0.29, -0.16, 1649.97)]);
        let o = Tracker::new(cfg()).update(&r).to_vec();
        assert_eq!(o.len(), 1, "the sink alone is an object");
        let o = &o[0];
        assert_eq!((o.id, o.mass, o.gas_mass, o.sink_mass, o.sinks), (1, 1649.97, 0.0, 1649.97, 1));
        assert_eq!((o.centre, o.radius), ([34.429, 30.581], 2.0), "c the sink, R half a block");
        assert_eq!(o.tau_dyn, (8.0f64 / 1649.97).sqrt());
        assert_eq!((o.luminosity, o.v_r, o.sigma_c), (0.0, 0.0, 0.0));
        // Beside a cloud, outside the box: the cloud and the sink, two objects; the sink's bit in no block.
        let mut r = report(8, 8);
        rect(&mut r, 2..5, 2..5, 100.0);
        r.origin = [64, 64];
        r.world_width = 256;
        let r = with_sinks(r, vec![sink(10.5, 10.5, 0.0, 0.0, 50.0)]);
        assert!(r.blocks.iter().all(|b| b.sinks == 0), "outside the box");
        let o = Tracker::new(cfg()).update(&r).to_vec();
        assert_eq!(o.iter().map(|o| (o.mass, o.sinks)).collect::<Vec<_>>(), vec![(900.0, 0), (50.0, 1)]);
        // A sink under the wisp mass is a wisp, as gas is.
        let empty = Report { sim_dt: 1.0, world_width: 64, ..Report::default() };
        let r = with_sinks(empty, vec![sink(10.5, 10.5, 0.0, 0.0, 0.5 * WISP_FRACTION * SUN_MASS_NOMINAL)]);
        assert!(Tracker::new(cfg()).update(&r).is_empty());
    }

    #[test]
    fn a_sink_crossing_a_block_keeps_its_object_s_id() {
        // M0-V9's F1 on the RTX 4090: 28,907 of 30,787 in the sink; by block alone, a sink one block over shares only the
        // gas, 1,880 — under 25 % — and the object took a fresh id each time. Its sink moves one cell a frame (vx 1).
        let mut t = Tracker::new(cfg());
        for frame in 0..6 {
            let mut r = report(8, 8);
            rect(&mut r, 2..6, 2..5, 1880.0 / 12.0);
            let r = with_sinks(r, vec![sink(12.6 + frame as f64, 13.5, 1.0, 0.0, 28_907.0 + frame as f64)]);
            t.update(&r);
            assert_eq!(ids(&t), vec![1], "frame {frame}: the sink in block {}", (12.6 + frame as f64) as u32 / 4);
            assert_eq!(t.objects()[0].sinks, 1);
        }
    }

    #[test]
    fn a_lone_sink_is_followed_where_its_velocity_carries_it() {
        let lone = |x: f64, vx: f64| with_sinks(Report { sim_dt: 1.0, world_width: 256, ..Report::default() }, vec![sink(x, 20.5, vx, 0.0, 500.0)]);
        // Three cells a frame, its velocity saying so: one id.
        let mut t = Tracker::new(cfg());
        for frame in 0..4 {
            t.update(&lone(20.5 + 3.0 * frame as f64, 3.0));
            assert_eq!(ids(&t), vec![1], "frame {frame}");
        }
        // The same three cells with a velocity of 0: the carried point is 3 cells off, past r_acc's 2 — a new object.
        let mut t = Tracker::new(cfg());
        t.update(&lone(20.5, 0.0));
        assert_eq!(ids_after(&mut t, &lone(23.5, 0.0)), vec![2]);
        // At r_acc exactly it is the same sink; a hair past, not.
        let mut t = Tracker::new(cfg());
        t.update(&lone(20.5, 0.0));
        assert_eq!(ids_after(&mut t, &lone(22.5, 0.0)), vec![1]);
        assert_eq!(ids_after(&mut t, &lone(24.5 + 1e-9, 0.0)), vec![2]);
        // r_acc grows with the mass, 2GM/c_sb²: 320,000 reads 4 cells at c_sb 400.
        let heavy = |x: f64| with_sinks(Report { sim_dt: 1.0, world_width: 256, ..Report::default() }, vec![sink(x, 20.5, 0.0, 0.0, 320_000.0)]);
        let mut t = Tracker::new(cfg());
        t.update(&heavy(20.5));
        assert_eq!(ids_after(&mut t, &heavy(24.5)), vec![1]);
    }

    #[test]
    fn sinks_are_matched_by_place_not_by_slot() {
        let frame = |sinks: Vec<SinkData>| with_sinks(Report { sim_dt: 1.0, world_width: 256, ..Report::default() }, sinks);
        // A (1,000), B (400) and C (300), far apart: three objects.
        let mut t = Tracker::new(cfg());
        t.update(&frame(vec![sink(10.0, 10.0, 0.0, 0.0, 1000.0), sink(12.5, 10.0, 0.0, 0.0, 400.0), sink(80.0, 80.0, 0.0, 0.0, 300.0)]));
        assert_eq!(ids(&t), vec![1, 2, 3]);
        // B merges into A (P3 folds the later slot into the earlier, at the mass-weighted centre): C moves to slot 1. The
        // merged sink holds both A and B — the heavier's id survives — and C keeps its own.
        let merged = sink((10.0 * 1000.0 + 12.5 * 400.0) / 1400.0, 10.0, 0.0, 0.0, 1400.0);
        t.update(&frame(vec![merged, sink(80.0, 80.0, 0.0, 0.0, 300.0)]));
        let found: Vec<(u32, f64)> = t.objects().iter().map(|o| (o.id, o.mass)).collect();
        assert_eq!(found, vec![(1, 1400.0), (3, 300.0)]);
        // A new sink takes the next slot and a fresh id.
        t.update(&frame(vec![sink(10.7, 10.0, 0.0, 0.0, 1400.0), sink(80.0, 80.0, 0.0, 0.0, 300.0), sink(40.0, 40.0, 0.0, 0.0, 100.0)]));
        assert_eq!(ids(&t), vec![1, 3, 4]);
    }

    #[test]
    fn the_snapshot_carries_the_sinks_and_refuses_one_without_them() {
        let mut r = report(8, 8);
        rect(&mut r, 2..6, 2..5, 1880.0 / 12.0);
        let mut t = Tracker::new(cfg());
        t.update(&with_sinks(r.clone(), vec![sink(13.6, 13.5, 1.0, 0.0, 28_907.0)]));
        let snap = t.snapshot();
        let mut back = Tracker::restore(cfg(), &snap).expect("the snapshot restores");
        assert_eq!(back.snapshot(), snap, "round trip");
        // The restored tracker follows the sink across the block as the live one does.
        let next = with_sinks(r, vec![sink(14.6, 13.5, 1.0, 0.0, 28_908.0)]);
        assert_eq!(ids_after(&mut back, &next), ids_after(&mut t, &next));
        assert_eq!(ids(&back), vec![1]);
        let mut old = snap.clone();
        old["previous"][0].as_object_mut().unwrap().remove("sinks");
        let err = Tracker::restore(cfg(), &old).err().expect("a snapshot without the sinks is refused");
        assert!(err.contains("sinks"), "{err}");
    }
}
