//! The ledger (m0_contrat.md §2.9, §2.12.2; M0-T57): mass, energy and momentum in f64, from the block summary's
//! fixed-order partials and the booked terms, cumulative since the ledger (re)started.
//!
//! What the state holds comes from a [`BlockSummary`] — the blocks' f32 sums added in f64 in the blocks' order, the box's
//! blocks row-major, so the same summary gives the same bits (§1.3.4) — and the sinks it carries. What left the grid by a
//! route (escaped, erased, vacuum_reset, painted, …) comes from the booking: the caller reads its totals
//! ([`take_booked`]: [`book_totals`]' fixed-order f64 sums, each booking buffer cleared after) and hands them over; the
//! ledger keeps the f64 cumulative, the slots hold one interval. The frame accumulators (radiated, nuclear, neutrino) are
//! the summary's own: each [`Step::summarise`](crate::step::Step::summarise) sums them and clears them, so every summary
//! holds what accrued since the one before.
//!
//! Readings the contract leaves open, declared here:
//! - The summary covers the active box; outside it every cell is the floor state (§1.3.3: vacuum, static), whose mass is
//!   `sigma_floor` a cell, so the grid mass adds `sigma_floor × (cells outside the box)` — a box that grows over floor gas
//!   would otherwise move the grid mass by that much. The floor gas's energy outside the box is not added (≈ 10⁻⁹ of a
//!   star's).
//! - The sinks' kinetic energy ½ M v² is part of `kinetic` (§2.9 names no sinks' kinetic term; M0-V7 observation 2). The
//!   potential W is ½ Σ m φ over the cells, which is what the summary holds: the sinks' own m φ is not in it.
//! - `cleared` is booked but not in the mass invariant: a clear restarts the ledger (§2.9), so it never reaches it.
//! - The ledger starts at a summary, not before step 0: the box and φ exist only after the first step, so a run's ledger
//!   starts after its first step and `start.step` says which.

use serde_json::{json, Value};
use sr_physics::observe::tracker::{exact, read_exact, read_list, read_whole};
use sr_physics::registry::Physics;

use crate::state::{book_totals, Booking, WorldConfig, BOOK_GROUPS, N_TERMS, TERMS};
use crate::summary::BlockSummary;

/// Where a record was taken: the step count and the sim time at the summary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct At {
    pub step: u64,
    pub sim_time: f64,
}

/// The ledger at one summary: every §2.9 term, cumulative since the ledger started, in f64.
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    pub at: At,
    pub mass: Mass,
    pub energy: Energy,
    pub momentum: Momentum,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Mass {
    pub grid: f64,
    pub sinks: f64,
    pub escaped: f64,
    pub painted: f64,
    pub erased: f64,
    pub preset_dropped: f64,
    pub vacuum_reset: f64,
    pub cleared: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Energy {
    /// The cells' and the sinks' ½ M v².
    pub kinetic: f64,
    pub thermal: f64,
    pub cold: f64,
    /// ½ Σ m φ.
    pub potential: f64,
    pub radiated: f64,
    pub neutrino_lost: f64,
    pub neutrino_deposited: f64,
    pub escaped: f64,
    pub swallowed: f64,
    /// Σ q ω Δm, signed: the seven nuclear groups.
    pub nuclear_released: f64,
    pub floor_added: f64,
    /// The tools' net energy: painted, erased, heated, cooled, preset, cleared.
    pub tools: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Momentum {
    pub grid: [f64; 2],
    pub sinks: [f64; 2],
    pub escaped: [f64; 2],
}

impl Mass {
    /// §2.9: grid + sinks + escaped + erased + vacuum_reset − painted − preset_dropped.
    pub fn invariant(&self) -> f64 {
        self.grid + self.sinks + self.escaped + self.erased + self.vacuum_reset - self.painted - self.preset_dropped
    }
}

impl Energy {
    /// §2.9: kinetic + thermal + cold + W + radiated + neutrino_lost − neutrino_deposited + escaped + swallowed −
    /// nuclear_released − floor_added − tools.
    pub fn invariant(&self) -> f64 {
        self.kinetic + self.thermal + self.cold + self.potential + self.radiated + self.neutrino_lost - self.neutrino_deposited
            + self.escaped
            + self.swallowed
            - self.nuclear_released
            - self.floor_added
            - self.tools
    }
}

impl Momentum {
    /// §2.9: grid + sinks + escaped, each axis.
    pub fn invariant(&self) -> [f64; 2] {
        std::array::from_fn(|k| self.grid[k] + self.sinks[k] + self.escaped[k])
    }
}

impl Record {
    /// The record as summary.json's `ledger.start` / `ledger.end` (§2.12.2): every term and the three invariants.
    pub fn json(&self) -> Value {
        self.json_with(|x| json!(x))
    }

    /// [`Record::json`]'s layout with every f64 written by `num` — the observers' snapshot writes [`exact`] (§0.5 [M0-D76]).
    fn json_with(&self, num: impl Fn(f64) -> Value) -> Value {
        let (m, e, p) = (&self.mass, &self.energy, &self.momentum);
        let pair = |v: [f64; 2]| json!([num(v[0]), num(v[1])]);
        json!({
            "step": self.at.step,
            "sim_time": num(self.at.sim_time),
            "mass": {
                "grid": num(m.grid), "sinks": num(m.sinks), "escaped": num(m.escaped), "painted": num(m.painted),
                "erased": num(m.erased), "preset_dropped": num(m.preset_dropped), "vacuum_reset": num(m.vacuum_reset),
                "cleared": num(m.cleared), "invariant": num(m.invariant()),
            },
            "energy": {
                "kinetic": num(e.kinetic), "thermal": num(e.thermal), "cold": num(e.cold), "potential": num(e.potential),
                "radiated": num(e.radiated), "neutrino_lost": num(e.neutrino_lost),
                "neutrino_deposited": num(e.neutrino_deposited), "escaped": num(e.escaped), "swallowed": num(e.swallowed),
                "nuclear_released": num(e.nuclear_released), "floor_added": num(e.floor_added), "tools": num(e.tools),
                "invariant": num(e.invariant()),
            },
            "momentum": {"grid": pair(p.grid), "sinks": pair(p.sinks), "escaped": pair(p.escaped), "invariant": pair(p.invariant())},
        })
    }

    /// The record the snapshot wrote with [`exact`] numbers (the invariants are recomputed, not read).
    fn restore(v: &Value) -> Result<Record, String> {
        let n = |group: &str, k: &str| read_exact(&v[group][k], &format!("ledger.{group}.{k}"));
        let pair = |k: &str| -> Result<[f64; 2], String> {
            let at = format!("ledger.momentum.{k}");
            Ok([read_exact(&v["momentum"][k][0], &at)?, read_exact(&v["momentum"][k][1], &at)?])
        };
        Ok(Record {
            at: At { step: read_whole(v, "step")?, sim_time: read_exact(&v["sim_time"], "ledger.sim_time")? },
            mass: Mass {
                grid: n("mass", "grid")?,
                sinks: n("mass", "sinks")?,
                escaped: n("mass", "escaped")?,
                painted: n("mass", "painted")?,
                erased: n("mass", "erased")?,
                preset_dropped: n("mass", "preset_dropped")?,
                vacuum_reset: n("mass", "vacuum_reset")?,
                cleared: n("mass", "cleared")?,
            },
            energy: Energy {
                kinetic: n("energy", "kinetic")?,
                thermal: n("energy", "thermal")?,
                cold: n("energy", "cold")?,
                potential: n("energy", "potential")?,
                radiated: n("energy", "radiated")?,
                neutrino_lost: n("energy", "neutrino_lost")?,
                neutrino_deposited: n("energy", "neutrino_deposited")?,
                escaped: n("energy", "escaped")?,
                swallowed: n("energy", "swallowed")?,
                nuclear_released: n("energy", "nuclear_released")?,
                floor_added: n("energy", "floor_added")?,
                tools: n("energy", "tools")?,
            },
            momentum: Momentum { grid: pair("grid")?, sinks: pair("sinks")?, escaped: pair("escaped")? },
        })
    }
}

/// The ledger of a run: the booked terms' f64 cumulative, and the records at its start and at the latest summary.
pub struct Ledger {
    world: WorldConfig,
    sigma_floor: f64,
    /// The booked terms' cumulative totals since the start ([`TERMS`]' order; the accumulators' entries stay 0 — they
    /// come from the summaries).
    booked: [f64; N_TERMS],
    /// The frame accumulators' cumulative: radiated, nuclear (the seven groups), neutrino lost, neutrino deposited.
    radiated: f64,
    nuclear: f64,
    neutrino_lost: f64,
    neutrino_deposited: f64,
    start: Option<Record>,
    last: Option<Record>,
}

impl Ledger {
    pub fn new(world: WorldConfig, physics: &Physics) -> Ledger {
        let sigma_floor = physics.get("sigma_floor").expect("physics.json holds sigma_floor");
        Ledger {
            world,
            sigma_floor,
            booked: [0.0; N_TERMS],
            radiated: 0.0,
            nuclear: 0.0,
            neutrino_lost: 0.0,
            neutrino_deposited: 0.0,
            start: None,
            last: None,
        }
    }

    /// (Re)starts the ledger at `summary`: every booked term and accumulator back to 0, the state as it stands the start.
    /// The summary's accumulators are what accrued before the start, and are dropped with the booked terms the caller
    /// read to clear the booking.
    pub fn restart(&mut self, summary: &BlockSummary, at: At) -> &Record {
        self.booked = [0.0; N_TERMS];
        (self.radiated, self.nuclear, self.neutrino_lost, self.neutrino_deposited) = (0.0, 0.0, 0.0, 0.0);
        let record = self.record(summary, at);
        self.start = Some(record.clone());
        self.last.insert(record)
    }

    /// Adds one interval: `booked` the booked terms' totals since the last call ([`take_booked`]), `summary` the state and
    /// the accumulators at its end. Returns the record there.
    pub fn update(&mut self, summary: &BlockSummary, booked: &[f64; N_TERMS], at: At) -> &Record {
        assert!(self.start.is_some(), "the ledger is updated after it started");
        for (total, (add, term)) in self.booked.iter_mut().zip(booked.iter().zip(&TERMS)) {
            if !term.accumulator {
                *total += add;
            }
        }
        self.accumulate(summary);
        let record = self.record(summary, at);
        self.last.insert(record)
    }

    pub fn start(&self) -> Option<&Record> {
        self.start.as_ref()
    }

    pub fn last(&self) -> Option<&Record> {
        self.last.as_ref()
    }

    /// The ledger for the observers' snapshot beside a dump (§0.5 [M0-D76]): the booked terms' and the accumulators'
    /// cumulative, the start and the latest record, every f64 [`exact`].
    pub fn snapshot(&self) -> Value {
        let record = |r: &Option<Record>| r.as_ref().map_or(Value::Null, |r| r.json_with(exact));
        json!({
            "booked": self.booked.iter().map(|&x| exact(x)).collect::<Vec<_>>(),
            "radiated": exact(self.radiated),
            "nuclear": exact(self.nuclear),
            "neutrino_lost": exact(self.neutrino_lost),
            "neutrino_deposited": exact(self.neutrino_deposited),
            "start": record(&self.start),
            "last": record(&self.last),
        })
    }

    /// The ledger [`Ledger::snapshot`] wrote, in `world` under `physics` (the run's own).
    pub fn restore(world: WorldConfig, physics: &Physics, v: &Value) -> Result<Ledger, String> {
        let mut ledger = Ledger::new(world, physics);
        let booked = read_list(v, "booked")?;
        if booked.len() != N_TERMS {
            return Err(format!("ledger.booked: {} terms, expected {N_TERMS}", booked.len()));
        }
        for (slot, x) in ledger.booked.iter_mut().zip(booked) {
            *slot = read_exact(x, "ledger.booked")?;
        }
        ledger.radiated = read_exact(&v["radiated"], "ledger.radiated")?;
        ledger.nuclear = read_exact(&v["nuclear"], "ledger.nuclear")?;
        ledger.neutrino_lost = read_exact(&v["neutrino_lost"], "ledger.neutrino_lost")?;
        ledger.neutrino_deposited = read_exact(&v["neutrino_deposited"], "ledger.neutrino_deposited")?;
        let record = |k: &str| if v[k].is_null() { Ok(None) } else { Record::restore(&v[k]).map(Some) };
        (ledger.start, ledger.last) = (record("start")?, record("last")?);
        Ok(ledger)
    }

    /// summary.json's `ledger` (§2.12.2): `{start, end}`, both empty before the ledger started.
    pub fn json(&self) -> Value {
        match (&self.start, &self.last) {
            (Some(s), Some(l)) => json!({"start": s.json(), "end": l.json()}),
            _ => json!({"start": {}, "end": {}}),
        }
    }

    /// The summary's frame accumulators added to the cumulative: block by block, in the blocks' order, in f64.
    fn accumulate(&mut self, summary: &BlockSummary) {
        for b in &summary.blocks {
            self.radiated += f64::from(b.radiated);
            self.nuclear += b.nuclear.iter().map(|&v| f64::from(v)).sum::<f64>();
            self.neutrino_lost += f64::from(b.neutrino_lost);
            self.neutrino_deposited += f64::from(b.neutrino_deposited);
        }
    }

    /// The record at `summary`: the state from its blocks and sinks, the booked terms from the cumulative.
    fn record(&self, summary: &BlockSummary, at: At) -> Record {
        let t = |name: &str| self.booked[TERMS.iter().position(|t| t.name == name).expect("a booked term of that name")];
        let (mut grid, mut px, mut py, mut kinetic, mut thermal, mut cold, mut phi_m) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        for b in &summary.blocks {
            grid += f64::from(b.mass);
            px += f64::from(b.momentum[0]);
            py += f64::from(b.momentum[1]);
            kinetic += f64::from(b.kinetic);
            thermal += f64::from(b.thermal);
            cold += f64::from(b.cold);
            phi_m += f64::from(b.phi_m);
        }
        let cells = f64::from(self.world.width) * f64::from(self.world.height);
        let inside = f64::from(summary.size[0]) * f64::from(summary.size[1]);
        grid += self.sigma_floor * (cells - inside);
        let (mut sink_mass, mut sink_px, mut sink_py, mut sink_kinetic) = (0.0, 0.0, 0.0, 0.0);
        for s in &summary.sinks {
            sink_mass += s.m;
            sink_px += s.m * s.vx;
            sink_py += s.m * s.vy;
            sink_kinetic += 0.5 * s.m * (s.vx * s.vx + s.vy * s.vy);
        }
        let tools = ["painted", "erased", "heated", "cooled", "preset", "cleared"].iter().map(|n| t(&format!("energy.tools.{n}"))).sum();
        Record {
            at,
            mass: Mass {
                grid,
                sinks: sink_mass,
                escaped: t("escaped.mass"),
                painted: t("mass.painted"),
                erased: t("mass.erased"),
                preset_dropped: t("mass.preset_dropped"),
                vacuum_reset: t("mass.vacuum_reset"),
                cleared: t("mass.cleared"),
            },
            energy: Energy {
                kinetic: kinetic + sink_kinetic,
                thermal,
                cold,
                potential: 0.5 * phi_m,
                radiated: self.radiated,
                neutrino_lost: self.neutrino_lost,
                neutrino_deposited: self.neutrino_deposited,
                escaped: t("escaped.energy"),
                swallowed: t("energy.swallowed"),
                nuclear_released: self.nuclear,
                floor_added: t("energy.floor_added"),
                tools,
            },
            momentum: Momentum { grid: [px, py], sinks: [sink_px, sink_py], escaped: [t("escaped.mom_x"), t("escaped.mom_y")] },
        }
    }
}

/// Reads every booked term's total since the booking was last cleared ([`book_totals`]: f64, term by term, row by row, slot
/// by slot) and clears the booking buffers that hold booked terms; the frame accumulators' buffers are left to the
/// summary, which clears them ([`Step::summarise`](crate::step::Step::summarise)). Blocks on the device (native and
/// headless only, §6.3).
pub fn take_booked(device: &wgpu::Device, queue: &wgpu::Queue, booking: &Booking) -> [f64; N_TERMS] {
    let totals = book_totals(booking.world(), &booking.readback(device, queue));
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("ledger.clear") });
    for (g, group) in BOOK_GROUPS.iter().enumerate() {
        if !TERMS[group.terms.start].accumulator {
            encoder.clear_buffer(booking.group(g), 0, None);
        }
    }
    queue.submit([encoder.finish()]);
    totals
}
