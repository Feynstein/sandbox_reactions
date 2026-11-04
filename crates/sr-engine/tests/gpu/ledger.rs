//! M0-T57 — the ledger (contract §2.9, §2.12.2, §5.3's G-CONS; crates/sr-engine/src/ledger.rs): mass, energy and momentum in
//! f64 from the block summary's partials and the booked terms, cumulative since the ledger started, through `headless`'s
//! summary.json. Two runs of 500 steps of a collapsing disk (128 × 128, gravity and hydro, T 2.3 — U_th ≈ 0.1|W|₀ —, the cold
//! pressure off through `overrides` as scenes/cold_disk.json does, on the physics adapter, R12): at rest in the middle of a
//! *bounce* world, then
//! drifting out of a *leave* world's edge, so that mass escapes inside the 500 steps (the adversarial line: a leave run
//! where nothing reaches the edge cannot see the escaped term, so the escaped total is asserted and printed).
//!
//! Graded: the mass invariant constant to 10⁻⁶ of the mass; momentum (grid + sinks + escaped) to 10⁻⁶ of Σm|u|, Σm|u| read
//! from the run's final dump; the energy drift reported, not graded (G-CONS grades it over whole lives, M0-T106); the ledger's
//! grid mass and momentum against an f64 sum of the final dump's planes, which is independent of the summary and of the box.
//! The collapse at rest is symmetric, which is why its momentum holds: source-term gravity does not conserve momentum
//! (M0-V7 observation 1), so the drifting run's momentum is read as the ledger gives it and graded at the band the flag
//! measured, not at 10⁻⁶ — see `LEAVE_MOMENTUM`.
//!
//! M0-D64: the energy invariant graded on a hot compact remnant radiating into vacuum, where gravity's energy source must pay
//! for the mass HLL's diffusion moves across φ (§0.5 [M0-D64]) — G-CONS's bound, 10⁻³ |W| per τ_dyn, over 1,000 steps.
//!
//! M0-D71d: the momentum term `held` (§2.9 [M0-TG2]) — the scenes above switch the cold pressure off, so none holds a cell.
//! Graded where the held cells feel a net force (§0.5 [M0-D71]: the booking graded where it holds by construction): gas
//! driven against a held C/O block, gravity off, in a leave world. G-REST's M 900 SCF disk: its held cores feel almost no
//! net force. What the GPU booked there until M0-D151 was not its f32 slots' rounding but P2's f32 field's net force,
//! carried by the held core's own pull (2.0e-3 of Σm|u| over 1,000 steps on the RTX 4090; the twin's 1e-11; M0-D149's
//! measurement); since §1.4.2 [M0-D149] a held cell's Σg·Δt reads g_pull, the field of all but the held cells, and the disk
//! is graded on its `held` at or under the drift without held, the grid's own source-term error (§0.5 [M0-D151], the lead
//! at M0-D151: «Grade it (Recommended)»; measured 6.6e-6 under 2.0e-5) — the free gas's pull, not P2's rounding; its
//! invariant's drifts stay printed (§0.5 [M0-D71]).
//! A snapshot written before [M0-TG2] is restored with the hold's terms 0.

use std::path::PathBuf;

use serde_json::Value;
use sr_engine::calibrate::{cold_disk_scene, radius_holding, Matter};
use sr_engine::dump::{Dump, Sink};
use sr_engine::headless::{parse_args, run_scene_captured};
use sr_engine::ledger::{At, Ledger};
use sr_engine::state::{WorldConfig, N_TERMS, TERMS};
use sr_engine::summary::{Block, BlockSummary};
use sr_physics::registry::{Elements, Physics};

const STEPS: u64 = 500;
const N: u32 = 128;
/// G-CONS's band on the mass invariant and on momentum (§5.3), relative.
const BAND: f64 = 1e-6;
/// The drifting run's momentum band, relative to Σm|u|: a cloud that moves is held by source-term gravity's momentum error
/// (M0-V7 observation 1: 3.9 × 10⁻³ on sink_cloud), so the number is reported and bounded by the measured error, not 10⁻⁶.
/// 7e-3 since M0-D69 (§0.5 [M0-D69], the lead's «7e-3», 2026-10-28): gravity's ring reads the free-space φ past the world, and
/// this case's drift fell from 8.05 × 10⁻³ (the one-sided ring) to 5.54 × 10⁻³ (RTX 4090). What holds it is source-term
/// gravity on §0.5 [M0-D68]'s source, not the edge: the same disk with no edge near drifts 1.15 × 10⁻², and one whose rim
/// just touches the edge drifts as it does in a wider world (M0-T106 carries it). The one-sided ring's 8.05e-3 stays red.
const LEAVE_MOMENTUM: f64 = 7e-3;
/// The disk's T: U_th ≈ 0.1|W|₀ (M0-D66, the lead's «Disk T 2.3 + edge D», 2026-10-20). At M0-T57's T 0.05 the pressureless
/// disk falls to a point; once §0.5 [M0-D64]'s face work stopped the Σu·g source heating it there, it stays collapsed:
/// bounce's Σm|u| fell 9 × under an alike drift (2.8e-6), and leave's collapse beside the world's edge drifted 5.7e-2 (21 in a
/// 256-wide world, M0-D66's D). On §0.5 [M0-D68]'s source with M0-D69's ring, T 0.05 reads 2.06e-2 at x 116 — the disk cut
/// by the edge — and 1.04e-2 with no edge near (RTX 4090): source-term gravity, not the edge.
const DISK_T: f64 = 2.3;

fn scene(edge: &str, x: f64, v: f64, gravity: bool) -> String {
    format!(
        r#"{{"version": 1,
 "world": {{"width": {N}, "height": {N}, "edge": "{edge}"}},
 "slowdown": false,
 "switches": {{"gravity": {gravity}, "hydro": true, "heat": false, "reactions": false, "neutrinos": false, "sinks": false}},
 "objects": [
   {{"kind": "disk", "x": {x}, "y": 64, "a": 24, "sigma_c": 1.0, "t": {DISK_T}, "composition": {{"H": 1.0}}, "v": [{v}, 0]}}
 ],
 "overrides": {{"k1e": 0, "k2e": 0, "k1n": 0, "k2n": 0}}}}"#
    )
}

fn temp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sr-ledger-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A run's outputs: summary.json and the final dump.
struct Out {
    summary: Value,
    dump: Dump,
}

/// Runs `headless` in-process on `scene_json` for [`STEPS`] steps and reads back its summary.json and final dump.
fn run(name: &str, scene_json: &str) -> Out {
    run_for(name, scene_json, STEPS)
}

/// As [`run`], for `steps` steps.
fn run_for(name: &str, scene_json: &str, steps: u64) -> Out {
    let dir = temp(name);
    let file = dir.join("scene.json");
    std::fs::write(&file, scene_json).unwrap();
    let out = dir.join("out");
    let mut args: Vec<String> =
        ["--scene", &file.display().to_string(), "--steps", &steps.to_string(), "--out", &out.display().to_string()].map(String::from).to_vec();
    if let Ok(a) = std::env::var("SR_TEST_ADAPTER") {
        args.extend(["--adapter".to_string(), a]);
    }
    assert_eq!(sr_engine::headless::run(&args), 0, "headless");
    let summary: Value = serde_json::from_slice(&std::fs::read(out.join("summary.json")).unwrap()).unwrap();
    let dump = Dump::read(&out.join("dumps").join(format!("s_{steps}.bin"))).unwrap();
    Out { summary, dump }
}

fn num(v: &Value, path: &[&str]) -> f64 {
    path.iter().fold(v, |v, k| &v[*k]).as_f64().unwrap_or_else(|| panic!("summary.json's ledger has no number at {path:?}: {v}"))
}

impl Out {
    fn start(&self) -> &Value {
        &self.summary["ledger"]["start"]
    }

    fn end(&self) -> &Value {
        &self.summary["ledger"]["end"]
    }

    /// The final dump's plane `k`, summed in f64 over every cell.
    fn plane_sum(&self, k: usize) -> f64 {
        let cells = self.dump.world.cells();
        self.dump.planes[k * cells..(k + 1) * cells].iter().map(|&v| f64::from(v)).sum()
    }

    /// Σ |σu| over every cell: the momentum's scale, Σ m |u|.
    fn momentum_scale(&self) -> f64 {
        let cells = self.dump.world.cells();
        (0..cells).map(|i| f64::from(self.dump.planes[cells + i]).hypot(f64::from(self.dump.planes[2 * cells + i]))).sum()
    }
}

/// The three invariants' drifts from the start: mass relative to the mass, momentum (each axis) relative to Σm|u|, energy
/// relative to the energy's own scale (reported only).
struct Drift {
    mass: f64,
    momentum: f64,
    energy: f64,
}

fn drift(out: &Out) -> Drift {
    let (s, e) = (out.start(), out.end());
    let mass = num(e, &["mass", "invariant"]);
    let momentum = (0..2)
        .map(|k| {
            let at = |v: &Value| v["momentum"]["invariant"][k].as_f64().expect("a number");
            (at(e) - at(s)).powi(2)
        })
        .sum::<f64>()
        .sqrt();
    let scale = ["kinetic", "thermal", "cold", "potential"].iter().map(|k| num(e, &["energy", k]).abs()).sum::<f64>();
    Drift {
        mass: (mass - num(s, &["mass", "invariant"])).abs() / mass,
        momentum: momentum / out.momentum_scale(),
        energy: (num(e, &["energy", "invariant"]) - num(s, &["energy", "invariant"])) / scale,
    }
}

/// The ledger's grid terms against the final dump's planes: the ledger sums the box's blocks and the floor outside, the dump
/// sums every cell — each in f64 from f32 values.
fn assert_grid_matches_the_state(out: &Out) {
    let grid = num(out.end(), &["mass", "grid"]);
    let state = out.plane_sum(0);
    assert!((grid - state).abs() <= BAND * state, "grid mass {grid} against the dump's {state}");
    let scale = out.momentum_scale();
    for (k, plane) in [(0usize, 1usize), (1, 2)] {
        let ledger = out.end()["momentum"]["grid"][k].as_f64().expect("a number");
        let state = out.plane_sum(plane);
        assert!((ledger - state).abs() <= BAND * scale, "grid momentum {k}: {ledger} against the dump's {state} (Σm|u| {scale})");
    }
}

#[test]
fn bounce_500_steps_the_mass_and_the_momentum_hold() {
    let out = run("bounce", &scene("bounce", 64.0, 0.0, true));
    let d = drift(&out);
    println!(
        "bounce: steps {}..{}, mass drift {:.3e}, momentum drift {:.3e} of Σm|u| = {:.3e}, energy drift {:.3e} (reported), escaped {}",
        num(out.start(), &["step"]),
        num(out.end(), &["step"]),
        d.mass,
        d.momentum,
        out.momentum_scale(),
        d.energy,
        num(out.end(), &["mass", "escaped"]),
    );
    assert_eq!(num(out.start(), &["step"]), 1.0, "the ledger starts after the run's first step");
    assert_eq!(num(out.end(), &["step"]), STEPS as f64);
    assert!(out.momentum_scale() > 1.0, "the disk collapsed: Σm|u| {}", out.momentum_scale());
    assert_eq!(num(out.end(), &["mass", "escaped"]), 0.0, "bounce: nothing escapes");
    assert!(d.mass <= BAND, "mass invariant drift {:.3e}", d.mass);
    assert!(d.momentum <= BAND, "momentum drift {:.3e} of Σm|u|", d.momentum);
    assert_grid_matches_the_state(&out);
}

#[test]
fn leave_500_steps_collapsing_with_mass_leaving_the_mass_holds_and_the_escaped_term_is_in_it() {
    // The disk sits across the right edge and drifts out of it: gas crosses from the first steps on.
    let out = run("leave", &scene("leave", 116.0, 0.5, true));
    let d = drift(&out);
    let (escaped, total) = (num(out.end(), &["mass", "escaped"]), num(out.end(), &["mass", "invariant"]));
    println!(
        "leave: escaped mass {escaped:.6e} of {total:.6e} ({:.3e}), momentum escaped {:?}, mass drift {:.3e}, momentum drift {:.3e} of Σm|u| = {:.3e}, \
         energy drift {:.3e} (reported), escaped energy {:.6e}",
        escaped / total,
        out.end()["momentum"]["escaped"],
        d.mass,
        d.momentum,
        out.momentum_scale(),
        d.energy,
        num(out.end(), &["energy", "escaped"]),
    );
    // The escaped term is what makes this run worth reading: mass left, by an amount the band could not hide.
    assert!(escaped > 1e-3 * total, "mass left through the edge: {escaped} of {total}");
    assert!(d.mass <= BAND, "mass invariant drift {:.3e} (escaped {escaped})", d.mass);
    assert!(d.momentum <= LEAVE_MOMENTUM, "momentum drift {:.3e} of Σm|u|", d.momentum);
    assert_grid_matches_the_state(&out);
    let grid_start = num(out.start(), &["mass", "grid"]);
    assert!(num(out.end(), &["mass", "grid"]) < grid_start - escaped * 0.99, "the grid lost what escaped");
}

/// The same drifting disk with gravity off (hydro only): the momentum is then graded tightly. Two terms stay outside the sum
/// by the contract's own words and are accounted for, not hidden: the momentum a vacuum reset removes is booked nowhere
/// (§2.7 [M0-T19]) — at most `vacuum_reset` × the fastest gas, both read from the run — and the f32 round-off, 10⁻⁶ of Σm|u|.
/// The escaped term stands out of the sum by 10⁴ times that bound: leaving it out is off by 3.6 of Σm|u|.
#[test]
fn leave_500_steps_without_gravity_the_momentum_holds_to_what_a_vacuum_reset_removes() {
    let out = run("leave-flat", &scene("leave", 116.0, 0.5, false));
    let d = drift(&out);
    let cells = out.dump.world.cells();
    let u_max = (0..cells)
        .filter(|&i| out.dump.planes[i] > 1e-6)
        .map(|i| f64::from(out.dump.planes[cells + i]).hypot(f64::from(out.dump.planes[2 * cells + i])) / f64::from(out.dump.planes[i]))
        .fold(0.0, f64::max);
    let reset = num(out.end(), &["mass", "vacuum_reset"]);
    let (escaped, total) = (num(out.end(), &["mass", "escaped"]), num(out.end(), &["mass", "invariant"]));
    let px = out.end()["momentum"]["escaped"][0].as_f64().expect("a number");
    let bound = BAND + reset * u_max / out.momentum_scale();
    println!(
        "leave without gravity: escaped mass {escaped:.6e} of {total:.6e}, escaped x momentum {px:.6e}, vacuum_reset {reset:.3e} × u_max {u_max:.3} \
         → bound {bound:.3e}; mass drift {:.3e}, momentum drift {:.3e} of Σm|u| = {:.3e}",
        d.mass,
        d.momentum,
        out.momentum_scale(),
    );
    assert!(escaped > 1e-3 * total && px > 1e-3 * out.momentum_scale(), "mass and momentum left: {escaped}, {px}");
    assert!(d.mass <= BAND, "mass invariant drift {:.3e}", d.mass);
    assert!(d.momentum <= bound, "momentum drift {:.3e} of Σm|u| against {bound:.3e}", d.momentum);
    assert_grid_matches_the_state(&out);
}

/// G-CONS's energy bound (§5.3): the invariant's drift at most 10⁻³ |W| per τ_dyn.
const CONS_PER_TAU: f64 = 1e-3;

/// M0-D64: a hot compact disk of helium with some carbon and oxygen (the Sun-like life's remnant, made small), heat on and
/// burning off, in a leave world: it contracts, radiates into vacuum and HLL's diffusion moves mass up its steep φ every step.
/// The energy invariant (§2.9) is graded over 1,000 steps at G-CONS's bound, τ_dyn = √(R₉₀³ / G M) of the end state (G = 1,
/// the tracker's rule) — Σu·g alone (the source before §0.5 [M0-D64]) read 0.171 |W| per sim time unit here.
#[test]
fn a_hot_compact_remnant_radiating_into_vacuum_keeps_the_energy_invariant_within_g_cons() {
    let scene = format!(
        r#"{{"version": 1,
 "world": {{"width": {N}, "height": {N}, "edge": "leave"}},
 "slowdown": false,
 "switches": {{"gravity": true, "hydro": true, "heat": true, "reactions": false, "neutrinos": false, "sinks": false}},
 "objects": [
   {{"kind": "disk", "x": 64, "y": 64, "a": 9, "sigma_c": 8.8, "t": 50, "composition": {{"He": 0.9, "C": 0.05, "O": 0.05}}}}
 ]}}"#
    );
    let steps = 1000;
    let out = run_for("remnant", &scene, steps);
    let (s, e) = (out.start(), out.end());
    let cells = out.dump.world.cells();
    let vac = Physics::shipped().unwrap().get("sigma_vac").expect("sigma_vac is a physics.json key");
    let sigma = &out.dump.planes[..cells];
    let mass: f64 = sigma.iter().filter(|&&v| f64::from(v) >= vac).map(|&v| f64::from(v)).sum();
    let r90 = radius_holding(sigma, out.dump.world.width as usize, vac, 0.9);
    let tau_dyn = (r90.powi(3) / mass).sqrt();
    let w0 = num(s, &["energy", "potential"]).abs();
    let span = num(e, &["sim_time"]) - num(s, &["sim_time"]);
    let d_inv = num(e, &["energy", "invariant"]) - num(s, &["energy", "invariant"]);
    let per_tau = d_inv.abs() / w0 / (span / tau_dyn);
    let term = |v: &Value, k: &str| num(v, &["energy", k]);
    println!(
        "remnant: steps {}..{}, sim time {span:.3}, R₉₀ {r90:.3}, M {mass:.1}, τ_dyn {tau_dyn:.4}, |W|₀ {w0:.4e}; Δinvariant {d_inv:.4e} → \
         {per_tau:.3e} |W| per τ_dyn (bound {CONS_PER_TAU:.0e}); radiated {:.4e}, floor_added {:.4e}, escaped {:.4e}, ΔW {:.4e}",
        num(s, &["step"]),
        num(e, &["step"]),
        term(e, "radiated") - term(s, "radiated"),
        term(e, "floor_added") - term(s, "floor_added"),
        term(e, "escaped") - term(s, "escaped"),
        term(e, "potential") - term(s, "potential"),
    );
    assert_eq!(num(e, &["step"]), steps as f64);
    // The adversarial line: a remnant that neither shines nor contracts cannot show the leak.
    assert!(term(e, "radiated") - term(s, "radiated") > 1e-2 * w0, "the remnant radiated");
    assert!(per_tau <= CONS_PER_TAU, "energy invariant drift {per_tau:.3e} |W| per τ_dyn (Δ {d_inv:.4e}, |W|₀ {w0:.4e}, τ_dyn {tau_dyn:.4})");
}

fn block(mass: f32, px: f32, py: f32, kinetic: f32) -> Block {
    let mut b = Block {
        mass,
        mx: 0.0,
        my: 0.0,
        momentum: [px, py],
        kinetic,
        thermal: 0.0,
        cold: 0.0,
        pressure: 0.0,
        species: [0.0; 10],
        phi_m: 0.0,
        max_cell: 0,
        max_sigma: 0.0,
        max_t: 0.0,
        max_x: [0.0; 10],
        cold_ratio: 0.0,
        radiated: 0.0,
        nuclear: [0.0; 7],
        neutrino_lost: 0.0,
        neutrino_deposited: 0.0,
        unbound: 0.0,
        sinks: 0,
    };
    b.thermal = 0.5 * mass;
    b.cold = 0.25 * mass;
    b.phi_m = -2.0 * mass;
    b
}

fn index(name: &str) -> usize {
    TERMS.iter().position(|t| t.name == name).unwrap()
}

/// The terms of §2.9 on a hand-made summary (no GPU): the grid from the blocks and the floor outside the box, the sinks'
/// mass, momentum and kinetic energy, the booked terms cumulative across two updates, each invariant's signs, and a restart
/// that zeroes them. NOT PROVEN (synthetic) for anything past the arithmetic.
#[test]
fn the_terms_and_the_invariants_on_a_hand_made_summary() {
    let physics = Physics::shipped().unwrap();
    let floor = physics.get("sigma_floor").unwrap();
    let world = WorldConfig::new(64, 64).unwrap();
    // A box of 8 × 8 cells: two blocks of 4 × 4, then two more.
    let summary = |blocks: Vec<Block>, sinks: Vec<Sink>| BlockSummary { origin: [8, 8], size: [8, 8], columns: 2, blocks, sinks };
    let blocks = vec![block(3.0, 1.0, -2.0, 0.5), block(5.0, 0.5, 0.0, 1.5), block(7.0, 0.0, 0.25, 2.0), block(9.0, 0.0, 0.0, 0.0)];
    let sink = Sink { x: 10.0, y: 10.0, vx: 0.5, vy: -0.25, m: 100.0 };
    let outside = floor * (64.0 * 64.0 - 64.0);
    let mut ledger = Ledger::new(world, &physics);
    let a = At { step: 1, sim_time: 0.5 };
    let start = ledger.restart(&summary(blocks.clone(), vec![sink]), a).clone();
    assert_eq!(start.mass.grid, 24.0 + outside);
    assert_eq!((start.mass.sinks, start.momentum.sinks), (100.0, [50.0, -25.0]));
    assert_eq!(start.momentum.grid, [1.5, -1.75]);
    assert_eq!(start.energy.kinetic, 4.0 + 0.5 * 100.0 * (0.25 + 0.0625));
    assert_eq!((start.energy.thermal, start.energy.cold, start.energy.potential), (12.0, 6.0, -24.0));
    assert_eq!(start.mass.escaped, 0.0);

    // Two intervals, each booking some escaped mass, momentum and energy, some erased, some floor energy: cumulative.
    let mut booked = [0f64; N_TERMS];
    booked[index("escaped.mass")] = 2.0;
    booked[index("escaped.mom_x")] = 0.5;
    booked[index("escaped.mom_y")] = -0.25;
    booked[index("escaped.energy")] = 3.0;
    booked[index("mass.erased")] = 1.0;
    booked[index("mass.painted")] = 4.0;
    booked[index("energy.floor_added")] = 0.5;
    booked[index("energy.tools.heated")] = 0.25;
    booked[index("energy.tools.erased")] = -1.0;
    booked[index("energy.swallowed")] = 0.125;
    // The accumulators in `booked` are ignored: the summaries' own are the ledger's.
    booked[index("radiated")] = 1e6;
    let mut acc = blocks.clone();
    acc[0].radiated = 0.5;
    acc[1].radiated = 0.25;
    acc[2].nuclear = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.5];
    acc[3].neutrino_lost = 2.0;
    acc[3].neutrino_deposited = 0.5;
    ledger.update(&summary(acc.clone(), vec![sink]), &booked, At { step: 2, sim_time: 1.0 });
    let end = ledger.update(&summary(acc, vec![sink]), &booked, At { step: 3, sim_time: 1.5 }).clone();
    assert_eq!((end.mass.escaped, end.mass.erased, end.mass.painted), (4.0, 2.0, 8.0));
    assert_eq!(end.momentum.escaped, [1.0, -0.5]);
    assert_eq!(end.mass.invariant(), end.mass.grid + 100.0 + 4.0 + 2.0 - 8.0);
    assert_eq!((end.energy.radiated, end.energy.nuclear_released, end.energy.neutrino_lost, end.energy.neutrino_deposited), (1.5, 1.0, 4.0, 1.0));
    assert_eq!((end.energy.escaped, end.energy.floor_added, end.energy.tools, end.energy.swallowed), (6.0, 1.0, -1.5, 0.25));
    // §2.9: K + U + C + W + radiated + ν lost − ν deposited + escaped + swallowed − nuclear − floor − tools.
    let e = &end.energy;
    let want = e.kinetic + 12.0 + 6.0 - 24.0 + 1.5 + 4.0 - 1.0 + 6.0 + 0.25 - 1.0 - 1.0 + 1.5;
    assert!((e.invariant() - want).abs() < 1e-12, "{} against {want}", e.invariant());
    assert_eq!(end.momentum.invariant(), [1.5 + 50.0 + 1.0, -1.75 - 25.0 - 0.5]);
    // The json carries the start, the end and the three invariants.
    let json = ledger.json();
    assert_eq!(json["start"]["step"], 1);
    assert_eq!(json["end"]["step"], 3);
    assert_eq!(json["end"]["mass"]["escaped"], 4.0);
    assert_eq!(json["end"]["mass"]["invariant"].as_f64().unwrap(), end.mass.invariant());

    // A restart is a new start: every booked term and accumulator zero again.
    let again = ledger.restart(&summary(blocks, Vec::new()), At { step: 9, sim_time: 4.0 }).clone();
    assert_eq!((again.mass.escaped, again.mass.erased, again.energy.radiated, again.energy.tools), (0.0, 0.0, 0.0, 0.0));
    assert_eq!(ledger.start(), Some(&again));
}

/// The hold's four booked terms (§2.9 [M0-TG2]): P8's, then P4's.
const HELD_TERMS: [&str; 4] = ["held.mom_x", "held.mom_y", "held.spared.mom_x", "held.spared.mom_y"];
/// The held disk's span: G-REST's step 1,000.
const HELD_STEPS: u64 = 1_000;

/// M0-D71d: G-REST's M 900 SCF disk (`calibrate`'s cold C/O disk, the shipped K's, 64 × 64 bounce, gravity and hydro) through
/// `headless`, 1,000 steps, so cells are held and the hold books momentum. summary.json's `momentum.held` is the ledger's —
/// the four `held` terms of the observers' snapshot beside the final dump, cumulative since the start — and its invariant
/// is grid + sinks + escaped + held. The drift with and without `held` is printed (§0.5 [M0-D71]); `held` is graded at or
/// under the drift without it (§0.5 [M0-D151]: the free gas's pull, not P2's f32 net force, which read 2.0e-3 against 2.0e-5).
#[test]
fn the_held_disk_writes_the_ledgers_held_into_summary_json() {
    let (physics, elements) = (Physics::shipped().unwrap(), Elements::shipped().unwrap());
    let (scene, _) = cold_disk_scene(900.0, Matter::CarbonOxygen, &physics, &elements, &[]).unwrap().expect("an equilibrium");
    let dir = temp("held");
    let out = dir.join("out");
    let mut args: Vec<String> =
        ["--scene", "disk:m900", "--steps", &HELD_STEPS.to_string(), "--out", &out.display().to_string()].map(String::from).to_vec();
    if let Ok(a) = std::env::var("SR_TEST_ADAPTER") {
        args.extend(["--adapter".to_string(), a]);
    }
    let (code, lines) = run_scene_captured(parse_args(&args).unwrap(), scene, None);
    assert_eq!(code, 0, "headless: {lines:?}");
    let summary: Value = serde_json::from_slice(&std::fs::read(out.join("summary.json")).unwrap()).unwrap();
    let final_dump = out.join("dumps").join(format!("s_{HELD_STEPS}.bin"));
    let out = Out { summary, dump: Dump::read(&final_dump).unwrap() };
    let snapshot: Value = serde_json::from_slice(&std::fs::read(final_dump.with_extension("observers.json")).unwrap()).unwrap();
    let booked = |name: &str| {
        let v = &snapshot["ledger"]["booked"][index(name)];
        v.as_str().expect("an exact f64").parse::<f64>().unwrap()
    };
    let (s, e) = (out.start(), out.end());
    let pair = |v: &Value, k: &str| [0, 1].map(|i| v["momentum"][k][i].as_f64().unwrap_or_else(|| panic!("momentum.{k}: {v}")));
    let (held, grid, inv0, inv1) = (pair(e, "held"), pair(e, "grid"), pair(s, "invariant"), pair(e, "invariant"));
    let scale = out.momentum_scale();
    let drift = (inv1[0] - inv0[0]).hypot(inv1[1] - inv0[1]) / scale;
    let without = (inv1[0] - held[0] - inv0[0] + pair(s, "held")[0]).hypot(inv1[1] - held[1] - inv0[1] + pair(s, "held")[1]) / scale;
    println!(
        "SR-LEDGER-HELD disk M 900: steps {}..{}, held ({:.4e}, {:.4e}) = {:.3e} of Σm|u| {scale:.4e}; grid ({:.4e}, {:.4e}); \
         invariant drift {drift:.3e} of Σm|u| with held, {without:.3e} without (held {} the drift without); snapshot's held ({:.4e}, \
         {:.4e}) + spared ({:.4e}, {:.4e})",
        num(s, &["step"]),
        num(e, &["step"]),
        held[0],
        held[1],
        held[0].hypot(held[1]) / scale,
        grid[0],
        grid[1],
        if held[0].hypot(held[1]) / scale <= without { "at or under" } else { "over" },
        booked(HELD_TERMS[0]),
        booked(HELD_TERMS[1]),
        booked(HELD_TERMS[2]),
        booked(HELD_TERMS[3]),
    );
    assert_eq!(num(e, &["step"]), HELD_STEPS as f64);
    assert_eq!(held, [booked(HELD_TERMS[0]) + booked(HELD_TERMS[2]), booked(HELD_TERMS[1]) + booked(HELD_TERMS[3])], "summary.json's held");
    for (k, &inv) in inv1.iter().enumerate() {
        let sum = ["grid", "sinks", "escaped", "held"].iter().map(|t| pair(e, t)[k]).sum::<f64>();
        assert_eq!(inv, sum, "the invariant, axis {k}: grid + sinks + escaped + held");
    }
    let taken = held[0].hypot(held[1]) / scale;
    assert!(taken <= without, "held {taken:.3e} of Σm|u| over the drift without held {without:.3e}: the held cores book more than the free gas's pull");
    assert_grid_matches_the_state(&out);
}

/// M0-D71d: an H disk (a = 16, Σc = 1, T 2) driven at (1, 0.3) into a held C/O block (4 × 64, Σ 20 ≥ σ_hold, T_floor, the
/// shipped K's), gravity off, a leave world, 500 steps: the gas is turned back by the block's walls and the impulse it gives
/// the block is booked `held` (§2.3.4 [M0-TG2]). Graded: momentum (grid + sinks + escaped + held) to 1e-6 of Σm|u| plus what
/// a vacuum reset removes, as the gravity-off leave case; `held` over 100 times that bound, so leaving it out of the
/// invariant breaks it (measured on the RTX 4090: held 0.26 of Σm|u|, drift 6.9e-6 against 2.2e-4). The block is 4 wide so
/// its mass within r ≤ 24 of any of its cells (at most 4 × 49 × 20 = 3,920, the disk's gas on top) stays under m_tov_sb
/// (4,853.17), the hold's gate (§0.5 [M0-T130]); 16 wide (≈ 15,700) it was never held (M0-T130's claim, 2026-11-04).
#[test]
fn gas_driven_against_a_held_block_books_held_and_the_momentum_holds() {
    let scene = format!(
        r#"{{"version": 1,
 "world": {{"width": {N}, "height": {N}, "edge": "leave"}},
 "slowdown": false,
 "switches": {{"gravity": false, "hydro": true, "heat": false, "reactions": false, "neutrinos": false, "sinks": false}},
 "objects": [
   {{"kind": "rect", "x0": 76, "y0": 32, "w": 4, "h": 64, "sigma": 20.0, "t": 0.05, "composition": {{"C": 0.5, "O": 0.5}}, "v": [0, 0]}},
   {{"kind": "disk", "x": 44, "y": 60, "a": 16, "sigma_c": 1.0, "t": 2.0, "composition": {{"H": 1.0}}, "v": [1.0, 0.3]}}
 ]}}"#
    );
    let out = run("held-wall", &scene);
    let d = drift(&out);
    let (s, e) = (out.start(), out.end());
    let pair = |v: &Value, k: &str| [0, 1].map(|i| v["momentum"][k][i].as_f64().unwrap_or_else(|| panic!("momentum.{k}: {v}")));
    let (held, held0) = (pair(e, "held"), pair(s, "held"));
    let scale = out.momentum_scale();
    let taken = (held[0] - held0[0]).hypot(held[1] - held0[1]) / scale;
    let cells = out.dump.world.cells();
    let u_max = (0..cells)
        .filter(|&i| out.dump.planes[i] > 1e-6)
        .map(|i| f64::from(out.dump.planes[cells + i]).hypot(f64::from(out.dump.planes[2 * cells + i])) / f64::from(out.dump.planes[i]))
        .fold(0.0, f64::max);
    let reset = num(e, &["mass", "vacuum_reset"]);
    let bound = BAND + reset * u_max / scale;
    println!(
        "SR-LEDGER-HELD wall: held ({:.4e}, {:.4e}) = {taken:.3e} of Σm|u| {scale:.4e}, escaped {}; vacuum_reset {reset:.3e} × u_max \
         {u_max:.3} → bound {bound:.3e}; mass drift {:.3e}, momentum drift {:.3e} of Σm|u|",
        held[0],
        held[1],
        e["momentum"]["escaped"],
        d.mass,
        d.momentum,
    );
    assert!(taken > 100.0 * bound, "held {taken:.3e} of Σm|u|: not over 100 × the bound {bound:.3e} — the block took no impulse");
    assert!(d.mass <= BAND, "mass invariant drift {:.3e}", d.mass);
    assert!(d.momentum <= bound, "momentum drift {:.3e} of Σm|u| against {bound:.3e}", d.momentum);
    assert_grid_matches_the_state(&out);
}

/// A snapshot written before [M0-TG2] (§2.9, §0.5 [M0-D76]): its `booked` list ends before the hold's four terms and its
/// records carry no `momentum.held` — restored, they read 0, every other term as written; today's snapshot keeps `held`
/// through the round trip; a list of any other length is refused. NOT PROVEN (synthetic): a hand-made ledger.
#[test]
fn a_snapshot_from_before_the_hold_restores_with_the_held_terms_zero() {
    let physics = Physics::shipped().unwrap();
    let world = WorldConfig::new(64, 64).unwrap();
    let summary = BlockSummary { origin: [8, 8], size: [8, 8], columns: 2, blocks: vec![block(3.0, 1.0, -2.0, 0.5); 4], sinks: Vec::new() };
    let mut ledger = Ledger::new(world, &physics);
    ledger.restart(&summary, At { step: 1, sim_time: 0.5 });
    let mut booked = [0f64; N_TERMS];
    booked[index("escaped.mom_x")] = 0.5;
    for (k, name) in HELD_TERMS.iter().enumerate() {
        booked[index(name)] = 0.25 * (k + 1) as f64;
    }
    let end = ledger.update(&summary, &booked, At { step: 2, sim_time: 1.0 }).clone();
    assert_eq!(end.momentum.held, [0.25 + 0.75, 0.5 + 1.0]);
    assert_eq!(end.momentum.invariant(), [4.0 + 0.5 + 1.0, -8.0 + 1.5]);
    assert_eq!(ledger.json()["end"]["momentum"]["held"], serde_json::json!([1.0, 1.5]));

    // Today's snapshot round-trips the hold.
    let now = ledger.snapshot();
    let back = Ledger::restore(world, &physics, &now).unwrap();
    assert_eq!(back.last(), Some(&end));

    // The same ledger as a pre-[M0-TG2] tree wrote it: 37 terms, no `momentum.held` in either record.
    let first = index(HELD_TERMS[0]);
    assert_eq!((first, N_TERMS - first), (37, 4), "the hold's terms are TERMS' last four");
    let mut old = now.clone();
    old["booked"].as_array_mut().unwrap().truncate(first);
    for r in ["start", "last"] {
        old[r]["momentum"].as_object_mut().unwrap().remove("held");
    }
    let back = Ledger::restore(world, &physics, &old).unwrap();
    let last = back.last().unwrap();
    assert_eq!(last.momentum.held, [0.0, 0.0]);
    assert_eq!((last.momentum.grid, last.momentum.escaped), (end.momentum.grid, end.momentum.escaped));
    assert_eq!(last.momentum.invariant(), [4.0 + 0.5, -8.0]);
    assert_eq!((last.mass.clone(), last.energy.clone()), (end.mass.clone(), end.energy.clone()));
    let restored = back.snapshot();
    let tail: Vec<&str> = restored["booked"].as_array().unwrap()[first..].iter().map(|v| v.as_str().unwrap()).collect();
    assert_eq!(tail, ["0.0"; 4], "the hold's terms restored");
    for k in 0..first {
        assert_eq!(restored["booked"][k], now["booked"][k], "booked term {k}");
    }

    // Any other length is refused.
    for len in [first - 1, first + 2] {
        let mut bad = now.clone();
        bad["booked"].as_array_mut().unwrap().truncate(len);
        assert!(Ledger::restore(world, &physics, &bad).is_err(), "a {len}-term booked list restored");
    }
}

