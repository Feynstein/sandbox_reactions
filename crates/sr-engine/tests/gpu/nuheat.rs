//! M0-T53 — neutrino heating on the CPU twin (contract §2.5.4, §1.6.4, §2.7's f_dep, §2.9, R8, R13); no adapter is touched.
//! M0-T54 — P7 on the GPU (`Step::with_neutrino_heating`, shaders/nuheat.wgsl; the cases `gpu_…`, the physics adapter, R12), graded
//! against the contract's arithmetic and the twin from the same f32 start.
//! The oracle is the contract's own arithmetic, kept apart from the FFT: D(x) = Σ(x) Σ_y S_ν(y) / (r² + 1) summed directly
//! over the cells, q(x) = f_dep (Σ S_ν) D(x) / Σ D. `NOT PROVEN (synthetic)` beyond the formula itself: the worlds are
//! hand-made fields, not a collapsed star's.

use sr_engine::gpu::Gpu;
use sr_engine::state::{book_totals, Booking, SideFields, State as GpuState, WorldConfig, BOOK_GROUPS, TERMS};
use sr_engine::step::boxfit::BoxMode;
use sr_engine::step::latch::{FramePlan, FrameReport};
use sr_engine::step::nuheat::{NuHeatReport, NuHeatSetup};
use sr_engine::step::{CostSource, EosGpu, Passes, Pass, Step, Timing};
use sr_physics::reference::hydro::State;
use sr_physics::reference::nuheat::NuHeat;
use sr_physics::registry::{Elements, Physics, N_SPECIES};

const W: usize = 24;
const H: usize = 20;
/// Neutron matter's species plane (§2.2: channels 4–13, slot 9).
const X_N: usize = 4 + 9;
const E: usize = 3;

fn physics(f_dep: f64) -> Physics {
    Physics::shipped().unwrap().with(&[("f_dep", f_dep)]).unwrap_or_else(|e| panic!("{e}"))
}

/// A 24 × 20 world: Σ from a fixed LCG in [0.5, 1.5) (0 where `empty`), hydrogen, E = Σ; the cells `star` neutron matter
/// (X_n = 1, S_ν = 3 + their index mod 5).
fn world(star: &dyn Fn(usize, usize) -> bool, empty: &dyn Fn(usize, usize) -> bool) -> (State, Vec<f64>) {
    build(W, H, star, empty)
}

/// [`world`] at any size.
fn build(w: usize, h: usize, star: &dyn Fn(usize, usize) -> bool, empty: &dyn Fn(usize, usize) -> bool) -> (State, Vec<f64>) {
    let n = w * h;
    let hydrogen = Elements::shipped().unwrap().index_of("H").unwrap();
    assert!(hydrogen < N_SPECIES);
    let mut planes = vec![0.0; (4 + N_SPECIES) * n];
    let mut s_nu = vec![0.0; n];
    let mut seed = 12345u64;
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let sigma = if empty(x, y) { 0.0 } else { 0.5 + (seed >> 11) as f64 / (1u64 << 53) as f64 };
            planes[i] = sigma;
            planes[E * n + i] = sigma;
            if star(x, y) {
                planes[X_N * n + i] = 1.0;
                s_nu[i] = 3.0 + (i % 5) as f64;
            } else {
                planes[(4 + hydrogen) * n + i] = 1.0;
            }
        }
    }
    (State::new(w, h, planes), s_nu)
}

fn blob(x: usize, y: usize) -> bool {
    (4..8).contains(&x) && (5..9).contains(&y)
}

fn nowhere(_: usize, _: usize) -> bool {
    false
}

fn rel(got: f64, want: f64) -> f64 {
    (got - want).abs() / want.abs().max(f64::MIN_POSITIVE)
}

/// The contract's D(x), by the direct double sum over the cells (no FFT).
fn direct_d(st: &State, s_nu: &[f64]) -> Vec<f64> {
    let (sigma, x_n) = (st.plane(0), st.plane(X_N));
    (0..W * H)
        .map(|i| {
            if x_n[i] >= 0.5 {
                return 0.0;
            }
            let (xi, yi) = ((i % W) as f64, (i / W) as f64);
            let reach: f64 = (0..W * H)
                .map(|j| {
                    let (dx, dy) = (xi - (j % W) as f64, yi - (j / W) as f64);
                    s_nu[j] / (dx * dx + dy * dy + 1.0)
                })
                .sum();
            sigma[i] * reach
        })
        .collect()
}

/// Σ q = f_dep × Σ S_ν to 1e-9, at the shipped share, at the bound 0.1 and at 0 (nothing) — the deposit is the share of
/// the emitted power, never more; `apply` adds exactly Σ q Δt to E and books it.
#[test]
fn deposited_is_f_dep_of_emitted() {
    for &f_dep in &[0.05, 0.1, 0.01] {
        let (mut st, s_nu) = world(&blob, &nowhere);
        let mut p7 = NuHeat::new(&physics(f_dep));
        let pass = p7.heating(&st, &s_nu).unwrap();
        let emitted: f64 = s_nu.iter().sum();
        assert_eq!(pass.emitted, emitted);
        let got: f64 = pass.q.iter().sum();
        assert!(rel(got, f_dep * emitted) < 1e-9, "f_dep {f_dep}: deposited {got} against {}", f_dep * emitted);
        assert!(rel(pass.deposited, got) < 1e-12);
        assert!(pass.q.iter().all(|&q| q >= 0.0));

        let (dt, e0): (f64, f64) = (0.25, st.plane(E).iter().sum());
        let booked = p7.apply(&mut st, &s_nu, dt).unwrap();
        let e1: f64 = st.plane(E).iter().sum();
        assert!(rel(booked, f_dep * emitted * dt) < 1e-9, "booked {booked}");
        assert!(rel(e1 - e0, booked) < 1e-9, "E rose {} against the booking {booked}", e1 - e0);
        assert_eq!(p7.deposited, booked);
    }
    let (st, s_nu) = world(&blob, &nowhere);
    let none = NuHeat::new(&physics(0.0)).heating(&st, &s_nu).unwrap();
    assert!(none.q.iter().all(|&q| q == 0.0) && none.deposited == 0.0, "f_dep = 0 deposits nothing");
}

/// The FFT convolution against the contract's direct sum: q(x) = f_dep (Σ S_ν) D(x) / Σ D(x), every cell within 1e-9 of
/// the largest.
#[test]
fn matches_the_direct_sum() {
    let (st, s_nu) = world(&blob, &nowhere);
    let mut p7 = NuHeat::new(&physics(0.05));
    let pass = p7.heating(&st, &s_nu).unwrap();
    let d = direct_d(&st, &s_nu);
    let (total, emitted): (f64, f64) = (d.iter().sum(), s_nu.iter().sum());
    let top = pass.q.iter().cloned().fold(0.0, f64::max);
    let worst = (0..W * H).map(|i| (pass.q[i] - 0.05 * emitted * d[i] / total).abs()).fold(0.0, f64::max);
    assert!(worst < 1e-9 * top, "worst cell off by {worst:e} of {top:e}");
}

/// None inside the neutron star: X_n = 1 and X_n = 0.5 exactly get 0 (bit for bit), 0.4999 gets its share.
#[test]
fn none_inside_the_neutron_star() {
    let (mut st, s_nu) = world(&blob, &nowhere);
    let n = W * H;
    let (half, almost) = (10 * W + 12, 10 * W + 13);
    st.planes[X_N * n + half] = 0.5;
    st.planes[X_N * n + almost] = 0.4999;
    let pass = NuHeat::new(&physics(0.05)).heating(&st, &s_nu).unwrap();
    for i in 0..n {
        if st.get(X_N, i) >= 0.5 {
            assert_eq!(pass.q[i], 0.0, "cell {i} is neutron star");
        }
    }
    assert!(pass.q[almost] > 0.0, "X_n 0.4999 is gas");
    assert_eq!(pass.q[half], 0.0);
    assert!(pass.q.iter().sum::<f64>() > 0.0);
}

/// Nearer and denser receive more: equal Σ at 3 and at 8 cells from a lone source cell, the nearer one more by the kernel's
/// ratio (64 + 1)/(9 + 1) = 6.5; equal distance, Σ 1 against 2: twice. The energy stays f_dep of the emitted.
#[test]
fn nearer_and_denser_receive_more() {
    let n = W * H;
    let (mut st, _) = world(&nowhere, &|_, _| true);
    let h = Elements::shipped().unwrap().index_of("H").unwrap();
    let at = |x: usize, y: usize| y * W + x;
    let mut s_nu = vec![0.0; n];
    let src = at(2, 10);
    s_nu[src] = 7.0;
    st.planes[X_N * n + src] = 1.0;
    for (x, y, sigma) in [(5, 10, 1.0), (10, 10, 1.0), (2, 4, 1.0), (2, 16, 2.0)] {
        st.planes[at(x, y)] = sigma;
        st.planes[(4 + h) * n + at(x, y)] = 1.0;
    }
    let pass = NuHeat::new(&physics(0.05)).heating(&st, &s_nu).unwrap();
    let (near, far, a, b) = (pass.q[at(5, 10)], pass.q[at(10, 10)], pass.q[at(2, 4)], pass.q[at(2, 16)]);
    assert!(near > far, "nearer {near} against {far}");
    // the kernel at 3 and at 8 cells, plus the unit's 1
    assert!(rel(near / far, (64.0 + 1.0) / (9.0 + 1.0)) < 1e-6, "ratio {}", near / far);
    assert!(rel(b / a, 2.0) < 1e-6, "twice the Σ at the same distance: {}", b / a);
    assert!(rel(pass.q.iter().sum::<f64>(), 0.05 * 7.0) < 1e-9);
}

/// Adversarial: normalised over ΣD, not per cell. One lone gas cell takes exactly f_dep of what the core emitted, whatever
/// its own D — a share normalised per cell instead would hand it an amount tied to its D, and a sparse world would exceed
/// what the core emitted.
#[test]
fn a_lone_gas_cell_takes_exactly_f_dep() {
    let n = W * H;
    let (mut st, _) = world(&nowhere, &|_, _| true);
    let h = Elements::shipped().unwrap().index_of("H").unwrap();
    let mut s_nu = vec![0.0; n];
    s_nu[3 * W + 3] = 11.0;
    st.planes[X_N * n + 3 * W + 3] = 1.0;
    let gas = 15 * W + 20;
    st.planes[gas] = 0.7;
    st.planes[(4 + h) * n + gas] = 1.0;
    let pass = NuHeat::new(&physics(0.1)).heating(&st, &s_nu).unwrap();
    assert!(rel(pass.q[gas], 0.1 * 11.0) < 1e-9, "the lone cell got {}", pass.q[gas]);
    assert!(pass.deposited <= pass.emitted);
}

/// Nothing emitted, nothing deposited: ΣS_ν = 0 leaves E bit for bit and books 0; a world that is all neutron star has
/// no gas to take the share (ΣD = 0) and deposits nothing, finite.
#[test]
fn nothing_when_nothing_is_emitted_or_taken() {
    let (mut st, _) = world(&blob, &nowhere);
    let before = st.clone();
    let mut p7 = NuHeat::new(&physics(0.05));
    let silent = vec![0.0; W * H];
    assert_eq!(p7.apply(&mut st, &silent, 1.0).unwrap(), 0.0);
    assert_eq!(st, before, "E untouched");
    assert_eq!(p7.deposited, 0.0);

    let (all_star, s_all) = world(&|_, _| true, &nowhere);
    let pass = p7.heating(&all_star, &s_all).unwrap();
    assert!(pass.q.iter().all(|&q| q == 0.0) && pass.deposited == 0.0 && pass.emitted > 0.0);
}

/// §1.6.4: f_dep above 0.1 is refused by the registry while S2 is off — the bound at 0.1 is accepted, a hair above is not,
/// and so is a negative share; the stand-in stays off in the shipped constants.
#[test]
fn f_dep_above_the_bound_is_refused() {
    let shipped = Physics::shipped().unwrap();
    assert_eq!(shipped.get("f_dep"), Some(0.05));
    assert!(shipped.with(&[("f_dep", 0.1)]).is_ok());
    for bad in [0.1 + 1e-9, 0.5, 1.0, -1e-9] {
        let err = shipped.with(&[("f_dep", bad)]).expect_err("S2 is off");
        assert!(err.to_string().contains("f_dep"), "{err}");
    }
}

// ---- P7 on the GPU (M0-T54) ----
//
// Tolerances, fixed before the first GPU run: the deposit Σ q Δt = f_dep ΣS_ν Δt within 1e-5 relative (the plan's number: f32 tile
// sums, 64 cells a tile then a fixed-order scan); each cell's q within 1e-4 of the largest q of the twin's (the FFT's f32 error
// against the twin's f64: the transforms' round-off is absolute, so the far, small q are judged against the top); E raised by what
// was booked to 2 ulp of E (both are f32 sums of the same product); the ratios of two cells (6.5 and 2) within 1e-3 (a cell's
// reach is an FFT output, of 1e-6 of the largest reach in absolute terms); "bit for bit", the dispatch counters and the zeros exact.
// The worlds are hand-made fields on the 64 × 64 world, the smallest §2.8 allows: the formula is proven, a collapsed star's heating
// is NOT PROVEN (synthetic).

const GW: usize = 64;
const GH: usize = 64;

/// A GPU world for P7 alone (with P1, P8 and P9): the state and S_ν uploaded, one run at a time.
struct Rig {
    gpu: Gpu,
    state: GpuState,
    booking: Booking,
    side: SideFields,
    step: Step,
    w: usize,
    h: usize,
}

fn f32s(v: &[f64]) -> Vec<f32> {
    v.iter().map(|&x| x as f32).collect()
}

fn le_bytes(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|x| x.to_le_bytes()).collect()
}

/// `st` rounded through f32, as the GPU holds it: the twin's start.
fn rounded(st: &State) -> State {
    State::new(st.width, st.height, st.planes.iter().map(|&v| f64::from(v as f32)).collect())
}

/// The shipped constants with the cold pressure off (the four K's 0: the hand-made worlds' E = Σ is far below a cold electron
/// gas's energy at Σ ~ 1, which P8 would raise to) and `f_dep`.
fn gpu_physics(f_dep: f64) -> Physics {
    Physics::shipped().unwrap().with(&[("k1e", 0.0), ("k2e", 0.0), ("k1n", 0.0), ("k2n", 0.0), ("f_dep", f_dep)]).unwrap_or_else(|e| panic!("{e}"))
}

/// The step: P7 when `p7`, over the box `mode`.
fn rig(st: &State, s_nu: &[f64], f_dep: f64, mode: BoxMode, p7: bool) -> Rig {
    let physics = gpu_physics(f_dep);
    let gpu = super::device();
    let world = WorldConfig::new(st.width as u32, st.height as u32).unwrap();
    let state = GpuState::new(&gpu.device, world).unwrap();
    state.upload(&gpu.queue, &f32s(&st.planes)).unwrap();
    let booking = Booking::new(&gpu.device, world).unwrap();
    let side = SideFields::new(&gpu.device, world).unwrap();
    gpu.queue.write_buffer(side.s_nu(), 0, &le_bytes(&f32s(s_nu)));
    let eos = EosGpu::new(&gpu.device, &physics, &Elements::shipped().unwrap());
    let passes = Passes { mode, gravity: None, hydro: None };
    let setup = NuHeatSetup { side: &side };
    let step = Step::with_neutrino_heating(&gpu.device, &state, &booking, &eos, &physics, &passes, None, None, None, p7.then_some(&setup))
        .unwrap_or_else(|e| panic!("{e}"));
    Rig { gpu, state, booking, side, step, w: st.width, h: st.height }
}

impl Rig {
    fn run(&self, steps: u32) {
        self.step.run(&self.gpu.device, &self.gpu.queue, steps);
        let dt = self.step.read_dt(&self.gpu.device, &self.gpu.queue);
        assert!(dt.non_finite.is_none(), "non-finite state: {:?}", dt.non_finite);
    }

    /// One frame of `steps` steps, submitted and waited for, and its report.
    fn frame(&self, steps: u32, slow_down: bool) -> FrameReport {
        let mut encoder = self.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
        self.step.encode_frame(&FramePlan { steps, slow_down }, &mut encoder);
        self.gpu.queue.submit([encoder.finish()]);
        self.gpu.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        let reports = self.step.poll(&self.gpu.device);
        assert_eq!(reports.len(), 1, "one report per frame: {reports:?}");
        reports[0]
    }

    /// Δt_n of the last step.
    fn dt(&self) -> f64 {
        f64::from(self.step.read_dt(&self.gpu.device, &self.gpu.queue).dt)
    }

    fn set_s_nu(&self, s_nu: &[f64]) {
        self.gpu.queue.write_buffer(self.side.s_nu(), 0, &le_bytes(&f32s(s_nu)));
    }

    /// The cell state's bits.
    fn bits(&self) -> Vec<u32> {
        self.state.readback(&self.gpu.device, &self.gpu.queue).iter().map(|v| v.to_bits()).collect()
    }

    fn cells(&self) -> Vec<f32> {
        self.state.readback(&self.gpu.device, &self.gpu.queue)
    }

    /// `neutrino_deposited`'s plane, one slot per cell: Σ q Δt booked at the cell since the rig was made.
    fn booked(&self) -> Vec<f32> {
        let book = self.booking.readback(&self.gpu.device, &self.gpu.queue);
        let world = self.booking.world();
        let at = TERMS.iter().position(|t| t.name == "neutrino_deposited").expect("a booked term");
        let mut base = 0;
        for g in &BOOK_GROUPS {
            let slots = g.domain.slots(world);
            if g.terms.contains(&at) {
                let from = base + (at - g.terms.start) * slots;
                return book[from..from + slots].to_vec();
            }
            base += (g.terms.end - g.terms.start) * slots;
        }
        unreachable!("every term is in a group")
    }

    fn booked_bits(&self) -> Vec<u32> {
        self.booking.readback(&self.gpu.device, &self.gpu.queue).iter().map(|v| v.to_bits()).collect()
    }

    /// Σ over the world of `neutrino_deposited`, in f64 (the ledger's own sum).
    fn deposited(&self) -> f64 {
        let at = TERMS.iter().position(|t| t.name == "neutrino_deposited").expect("a booked term");
        book_totals(self.booking.world(), &self.booking.readback(&self.gpu.device, &self.gpu.queue))[at]
    }

    fn report(&self) -> NuHeatReport {
        self.step.read_nuheat(&self.gpu.device, &self.gpu.queue).expect("a step with P7")
    }

    /// E of the state, the plane `cells()` holds after the Σ, mom_x and mom_y planes.
    fn energy(&self) -> Vec<f32> {
        self.cells()[3 * self.w * self.h..4 * self.w * self.h].to_vec()
    }
}

/// A 64 × 64 world: the 4 × 4 star at (20, 24) in a 40 × 40 square of gas (x and y in 12 … 52).
fn star(x: usize, y: usize) -> bool {
    (20..24).contains(&x) && (24..28).contains(&y)
}

fn outside_the_square(x: usize, y: usize) -> bool {
    !((12..52).contains(&x) && (12..52).contains(&y))
}

fn the_world() -> (State, Vec<f64>) {
    build(GW, GH, &star, &outside_the_square)
}

/// q(x) per cell as the GPU booked it: the cell's `neutrino_deposited` over Δt_n of the one step run.
fn q_of(rig: &Rig) -> Vec<f64> {
    let dt = rig.dt();
    rig.booked().iter().map(|&b| f64::from(b) / dt).collect()
}

#[test]
fn gpu_q_matches_the_twin_cell_by_cell_over_the_box_and_the_world_and_a_second_run_is_bit_for_bit() {
    let (st, s_nu) = the_world();
    let (st32, s32) = (rounded(&st), s_nu.iter().map(|&v| f64::from(v as f32)).collect::<Vec<_>>());
    let want = NuHeat::new(&physics(0.05)).heating(&st32, &s32).unwrap();
    let top = want.q.iter().cloned().fold(0.0, f64::max);
    assert!(top > 0.0);
    let mut first: Option<(Vec<u32>, Vec<u32>)> = None;
    for mode in [BoxMode::Fit, BoxMode::WholeWorld] {
        let r = rig(&st, &s_nu, 0.05, mode, true);
        r.run(1);
        let q = q_of(&r);
        let worst = (0..GW * GH).map(|i| (q[i] - want.q[i]).abs()).fold(0.0, f64::max);
        println!("SR-NUHEAT q against the twin ({mode:?}): worst cell off by {:.3e} of the top {top:.3e}", worst / top);
        assert!(worst < 1e-4 * top, "{mode:?}: worst cell off by {worst:e} of {top:e}");
        // nothing lands in the star, nor outside the square (Σ = 0 there)
        for i in 0..GW * GH {
            let (x, y) = (i % GW, i / GW);
            if star(x, y) || outside_the_square(x, y) {
                assert_eq!(q[i], 0.0, "cell ({x}, {y}) took heat");
            }
        }
        if mode == BoxMode::Fit {
            let again = rig(&st, &s_nu, 0.05, mode, true);
            again.run(1);
            first = Some((r.bits(), r.booked_bits()));
            assert_eq!(first.as_ref().unwrap(), &(again.bits(), again.booked_bits()), "the same state gives the same bits (§1.3.4)");
        }
    }
    assert!(first.is_some());
}

#[test]
fn gpu_deposited_is_f_dep_of_emitted_and_e_rises_by_what_was_booked() {
    let (st, s_nu) = the_world();
    let s32: Vec<f64> = s_nu.iter().map(|&v| f64::from(v as f32)).collect();
    let emitted: f64 = s32.iter().sum();
    for &f_dep in &[0.05, 0.1, 0.01] {
        let r = rig(&st, &s_nu, f_dep, BoxMode::Fit, true);
        let e0 = r.energy();
        r.run(1);
        let dt = r.dt();
        let got = r.deposited();
        let err = rel(got, f_dep * emitted * dt);
        println!("SR-NUHEAT f_dep {f_dep}: booked {got:.6e} against {:.6e}, off by {err:.2e}", f_dep * emitted * dt);
        assert!(err < 1e-5, "f_dep {f_dep}: deposited {got} against {}", f_dep * emitted * dt);
        let report = r.report();
        assert!(report.opened && report.ran && report.steps == 1, "{report:?}");
        assert!(rel(f64::from(report.emitted), emitted) < 1e-5, "ΣS_ν {} against {emitted}", report.emitted);
        // E rose by what was booked, cell by cell (cells of Σ ≥ Σ_vac: P8 resets the vacuum's E)
        let (e1, booked, sigma) = (r.energy(), r.booked(), &st.plane(0)[..]);
        for i in (0..GW * GH).filter(|&i| sigma[i] >= 1e-6) {
            let rose = f64::from(e1[i]) - f64::from(e0[i]);
            let ulp = f64::from(e1[i]) * f64::from(f32::EPSILON);
            assert!((rose - f64::from(booked[i])).abs() <= 2.0 * ulp, "cell {i}: E rose {rose:e}, booked {:e}", booked[i]);
        }
    }
    // f_dep = 0: nothing deposited, E untouched
    let off = rig(&st, &s_nu, 0.0, BoxMode::Fit, true);
    let plain = rig(&st, &s_nu, 0.0, BoxMode::Fit, false);
    off.run(2);
    plain.run(2);
    assert_eq!(off.deposited(), 0.0);
    assert_eq!(off.bits(), plain.bits(), "f_dep = 0 leaves the state bit for bit a P7-less step's");
}

#[test]
fn gpu_none_inside_the_neutron_star() {
    let (mut st, s_nu) = the_world();
    let n = GW * GH;
    let at = |x: usize, y: usize| y * GW + x;
    // the same three cells as the twin's case: X_n 0.5 exactly and 1, and 0.4999
    let (half, almost) = (at(30, 30), at(31, 30));
    st.planes[X_N * n + half] = 0.5;
    st.planes[X_N * n + almost] = 0.4999;
    let r = rig(&st, &s_nu, 0.05, BoxMode::Fit, true);
    let e0 = r.energy();
    r.run(1);
    let (booked, e1) = (r.booked(), r.energy());
    let twin = rounded(&st);
    for i in 0..n {
        if twin.get(X_N, i) >= 0.5 {
            assert_eq!(booked[i].to_bits(), 0, "cell {i} is neutron star");
            assert_eq!(e1[i].to_bits(), e0[i].to_bits(), "cell {i}'s E moved");
        }
    }
    assert!(booked[almost] > 0.0, "X_n 0.4999 is gas");
    assert_eq!(booked[half], 0.0);
    assert!(r.deposited() > 0.0);
}

#[test]
fn gpu_nearer_and_denser_receive_more() {
    let n = GW * GH;
    let (mut st, _) = build(GW, GH, &|_, _| false, &|_, _| true);
    let h = Elements::shipped().unwrap().index_of("H").unwrap();
    let at = |x: usize, y: usize| y * GW + x;
    let mut s_nu = vec![0.0; n];
    // the twin's case at 8 cells in: a source at (8, 32), gas 3 and 8 cells east of it, and Σ 1 against 2 six cells north and south
    let src = at(8, 32);
    s_nu[src] = 7.0;
    st.planes[src] = 1.0;
    st.planes[E * n + src] = 1.0;
    st.planes[X_N * n + src] = 1.0;
    for (x, y, sigma) in [(11, 32, 1.0), (16, 32, 1.0), (8, 26, 1.0), (8, 38, 2.0)] {
        for (plane, v) in [(0, sigma), (E, sigma), (4 + h, 1.0)] {
            st.planes[plane * n + at(x, y)] = v;
        }
    }
    let r = rig(&st, &s_nu, 0.05, BoxMode::Fit, true);
    r.run(1);
    let q = q_of(&r);
    let (near, far, a, b) = (q[at(11, 32)], q[at(16, 32)], q[at(8, 26)], q[at(8, 38)]);
    println!("SR-NUHEAT nearer/farther {:.4} (6.5), denser/thinner {:.4} (2)", near / far, b / a);
    assert!(near > far);
    assert!(rel(near / far, (64.0 + 1.0) / (9.0 + 1.0)) < 1e-3, "ratio {}", near / far);
    assert!(rel(b / a, 2.0) < 1e-3, "twice the Σ at the same distance: {}", b / a);
    assert!(rel(q.iter().sum::<f64>(), 0.05 * 7.0) < 1e-5);
}

/// Adversarial: normalised over ΣD, not per cell. One lone gas cell takes exactly f_dep of what the core emitted, whatever its own D.
#[test]
fn gpu_a_lone_gas_cell_takes_exactly_f_dep() {
    let n = GW * GH;
    let (mut st, _) = build(GW, GH, &|_, _| false, &|_, _| true);
    let h = Elements::shipped().unwrap().index_of("H").unwrap();
    let mut s_nu = vec![0.0; n];
    let (core, gas) = (6 * GW + 6, 40 * GW + 50);
    s_nu[core] = 11.0;
    for (plane, v) in [(0, 1.0), (E, 1.0), (X_N, 1.0)] {
        st.planes[plane * n + core] = v;
    }
    for (plane, v) in [(0, 0.7), (E, 0.7), (4 + h, 1.0)] {
        st.planes[plane * n + gas] = v;
    }
    let r = rig(&st, &s_nu, 0.1, BoxMode::Fit, true);
    r.run(1);
    let q = q_of(&r);
    println!("SR-NUHEAT the lone cell took {:.6} of {:.6}", q[gas], 0.1 * 11.0);
    assert!(rel(q[gas], 0.1 * 11.0) < 1e-5, "the lone cell got {}", q[gas]);
    assert!(q.iter().sum::<f64>() <= 11.0 * (1.0 + 1e-5));
}

/// Nothing emitted, nothing dispatched: ΣS_ν = 0 leaves the state and the booking bit for bit a P7-less step's, the chain's
/// `nu_share` never runs, and the pass costs less than the one that convolves (the timing report, where the device has timestamps).
/// Then S_ν comes back and the chain with it, and goes again.
#[test]
fn gpu_no_dispatch_when_nothing_is_emitted() {
    let (st, s_nu) = the_world();
    let silent = vec![0.0; GW * GH];
    let r = rig(&st, &silent, 0.05, BoxMode::Fit, true);
    let plain = rig(&st, &silent, 0.05, BoxMode::Fit, false);
    r.run(3);
    plain.run(3);
    let report = r.report();
    assert_eq!((report.steps, report.opened, report.ran), (3, false, false), "{report:?}");
    assert_eq!(r.bits(), plain.bits(), "a silent P7 changes nothing");
    assert!(r.booked().iter().all(|b| b.to_bits() == 0), "and books nothing");
    // the gate is not sticky: S_ν back, the chain runs; S_ν gone, it does not
    r.set_s_nu(&s_nu);
    r.run(1);
    let on = r.report();
    assert_eq!((on.steps, on.opened, on.ran), (4, true, true), "{on:?}");
    assert!(r.deposited() > 0.0);
    r.set_s_nu(&silent);
    r.run(1);
    let off = r.report();
    assert_eq!((off.steps, off.opened, off.ran), (5, false, false), "{off:?}");

    // the timing report: the same pass, with and without S_ν, on a 512 × 512 world (1024 × 1024 padded) where the transforms are not nothing
    let big = (512usize, 512usize);
    let (st, s_nu) = build(big.0, big.1, &|x, y| (120..128).contains(&x) && (120..128).contains(&y), &|_, _| false);
    let pass = Pass::NeutrinoHeating as usize;
    let mut ms = [0.0f64; 2];
    for (k, s) in [vec![0.0; big.0 * big.1], s_nu].iter().enumerate() {
        let r = rig(&st, s, 0.05, BoxMode::Fit, true);
        match r.step.set_timing(&r.gpu.device, &r.gpu.queue, Timing::Auto) {
            Some(CostSource::GpuTimestamp) => {}
            other => {
                println!("SR-NUHEAT timing: NOT RUN ({other:?}: this adapter has no timestamp-query); the counters above are the proof");
                return;
            }
        }
        r.run(2);
        r.step.read_cost(&r.gpu.device, &r.gpu.queue);
        r.run(8);
        ms[k] = r.step.read_cost(&r.gpu.device, &r.gpu.queue).and_then(|c| c.pass_ms[pass]).expect("P7 was timed");
    }
    println!("SR-NUHEAT P7 costs {:.4} ms with nothing emitted, {:.4} ms with a star (512 × 512)", ms[0], ms[1]);
    assert!(ms[0] < 0.5 * ms[1], "the silent pass costs {} ms against {} ms", ms[0], ms[1]);
}

/// ΣD = 0 (every cell with mass is neutron star): the chain runs, hands the energy to no one, and nothing is non-finite.
#[test]
fn gpu_an_all_star_world_deposits_nothing() {
    let (st, s_nu) = build(GW, GH, &|_, _| true, &outside_the_square);
    let r = rig(&st, &s_nu, 0.05, BoxMode::Fit, true);
    let plain = rig(&st, &s_nu, 0.05, BoxMode::Fit, false);
    r.run(2);
    plain.run(2);
    let report = r.report();
    assert!(report.opened && report.ran && report.emitted > 0.0, "{report:?}");
    assert_eq!((report.total, report.scale), (0.0, 0.0), "{report:?}");
    assert_eq!(r.deposited(), 0.0);
    assert_eq!(r.bits(), plain.bits(), "no cell took anything");
}

/// A frame the latch stops: P7 runs on the steps that ran and on no later one (it is reached through its fixed-size gate and close,
/// P5's way). The hook sets the flag in the step with index 1 of a frame of 5 with the slow-down on: the frame stops after it.
#[test]
fn gpu_a_stopped_frame_runs_no_p7_after_it() {
    let (st, s_nu) = the_world();
    let stopped = rig(&st, &s_nu, 0.05, BoxMode::Fit, true);
    stopped.step.set_latch_hook(&stopped.gpu.queue, Some((1, 2, 1)));
    let report = stopped.frame(5, true);
    assert_eq!((report.steps_run, report.latch_fired), (2, Some(2)), "{report:?}");
    let two = rig(&st, &s_nu, 0.05, BoxMode::Fit, true);
    two.run(2);
    assert_eq!(stopped.report(), two.report(), "P7's gate ran on the two steps the frame ran");
    assert_eq!(stopped.report().steps, 2);
    assert_eq!(stopped.bits(), two.bits(), "the stopped frame's cells are a two-step run's");
    assert_eq!(stopped.booked_bits(), two.booked_bits(), "and its booking");
    // the sizes are back to 0: the next frame, with the hook out and the latch disarmed for 1.0 t.u., runs P7 on every step
    stopped.step.set_latch_hook(&stopped.gpu.queue, None);
    let next = stopped.frame(3, true);
    assert_eq!((next.steps_run, next.latch_fired), (3, None), "{next:?}");
    let after = stopped.report();
    assert_eq!((after.steps, after.ran), (5, true), "{after:?}");
}
