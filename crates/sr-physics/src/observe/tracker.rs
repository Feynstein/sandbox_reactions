//! Objects (m0_contrat.md §1.9.2; M0-T58): 8-connected blocks around dense cells, their identity from frame to frame,
//! and every per-object quantity the stages and the readouts read.
//!
//! Choices where §1.9.2 is silent (declared in M0-T58's handoff):
//! - a block holding a sink joins an object even when no cell of it reaches Σ_obj (a sink at the centre of swallowed
//!   gas is still an object);
//! - R and R_core never read below half a block (the summary's resolution), so τ_dyn is finite for a point-like core;
//! - identity: a pair (new object, previous object) shares, per common block, the lesser of the two frames' block
//!   weights (gas plus sinks in it); the pair counts when that is ≥ 25 % of the previous object's weight; pairs are taken
//!   by the shared weight, largest first, ties by the lower id, each id once — so on a merge the heavier id survives and
//!   on a split the larger part keeps it;
//! - K is the gas's kinetic energy (the sinks' ½Mv² is not in it), and v_r weights the sinks in with their mass.

use std::collections::BTreeMap;

use super::{BlockData, Report, GROUPS, GROUP_N};
use crate::registry::Physics;

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

/// What the tracker needs of the constants: Σ_obj and the block edge (§2.7), G (§2.1), the wisp mass (§1.9.2).
#[derive(Clone, Debug, PartialEq)]
pub struct TrackerConfig {
    pub sigma_obj: f64,
    pub block: u32,
    pub g: f64,
    pub wisp_mass: f64,
}

impl TrackerConfig {
    /// `physics`' Σ_obj and block edge, G = 1, wisps under 1 % of `sun_mass` (calibration's `preset_mass_sb.sun`).
    pub fn from_physics(physics: &Physics, sun_mass: f64) -> TrackerConfig {
        TrackerConfig {
            sigma_obj: physics.get("sigma_obj").expect("sigma_obj is a physics.json key"),
            block: physics.get("block").expect("block is a physics.json key") as u32,
            g: 1.0,
            wisp_mass: WISP_FRACTION * sun_mass,
        }
    }
}

/// One object after a frame, in sandbox units (§2.1).
#[derive(Clone, Debug, PartialEq)]
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
    /// L, the frame's radiated energy ÷ its sim time.
    pub luminosity: f64,
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

/// A component's blocks as the next frame's identity reads them: absolute block coordinates and weights.
type Weights = BTreeMap<(i64, i64), f64>;

/// An object's blocks (absolute block coordinates, weights) as the next frame's identity reads them.
struct Previous {
    id: u32,
    weight: f64,
    blocks: Weights,
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
        let mut found: Vec<(Object, Weights)> = components(&self.config, report)
            .into_iter()
            .map(|c| measure(&self.config, report, &c))
            .filter(|(o, _)| o.mass >= self.config.wisp_mass)
            .collect();
        let ids = self.inherit(&found.iter().map(|(_, b)| b).collect::<Vec<_>>());
        let mut previous = Vec::with_capacity(found.len());
        for ((object, blocks), id) in found.iter_mut().zip(ids) {
            object.id = id;
            previous.push(Previous { id, weight: blocks.values().sum(), blocks: std::mem::take(blocks) });
        }
        self.objects = found.into_iter().map(|(o, _)| o).collect();
        self.objects.sort_by_key(|o| o.id);
        self.previous = previous;
        &self.objects
    }

    /// The id of each new object (given by its blocks): the previous object it shares the most weight with when that is
    /// ≥ 25 % of the previous object's, each previous id to one successor, else a fresh id.
    fn inherit(&mut self, blocks: &[&Weights]) -> Vec<u32> {
        let mut pairs: Vec<(f64, u32, usize)> = Vec::new();
        for (i, new) in blocks.iter().enumerate() {
            for old in &self.previous {
                let shared: f64 = new.iter().filter_map(|(k, w)| old.blocks.get(k).map(|o| w.min(*o))).sum();
                if shared > 0.0 && shared >= INHERIT * old.weight {
                    pairs.push((shared, old.id, i));
                }
            }
        }
        pairs.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
        let mut ids: Vec<Option<u32>> = vec![None; blocks.len()];
        let mut taken: Vec<u32> = Vec::new();
        for (_, id, i) in pairs {
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

/// One component's object (id 0 until the tracker names it) and its blocks' weights for the next frame's identity.
fn measure(config: &TrackerConfig, report: &Report, list: &[usize]) -> (Object, Weights) {
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

    let weights: Weights = list
        .iter()
        .map(|&i| {
            let in_block: f64 = sinks.iter().filter(|&&(_, b)| b == i).map(|&(k, _)| report.sinks[k].m).sum();
            (key(i), at(i).mass + in_block)
        })
        .collect();
    let object = Object {
        id: 0,
        mass,
        gas_mass,
        sink_mass,
        sinks: sinks.len(),
        centre,
        radius,
        tau_dyn,
        luminosity: per_time(sum(&|b| b.radiated)),
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
    (object, weights)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observe::SinkData;

    fn cfg() -> TrackerConfig {
        TrackerConfig { sigma_obj: 1e-3, block: 4, g: 1.0, wisp_mass: WISP_FRACTION * SUN_MASS_NOMINAL }
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
}
