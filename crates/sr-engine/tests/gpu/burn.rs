//! M0-T44b — burning on the CPU twin (contract §1.6, §1.7, §2.5.1, §2.5.2, §5.1's G-BURN and G-ORDER, R8, R13): P6's f64
//! reference, `sr_physics::reference::react`, per-cell linearised backward-Euler sub-cycles under `burn_dx` and `burn_dt`;
//! the twin's cases touch no adapter. M0-T46 adds the GPU's cases to these scopes.
//!
//! Two scopes, one module each. `cell` — G-BURN: one cell, T pinned, flow off, k = A Σ^a f(T), one record alone (the others
//! disabled in the registry text, so a product is not eaten by the next record): H_burn (order 2) X = X₀/(1 + kX₀t), He_burn
//! (order 3) X = X₀/√(1 + 2kX₀²t), Ne_burn (order 1) X = X₀e^{−kt}, each down to X₀/10; the heat booked equals q·Δm of the
//! mass the planes say moved, within 1e-5 — summed from the sub-steps, not once from the step's first rate; E in a free run
//! gains exactly that; ΣX = 1 and every fraction ≥ 0; the sub-cycle limits hold at the shipped `burn_dx` and `burn_dt`; a cell
//! burns alone. `order` — G-ORDER: a cell of equal parts H, He, C, O, Ne and Si heated at a steady rate; each group's specific
//! power first passes ε* = 10⁻³ Q_H per t.u. in the order H, He, C, {Ne, O}, Si; every record books into its own group.
//!
//! The oracles are written here from the contract: f(T) from §1.6.2's terms with §2.5.1's temperatures, k from A, Σ and f,
//! q = q_ratio · Q_H from the table, T from E with the cold pressure off and the gas at rest (T = E/Σ/(1/μ)). G-BURN's closed
//! forms were written at M0-TC's ladder (T_H 100, T_He 140, T_Ne 274) and pin it (`cell::LADDER`, M0-D55); G-ORDER grades
//! the shipped ladder, §0.5 [M0-D50]'s (T_H 56, T_He 78.4).
//!
//! Tolerances: the contract's own — X within 1e-4 absolute, heat within 1e-5 relative, ΣX = 1 within 1e-6, the order exact.
//! G-BURN's runs set `burn_dx` to 1e-4 (physics.json's key, a scene's `overrides`): backward Euler is first order, and a
//! sub-step of 0.1 in X is a step of kh = 0.1/X at the tail of a decay, whose error is far above 1e-4 (measured in
//! `shipped_limits_error`, which reads the error at the shipped 0.1 and only bounds it); the limit's own check runs at the
//! shipped values.

use sr_physics::reference::hydro::State;
use sr_physics::reference::react::{Pass, React, TestFlags, N_GROUPS};
use sr_physics::registry::{Elements, Physics, Reactions, GROUPS, N_SPECIES};

const NS: usize = N_SPECIES;
/// §2.5.1: Q_H.
const Q_H: f64 = 8_700.0;

fn elements() -> Elements {
    Elements::shipped().unwrap()
}

/// The shipped constants with the cold pressure off (so T = μ E/Σ at rest), and `extra` overrides.
fn physics(extra: &[(&str, f64)]) -> Physics {
    let mut keys = vec![("k1e", 0.0), ("k2e", 0.0), ("k1n", 0.0), ("k2n", 0.0)];
    keys.extend_from_slice(extra);
    Physics::shipped().unwrap().with(&keys).unwrap()
}

/// assets/reactions.json with only `ids` enabled.
fn only(ids: &[&str], physics: &Physics) -> Reactions {
    let text = include_str!("../../../../assets/reactions.json");
    let mut root: serde_json::Value = serde_json::from_str(text).unwrap();
    for rec in root["records"].as_array_mut().unwrap() {
        let on = ids.contains(&rec["id"].as_str().unwrap());
        rec["enabled"] = serde_json::Value::Bool(on);
    }
    Reactions::parse(&root.to_string(), &elements(), physics).unwrap()
}

fn react(physics: &Physics, reactions: &Reactions, pin: bool) -> React {
    React::new(physics, &elements(), reactions, TestFlags { pin_temperature: pin }).unwrap()
}

/// Fractions from (key, value) parts; everything else 0.
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

fn temp(st: &State, i: usize) -> f64 {
    let x: [f64; NS] = std::array::from_fn(|s| st.get(4 + s, i));
    st.get(3, i) / st.get(0, i) / inv_mu(&x)
}

fn frac(st: &State, species: &str, i: usize) -> f64 {
    st.get(4 + elements().index_of(species).unwrap(), i)
}

/// One Gamow-shaped term from §0.5 [M0-D34]: (T/T_k)^{−2/3} exp(τ (1 − (T_k/T)^{1/3})), τ = 3ν + 2.
fn gamow(t: f64, t_k: f64, nu: f64) -> f64 {
    (t / t_k).powf(-2.0 / 3.0) * ((3.0 * nu + 2.0) * (1.0 - (t_k / t).cbrt())).exp()
}

/// f(T) of H_burn from §1.6.2's terms (T_H, 4) and (1.15 T_H, 18), each Gamow-shaped, above 0.5 T_H.
fn f_h(t: f64, t_h: f64) -> f64 {
    if t < 0.5 * t_h {
        return 0.0;
    }
    gamow(t, t_h, 4.0) + gamow(t, 1.15 * t_h, 18.0)
}

fn sum_x(st: &State, i: usize) -> f64 {
    (0..NS).map(|s| st.get(4 + s, i)).sum()
}

/// P6 on the GPU (M0-T46): a twin world of up to 48 × 48 cells laid at (8, 8) in a 64 × 64 world (§2.8's smallest), vacuum
/// around it; P6 alone with P1, P8 and P9, the box fit. As heat.rs's rig: each [`OnGpu::pass`] is one step at the Δt the test
/// writes as the Δt record's Δt_{n+1}; step 0 (the initial reduction's Δt) runs at build and the block is loaded again.
mod rig {
    use super::*;
    use sr_engine::gpu::Gpu;
    use sr_engine::scene::TestSettings;
    use sr_engine::state::{Booking, SideFields, State as GpuState, WorldConfig, BOOK_GROUPS, TERMS};
    use sr_engine::step::boxfit::BoxMode;
    use sr_engine::step::react::{ReactReport, ReactSetup};
    use sr_engine::step::{EosGpu, Passes, Step, Timing};

    const SIDE: usize = 64;
    const AT: usize = 8;

    pub struct OnGpu<'g> {
        gpu: &'g Gpu,
        state: GpuState,
        booking: Booking,
        step: Step,
        width: usize,
        height: usize,
    }

    impl<'g> OnGpu<'g> {
        pub fn new(gpu: &'g Gpu, physics: &Physics, reactions: &Reactions, pin: bool, block: &State) -> OnGpu<'g> {
            assert!(block.width <= SIDE - 2 * AT && block.height <= SIDE - 2 * AT);
            let world = WorldConfig::new(SIDE as u32, SIDE as u32).unwrap();
            let state = GpuState::new(&gpu.device, world).unwrap();
            let booking = Booking::new(&gpu.device, world).unwrap();
            let eos = EosGpu::new(&gpu.device, physics, &elements());
            let passes = Passes { mode: BoxMode::Fit, gravity: None, hydro: None };
            let test = TestSettings { pin_temperature: pin, ..TestSettings::default() };
            let side = SideFields::new(&gpu.device, world).unwrap();
            let react = ReactSetup { reactions, test, neutrinos: false, side: &side };
            let step = Step::with_reactions(&gpu.device, &state, &booking, &eos, physics, &passes, None, Some(&react)).unwrap();
            let run = OnGpu { gpu, state, booking, step, width: block.width, height: block.height };
            run.load(block);
            run.step.run(&gpu.device, &gpu.queue, 1);
            run.load(block);
            run
        }

        fn at(&self, i: usize) -> usize {
            (AT + i / self.width) * SIDE + AT + i % self.width
        }

        /// `block` uploaded (rounded to f32), vacuum around it, the booking cleared.
        pub fn load(&self, block: &State) {
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
        }

        /// One step whose P6 runs over `dt`; its counters.
        pub fn pass(&self, dt: f64) -> ReactReport {
            let (device, queue) = (&self.gpu.device, &self.gpu.queue);
            queue.write_buffer(self.step.dt_record(), 4, &(dt as f32).to_le_bytes());
            self.step.run(device, queue, 1);
            assert_eq!(self.step.read_dt(device, queue).dt, dt as f32, "P1 took the Δt written");
            self.step.read_react(device, queue).expect("the step runs P6")
        }

        /// P6's ms in one timed step of `dt` (GPU timestamps; the wall clock where the adapter has none).
        pub fn step_report(&self) -> ReactReport {
            self.step.read_react(&self.gpu.device, &self.gpu.queue).expect("the step runs P6")
        }

        pub fn timed_ms(&self, dt: f64) -> f64 {
            let (device, queue) = (&self.gpu.device, &self.gpu.queue);
            self.step.set_timing(device, queue, Timing::Auto);
            self.pass(dt);
            let cost = self.step.read_cost(device, queue).expect("timed");
            self.step.set_timing(device, queue, Timing::Off);
            cost.pass_ms[6].unwrap_or(cost.step_ms)
        }

        /// The block's cells as the GPU holds them, in f64.
        pub fn planes(&self) -> State {
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

        /// The block's `nuclear.<group>` slots since the last load, as [`Pass::nuclear`] lays them (group-major).
        pub fn nuclear(&self) -> Vec<f64> {
            let book = self.booking.readback(&self.gpu.device, &self.gpu.queue);
            let world = self.state.world();
            let g = BOOK_GROUPS.iter().position(|g| TERMS[g.terms.start].name == "nuclear.H").unwrap();
            let base: usize = BOOK_GROUPS[..g].iter().map(|g| g.terms.len() * g.domain.slots(world)).sum();
            let cells = self.width * self.height;
            (0..N_GROUPS * cells).map(|k| f64::from(book[base + (k / cells) * SIDE * SIDE + self.at(k % cells)])).collect()
        }
    }

    /// A [`Pass`] holding only what the GPU books, for [`Pass::nuclear_total`].
    pub fn booked(nuclear: Vec<f64>) -> Pass {
        Pass { nuclear, ..Pass::default() }
    }
}

/// G-BURN.
mod cell {
    use super::*;

    /// burn_dx for the analytic runs (module doc).
    const DX: f64 = 1e-4;
    /// X within this, absolute (§5.1).
    const X_TOL: f64 = 1e-4;
    /// M0-TC's ladder (§2.5.1 before §0.5 [M0-D50]), the one the closed forms were written at: T pinned at T_H, T_He or
    /// T_Ne, k from f there. The analytic runs and the heat they book pin it through `Physics::with` (G-COLL's pattern);
    /// f reads T/T_k alone, so the law graded is the shipped one. The shipped limits' runaway, the mixed cells and the
    /// free run read the shipped ladder.
    const LADDER: [(&str, f64); 6] = [("t_h", 100.0), ("t_he", 140.0), ("t_c", 196.0), ("t_ne", 274.0), ("t_o", 300.0), ("t_si", 420.0)];
    const T_H: f64 = 100.0;
    const T_HE: f64 = 140.0;
    const T_NE: f64 = 274.0;

    /// [`physics`] with [`LADDER`] pinned, and `extra`.
    fn pinned(extra: &[(&str, f64)]) -> Physics {
        let mut keys = LADDER.to_vec();
        keys.extend_from_slice(extra);
        physics(&keys)
    }

    /// Steps `st` (one cell, T pinned) over the checkpoints `(t_i)`, the twin's T pinned, `oracle(t)` the analytic X of the
    /// burning species; X within [`X_TOL`] at each, ΣX = 1 within 1e-6. Returns the last pass.
    fn follow(st: &mut State, r: &React, species: &str, checkpoints: &[f64], oracle: impl Fn(f64) -> f64) -> Pass {
        let mut t_prev = 0.0;
        let mut pass = Pass::default();
        for &t in checkpoints {
            pass = r.step(st, t - t_prev);
            t_prev = t;
            let (got, want) = (frac(st, species, 0), oracle(t));
            assert!((got - want).abs() <= X_TOL, "{species} at t = {t}: X = {got}, the closed form {want} (Δ {:.2e})", got - want);
            assert!((sum_x(st, 0) - 1.0).abs() <= 1e-6, "ΣX = {} at t = {t}", sum_x(st, 0));
            assert_eq!(pass.unfinished, 0, "a sub-cycle ran out of attempts at t = {t}");
        }
        pass
    }

    /// H_burn, order 2: X = X₀/(1 + kX₀t), k = A Σ f(T), down to X₀/10.
    #[test]
    fn h_burn_second_order() {
        let physics = pinned(&[("burn_dx", DX)]);
        let r = react(&physics, &only(&["H_burn"], &physics), true);
        let (sigma, t, x0) = (100.0, T_H, 1.0);
        let k = 5e-4 * sigma * f_h(t, T_H);
        let mut st = world(1, 1, &[(sigma, t, x_of(&[("H", x0)]))]);
        let at: Vec<f64> = [0.5, 1.0, 2.0, 4.0, 6.0, 9.0].iter().map(|v| v / (k * x0)).collect();
        let pass = follow(&mut st, &r, "H", &at, |t| x0 / (1.0 + k * x0 * t));
        println!("h_burn: k = {k:.4}, X_H = {:.6} (X₀/10 = {}), sub-steps {}, retried {}", frac(&st, "H", 0), x0 / 10.0, pass.substeps, pass.retried);
        assert!((temp(&st, 0) - t).abs() < 1e-9 * t, "T pinned: {}", temp(&st, 0));
        // The product is He, mass for mass: nothing else in the cell.
        assert!((frac(&st, "He", 0) - (1.0 - frac(&st, "H", 0))).abs() < 1e-9);
    }

    /// He_burn, order 3: X = X₀/√(1 + 2kX₀²t), k = A Σ² f(T), f = (T/T_He)⁴⁰ = 1 at T = T_He.
    #[test]
    fn he_burn_third_order() {
        let physics = pinned(&[("burn_dx", DX)]);
        let r = react(&physics, &only(&["He_burn"], &physics), true);
        let (sigma, t, x0) = (10.0, T_HE, 1.0);
        let k = 1e-3 * sigma * sigma;
        let mut st = world(1, 1, &[(sigma, t, x_of(&[("He", x0)]))]);
        // 1 + 2kX₀²t = 1.25, 2, 4, 9, 25, 100 → X/X₀ = 0.894 … 0.1.
        let at: Vec<f64> = [1.25, 2.0, 4.0, 9.0, 25.0, 100.0].iter().map(|m| (m - 1.0) / (2.0 * k * x0 * x0)).collect();
        let pass = follow(&mut st, &r, "He", &at, |t| x0 / (1.0 + 2.0 * k * x0 * x0 * t).sqrt());
        println!("he_burn: k = {k}, X_He = {:.6} (X₀/10 = {}), sub-steps {}, retried {}", frac(&st, "He", 0), x0 / 10.0, pass.substeps, pass.retried);
        assert!((frac(&st, "C", 0) - (1.0 - frac(&st, "He", 0))).abs() < 1e-9);
    }

    /// Ne_burn, order 1: X = X₀e^{−kt}, with an inert spectator (Fe: no record burns it below the gate).
    #[test]
    fn ne_burn_first_order() {
        let physics = pinned(&[("burn_dx", DX)]);
        let r = react(&physics, &only(&["Ne_burn"], &physics), true);
        let (sigma, t, x0) = (50.0, T_NE, 0.7);
        let k = 1e-3 * sigma;
        let mut st = world(1, 1, &[(sigma, t, x_of(&[("Ne", x0), ("Fe", 0.3)]))]);
        let at: Vec<f64> = [0.2, 0.5, 1.0, 1.5, 2.0, 10f64.ln()].iter().map(|v| v / k).collect();
        let pass = follow(&mut st, &r, "Ne", &at, |t| x0 * (-k * t).exp());
        println!("ne_burn: k = {k}, X_Ne = {:.6} (X₀/10 = {}), sub-steps {}, retried {}", frac(&st, "Ne", 0), x0 / 10.0, pass.substeps, pass.retried);
        assert!((frac(&st, "Fe", 0) - 0.3).abs() < 1e-12, "the spectator is untouched (renormalising only): {}", frac(&st, "Fe", 0));
        // Ne → O 0.4 + Mg 0.6, mass for mass.
        let lost = x0 - frac(&st, "Ne", 0);
        assert!((frac(&st, "O", 0) - 0.4 * lost).abs() < 1e-9 && (frac(&st, "Mg", 0) - 0.6 * lost).abs() < 1e-9, "O {} Mg {}", frac(&st, "O", 0), frac(&st, "Mg", 0));
    }

    /// The heat booked is q · Δm of the mass the planes say moved, within 1e-5 relative, per group and by record: H_burn's
    /// run of k X₀ t = 9 books it as the fuel falls — once from the first rate it would be 9× the mass moved, once from the
    /// last, a tenth. The accumulator of any other group stays 0.
    #[test]
    fn heat_booked_per_substep() {
        for (id, group, q_ratio, parts, sigma, t, run) in [
            ("H_burn", "H", 1.0, vec![("H", 1.0)], 100.0, T_H, 9.0 / (5e-4 * 100.0 * f_h(T_H, T_H))),
            ("Ne_burn", "Ne", 1.0 / 57.0, vec![("Ne", 0.7), ("Fe", 0.3)], 50.0, T_NE, 10f64.ln() / (1e-3 * 50.0)),
        ] {
            let physics = pinned(&[("burn_dx", DX)]);
            let r = react(&physics, &only(&[id], &physics), true);
            let mut st = world(1, 1, &[(sigma, t, x_of(&parts))]);
            let before = frac(&st, parts[0].0, 0);
            let pass = r.step(&mut st, run);
            let dm = sigma * (before - frac(&st, parts[0].0, 0));
            let want = q_ratio * Q_H * dm;
            let g = GROUPS.iter().position(|n| *n == group).unwrap();
            let got = pass.nuclear_total(g);
            println!("{id}: booked {got:.6} for q·Δm = {want:.6} (rel {:.2e}), sub-steps {}", (got - want).abs() / want, pass.substeps);
            assert!((got - want).abs() <= 1e-5 * want, "{id}: booked {got}, q·Δm {want}");
            for other in (0..N_GROUPS).filter(|&o| o != g) {
                assert_eq!(pass.nuclear_total(other), 0.0, "{id} books into {}", GROUPS[other]);
            }
        }
    }

    /// In a free run the cell's E gains exactly what is booked, and what is booked is q · Δm: the heat goes into the energy
    /// the same sub-step it is booked, T rises, and the sub-cycle's rate follows the T it reaches (a pinned T would not).
    #[test]
    fn heat_enters_the_energy() {
        let physics = physics(&[("burn_dx", 1e-3), ("burn_dt", 0.01)]);
        let r = react(&physics, &only(&["H_burn"], &physics), false);
        let (sigma, t0) = (1.0, 90.0);
        let mut st = world(1, 1, &[(sigma, t0, x_of(&[("H", 0.3), ("He", 0.7)]))]);
        let (e0, h0) = (st.get(3, 0), frac(&st, "H", 0));
        let pass = r.step(&mut st, 400.0);
        let dm = sigma * (h0 - frac(&st, "H", 0));
        let (de, booked) = (st.get(3, 0) - e0, pass.nuclear_total(0));
        println!("free H_burn: ΔX_H = {:.4}, T {t0} → {:.3}, E gained {de:.6}, booked {booked:.6}, q·Δm {:.6}, sub-steps {}", dm / sigma, temp(&st, 0), Q_H * dm, pass.substeps);
        assert!(dm > 0.05 * sigma, "the cell burned: ΔX {}", dm / sigma);
        assert!(temp(&st, 0) > 1.1 * t0, "T rose");
        assert!((de - booked).abs() <= 1e-9 * booked, "E gained {de}, booked {booked}");
        assert!((booked - Q_H * dm).abs() <= 1e-5 * Q_H * dm, "booked {booked}, q·Δm {}", Q_H * dm);
    }

    /// The limits hold at the shipped values (§2.5.1): no fraction moves more than burn_dx = 0.1 and T no more than
    /// burn_dt = 5 % in any sub-step, in a runaway (H's heat is 8,700 against ε_th of ~200: T climbs by a factor 40 once it
    /// lights). The limit is read from the sub-steps the pass accepted (`max_dx`, `max_dt_rel`), and the run did sub-cycle.
    #[test]
    fn shipped_limits_hold() {
        let physics = physics(&[]);
        assert_eq!((physics.get("burn_dx"), physics.get("burn_dt")), (Some(0.1), Some(0.05)));
        let r = react(&physics, &Reactions::shipped(&elements(), &physics).unwrap(), false);
        let mut st = world(1, 1, &[(1.0, 100.0, x_of(&[("H", 0.7346), ("He", 0.2485), ("O", 0.0077), ("C", 0.0092)]))]);
        let pass = r.step(&mut st, 50.0);
        println!("runaway at the shipped limits: max ΔX {:.4}, max ΔT/T {:.4}, sub-steps {}, retried {}, T {:.1}", pass.max_dx, pass.max_dt_rel, pass.substeps, pass.retried, temp(&st, 0));
        assert!(pass.substeps > 20, "the run did not sub-cycle: {} sub-steps", pass.substeps);
        assert!(pass.max_dx <= 0.1 * (1.0 + 1e-12), "ΔX {} > burn_dx", pass.max_dx);
        assert!(pass.max_dt_rel <= 0.05 * (1.0 + 1e-12), "ΔT/T {} > burn_dt", pass.max_dt_rel);
        assert!(pass.max_dt_rel > 0.02, "burn_dt was never what bound: the runaway is not one ({})", pass.max_dt_rel);
        assert_eq!(pass.unfinished, 0);
    }

    /// What the shipped burn_dx = 0.1 costs in accuracy, measured and only bounded: backward Euler at ΔX = 0.1 per sub-step
    /// reads H_burn's X₀/10 within a few percent of X₀ — far above G-BURN's 1e-4, which is why the analytic cases tighten
    /// burn_dx (module doc). The twin at the shipped limits stays on the right side of the closed form (BE overestimates a
    /// decay's remainder) and within 0.05. The limits are the shipped ones; the ladder the closed form's (a decay graded in
    /// k t, the same at any k).
    #[test]
    fn shipped_limits_error() {
        let physics = pinned(&[]);
        let r = react(&physics, &only(&["H_burn"], &physics), true);
        let (sigma, t, x0) = (100.0, T_H, 1.0);
        let k = 5e-4 * sigma * f_h(t, T_H);
        let run = 9.0 / (k * x0);
        let mut st = world(1, 1, &[(sigma, t, x_of(&[("H", x0)]))]);
        let pass = r.step(&mut st, run);
        let (got, want) = (frac(&st, "H", 0), x0 / (1.0 + k * x0 * run));
        println!("shipped burn_dx: X_H = {got:.5}, closed form {want:.5} (error {:.2e}), sub-steps {}", got - want, pass.substeps);
        assert!(got >= want && got - want < 0.05, "X {got}, closed form {want}");
    }

    /// ΣX = 1 within 1e-6 and every fraction ≥ 0 after a violent step over every record at once: all ten species present,
    /// two Σ, hot and cold, the gate open for iron.
    #[test]
    fn fractions_stay_physical() {
        let physics = physics(&[]);
        let r = react(&physics, &Reactions::shipped(&elements(), &physics).unwrap(), false);
        let mix = x_of(&[("H", 0.2), ("He", 0.2), ("C", 0.15), ("O", 0.15), ("Ne", 0.1), ("Mg", 0.05), ("Si", 0.05), ("S", 0.04), ("Fe", 0.06)]);
        let cells: Vec<_> = [(1.0, 150.0), (10.0, 300.0), (100.0, 450.0), (400.0, 520.0), (0.5, 60.0), (30.0, 250.0)].iter().map(|&(s, t)| (s, t, mix)).collect();
        let mut st = world(3, 2, &cells);
        let pass = r.step(&mut st, 5.0);
        println!("mixed cells: {} sub-steps, max in one cell {}, retried {}", pass.substeps, pass.max_substeps, pass.retried);
        assert_eq!(pass.unfinished, 0);
        for i in 0..6 {
            assert!((sum_x(&st, i) - 1.0).abs() <= 1e-6, "cell {i}: ΣX = {}", sum_x(&st, i));
            for s in 0..NS {
                assert!(st.get(4 + s, i) >= 0.0, "cell {i}: X[{s}] = {}", st.get(4 + s, i));
            }
        }
        assert!(pass.substeps > 6, "the cells burned");
    }

    /// A cell burns alone: a world of mixed cells steps each exactly as the same cell alone in a 1 × 1 world (bit for bit),
    /// and a cold cell and a vacuum cell are left as they were (bit for bit, nothing booked).
    #[test]
    fn cells_burn_alone() {
        let physics = physics(&[]);
        let r = react(&physics, &Reactions::shipped(&elements(), &physics).unwrap(), false);
        let cells = [
            (5.0, 120.0, x_of(&[("H", 1.0)])),
            (20.0, 200.0, x_of(&[("He", 0.6), ("C", 0.4)])),
            (1.0e-8, 400.0, x_of(&[("H", 1.0)])),
            (5.0, 20.0, x_of(&[("H", 0.5), ("He", 0.5)])),
            (50.0, 330.0, x_of(&[("Ne", 0.5), ("O", 0.3), ("Mg", 0.2)])),
            (50.0, 480.0, x_of(&[("Si", 0.6), ("S", 0.4)])),
        ];
        let mut st = world(3, 2, &cells);
        let before = st.clone();
        let pass = r.step(&mut st, 2.0);
        let n = 6;
        for (i, c) in cells.iter().enumerate() {
            let mut alone = world(1, 1, &[*c]);
            let solo = r.step(&mut alone, 2.0);
            for ch in 0..4 + NS {
                assert_eq!(st.planes[ch * n + i].to_bits(), alone.planes[ch].to_bits(), "cell {i}, channel {ch}");
            }
            for g in 0..N_GROUPS {
                assert_eq!(pass.nuclear[g * n + i].to_bits(), solo.nuclear[g].to_bits(), "cell {i}, group {g}");
            }
        }
        for i in [2, 3] {
            for ch in 0..4 + NS {
                assert_eq!(st.planes[ch * n + i].to_bits(), before.planes[ch * n + i].to_bits(), "cell {i} (vacuum or cold) channel {ch}");
            }
            assert!((0..N_GROUPS).all(|g| pass.nuclear[g * n + i] == 0.0), "cell {i} booked");
        }
        assert!((0..N_GROUPS).any(|g| pass.nuclear[g * n] != 0.0), "the hydrogen cell burned");
    }

    // P6 on the GPU (M0-T46): the same oracles, the same tolerances, f32.

    /// The GPU's `follow`: one pass per checkpoint, X within [`X_TOL`] of the closed form, ΣX = 1 within 1e-6, no cell
    /// unfinished. Returns the worst |ΔX| and the sub-steps taken.
    fn gpu_follow(r: &rig::OnGpu, species: &str, checkpoints: &[f64], oracle: impl Fn(f64) -> f64) -> (f64, u32) {
        let (mut t_prev, mut worst, mut substeps) = (0.0, 0.0f64, 0);
        for &t in checkpoints {
            let report = r.pass(t - t_prev);
            t_prev = t;
            let st = r.planes();
            let (got, want) = (frac(&st, species, 0), oracle(t));
            worst = worst.max((got - want).abs());
            substeps += report.substeps;
            assert!((got - want).abs() <= X_TOL, "GPU {species} at t = {t}: X = {got}, the closed form {want} (Δ {:.2e})", got - want);
            assert!((sum_x(&st, 0) - 1.0).abs() <= 1e-6, "GPU ΣX = {} at t = {t}", sum_x(&st, 0));
            assert_eq!(report.unfinished, 0, "a GPU cell ran out of attempts at t = {t}");
        }
        (worst, substeps)
    }

    /// G-BURN's three laws on the GPU, each to X₀/10: H_burn (order 2), He_burn (order 3), Ne_burn (order 1, Fe a spectator).
    #[test]
    fn gpu_the_three_laws_follow_their_closed_forms() {
        let gpu = super::super::device();
        let physics = pinned(&[("burn_dx", DX)]);
        let k_h = 5e-4 * 100.0 * f_h(T_H, T_H);
        let k_he = 1e-3 * 10.0 * 10.0;
        let k_ne = 1e-3 * 50.0;
        type Oracle = Box<dyn Fn(f64) -> f64>;
        let cases: [(&str, &str, (f64, f64, [f64; NS]), Vec<f64>, Oracle); 3] = [
            ("H_burn", "H", (100.0, T_H, x_of(&[("H", 1.0)])), [0.5, 1.0, 2.0, 4.0, 6.0, 9.0].iter().map(|v| v / k_h).collect(), Box::new(move |t| 1.0 / (1.0 + k_h * t))),
            ("He_burn", "He", (10.0, T_HE, x_of(&[("He", 1.0)])), [1.25, 2.0, 4.0, 9.0, 25.0, 100.0].iter().map(|m| (m - 1.0) / (2.0 * k_he)).collect(), Box::new(move |t| 1.0 / (1.0 + 2.0 * k_he * t).sqrt())),
            ("Ne_burn", "Ne", (50.0, T_NE, x_of(&[("Ne", 0.7), ("Fe", 0.3)])), [0.2, 0.5, 1.0, 1.5, 2.0, 10f64.ln()].iter().map(|v| v / k_ne).collect(), Box::new(move |t| 0.7 * (-k_ne * t).exp())),
        ];
        for (id, species, cell, at, oracle) in cases {
            let block = world(1, 1, &[cell]);
            let r = rig::OnGpu::new(&gpu, &physics, &only(&[id], &physics), true, &block);
            let (worst, substeps) = gpu_follow(&r, species, &at, oracle);
            let st = r.planes();
            println!("GPU {id}: worst |ΔX| {worst:.2e} (limit {X_TOL:e}), X = {:.6}, sub-steps {substeps}, T {:.4}", frac(&st, species, 0), temp(&st, 0));
            assert!((temp(&st, 0) - cell.1).abs() <= 1e-5 * cell.1, "T pinned: {}", temp(&st, 0));
            if id == "Ne_burn" {
                assert!((frac(&st, "Fe", 0) - 0.3).abs() <= 1e-6, "the spectator: {}", frac(&st, "Fe", 0));
            }
        }
    }

    /// The heat booked on the GPU is q · Δm of the mass the planes say moved, within 1e-5, in its group alone — H_burn and
    /// Ne_burn in one pass each down to X₀/10 (thousands of sub-steps): heat booked once per step would be 9× or a tenth off.
    #[test]
    fn gpu_heat_booked_per_substep() {
        let gpu = super::super::device();
        for (id, group, q_ratio, parts, sigma, t, run) in [
            ("H_burn", "H", 1.0, vec![("H", 1.0)], 100.0, T_H, 9.0 / (5e-4 * 100.0 * f_h(T_H, T_H))),
            ("Ne_burn", "Ne", 1.0 / 57.0, vec![("Ne", 0.7), ("Fe", 0.3)], 50.0, T_NE, 10f64.ln() / (1e-3 * 50.0)),
        ] {
            let physics = pinned(&[("burn_dx", DX)]);
            let block = world(1, 1, &[(sigma, t, x_of(&parts))]);
            let r = rig::OnGpu::new(&gpu, &physics, &only(&[id], &physics), true, &block);
            let before = frac(&r.planes(), parts[0].0, 0);
            let report = r.pass(run);
            let dm = sigma * (before - frac(&r.planes(), parts[0].0, 0));
            let want = q_ratio * Q_H * dm;
            let pass = rig::booked(r.nuclear());
            let g = GROUPS.iter().position(|n| *n == group).unwrap();
            let got = pass.nuclear_total(g);
            println!("GPU {id}: booked {got:.6} for q·Δm = {want:.6} (rel {:.2e}), sub-steps {}, retried {}", (got - want).abs() / want, report.substeps, report.retried);
            assert_eq!(report.unfinished, 0);
            assert!((got - want).abs() <= 1e-5 * want, "GPU {id}: booked {got}, q·Δm {want}");
            for other in (0..N_GROUPS).filter(|&o| o != g) {
                assert_eq!(pass.nuclear_total(other), 0.0, "GPU {id} books into {}", GROUPS[other]);
            }
        }
    }

    /// A free GPU run: E gains what is booked and what is booked is q · Δm, each within 1e-5 (f32 where the twin holds 1e-9
    /// on the first), and T rises.
    #[test]
    fn gpu_heat_enters_the_energy() {
        let gpu = super::super::device();
        let physics = physics(&[("burn_dx", 1e-3), ("burn_dt", 0.01)]);
        let (sigma, t0) = (1.0, 90.0);
        let block = world(1, 1, &[(sigma, t0, x_of(&[("H", 0.3), ("He", 0.7)]))]);
        let r = rig::OnGpu::new(&gpu, &physics, &only(&["H_burn"], &physics), false, &block);
        let start = r.planes();
        let report = r.pass(400.0);
        let st = r.planes();
        let dm = sigma * (frac(&start, "H", 0) - frac(&st, "H", 0));
        let (de, booked) = (st.get(3, 0) - start.get(3, 0), rig::booked(r.nuclear()).nuclear_total(0));
        println!("GPU free H_burn: ΔX_H = {:.4}, T {t0} → {:.3}, E gained {de:.6}, booked {booked:.6}, q·Δm {:.6}, sub-steps {}", dm / sigma, temp(&st, 0), Q_H * dm, report.substeps);
        assert!(dm > 0.05 * sigma && temp(&st, 0) > 1.1 * t0, "the cell burned and heated");
        assert!((de - booked).abs() <= 1e-5 * booked, "E gained {de}, booked {booked}");
        assert!((booked - Q_H * dm).abs() <= 1e-5 * Q_H * dm, "booked {booked}, q·Δm {}", Q_H * dm);
    }

    /// The shipped limits on the GPU (§2.5.1): a runaway (the Sun's mix from T_H, every record) and the six mixed cells over
    /// every record finish their pass, sub-cycled, ΣX = 1 within 1e-6 and every fraction ≥ 0. The runaway starts at the
    /// shipped T_H so it falls in the second pass (measured, M0-D55, twin and GPU alike: from T_H 56, T 67.1 after the first
    /// 5 t.u. in 4 sub-steps, then 198 to T 17,882; from 100, M0-TC's T_H, it runs away inside the first pass).
    #[test]
    fn gpu_fractions_stay_physical_at_the_shipped_limits() {
        let gpu = super::super::device();
        let physics = physics(&[]);
        let all = Reactions::shipped(&elements(), &physics).unwrap();
        let mix = x_of(&[("H", 0.2), ("He", 0.2), ("C", 0.15), ("O", 0.15), ("Ne", 0.1), ("Mg", 0.05), ("Si", 0.05), ("S", 0.04), ("Fe", 0.06)]);
        let sun = x_of(&[("H", 0.7346), ("He", 0.2485), ("O", 0.0077), ("C", 0.0092)]);
        let mut cells: Vec<_> = [(1.0, 150.0), (10.0, 300.0), (100.0, 450.0), (400.0, 520.0), (0.5, 60.0), (30.0, 250.0)].iter().map(|&(s, t)| (s, t, mix)).collect();
        let t_h = physics.get("t_h").unwrap();
        cells.extend([(1.0, t_h, sun), (1.0, t_h, sun)]);
        let block = world(4, 2, &cells);
        let r = rig::OnGpu::new(&gpu, &physics, &all, false, &block);
        let report = r.pass(5.0);
        let more = r.pass(45.0);
        let st = r.planes();
        println!("GPU shipped limits: {} + {} sub-steps, most in one cell {} / {}, retried {}, the runaway at T {:.1}", report.substeps, more.substeps, report.max_substeps, more.max_substeps, report.retried + more.retried, temp(&st, 6));
        assert_eq!(report.unfinished + more.unfinished, 0);
        assert!(more.max_substeps > 20 && temp(&st, 6) > 1000.0, "the runaway ran away, sub-cycled");
        for i in 0..8 {
            assert!((sum_x(&st, i) - 1.0).abs() <= 1e-6, "cell {i}: ΣX = {}", sum_x(&st, i));
            assert!((0..NS).all(|s| st.get(4 + s, i) >= 0.0), "cell {i}: a negative fraction");
        }
    }

    /// A GPU cell burns alone: a 3 × 2 world of mixed cells steps each as the same cell alone (every channel and every
    /// group's slot, bit for bit); the cold cell is left bit for bit and the vacuum cell books nothing.
    #[test]
    fn gpu_cells_burn_alone() {
        let gpu = super::super::device();
        let physics = physics(&[]);
        let all = Reactions::shipped(&elements(), &physics).unwrap();
        let cells = [
            (5.0, 120.0, x_of(&[("H", 1.0)])),
            (20.0, 200.0, x_of(&[("He", 0.625), ("C", 0.375)])),
            (1.0e-8, 400.0, x_of(&[("H", 1.0)])),
            (5.0, 20.0, x_of(&[("H", 0.5), ("He", 0.5)])),
            (50.0, 330.0, x_of(&[("Ne", 0.5), ("O", 0.25), ("Mg", 0.25)])),
            (50.0, 480.0, x_of(&[("Si", 0.625), ("S", 0.375)])),
        ];
        let block = world(3, 2, &cells);
        let r = rig::OnGpu::new(&gpu, &physics, &all, false, &block);
        let before = r.planes();
        r.pass(2.0);
        let (st, nuclear) = (r.planes(), r.nuclear());
        let solo = rig::OnGpu::new(&gpu, &physics, &all, false, &world(1, 1, &[cells[0]]));
        for (i, c) in cells.iter().enumerate().filter(|(i, _)| *i != 2) {
            solo.load(&world(1, 1, &[*c]));
            solo.pass(2.0);
            let (alone, booked) = (solo.planes(), solo.nuclear());
            for ch in 0..4 + NS {
                assert_eq!(st.planes[ch * 6 + i].to_bits(), alone.planes[ch].to_bits(), "cell {i}, channel {ch}");
            }
            for g in 0..N_GROUPS {
                assert_eq!(nuclear[g * 6 + i].to_bits(), booked[g].to_bits(), "cell {i}, group {g}");
            }
        }
        for ch in 0..4 + NS {
            assert_eq!(st.planes[ch * 6 + 3].to_bits(), before.planes[ch * 6 + 3].to_bits(), "the cold cell, channel {ch}");
        }
        assert!((0..N_GROUPS).all(|g| nuclear[g * 6 + 2] == 0.0 && nuclear[g * 6 + 3] == 0.0), "the vacuum or the cold cell booked");
        assert!((0..N_GROUPS).any(|g| nuclear[g * 6] != 0.0), "the hydrogen cell burned");
    }

    /// The adversarial cost: a hot core (8 × 8 cells burning hydrogen at the shipped limits, a runaway) inside a cold field
    /// (40 × 40 at T 20) against the same field all cold — P6's ms for one step of 1 t.u. are printed (timing is the
    /// Quadro's, R12: run with SR_TEST_ADAPTER); asserted: the core finishes its pass and the field books nothing.
    #[test]
    fn gpu_a_hot_core_in_a_cold_field() {
        let gpu = super::super::device();
        let physics = physics(&[]);
        let all = Reactions::shipped(&elements(), &physics).unwrap();
        let sun = x_of(&[("H", 0.7346), ("He", 0.2485), ("O", 0.0077), ("C", 0.0092)]);
        let side = 40;
        let core = |i: usize| (16..24).contains(&(i % side)) && (16..24).contains(&(i / side));
        let field: Vec<_> = (0..side * side).map(|i| (1.0, if core(i) { 140.0 } else { 20.0 }, sun)).collect();
        let cold: Vec<_> = (0..side * side).map(|_| (1.0, 20.0, sun)).collect();
        let r = rig::OnGpu::new(&gpu, &physics, &all, false, &world(side, side, &cold));
        let cold_ms = r.timed_ms(1.0);
        r.load(&world(side, side, &field));
        let hot_ms = r.timed_ms(1.0);
        let report = r.step_report();
        let nuclear = r.nuclear();
        println!("GPU hot core: P6 {hot_ms:.4} ms against {cold_ms:.4} ms all cold; {} sub-steps, at most {} in one cell, retried {}", report.substeps, report.max_substeps, report.retried);
        assert_eq!(report.unfinished, 0, "a core cell ran out of attempts");
        assert!(report.max_substeps > 10, "the core sub-cycled");
        let n = side * side;
        for i in (0..n).filter(|&i| !core(i)) {
            assert!((0..N_GROUPS).all(|g| nuclear[g * n + i] == 0.0), "field cell {i} booked");
        }
        assert!((0..n).filter(|&i| core(i)).all(|i| nuclear[i] > 0.0), "every core cell burned");
    }

    /// M0-D35 — a runaway on the GPU books as the twin (§0.5 [M0-D34]: the Gamow-shaped law on both sides, the GPU's rates
    /// past f32's e^88 held by its rate scale, never by clipping one record's f). Under the shipped constants (ref_burn.json's,
    /// the cold pressure on) and every record, one cell of G-REF's burning field (Σ 30, T 100, X_H 0.75, X_He 0.25) is
    /// marched by the twin at M0-D34's step-44 Δt (6.3056e-3) until the pass in which it runs away to iron (pass 47, T 143 →
    /// 18,800 under the Gamow law; step 44 was the power law's); the GPU runs that pass from the twin's state before it, and
    /// again from the same matter at T 2,520 (6 T_Si: He_burn's ln f ≈ 72 from the first sub-step). On both: each group's
    /// booking within 1e-5 of the pass's nuclear energy (Σ_g |booked_g|, the twin's) of the twin's — the groups that carry it
    /// read ≤ 4e-8 of their own value, the trace groups C and Ne (≤ 1.6e-6 of the pass) 1.3e-5 of theirs, f32's cancellation
    /// in a species burnt as fast as it is made — and ΔE within 1e-5 of the twin's, every plane and slot finite, ΣX = 1 within
    /// 1e-6 and nothing negative, the pass finished. Two cells guard the scale's edges the same way, bookings and ΔE graded on
    /// the energy f32 resolves: M0-D34's open NaN (T 47,000, iron with He 1e-13 and C 1e-25 — trace records with the largest
    /// f set the shift) and He_burn's peak (T 3.2 × 10⁷, ln f = 111.8, the largest shift any shipped record asks). Clipping
    /// each record's ln f at the scale's ceiling flattens the ratios between competing records (C_burn's e^69 against
    /// C_alpha's e^95 at T 18,800) and moves the groups' shares — the burn-clip-ln-f plant.
    #[test]
    fn gpu_the_runaway_books_as_the_twin() {
        use sr_physics::eos::Eos;
        let gpu = super::super::device();
        let physics = Physics::shipped().unwrap();
        let all = Reactions::shipped(&elements(), &physics).unwrap();
        let r = react(&physics, &all, false);
        let eos = Eos::from_physics(&physics).unwrap();
        let id_n = elements().index_of("n").unwrap();
        let sigma = 30.0;
        let dt = 6.3056e-3;
        let t_of = |st: &State| {
            let x: [f64; NS] = std::array::from_fn(|s| st.get(4 + s, 0));
            let comp = elements().composition(&x);
            Eos::temperature(st.get(3, 0) / sigma - eos.eps_cold(sigma, comp.y_e, x[id_n]), 1.0 / comp.inv_mu)
        };
        let cell = |t: f64, x: [f64; NS]| {
            let comp = elements().composition(&x);
            let mut planes = vec![0.0; 4 + NS];
            planes[0] = sigma;
            planes[3] = sigma * (Eos::eps_th_of_t(t, 1.0 / comp.inv_mu) + eos.eps_cold(sigma, comp.y_e, x[id_n]));
            planes[4..].copy_from_slice(&x);
            State::new(1, 1, planes)
        };
        let field = x_of(&[("H", 0.75), ("He", 0.25)]);
        let mut runaway = cell(100.0, field);
        let mut steps = 0;
        loop {
            let mut next = runaway.clone();
            r.step(&mut next, dt);
            if frac(&next, "Fe", 0) > 0.5 {
                break;
            }
            runaway = next;
            steps += 1;
            assert!(steps < 1000, "the field never ran away");
        }
        println!("the twin's field runs away in pass {steps} at Δt {dt}, from T {:.2}", t_of(&runaway));
        let cells = [
            (format!("pass {steps}"), runaway, true),
            ("T 2,520".to_string(), cell(6.0 * 420.0, field), true),
            ("trace He in iron, T 47,000".to_string(), cell(47_000.0, x_of(&[("He", 1e-13), ("C", 1e-25), ("Fe", 1.0 - 1e-13)])), false),
            ("He_burn's peak, T 3.2e7".to_string(), cell(140.0 * 61f64.powi(3), field), false),
        ];
        for (name, block, graded) in cells {
            let energy = block.get(3, 0);
            let mut twin = block.clone();
            let t0 = t_of(&block);
            let pass = r.step(&mut twin, dt);
            let on = rig::OnGpu::new(&gpu, &physics, &all, false, &block);
            let report = on.pass(dt);
            let (st, nuclear) = (on.planes(), on.nuclear());
            let (de_twin, de_gpu) = (twin.get(3, 0) - energy, st.get(3, 0) - f64::from(energy as f32));
            let groups: Vec<(f64, f64)> = (0..N_GROUPS).map(|g| (nuclear[g], pass.nuclear_total(g))).collect();
            let total: f64 = groups.iter().map(|(_, want)| want.abs()).sum();
            // What f32 resolves of the cell's energy, 2^-23 E: the floor of every energy comparison.
            let resolved = energy * f64::from(f32::EPSILON);
            println!(
                "GPU runaway, {name}: T {t0:.1} → twin {:.1}, X_Fe twin {:.6} GPU {:.6}; ΔE GPU {de_gpu:.6e} twin {de_twin:.6e}, f32 resolves {resolved:.1e}; \
                 sub-steps GPU {} twin {}, retried {} / {}",
                t_of(&twin),
                frac(&twin, "Fe", 0),
                frac(&st, "Fe", 0),
                report.substeps,
                pass.substeps,
                report.retried,
                pass.retried,
            );
            for (g, (got, want)) in groups.iter().enumerate() {
                println!("  group {}: GPU {got:.7e} twin {want:.7e} ({:.2e} of the group, {:.2e} of the pass)", GROUPS[g], (got - want).abs() / want.abs().max(f64::MIN_POSITIVE), (got - want).abs() / total);
            }
            assert_eq!((report.unfinished, pass.unfinished), (0, 0), "{name}: a pass ran out of attempts");
            assert!((0..4 + NS).all(|c| st.get(c, 0).is_finite()) && nuclear.iter().all(|v| v.is_finite()), "{name}: a non-finite value");
            assert!((sum_x(&st, 0) - 1.0).abs() <= 1e-6 && (0..NS).all(|c| st.get(4 + c, 0) >= 0.0), "{name}: ΣX {} or a negative fraction", sum_x(&st, 0));
            assert!(frac(&twin, "Fe", 0) > 0.5 && frac(&st, "Fe", 0) > 0.5, "{name}: the cell did not run away to iron");
            let floor = if graded { 1e-5 * total } else { (1e-5 * total).max(resolved) };
            for (g, (got, want)) in groups.iter().enumerate() {
                assert!((got - want).abs() <= floor, "{name}: group {} booked {got:e} on the GPU, {want:e} in the twin", GROUPS[g]);
            }
            let de_floor = if graded { 1e-5 * de_twin.abs() } else { (1e-5 * de_twin.abs()).max(resolved) };
            assert!((de_gpu - de_twin).abs() <= de_floor, "{name}: ΔE {de_gpu:e} on the GPU, {de_twin:e} in the twin");
        }
    }
}

/// G-ORDER.
mod order {
    use super::*;

    /// ε* = 10⁻³ Q_H per t.u., specific power (§5.1).
    const EPS_STAR: f64 = 1e-3 * Q_H;
    /// The shipped ladder's T_H and T_He (§0.5 [M0-D50]): G-ORDER grades the order the game's constants give, H's power at
    /// its crossing by hand at them.
    const T_H: f64 = 56.0;
    const T_HE: f64 = 78.4;
    /// The ramp: T = T₀ + rate · t, sampled every `DT_RAMP` t.u.; each sample reads the power of a fresh cell of equal parts
    /// from one pass of `DT_BURN`, short enough that no fuel is spent in it (the rate over the fraction, times 10⁻⁹, is far
    /// under 1 at every crossing): the instantaneous power at the sample's T. A cell left to burn along the ramp would not
    /// define the order — C_alpha spends the carbon C_burn would have used, the products of an earlier rung feed the later
    /// ones and a slow ramp burns every fuel out below ε* (measured, the scratch run of M0-T44b) — so the composition is the
    /// contract's equal parts, held.
    const T0: f64 = 40.0;
    const RATE: f64 = 100.0;
    const DT_RAMP: f64 = 0.005;
    const DT_BURN: f64 = 1e-9;

    fn parts() -> [(&'static str, f64); 6] {
        [("H", 1.0 / 6.0), ("He", 1.0 / 6.0), ("C", 1.0 / 6.0), ("O", 1.0 / 6.0), ("Ne", 1.0 / 6.0), ("Si", 1.0 / 6.0)]
    }

    /// The group's specific power in a cell of equal parts at `t`, from one short pass (T pinned, Σ = 1).
    fn powers(r: &React, t: f64) -> [f64; N_GROUPS] {
        let mut st = world(1, 1, &[(1.0, t, x_of(&parts()))]);
        let pass = r.step(&mut st, DT_BURN);
        std::array::from_fn(|g| pass.nuclear_total(g) / (1.0 * DT_BURN))
    }

    /// The groups first passing ε* in the order H, He, C, {Ne, O}, Si (K6; SM-O12): the first sample at which each group's
    /// specific power reaches ε* is strictly later than the previous rung's (Ne and O may tie or swap), the crossings
    /// bracket the ladder's own temperatures, and a group the cell does not hold (N) never passes.
    #[test]
    fn ignition_order() {
        let physics = physics(&[]);
        let r = react(&physics, &Reactions::shipped(&elements(), &physics).unwrap(), true);
        let mut first = [None::<f64>; N_GROUPS];
        let samples = ((600.0 - T0) / (RATE * DT_RAMP)) as usize;
        for k in 0..=samples {
            let t_time = k as f64 * DT_RAMP;
            let t = T0 + RATE * t_time;
            for (g, p) in powers(&r, t).iter().enumerate() {
                if first[g].is_none() && *p >= EPS_STAR {
                    first[g] = Some(t);
                }
            }
        }
        let at = |g: &str| first[GROUPS.iter().position(|n| *n == g).unwrap()].unwrap_or_else(|| panic!("group {g} never passed ε* = {EPS_STAR}"));
        let (h, he, c, ne, o, si) = (at("H"), at("He"), at("C"), at("Ne"), at("O"), at("Si"));
        println!("ε* crossings at T: H {h}, He {he}, C {c}, Ne {ne}, O {o}, Si {si}");
        assert!(h < he, "H {h} before He {he}");
        assert!(he < c, "He {he} before C {c}");
        assert!(c < ne.min(o), "C {c} before Ne {ne} and O {o}");
        assert!(ne.max(o) < si, "Ne {ne} and O {o} before Si {si}");
        // The measure itself: H's power at its crossing is the formula's, q A Σ X² f(T), from §1.6.2 and §2.5.1 by hand.
        let ih = GROUPS.iter().position(|g| *g == "H").unwrap();
        let (got, want) = (powers(&r, h)[ih], Q_H * 5e-4 * (1.0 / 36.0) * f_h(h, T_H));
        assert!((got - want).abs() <= 1e-6 * want, "H's power at T = {h}: {got}, by hand {want}");
        // Each rung sits at or above its own reference temperature's order: no group lights below T_thr of its first term.
        assert!(h >= 0.5 * T_H && he >= 0.5 * T_HE, "ignition below T_thr: H {h}, He {he}");
        let n = GROUPS.iter().position(|g| *g == "N").unwrap();
        assert!(first[n].is_none(), "a cell with no iron books N");
    }

    /// Every record books into its own group and no other: each record alone, its inputs only, one short pass (T pinned at
    /// twice its threshold temperature, Σ chosen above the gate for N_Fe); the group's accumulator is the heat of the mass
    /// moved, q · Δm to 1e-5, every other group's is 0 — C_alpha books into He, Mg_burn into O, S_burn into Si.
    #[test]
    fn records_book_into_their_group() {
        let physics = physics(&[]);
        let e = elements();
        let all = Reactions::shipped(&e, &physics).unwrap();
        for rec in all.records() {
            let r = react(&physics, &only(&[rec.id.as_str()], &physics), true);
            let parts: Vec<(&str, f64)> = rec.inputs.iter().map(|s| (e.species()[s.species].key.as_str(), s.share)).collect();
            // Σ above the gate for N_Fe (Y_e Σ > Σ_N = 160, Y_e of iron ≈ 0.46); T at 1.2 × the record's first term.
            let sigma = if rec.rate.gate.is_some() { 600.0 } else { 2.0 };
            let t = rec.rate.terms.first().map_or(1.0, |t| 1.2 * t.t_k);
            let mut st = world(1, 1, &[(sigma, t, x_of(&parts))]);
            let before: Vec<f64> = (0..NS).map(|s| st.get(4 + s, 0)).collect();
            let pass = r.step(&mut st, 1e-3);
            let moved = sigma * (before[rec.inputs[0].species] - st.get(4 + rec.inputs[0].species, 0)) / rec.inputs[0].share;
            let g = GROUPS.iter().position(|n| *n == rec.group).unwrap();
            let want = rec.q_ratio * Q_H * moved;
            println!("{}: group {}, booked {:.6e}, q·Δm {want:.6e}", rec.id, rec.group, pass.nuclear_total(g));
            assert!(moved > 0.0, "{} did not burn", rec.id);
            assert!((pass.nuclear_total(g) - want).abs() <= 1e-5 * want.abs(), "{}: booked {}, q·Δm {want}", rec.id, pass.nuclear_total(g));
            for other in (0..N_GROUPS).filter(|&o| o != g) {
                assert_eq!(pass.nuclear_total(other), 0.0, "{} books into {}", rec.id, GROUPS[other]);
            }
        }
    }
    /// G-ORDER on the GPU: every sample of the ramp is one cell of a 40 × 29 world (T = 40 + 0.5 k, the twin's samples), one
    /// pass of 1e-9 — each group's specific power first reaches ε* in the order H, He, C, {Ne, O}, Si, each crossing within
    /// one sample of the twin's, H's power at its crossing q A Σ X² f(T) by hand within 1e-4 (§1.6.1's table budget), N never.
    #[test]
    fn gpu_ignition_order() {
        let gpu = super::super::device();
        let physics = physics(&[]);
        let all = Reactions::shipped(&elements(), &physics).unwrap();
        let samples = ((600.0 - T0) / (RATE * DT_RAMP)) as usize + 1;
        let (w, h) = (40, samples.div_ceil(40));
        let t_of = |k: usize| T0 + RATE * (k as f64 * DT_RAMP);
        let cells: Vec<_> = (0..w * h).map(|k| (1.0, t_of(k.min(samples - 1)), x_of(&parts()))).collect();
        let r = rig::OnGpu::new(&gpu, &physics, &all, true, &world(w, h, &cells));
        r.pass(DT_BURN);
        let nuclear = r.nuclear();
        let n = w * h;
        let power = |g: usize, k: usize| nuclear[g * n + k] / DT_BURN;
        let first: Vec<Option<usize>> = (0..N_GROUPS).map(|g| (0..samples).find(|&k| power(g, k) >= EPS_STAR)).collect();
        let twin = react(&physics, &all, true);
        let at = |name: &str| {
            let g = GROUPS.iter().position(|n| *n == name).unwrap();
            let k = first[g].unwrap_or_else(|| panic!("GPU: group {name} never passed ε*"));
            let k_twin = (0..samples).find(|&k| powers(&twin, t_of(k))[g] >= EPS_STAR).unwrap();
            assert!(k.abs_diff(k_twin) <= 1, "{name}: the GPU crosses at T {}, the twin at {}", t_of(k), t_of(k_twin));
            (t_of(k), k)
        };
        let ((h_t, h_k), (he, _), (c, _), (ne, _), (o, _), (si, _)) = (at("H"), at("He"), at("C"), at("Ne"), at("O"), at("Si"));
        println!("GPU ε* crossings at T: H {h_t}, He {he}, C {c}, Ne {ne}, O {o}, Si {si}");
        assert!(h_t < he && he < c && c < ne.min(o) && ne.max(o) < si, "the order H, He, C, {{Ne, O}}, Si");
        let want = Q_H * 5e-4 * (1.0 / 36.0) * f_h(h_t, T_H);
        assert!((power(0, h_k) - want).abs() <= 1e-4 * want, "H's power at T = {h_t}: {}, by hand {want}", power(0, h_k));
        assert!(first[GROUPS.iter().position(|g| *g == "N").unwrap()].is_none(), "a cell with no iron books N");
    }

    /// Every record alone on the GPU books into its own group and no other, q · Δm of the mass moved within 1e-5.
    #[test]
    fn gpu_records_book_into_their_group() {
        let gpu = super::super::device();
        let physics = physics(&[]);
        let e = elements();
        for rec in Reactions::shipped(&e, &physics).unwrap().records() {
            let parts: Vec<(&str, f64)> = rec.inputs.iter().map(|s| (e.species()[s.species].key.as_str(), s.share)).collect();
            let sigma = if rec.rate.gate.is_some() { 600.0 } else { 2.0 };
            let t = rec.rate.terms.first().map_or(1.0, |t| 1.2 * t.t_k);
            let block = world(1, 1, &[(sigma, t, x_of(&parts))]);
            let alone = only(&[rec.id.as_str()], &physics);
            // A pass long enough to burn about a fifth of the input, so f32's planes resolve Δm to 1e-5 (the twin's 1e-3
            // moves 4e-6 of hydrogen, under f32's step at 1); its length from the twin's rate.
            let input = rec.inputs[0].species;
            let mut probe = block.clone();
            react(&physics, &alone, true).step(&mut probe, 1e-3);
            let dt = 1e-3 * 0.2 * block.get(4 + input, 0) / (block.get(4 + input, 0) - probe.get(4 + input, 0));
            let r = rig::OnGpu::new(&gpu, &physics, &alone, true, &block);
            let before = r.planes().get(4 + input, 0);
            r.pass(dt);
            let moved = sigma * (before - r.planes().get(4 + input, 0)) / rec.inputs[0].share;
            let pass = rig::booked(r.nuclear());
            let g = GROUPS.iter().position(|n| *n == rec.group).unwrap();
            let want = rec.q_ratio * Q_H * moved;
            println!("GPU {}: group {}, ΔX {:.3} over {dt:.3e}, booked {:.6e}, q·Δm {want:.6e}", rec.id, rec.group, moved * rec.inputs[0].share / sigma, pass.nuclear_total(g));
            assert!(moved > 0.0, "{} did not burn", rec.id);
            assert!((pass.nuclear_total(g) - want).abs() <= 1e-5 * want.abs(), "GPU {}: booked {}, q·Δm {want}", rec.id, pass.nuclear_total(g));
            for other in (0..N_GROUPS).filter(|&o| o != g) {
                assert_eq!(pass.nuclear_total(other), 0.0, "GPU {} books into {}", rec.id, GROUPS[other]);
            }
        }
    }
}

