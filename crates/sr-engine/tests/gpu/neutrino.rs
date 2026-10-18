//! M0-T45 — neutrino cooling and the iron gate on the CPU twin (contract §1.6.2's N_Fe, §1.8.5's latch trigger, §2.5.3, R8,
//! R13): `sr_physics::reference::react`'s cooling term, the source S_ν it keeps and N_Fe's gate; the twin's cases touch no
//! adapter. One scope, `neutrino`.
//!
//! The oracles are written here from the contract. §2.5.3: ε_ν = A_ν (T/T_ν)^m per unit mass for T ≥ T_ν, else 0, with
//! A_ν = 10⁻³, m = 8 (physics.json) and T_ν = 166 (M0-TC's: §0.5 [M0-D50] ships 93, so the cooling's cases pin 166 through
//! [`cooling`], M0-D55); booked `neutrino_lost` (energy per area, Σ ε_ν Δt in a cell held at T);
//! S_ν = ε_νΣ in cells with X_n ≥ 0.5. §1.6.2: N_Fe, Fe → n, q = −1.14 Q_H, a = 0, f ≡ 1, g = (Y_eΣ/Σ_N − 1) for Y_eΣ > Σ_N
//! (Σ_N = 160, p = 1) and 0 at or below it; §1.8.5: the latch is set when any cell's N_Fe rate is > 0. The cold pressure is
//! off (the four K's 0) so T = μE/Σ for a cell at rest, and a record is alone when the others are disabled in the registry text.
//!
//! Tolerances: the booked loss at a held T within 1e-6 relative (the contract's number for the closed form); the unpinned
//! cooling against the ODE's closed form T⁻⁷ = T₀⁻⁷ + 7 μ A_ν t / T_ν⁸ within 5e-3 of the temperature drop at `burn_dt` 10⁻³
//! (explicit first-order sub-steps: measured 1.8e-3); energy and mass conservation within 1e-9 / 1e-12.
//!
//! M0-T47 — the same oracles on the GPU (`gpu::`, cases `gpu_…`, the physics adapter, R12): P6's cooling, `neutrino_lost`
//! (booking plane 7 of the reactions group), S_ν in `side.s_nu` and the latch flag, P6 alone with P1, P8 and P9 ([`Rig`]) as
//! burn.rs's, and one frame run (P4 on) where the physics opens N_Fe's gate at a step k. The f32 tolerances, fixed before
//! the first GPU run: the booked loss at a held T within 1e-4 (f32's `T/T_ν` to the eighth power: 8 × 6e-8 × a few), T held
//! within 1e-5, S_ν within 1e-4, the ODE's temperature drop within 1e-2, E against what was booked within 1e-4 of it; the
//! fractions Fe → n within 1e-6 (f32 at X ≈ 1), q ΔX_n Σ within 1e-4; "bit for bit" and the flag exact.

use sr_physics::reference::hydro::State;
use sr_physics::reference::react::{Pass, React, TestFlags, N_GROUPS};
use sr_physics::registry::{Elements, Physics, Reactions, GROUPS, N_SPECIES};

const NS: usize = N_SPECIES;
/// §2.5.3 / physics.json; T_ν as M0-TC wrote it, the one the closed forms below are written at (pinned by [`cooling`]).
const A_NU: f64 = 1e-3;
const T_NU: f64 = 166.0;
const M_NU: i32 = 8;
/// §2.5.1: Σ_N, and Q_H for N_Fe's q.
const SIGMA_N: f64 = 160.0;
const Q_H: f64 = 8_700.0;
const Q_NFE: f64 = -1.14 * Q_H;

fn elements() -> Elements {
    Elements::shipped().unwrap()
}

/// The shipped constants with the cold pressure off, and `extra` overrides.
fn physics(extra: &[(&str, f64)]) -> Physics {
    let mut keys = vec![("k1e", 0.0), ("k2e", 0.0), ("k1n", 0.0), ("k2n", 0.0)];
    keys.extend_from_slice(extra);
    Physics::shipped().unwrap().with(&keys).unwrap()
}

/// [`physics`] with T_ν pinned at [`T_NU`] (G-COLL's pattern): the law A_ν (T/T_ν)^m graded at the T_ν its closed forms and
/// temperatures (170 … 500 above it, 100 below) were written for; the gate, N_Fe and the latch read the shipped T_ν.
fn cooling(extra: &[(&str, f64)]) -> Physics {
    let mut keys = vec![("t_nu", T_NU)];
    keys.extend_from_slice(extra);
    physics(&keys)
}

/// assets/reactions.json with only `ids` enabled (none: cooling alone).
fn only(ids: &[&str], physics: &Physics) -> Reactions {
    let text = include_str!("../../../../assets/reactions.json");
    let mut root: serde_json::Value = serde_json::from_str(text).unwrap();
    for rec in root["records"].as_array_mut().unwrap() {
        let on = ids.contains(&rec["id"].as_str().unwrap());
        rec["enabled"] = serde_json::Value::Bool(on);
    }
    Reactions::parse(&root.to_string(), &elements(), physics).unwrap()
}

/// The twin with the neutrino switch as given.
fn react(physics: &Physics, reactions: &Reactions, pin: bool, neutrinos: bool) -> React {
    React::new(physics, &elements(), reactions, TestFlags { pin_temperature: pin }).unwrap().with_neutrinos(neutrinos)
}

fn x_of(parts: &[(&str, f64)]) -> [f64; NS] {
    let e = elements();
    let mut x = [0.0; NS];
    for &(k, v) in parts {
        x[e.index_of(k).unwrap_or_else(|| panic!("species {k}"))] = v;
    }
    x
}

fn inv_mu(x: &[f64; NS]) -> f64 {
    elements().composition(x).inv_mu
}

fn y_e(x: &[f64; NS]) -> f64 {
    elements().composition(x).y_e
}

/// A world of `cells`, each (Σ, T, X) at rest.
fn world(width: usize, height: usize, cells: &[(f64, f64, [f64; NS])]) -> State {
    let n = width * height;
    assert_eq!(cells.len(), n);
    let mut planes = vec![0.0; (4 + NS) * n];
    for (i, &(sigma, t, x)) in cells.iter().enumerate() {
        planes[i] = sigma;
        planes[3 * n + i] = sigma * t * inv_mu(&x);
        for s in 0..NS {
            planes[(4 + s) * n + i] = x[s];
        }
    }
    State::new(width, height, planes)
}

fn x_at(st: &State, i: usize) -> [f64; NS] {
    std::array::from_fn(|s| st.get(4 + s, i))
}

fn temp(st: &State, i: usize) -> f64 {
    st.get(3, i) / st.get(0, i) / inv_mu(&x_at(st, i))
}

fn frac(st: &State, species: &str, i: usize) -> f64 {
    st.get(4 + elements().index_of(species).unwrap(), i)
}

/// ε_ν by the contract's formula.
fn eps_nu(t: f64) -> f64 {
    if t < T_NU {
        0.0
    } else {
        A_NU * (t / T_NU).powi(M_NU)
    }
}

fn rel(got: f64, want: f64) -> f64 {
    (got - want).abs() / want.abs().max(f64::MIN_POSITIVE)
}

fn group(name: &str) -> usize {
    GROUPS.iter().position(|g| *g == name).unwrap()
}

/// The twin cases: cooling, S_ν, N_Fe's gate and the latch.
mod neutrino {
    use super::*;

    /// A cell held at T (pin) loses exactly ε_ν Σ Δt — booked `neutrino_lost` within 1e-6 of A_ν (T/T_ν)^8 Σ Δt — at four
    /// temperatures and two densities; T, Σ and the composition stay as they were and no nuclear group is touched.
    #[test]
    fn cooling_closed_form_at_pinned_t() {
        let physics = cooling(&[]);
        let r = react(&physics, &only(&[], &physics), true, true);
        let fe = x_of(&[("Fe", 1.0)]);
        let dt = 0.3;
        for &(sigma, t) in &[(50.0, 170.0), (50.0, 200.0), (50.0, 300.0), (120.0, 500.0)] {
            let mut st = world(1, 1, &[(sigma, t, fe)]);
            let pass = r.step(&mut st, dt);
            let want = A_NU * (t / T_NU).powi(M_NU) * sigma * dt;
            let got = pass.neutrino_lost[0];
            println!("pinned T = {t}, Σ = {sigma}: lost {got:.6e}, the closed form {want:.6e} (Δ {:.2e})", rel(got, want));
            assert!(rel(got, want) <= 1e-6, "T = {t}: lost {got}, the closed form {want}");
            assert!((pass.neutrino_lost_total() - got).abs() <= 1e-15 * got, "the total is the one cell's");
            assert!((temp(&st, 0) - t).abs() <= 1e-9 * t, "T pinned: {}", temp(&st, 0));
            assert_eq!(x_at(&st, 0), fe, "cooling moves no mass");
            assert_eq!(st.get(0, 0), sigma);
            assert!((0..N_GROUPS).all(|g| pass.nuclear_total(g) == 0.0), "no nuclear group is booked");
            assert!(!pass.latch);
        }
    }

    /// None below T_ν: a cell at 0.999999 T_ν, at 100 and at 1 loses nothing and is left bit for bit; at 1.000001 T_ν it
    /// loses A_ν Σ Δt; with the switch off (the default of [`React::new`]) a cell at 500 is left bit for bit too.
    #[test]
    fn none_below_t_nu_and_none_with_the_switch_off() {
        let physics = cooling(&[]);
        let reactions = only(&[], &physics);
        let on = react(&physics, &reactions, false, true);
        let fe = x_of(&[("Fe", 1.0)]);
        for t in [T_NU * (1.0 - 1e-6), 100.0, 1.0] {
            let mut st = world(1, 1, &[(50.0, t, fe)]);
            let before = st.planes.clone();
            let pass = on.step(&mut st, 0.5);
            assert_eq!(pass.neutrino_lost[0], 0.0, "T = {t} < T_ν: lost {}", pass.neutrino_lost[0]);
            assert_eq!(pass.s_nu[0], 0.0);
            assert_eq!(st.planes, before, "T = {t} < T_ν: the cell is left bit for bit");
        }
        let mut st = world(1, 1, &[(50.0, T_NU * (1.0 + 1e-6), fe)]);
        let pass = react(&physics, &reactions, true, true).step(&mut st, 0.5);
        let want = A_NU * 50.0 * 0.5;
        println!("just above T_ν: lost {:.6e}, A_ν Σ Δt {want:.6e}", pass.neutrino_lost[0]);
        assert!(rel(pass.neutrino_lost[0], want) <= 1e-6 * 10.0, "at T_ν the loss is A_ν Σ Δt: {}", pass.neutrino_lost[0]);
        let mut st = world(1, 1, &[(50.0, 500.0, fe)]);
        let before = st.planes.clone();
        let pass = react(&physics, &reactions, false, false).step(&mut st, 0.5);
        assert_eq!(pass.neutrino_lost_total(), 0.0);
        assert_eq!(st.planes, before, "neutrinos off: a hot cell is left bit for bit");
    }

    /// S_ν = ε_νΣ only where the cell leaves the pass with X_n ≥ 0.5: a neutron-rich world of seven cells at T = 300 held
    /// (X_n 0.9, 0.5, 0.4999, 0.1, none, the vacuum, and a cell cooler than T_ν with X_n 0.9) — S_ν is ε_νΣ in the first two
    /// (the boundary included) and 0 in the rest, while `neutrino_lost` is ε_νΣΔt in every gas cell hotter than T_ν; the
    /// deposited share P7 may take (the sum of S_ν Δt) never exceeds the emitted loss.
    #[test]
    fn source_only_where_x_n_is_half_or_more() {
        let physics = cooling(&[]);
        let r = react(&physics, &only(&[], &physics), true, true);
        let dt = 0.25;
        let sigma = 80.0;
        let cells = [
            (sigma, 300.0, x_of(&[("n", 0.9), ("Fe", 0.1)])),
            (sigma, 300.0, x_of(&[("n", 0.5), ("Fe", 0.5)])),
            (sigma, 300.0, x_of(&[("n", 0.4999), ("Fe", 0.5001)])),
            (sigma, 300.0, x_of(&[("n", 0.1), ("Fe", 0.9)])),
            (sigma, 300.0, x_of(&[("Fe", 1.0)])),
            (1e-9, 300.0, x_of(&[("n", 1.0)])),
            (sigma, 100.0, x_of(&[("n", 0.9), ("Fe", 0.1)])),
        ];
        let mut st = world(7, 1, &cells);
        let pass = r.step(&mut st, dt);
        let power = eps_nu(300.0) * sigma;
        println!("S_ν = {:?}, ε_νΣ = {power:.6e}", pass.s_nu);
        for (i, &want_source) in [true, true, false, false, false, false, false].iter().enumerate() {
            let got = pass.s_nu[i];
            if want_source {
                assert!(rel(got, power) <= 1e-6, "cell {i}: S_ν {got}, ε_νΣ {power}");
            } else {
                assert_eq!(got, 0.0, "cell {i}: S_ν {got} where X_n < 0.5, the vacuum, or T < T_ν");
            }
            let lost_want = if i < 5 { power * dt } else { 0.0 };
            assert!((pass.neutrino_lost[i] - lost_want).abs() <= 1e-6 * power * dt, "cell {i}: lost {}, ε_νΣΔt {lost_want}", pass.neutrino_lost[i]);
        }
        let emitted: f64 = pass.s_nu.iter().sum::<f64>() * dt;
        assert!(emitted <= pass.neutrino_lost_total(), "S_ν over the step ({emitted}) is more than was lost ({})", pass.neutrino_lost_total());
    }

    /// Cooling alone, T free: iron at T₀ = 500, Σ = 50, `burn_dt` 10⁻³, over t = 20 — the ODE dT/dt = −μ A_ν (T/T_ν)^8 gives
    /// T⁻⁷ = T₀⁻⁷ + 7 μ A_ν t/T_ν⁸ (T stays above T_ν, checked); the temperature drop is within 5e-3 of the closed form's, the
    /// energy taken from E is exactly the booked loss (1e-9), no mass moved, sub-cycled (cooling binds `burn_dt`).
    #[test]
    fn cooling_unpinned_follows_the_ode() {
        let physics = cooling(&[("burn_dt", 1e-3)]);
        let r = react(&physics, &only(&[], &physics), false, true);
        let fe = x_of(&[("Fe", 1.0)]);
        let (sigma, t0, t): (f64, f64, f64) = (50.0, 500.0, 20.0);
        let mu = 1.0 / inv_mu(&fe);
        let t_end = (t0.powi(-7) + 7.0 * mu * A_NU * t / T_NU.powi(8)).powf(-1.0 / 7.0);
        assert!(t_end > T_NU, "the case stays above T_ν: {t_end}");
        let mut st = world(1, 1, &[(sigma, t0, fe)]);
        let e0 = st.get(3, 0);
        let pass = r.step(&mut st, t);
        let got = temp(&st, 0);
        let (drop_got, drop_want) = (t0 - got, t0 - t_end);
        println!("unpinned: T {t0} → {got:.4} (closed form {t_end:.4}), drop Δ {:.2e}, {} sub-steps, max ΔT/T {:.2e}", rel(drop_got, drop_want), pass.substeps, pass.max_dt_rel);
        assert!(rel(drop_got, drop_want) <= 5e-3, "T fell by {drop_got}, the closed form {drop_want}");
        let taken = e0 - st.get(3, 0);
        assert!(rel(taken, pass.neutrino_lost[0]) <= 1e-9, "E lost {taken}, booked {}", pass.neutrino_lost[0]);
        assert_eq!(x_at(&st, 0), fe);
        assert!(pass.substeps > 100, "cooling did not bind the sub-step: {} sub-steps", pass.substeps);
        assert!(pass.max_dt_rel <= 1e-3 * (1.0 + 1e-12), "ΔT/T {} > burn_dt", pass.max_dt_rel);
        assert_eq!(pass.unfinished, 0);
    }

    /// Energy closes with burning and cooling together: a hot neon cell, free, over a short step — E gains exactly the nuclear
    /// heat booked minus the neutrinos lost (1e-9 of the heat), and the loss is 10⁻³ of that heat at least, so a pass that
    /// forgot to take it from E is caught.
    #[test]
    fn energy_closes_with_burning_and_cooling() {
        let physics = physics(&[]);
        let r = react(&physics, &only(&["Ne_burn"], &physics), false, true);
        let mut st = world(1, 1, &[(50.0, 300.0, x_of(&[("Ne", 1.0)]))]);
        let e0 = st.get(3, 0);
        let pass = r.step(&mut st, 0.05);
        let heat: f64 = (0..N_GROUPS).map(|g| pass.nuclear_total(g)).sum();
        let lost = pass.neutrino_lost_total();
        let de = st.get(3, 0) - e0;
        println!("Ne cell: heat {heat:.6e}, lost {lost:.6e} ({:.2e} of it), ΔE {de:.6e}", lost / heat);
        assert!(heat > 0.0 && lost > 1e-3 * heat, "the case needs both terms: heat {heat}, lost {lost}");
        assert!((de - (heat - lost)).abs() <= 1e-9 * heat, "ΔE {de} ≠ heat {heat} − lost {lost}");
        assert!(temp(&st, 0) > 300.0, "the cell heated");
    }

    /// N_Fe's gate on Y_eΣ: with only N_Fe enabled and T held, iron (Y_e = 26/56) does nothing just under Σ_N/Y_e (0.1 %) and
    /// converts just over it (0.1 %); at Σ = 160 ± 0.02 and Σ = 200 — above Σ_N, between Σ_N and Σ_N/Y_e, where a gate read on Σ
    /// would fire — it does nothing; a half-iron half-hydrogen cell (Y_e ≈ 0.73) opens at 218.7, not at Σ_N/Y_e(Fe). The gate
    /// closed leaves the cell bit for bit and the latch unset; open, Fe falls, the latch is set.
    #[test]
    fn n_fe_gate_reads_y_e_sigma() {
        let physics = physics(&[]);
        let r = react(&physics, &only(&["N_Fe"], &physics), true, false);
        let fe = x_of(&[("Fe", 1.0)]);
        let mixed = x_of(&[("Fe", 0.5), ("H", 0.5)]);
        let (ye_fe, ye_mixed) = (y_e(&fe), y_e(&mixed));
        println!("Y_e(Fe) = {ye_fe:.5}, Σ_N/Y_e = {:.3}; Y_e(mixed) = {ye_mixed:.5}, Σ_N/Y_e = {:.3}", SIGMA_N / ye_fe, SIGMA_N / ye_mixed);
        assert!(ye_fe < 0.47 && SIGMA_N / ye_fe > 340.0, "iron's Y_e is not 0.5");
        let cases = [
            (fe, SIGMA_N * 0.9999 / ye_fe, false),
            (fe, SIGMA_N * 1.0001 / ye_fe, true),
            (fe, SIGMA_N - 0.02, false),
            (fe, SIGMA_N + 0.02, false),
            (fe, 200.0, false),
            (mixed, SIGMA_N * 0.9999 / ye_mixed, false),
            (mixed, SIGMA_N * 1.0001 / ye_mixed, true),
            (mixed, SIGMA_N / ye_fe * 1.0001, true),
        ];
        for (k, &(x, sigma, open)) in cases.iter().enumerate() {
            let mut st = world(1, 1, &[(sigma, 200.0, x)]);
            let before = st.planes.clone();
            let pass = r.step(&mut st, 1.0);
            let moved = frac(&st, "Fe", 0) < x[elements().index_of("Fe").unwrap()];
            println!("case {k}: Σ = {sigma:.3}, Y_eΣ = {:.3} → {}", y_e(&x) * sigma, if moved { "converts" } else { "closed" });
            assert_eq!((moved, pass.latch), (open, open), "case {k} (Σ = {sigma}, Y_eΣ = {}): gate {open}", y_e(&x) * sigma);
            if !open {
                assert_eq!(st.planes, before, "case {k}: a closed gate leaves the cell bit for bit");
                assert_eq!(pass.nuclear_total(group("N")), 0.0);
            }
        }
    }

    /// N_Fe converts and absorbs: iron at Σ = 500 (g = 0.45), free, over 1 t.u. — Fe → n mass for mass in the planes
    /// (ΔX_Fe = −ΔX_n within 1e-12, Σ X = 1), nothing else moves, the heat booked in group N is q ΔX_n Σ with q = −1.14 Q_H
    /// (within 1e-5) and negative, no other group is booked, E gains exactly that (1e-9), T falls, the latch is set.
    #[test]
    fn n_fe_absorbs_and_conserves_mass() {
        let physics = physics(&[]);
        let r = react(&physics, &only(&["N_Fe"], &physics), false, false);
        let sigma = 500.0;
        let mut st = world(1, 1, &[(sigma, 1000.0, x_of(&[("Fe", 1.0)]))]);
        let e0 = st.get(3, 0);
        let pass = r.step(&mut st, 1.0);
        let (dfe, dn) = (frac(&st, "Fe", 0) - 1.0, frac(&st, "n", 0));
        let sum: f64 = x_at(&st, 0).iter().sum();
        println!("N_Fe: ΔX_Fe {dfe:.6e}, ΔX_n {dn:.6e}, booked {:.6e}, ΔE {:.6e}, T {:.2}", pass.nuclear_total(group("N")), st.get(3, 0) - e0, temp(&st, 0));
        assert!(dn > 1e-4, "the record converted {dn}");
        assert!((dfe + dn).abs() <= 1e-12, "Fe lost {dfe}, n gained {dn}");
        assert!((sum - 1.0).abs() <= 1e-12 && x_at(&st, 0).iter().all(|&v| v >= 0.0), "ΣX = {sum}");
        let others: f64 = x_at(&st, 0).iter().sum::<f64>() - frac(&st, "Fe", 0) - dn;
        assert!(others.abs() <= 1e-12, "another species moved: {others}");
        let booked = pass.nuclear_total(group("N"));
        let want = Q_NFE * dn * sigma;
        assert!(booked < 0.0 && rel(booked, want) <= 1e-5, "booked {booked}, q ΔX_n Σ {want}");
        assert!((0..N_GROUPS).filter(|&g| g != group("N")).all(|g| pass.nuclear_total(g) == 0.0), "only N is booked");
        assert!(rel(st.get(3, 0) - e0, booked) <= 1e-9, "E gained {}, booked {booked}", st.get(3, 0) - e0);
        assert!(temp(&st, 0) < 1000.0, "absorbing cooled the cell");
        assert!(pass.latch);
    }

    /// The latch's P6 trigger reads any cell: a world of five — iron under the gate, iron over it, hydrogen at high Σ (no iron),
    /// neutron matter at high Σ, vacuum — sets the latch exactly when the iron over the gate is there, in any position; and
    /// the world without that cell leaves it unset.
    #[test]
    fn latch_is_any_cells_n_fe_rate() {
        let physics = physics(&[]);
        let r = react(&physics, &only(&["N_Fe"], &physics), true, true);
        let fe = x_of(&[("Fe", 1.0)]);
        let closed = [
            (200.0, 200.0, fe),
            (900.0, 200.0, x_of(&[("H", 1.0)])),
            (900.0, 200.0, x_of(&[("n", 1.0)])),
            (1e-9, 200.0, fe),
        ];
        let open = (SIGMA_N * 1.05 / y_e(&fe), 200.0, fe);
        let mut st = world(4, 1, &closed);
        assert!(!r.step(&mut st, 1.0).latch, "no cell's N_Fe rate is > 0, the latch stays unset");
        for pos in 0..=closed.len() {
            let mut cells = closed.to_vec();
            cells.insert(pos, open);
            let mut st = world(5, 1, &cells);
            let pass: Pass = r.step(&mut st, 1.0);
            assert!(pass.latch, "the open cell at position {pos} did not set the latch");
            assert!(frac(&st, "Fe", pos) < 1.0, "the open cell converted nothing");
            for i in (0..5).filter(|&i| i != pos) {
                assert_eq!(x_at(&st, i), cells[i].2, "cell {i} moved, its gate is closed");
            }
        }
    }
}

/// The twin cases' counterparts on the GPU.
mod gpu {
    use super::*;
    use crate::device;
    use sr_engine::gpu::Gpu;
    use sr_engine::scene::{EdgeMode, TestSettings};
    use sr_engine::state::{Booking, SideFields, State as GpuState, WorldConfig, BOOK_GROUPS, CHANNELS, N_CHANNELS, TERMS};
    use sr_engine::step::boxfit::BoxMode;
    use sr_engine::step::latch::FramePlan;
    use sr_engine::step::react::{ReactReport, ReactSetup};
    use sr_engine::step::{EosGpu, Passes, Step};

    const SIDE: usize = 64;
    const AT: usize = 8;

    /// A buffer's words, read back through a staging buffer.
    fn words(gpu: &Gpu, buffer: &wgpu::Buffer, count: u64) -> Vec<u32> {
        let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("neutrino.readback"),
            size: count * 4,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = gpu.device.create_command_encoder(&Default::default());
        encoder.copy_buffer_to_buffer(buffer, 0, &staging, 0, count * 4);
        gpu.queue.submit([encoder.finish()]);
        staging.slice(..).map_async(wgpu::MapMode::Read, |r| r.unwrap());
        gpu.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        let out = staging.slice(..).get_mapped_range().expect("mapped").chunks_exact(4).map(|c| u32::from_le_bytes(c.try_into().unwrap())).collect();
        staging.unmap();
        out
    }

    /// P6 on a block of cells at (8, 8) of a 64 × 64 world, vacuum around it, with P1, P8 and P9; each [`Rig::pass`] is one
    /// step at the Δt the test writes as the Δt record's Δt_{n+1} (burn.rs's rig, with the side fields and the latch flag).
    struct Rig<'g> {
        gpu: &'g Gpu,
        state: GpuState,
        booking: Booking,
        side: SideFields,
        step: Step,
        width: usize,
        height: usize,
    }

    impl<'g> Rig<'g> {
        fn new(gpu: &'g Gpu, physics: &Physics, reactions: &Reactions, pin: bool, neutrinos: bool, block: &State) -> Rig<'g> {
            assert!(block.width <= SIDE - 2 * AT && block.height <= SIDE - 2 * AT);
            let world = WorldConfig::new(SIDE as u32, SIDE as u32).unwrap();
            let state = GpuState::new(&gpu.device, world).unwrap();
            let booking = Booking::new(&gpu.device, world).unwrap();
            let side = SideFields::new(&gpu.device, world).unwrap();
            let eos = EosGpu::new(&gpu.device, physics, &elements());
            let passes = Passes { mode: BoxMode::Fit, gravity: None, hydro: None };
            let test = TestSettings { pin_temperature: pin, ..TestSettings::default() };
            let react = ReactSetup { reactions, test, neutrinos, side: &side };
            let step = Step::with_reactions(&gpu.device, &state, &booking, &eos, physics, &passes, None, Some(&react)).unwrap();
            let rig = Rig { gpu, state, booking, side, step, width: block.width, height: block.height };
            rig.load(block);
            rig.step.run(&gpu.device, &gpu.queue, 1);
            rig.load(block);
            rig
        }

        fn at(&self, i: usize) -> usize {
            (AT + i / self.width) * SIDE + AT + i % self.width
        }

        /// `block` uploaded (rounded to f32), vacuum around it, the booking and the latch flag cleared; S_ν is not — P6 must
        /// rewrite it.
        fn load(&self, block: &State) {
            assert_eq!((block.width, block.height), (self.width, self.height));
            let n = SIDE * SIDE;
            let mut planes = vec![0f32; (4 + NS) * n];
            for i in 0..n {
                planes[i] = 1e-8;
                planes[3 * n + i] = 1e-8 * 0.05 * 2.0;
                planes[4 * n + i] = 1.0;
            }
            for i in 0..self.width * self.height {
                for c in 0..4 + NS {
                    planes[c * n + self.at(i)] = block.get(c, i) as f32;
                }
            }
            self.state.upload(&self.gpu.queue, &planes).unwrap();
            let mut encoder = self.gpu.device.create_command_encoder(&Default::default());
            self.booking.clear(&mut encoder);
            self.gpu.queue.submit([encoder.finish()]);
            self.gpu.queue.write_buffer(self.step.latch_flag(), 0, &0u32.to_le_bytes());
        }

        /// One step whose P6 runs over `dt`; its counters.
        fn pass(&self, dt: f64) -> ReactReport {
            let (device, queue) = (&self.gpu.device, &self.gpu.queue);
            queue.write_buffer(self.step.dt_record(), 4, &(dt as f32).to_le_bytes());
            self.step.run(device, queue, 1);
            assert_eq!(self.step.read_dt(device, queue).dt, dt as f32, "P1 took the Δt written");
            self.step.read_react(device, queue).expect("the step runs P6")
        }

        /// The block's cells as the GPU holds them, in f64.
        fn planes(&self) -> State {
            let all = self.state.readback(&self.gpu.device, &self.gpu.queue);
            let (n, cells) = (SIDE * SIDE, self.width * self.height);
            let mut planes = vec![0f64; (4 + NS) * cells];
            for i in 0..cells {
                for c in 0..4 + NS {
                    planes[c * cells + i] = f64::from(all[c * n + self.at(i)]);
                }
            }
            State::new(self.width, self.height, planes)
        }

        /// The block's slots of the booked term `name` since the last load.
        fn booked(&self, name: &str) -> Vec<f64> {
            let book = self.booking.readback(&self.gpu.device, &self.gpu.queue);
            let t = TERMS.iter().position(|t| t.name == name).unwrap_or_else(|| panic!("term {name}"));
            let g = BOOK_GROUPS.iter().position(|g| g.terms.contains(&t)).unwrap();
            let before: usize = BOOK_GROUPS[..g].iter().map(|g| g.terms.len() * g.domain.slots(self.state.world())).sum();
            let plane = before + (t - BOOK_GROUPS[g].terms.start) * SIDE * SIDE;
            (0..self.width * self.height).map(|i| f64::from(book[plane + self.at(i)])).collect()
        }

        /// The block's `nuclear.<group>` slots, summed over the block, by group.
        fn nuclear(&self, group: &str) -> f64 {
            self.booked(&format!("nuclear.{group}")).iter().sum()
        }

        /// side.s_nu over the block.
        fn s_nu(&self) -> Vec<f64> {
            let all = words(self.gpu, self.side.s_nu(), (SIDE * SIDE) as u64);
            (0..self.width * self.height).map(|i| f64::from(f32::from_bits(all[self.at(i)]))).collect()
        }

        /// The latch record's flag word.
        fn latch(&self) -> bool {
            words(self.gpu, self.step.latch_flag(), 1)[0] != 0
        }
    }

    /// Cells of one species at a time, for the cases below.
    fn fe() -> [f64; NS] {
        x_of(&[("Fe", 1.0)])
    }

    /// A cell held at T (pin) loses A_ν (T/T_ν)^8 Σ Δt, booked `neutrino_lost`, at four temperatures and two densities; T,
    /// Σ and the composition stay, no nuclear group is booked, the latch is not set.
    #[test]
    fn gpu_cooling_closed_form_at_pinned_t() {
        let gpu = device();
        let physics = cooling(&[]);
        let dt = 0.3;
        for &(sigma, t) in &[(50.0, 170.0), (50.0, 200.0), (50.0, 300.0), (120.0, 500.0)] {
            let block = world(1, 1, &[(sigma, t, fe())]);
            let rig = Rig::new(&gpu, &physics, &only(&[], &physics), true, true, &block);
            rig.pass(dt);
            let want = A_NU * (t / T_NU).powi(M_NU) * sigma * dt;
            let got = rig.booked("neutrino_lost")[0];
            let st = rig.planes();
            println!("pinned T = {t}, Σ = {sigma}: lost {got:.6e}, the closed form {want:.6e} (Δ {:.2e}), T {:.4}", rel(got, want), temp(&st, 0));
            assert!(rel(got, want) <= 1e-4, "T = {t}: lost {got}, the closed form {want}");
            assert!((temp(&st, 0) - t).abs() <= 1e-5 * t, "T pinned: {}", temp(&st, 0));
            assert_eq!(x_at(&st, 0), x_at(&block, 0), "cooling moves no mass");
            assert!(["H", "He", "C", "Ne", "O", "Si", "N"].iter().all(|g| rig.nuclear(g) == 0.0), "no nuclear group is booked");
            assert!(!rig.latch());
        }
    }

    /// None below T_ν: a cell at 0.9999 T_ν, at 100 and at 1 is left bit for bit, loses nothing and sources nothing; at
    /// 1.0001 T_ν it loses A_ν Σ Δt; with the switch off a cell at 500 is left bit for bit, books nothing and S_ν stays 0.
    #[test]
    fn gpu_none_below_t_nu_and_none_with_the_switch_off() {
        let gpu = device();
        let physics = cooling(&[]);
        let reactions = only(&[], &physics);
        let neutron_rich = x_of(&[("n", 0.9), ("Fe", 0.1)]);
        let block = world(1, 1, &[(50.0, 300.0, neutron_rich)]);
        let on = Rig::new(&gpu, &physics, &reactions, true, true, &block);
        for t in [T_NU * (1.0 - 1e-4), 100.0, 1.0] {
            let block = world(1, 1, &[(50.0, t, neutron_rich)]);
            on.load(&block);
            on.pass(0.5);
            let loaded = {
                on.load(&block);
                on.planes()
            };
            on.pass(0.5);
            assert_eq!(on.planes().planes, loaded.planes, "T = {t} < T_ν: the cell is left bit for bit");
            assert_eq!(on.booked("neutrino_lost")[0], 0.0, "T = {t} < T_ν");
            assert_eq!(on.s_nu()[0], 0.0, "T = {t} < T_ν: S_ν");
        }
        on.load(&world(1, 1, &[(50.0, T_NU * (1.0 + 1e-4), fe())]));
        on.pass(0.5);
        let (got, want) = (on.booked("neutrino_lost")[0], A_NU * 50.0 * 0.5);
        println!("just above T_ν: lost {got:.6e}, A_ν Σ Δt {want:.6e}");
        assert!(rel(got, want) <= 1e-3, "at T_ν the loss is A_ν Σ Δt: {got}");
        let off_block = world(1, 1, &[(50.0, 500.0, neutron_rich)]);
        let off = Rig::new(&gpu, &physics, &reactions, false, false, &off_block);
        let before = off.planes().planes;
        off.pass(0.5);
        assert_eq!(off.planes().planes, before, "neutrinos off: a hot cell is left bit for bit");
        assert_eq!(off.booked("neutrino_lost")[0], 0.0);
        assert_eq!(off.s_nu()[0], 0.0, "neutrinos off: S_ν is never written");
    }

    /// S_ν = ε_νΣ only where the cell leaves the pass with X_n ≥ 0.5 — the twin's seven cells at T = 300 held: X_n 0.9 and
    /// 0.5 (the boundary included) are sources, 0.4999, 0.1, iron, the vacuum and a cell cooler than T_ν are not; and S_ν is
    /// rewritten every step: the same world again with the first cell now at X_n 0.1 reads 0 there, not the last step's.
    #[test]
    fn gpu_source_only_where_x_n_is_half_or_more() {
        let gpu = device();
        let physics = cooling(&[]);
        let (dt, sigma) = (0.25, 80.0);
        let mut cells = vec![
            (sigma, 300.0, x_of(&[("n", 0.9), ("Fe", 0.1)])),
            (sigma, 300.0, x_of(&[("n", 0.5), ("Fe", 0.5)])),
            (sigma, 300.0, x_of(&[("n", 0.4999), ("Fe", 0.5001)])),
            (sigma, 300.0, x_of(&[("n", 0.1), ("Fe", 0.9)])),
            (sigma, 300.0, fe()),
            (1e-9, 300.0, x_of(&[("n", 1.0)])),
            (sigma, 100.0, x_of(&[("n", 0.9), ("Fe", 0.1)])),
        ];
        let rig = Rig::new(&gpu, &physics, &only(&[], &physics), true, true, &world(7, 1, &cells));
        rig.pass(dt);
        let power = eps_nu(300.0) * sigma;
        let s = rig.s_nu();
        println!("S_ν = {s:?}, ε_νΣ = {power:.6e}");
        for (i, &source) in [true, true, false, false, false, false, false].iter().enumerate() {
            if source {
                assert!(rel(s[i], power) <= 1e-4, "cell {i}: S_ν {}, ε_νΣ {power}", s[i]);
            } else {
                assert_eq!(s[i], 0.0, "cell {i}: S_ν {} where X_n < 0.5, the vacuum, or T < T_ν", s[i]);
            }
        }
        let lost = rig.booked("neutrino_lost");
        for i in 0..5 {
            assert!(rel(lost[i], power * dt) <= 1e-4, "cell {i}: lost {}, ε_νΣΔt {}", lost[i], power * dt);
        }
        assert!(lost[5] == 0.0 && lost[6] == 0.0, "the vacuum and the cool cell lose nothing: {lost:?}");
        let emitted: f64 = s.iter().sum::<f64>() * dt;
        assert!(emitted <= lost.iter().sum::<f64>(), "S_ν over the step ({emitted}) is more than was lost");
        cells[0] = (sigma, 300.0, x_of(&[("n", 0.1), ("Fe", 0.9)]));
        rig.load(&world(7, 1, &cells));
        rig.pass(dt);
        assert_eq!(rig.s_nu()[0], 0.0, "S_ν left from the last step in a cell that is no longer a source");
    }

    /// Cooling alone, T free: iron at T₀ = 500, Σ = 50, `burn_dt` 10⁻³, over t = 20 — the ODE's T⁻⁷ = T₀⁻⁷ + 7 μ A_ν t/T_ν⁸:
    /// the temperature drop within 1e-2 of the closed form's (f32), the energy taken from E the booked loss (1e-4), no mass
    /// moved, sub-cycled, nothing left unfinished.
    #[test]
    fn gpu_cooling_unpinned_follows_the_ode() {
        let gpu = device();
        let physics = cooling(&[("burn_dt", 1e-3)]);
        let (sigma, t0, t): (f64, f64, f64) = (50.0, 500.0, 20.0);
        let mu = 1.0 / inv_mu(&fe());
        let t_end = (t0.powi(-7) + 7.0 * mu * A_NU * t / T_NU.powi(8)).powf(-1.0 / 7.0);
        assert!(t_end > T_NU, "the case stays above T_ν: {t_end}");
        let block = world(1, 1, &[(sigma, t0, fe())]);
        let rig = Rig::new(&gpu, &physics, &only(&[], &physics), false, true, &block);
        let (e0, t_start) = (rig.planes().get(3, 0), temp(&rig.planes(), 0));
        let report = rig.pass(t);
        let st = rig.planes();
        let (drop_got, drop_want) = (t_start - temp(&st, 0), t0 - t_end);
        let (taken, booked) = (e0 - st.get(3, 0), rig.booked("neutrino_lost")[0]);
        println!("unpinned: T {t_start:.3} → {:.4} (closed form {t_end:.4}), drop Δ {:.2e}, {} sub-steps, E lost {taken:.6e}, booked {booked:.6e}", temp(&st, 0), rel(drop_got, drop_want), report.substeps);
        assert!(rel(drop_got, drop_want) <= 1e-2, "T fell by {drop_got}, the closed form {drop_want}");
        assert!(rel(taken, booked) <= 1e-4, "E lost {taken}, booked {booked}");
        assert_eq!(x_at(&st, 0), x_at(&block, 0));
        assert!(report.substeps > 100, "cooling did not bind the sub-step: {} sub-steps", report.substeps);
        assert_eq!(report.unfinished, 0);
    }

    /// Energy closes with burning and cooling together: a hot neon cell, free, over a short step — E gains the nuclear heat
    /// booked minus the neutrinos lost (1e-4 of the heat), and the loss is 10⁻³ of that heat at least, so a pass that forgot
    /// to take it from E is caught.
    #[test]
    fn gpu_energy_closes_with_burning_and_cooling() {
        let gpu = device();
        let physics = physics(&[]);
        let block = world(1, 1, &[(50.0, 300.0, x_of(&[("Ne", 1.0)]))]);
        let rig = Rig::new(&gpu, &physics, &only(&["Ne_burn"], &physics), false, true, &block);
        let e0 = rig.planes().get(3, 0);
        rig.pass(0.05);
        let heat: f64 = GROUPS.iter().map(|g| rig.nuclear(g)).sum();
        let lost = rig.booked("neutrino_lost")[0];
        let de = rig.planes().get(3, 0) - e0;
        println!("Ne cell: heat {heat:.6e}, lost {lost:.6e} ({:.2e} of it), ΔE {de:.6e}", lost / heat);
        assert!(heat > 0.0 && lost > 1e-3 * heat, "the case needs both terms: heat {heat}, lost {lost}");
        assert!((de - (heat - lost)).abs() <= 1e-4 * heat, "ΔE {de} ≠ heat {heat} − lost {lost}");
    }

    /// N_Fe's gate on Y_eΣ, the twin's eight cases: closed and bit for bit just under Σ_N/Y_e (0.01 %), at Σ = 160 ± 0.02 and
    /// at 200; open just over it for iron and for a half-hydrogen cell at its own threshold, and not at iron's. Open: Fe
    /// falls and the latch flag is set; closed: the cell is untouched and the flag is not.
    #[test]
    fn gpu_n_fe_gate_reads_y_e_sigma() {
        let gpu = device();
        let physics = physics(&[]);
        let (iron, mixed) = (fe(), x_of(&[("Fe", 0.5), ("H", 0.5)]));
        let (ye_fe, ye_mixed) = (y_e(&iron), y_e(&mixed));
        let cases = [
            (iron, SIGMA_N * 0.9999 / ye_fe, false),
            (iron, SIGMA_N * 1.0001 / ye_fe, true),
            (iron, SIGMA_N - 0.02, false),
            (iron, SIGMA_N + 0.02, false),
            (iron, 200.0, false),
            (mixed, SIGMA_N * 0.9999 / ye_mixed, false),
            (mixed, SIGMA_N * 1.0001 / ye_mixed, true),
            (mixed, SIGMA_N / ye_fe * 1.0001, true),
        ];
        let rig = Rig::new(&gpu, &physics, &only(&["N_Fe"], &physics), true, false, &world(1, 1, &[(10.0, 200.0, iron)]));
        for (k, &(x, sigma, open)) in cases.iter().enumerate() {
            let block = world(1, 1, &[(sigma, 200.0, x)]);
            rig.load(&block);
            let before = rig.planes().planes;
            rig.pass(1.0);
            let after = rig.planes();
            let fe_at = elements().index_of("Fe").unwrap();
            let moved = after.get(4 + fe_at, 0) < before[4 + fe_at];
            println!("case {k}: Σ = {sigma:.3}, Y_eΣ = {:.3} → {}, latch {}", y_e(&x) * sigma, if moved { "converts" } else { "closed" }, rig.latch());
            assert_eq!((moved, rig.latch()), (open, open), "case {k} (Σ = {sigma}, Y_eΣ = {}): gate {open}", y_e(&x) * sigma);
            if !open {
                assert_eq!(after.planes, before, "case {k}: a closed gate leaves the cell bit for bit");
                assert_eq!(rig.nuclear("N"), 0.0);
            }
        }
    }

    /// N_Fe converts and absorbs: iron at Σ = 500 (g = 0.45), free, over 1 t.u. — Fe → n mass for mass in the planes
    /// (within 1e-6, f32 at X ≈ 1), ΣX = 1, nothing else moves, the heat booked in group N is q ΔX_n Σ with q = −1.14 Q_H
    /// (1e-4) and negative, no other group is booked, E gains that (1e-4), T falls, the latch flag is set.
    #[test]
    fn gpu_n_fe_absorbs_and_conserves_mass() {
        let gpu = device();
        let physics = physics(&[]);
        let sigma = 500.0;
        let block = world(1, 1, &[(sigma, 1000.0, fe())]);
        let rig = Rig::new(&gpu, &physics, &only(&["N_Fe"], &physics), false, false, &block);
        let (e0, t0) = (rig.planes().get(3, 0), temp(&rig.planes(), 0));
        rig.pass(1.0);
        let st = rig.planes();
        let (dfe, dn) = (frac(&st, "Fe", 0) - frac(&block, "Fe", 0), frac(&st, "n", 0));
        let sum: f64 = x_at(&st, 0).iter().sum();
        let booked = rig.nuclear("N");
        println!("N_Fe: ΔX_Fe {dfe:.6e}, ΔX_n {dn:.6e}, booked {booked:.6e}, ΔE {:.6e}, T {:.2}", st.get(3, 0) - e0, temp(&st, 0));
        assert!(dn > 1e-4, "the record converted {dn}");
        assert!((dfe + dn).abs() <= 1e-6, "Fe lost {dfe}, n gained {dn}");
        assert!((sum - 1.0).abs() <= 1e-6 && x_at(&st, 0).iter().all(|&v| v >= 0.0), "ΣX = {sum}");
        let others: f64 = x_at(&st, 0).iter().sum::<f64>() - frac(&st, "Fe", 0) - dn;
        assert!(others.abs() <= 1e-6, "another species moved: {others}");
        let want = Q_NFE * dn * sigma;
        assert!(booked < 0.0 && rel(booked, want) <= 1e-4, "booked {booked}, q ΔX_n Σ {want}");
        assert!(GROUPS.iter().filter(|&&g| g != "N").all(|g| rig.nuclear(g) == 0.0), "only N is booked");
        assert!(rel(st.get(3, 0) - e0, booked) <= 1e-4, "E gained {}, booked {booked}", st.get(3, 0) - e0);
        assert!(temp(&st, 0) < t0, "absorbing cooled the cell");
        assert!(rig.latch());
    }

    /// The latch flag reads any cell: five cells — iron under the gate, iron over it, hydrogen at high Σ, neutron matter at
    /// high Σ, vacuum — set it exactly when the iron over the gate is there, in any of the five positions.
    #[test]
    fn gpu_latch_is_any_cells_n_fe_rate() {
        let gpu = device();
        let physics = physics(&[]);
        let closed = [
            (200.0, 200.0, fe()),
            (900.0, 200.0, x_of(&[("H", 1.0)])),
            (900.0, 200.0, x_of(&[("n", 1.0)])),
            (1e-9, 200.0, fe()),
        ];
        let open = (SIGMA_N * 1.05 / y_e(&fe()), 200.0, fe());
        let rig = Rig::new(&gpu, &physics, &only(&["N_Fe"], &physics), true, true, &world(5, 1, &[open, closed[0], closed[1], closed[2], closed[3]]));
        let mut cells = closed.to_vec();
        cells.push(closed[3]);
        rig.load(&world(5, 1, &cells));
        rig.pass(1.0);
        assert!(!rig.latch(), "no cell's N_Fe rate is > 0, the flag stays unset");
        for pos in 0..5 {
            let mut cells = closed.to_vec();
            cells.insert(pos, open);
            rig.load(&world(5, 1, &cells));
            rig.pass(1.0);
            assert!(rig.latch(), "the open cell at position {pos} did not set the flag");
            let st = rig.planes();
            assert!(frac(&st, "Fe", pos) < 1.0, "the open cell converted nothing");
            for i in (0..5).filter(|&i| i != pos) {
                assert!((x_at(&st, i)).iter().zip(&cells[i].2).all(|(a, b)| a == b), "cell {i} moved, its gate is closed");
            }
        }
    }

    /// The flag is set from any sub-cycle's rate, not the step's last: iron 1 % over the gate (g = 0.01) over a step of 10⁶ t.u.
    /// converts until its own Y_e has closed the gate (the first sub-cycle moves 9 % of the iron, the gate shuts at 1 %), so
    /// the rate is 0 from then to the step's end — the flag is set; the same cell over another step, its gate shut, leaves
    /// it unset.
    #[test]
    fn gpu_latch_is_set_by_a_rate_that_closes_within_the_step() {
        let gpu = device();
        let physics = physics(&[]);
        let g0 = 0.01;
        let sigma = SIGMA_N * (1.0 + g0) / y_e(&fe());
        let block = world(1, 1, &[(sigma, 200.0, fe())]);
        let rig = Rig::new(&gpu, &physics, &only(&["N_Fe"], &physics), true, false, &block);
        let report = rig.pass(1e6);
        let st = rig.planes();
        let closes = 1.0 - 1.0 / (1.0 + g0);
        let fallen = 1.0 - frac(&st, "Fe", 0);
        println!("gate closing: Fe fell by {fallen:.6e} (closes at {closes:.6e}), {} sub-steps", report.substeps);
        assert!(rig.latch(), "the first sub-cycle's rate was > 0, the flag is set");
        assert!(fallen > closes, "the gate stayed open: Fe fell by {fallen}, {closes} closes it");
        assert!(y_e(&x_at(&st, 0)) * sigma < SIGMA_N, "the gate is shut at the step's end");
        rig.gpu.queue.write_buffer(rig.step.latch_flag(), 0, &0u32.to_le_bytes());
        rig.pass(1.0);
        assert!(!rig.latch(), "a cell whose gate is shut sets nothing: the first pass's flag came from a sub-cycle before the last");
    }

    /// The latch's first trigger by the physics (§1.8.5): two walls of iron (Σ = 90, Y_eΣ = 42 < 160, cold, moving at ±2
    /// toward each other, P4 on, bounce edges) meet; the shock between them lifts Σ over Σ_N/Y_e at some step k, N_Fe turns
    /// on there and the flag is set. A bare run, a step at a time, finds k as the first step after which neutron matter exists
    /// anywhere; a 40-step frame with the slow-down on then runs exactly k steps and fires at k, leaving the state, the Δt
    /// record and the box bit-identical to the bare k-step run; with the slow-down off the same frame runs all 40. k ≥ 2: the
    /// gate is shut at the start.
    #[test]
    fn gpu_a_frame_whose_step_k_turns_n_fe_on_stops_after_step_k() {
        let gpu = device();
        let physics = physics(&[]);
        let reactions = only(&["N_Fe"], &physics);
        let fe_plane = CHANNELS.iter().position(|&c| c == "x_Fe").unwrap();
        let start = |gpu: &Gpu| -> (GpuState, Step) {
            let world = WorldConfig::new(SIDE as u32, SIDE as u32).unwrap();
            let n = world.cells();
            let mut p = vec![0f32; N_CHANNELS * n];
            let x_h = CHANNELS.iter().position(|&c| c == "x_H").unwrap();
            p[x_h * n..(x_h + 1) * n].fill(1.0);
            p[..n].fill(1e-8);
            p[3 * n..4 * n].fill(1e-8 * 0.05 * 2.0);
            let inv = inv_mu(&fe()) as f32;
            for y in 8..56 {
                for x in 8..56 {
                    let i = y * SIDE + x;
                    let u = if x < 32 { WALL_SPEED } else { -WALL_SPEED };
                    p[i] = WALL_SIGMA;
                    p[n + i] = WALL_SIGMA * u;
                    p[3 * n + i] = WALL_SIGMA * (WALL_T * inv + 0.5 * u * u);
                    p[x_h * n + i] = 0.0;
                    p[fe_plane * n + i] = 1.0;
                }
            }
            let state = GpuState::new(&gpu.device, world).unwrap();
            state.upload(&gpu.queue, &p).unwrap();
            let booking = Booking::new(&gpu.device, world).unwrap();
            let side = SideFields::new(&gpu.device, world).unwrap();
            let eos = EosGpu::new(&gpu.device, &physics, &elements());
            let passes = Passes { mode: BoxMode::Fit, gravity: None, hydro: Some(EdgeMode::Bounce) };
            let react = ReactSetup { reactions: &reactions, test: TestSettings::default(), neutrinos: false, side: &side };
            let step = Step::with_reactions(&gpu.device, &state, &booking, &eos, &physics, &passes, None, Some(&react)).unwrap();
            (state, step)
        };
        // Neutron matter appears only from N_Fe (the walls mix with the vacuum's hydrogen, so Fe alone says nothing).
        let n_plane = CHANNELS.iter().position(|&c| c == "x_n").unwrap();
        let max_n = |gpu: &Gpu, state: &GpuState| -> f32 {
            let n = SIDE * SIDE;
            state.readback(&gpu.device, &gpu.queue)[n_plane * n..(n_plane + 1) * n].iter().cloned().fold(0.0, f32::max)
        };
        let max_sigma = |gpu: &Gpu, state: &GpuState| -> f32 {
            state.readback(&gpu.device, &gpu.queue)[..SIDE * SIDE].iter().cloned().fold(0.0, f32::max)
        };
        // The bare run: the first step after which neutron matter exists anywhere.
        let (state, step) = start(&gpu);
        let mut k = 0;
        let mut at_k = Vec::new();
        for i in 1..=40 {
            step.run(&gpu.device, &gpu.queue, 1);
            println!("bare step {i}: max Σ {:.2} (Σ_N/Y_e = {:.1}), max X_n {:.3e}", max_sigma(&gpu, &state), SIGMA_N / y_e(&fe()), max_n(&gpu, &state));
            if max_n(&gpu, &state) > 0.0 && k == 0 {
                k = i;
                at_k = state.readback(&gpu.device, &gpu.queue).iter().map(|v| v.to_bits()).collect::<Vec<_>>();
                break;
            }
        }
        assert!(k >= 2, "the gate opened at step {k}: not a step the physics reached after the start");
        let want = (at_k, step.read_dt(&gpu.device, &gpu.queue), step.read_box(&gpu.device, &gpu.queue));
        let frame = |slow_down: bool| {
            let (state, step) = start(&gpu);
            let mut encoder = gpu.device.create_command_encoder(&Default::default());
            step.encode_frame(&FramePlan { steps: 40, slow_down }, &mut encoder);
            gpu.queue.submit([encoder.finish()]);
            gpu.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
            let reports = step.poll(&gpu.device);
            assert_eq!(reports.len(), 1);
            (reports[0], state, step)
        };
        let (report, state, step) = frame(true);
        println!("frame: {} steps run, fired at {:?} (the bare run opened the gate at step {k})", report.steps_run, report.latch_fired);
        assert_eq!((report.steps_run, report.latch_fired), (k, Some(k)), "the frame stops after the step that turned N_Fe on");
        let got: Vec<u32> = state.readback(&gpu.device, &gpu.queue).iter().map(|v| v.to_bits()).collect();
        assert_eq!(got.iter().zip(&want.0).filter(|(a, b)| a != b).count(), 0, "the state differs from the bare k-step run");
        assert_eq!(step.read_dt(&gpu.device, &gpu.queue), want.1, "the Δt record");
        assert_eq!(step.read_box(&gpu.device, &gpu.queue), want.2, "the box");
        let (report, ..) = frame(false);
        assert_eq!((report.steps_run, report.latch_fired), (40, None), "the slow-down off: all 40 steps run, nothing fires");
    }

    /// The wall scene of the frame case: cold iron at Σ = WALL_SIGMA, each half at ±WALL_SPEED toward the middle.
    const WALL_SIGMA: f32 = 150.0;
    const WALL_SPEED: f32 = 3.0;
    const WALL_T: f32 = 1.0;
}
