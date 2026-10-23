//! M0-T41 — heat and light on the CPU twin (contract §2.4, §1.7, §5.1's G-HEAT1, G-HEAT2 and G-FLD, R8, R13): P5's f64
//! reference, `sr_physics::reference::heat`; the twin's cases touch no adapter. M0-T117 — the twin's P5 is the implicit step
//! of contract §0.5 [M0-TJ4] (backward Euler in u = a_r T⁴, Newton passes, Jacobi-PCG). M0-T42 — the GPU's P5
//! (`step/heat.rs`, shaders/heat.wgsl), cases `gpu_…`, the same oracles on the physics adapter (R12), P5 alone between P1,
//! P8 and P9 ([`OnGpu`]). M0-T118 — the GPU's implicit step on the linear law; M0-T119 — on every scene (the radiative law,
//! RKL2 retired), every GPU case compared with the twin's `Heat::step` or `Heat::law`. M0-D83 — every GPU case of the four
//! scopes below on both of P5's preconditioners, Jacobi and the V-cycle (§0.5 [M0-D82]), a case parameter
//! ([`on_both_preconditioners`]).
//!
//! Three scopes, one module each. `gauss` — G-HEAT1: a Gaussian heat pulse in a uniform medium whose diffusivity is the
//! scene's `test.constant_diffusivity` (scenes/heat_pulse.json), the variance per axis σ²(t) = σ₀² + 2χt within 1 % (SM-O7,
//! SM-E21) and the total heat constant to 1e-5; the pass's F at a face against −χ·C·ΔT; the light leaving (a hot slab beside
//! vacuum and at the world's edge: what the pass books `radiated` equals the heat the grid lost, to 1e-6, and sits in the
//! cells that emitted it); the scene's `pin_temperature`. `implicit` — G-HEAT2 as §0.5 [M0-TJ4] re-reads it: (a) one pass of
//! Δt = 24 against an independent tight solve of the same backward-Euler system written here (Gauss–Seidel), within 1e-10 of
//! the peak, σ² within 1 %; (b) backward Euler's first order against explicit sub-steps of Δt = 1/64; a one-cell pulse in
//! many passes (the adversarial case: every wavenumber present — backward Euler keeps it non-negative where RKL2 rang); and
//! the radiative law's nonlinear step solved per cell on a hot slab. On the GPU: (a) one pass against the twin from the same
//! f32 start and run twice bit for bit, and the one-cell pulse. `limiter` — G-FLD: thin gas (Σ = 10⁻⁴,
//! scenes/heat_spot.json) around a hot spot, the law each pass freezes (`Heat::law`, the start state's flux) holds
//! |F| ≤ c_sb a_r T⁴ (1 + 10⁻⁶) at every face (SM-E19); thick gas, where the limiter must not act, carries the radiative
//! conductivity (4/3) a_r c_sb T³/(κΣ) in the solved F (SM-E20); the scene's `flux_limiter` off gives λ ≡ 1/3.
//!
//! The oracles are written here from the contract, never read from the scheme: temperatures from E with the cold pressure
//! off and the gas at rest and pure hydrogen (μ = 1/2), the explicit stepping, the Gauss–Seidel solve, the moments, the
//! closed forms.
//!
//! The GPU's f32 tolerances, fixed before its first run: the contract's own where a row names one (σ² 1 %, total heat 1e-5,
//! |F| ≤ c_sb a_r T⁴ (1 + 1e-6), G-HEAT2 (a)'s 1e-4 of the peak against the twin from the same f32 start); F within 1e-4 of
//! the largest face flux; `radiated` against the heat lost within 1e-5 and the twin's within 1e-4; the implicit one-cell
//! pulse never below −1e-5 of the peak (the solve's tolerance, M0-T118). The law a GPU pass freezes is read from a pinned
//! run on the same start state (pinned, F is the law: §0.5 [M0-TJ4] (1)).

use std::path::{Path, PathBuf};

use sr_engine::dump::Dump;
use sr_engine::gpu::Gpu;
use sr_engine::scene::{Scene, TestSettings};
use sr_engine::state::{book_totals, Booking, SideFields, State as GpuState, WorldConfig, BOOK_GROUPS, TERMS};
use sr_engine::step::boxfit::BoxMode;
use sr_engine::step::heat::{set_default_precond, HeatPrecond, HeatReport, HeatSetup, MIN_ROUNDS, SLOTS};
use sr_engine::step::{EosGpu, Passes, Step};
use sr_physics::eos::Eos;
use sr_physics::reference::heat::{lambda, Flux, Heat, Pass, TestFlags};
use sr_physics::reference::hydro::State;
use sr_physics::registry::{Elements, Physics, N_SPECIES};

/// Hydrogen's 1/μ (§1.5): ε_th = T/μ = 2T.
const INV_MU: f64 = 2.0;
/// Hydrogen's κ = κ₀ (1 + X_H) with κ₀ = 1 (§2.4.2, §2.7).
const KAPPA_H: f64 = 2.0;
/// The Gaussian of G-HEAT1: its σ₀ in cells, its peak over the background, the diffusivity of heat_pulse.json.
const SIGMA0: f64 = 4.0;
const PEAK: f64 = 1.0;
const CHI: f64 = 0.5;

fn scenes_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenes")
}

fn elements() -> Elements {
    Elements::shipped().unwrap()
}

fn scene(name: &str) -> Scene {
    Scene::load(&scenes_dir().join(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
}

fn flags(scene: &Scene) -> TestFlags {
    let t = scene.test;
    TestFlags { pin_temperature: t.pin_temperature, constant_diffusivity: t.constant_diffusivity, flux_limiter: t.flux_limiter }
}

/// A scene's planes as the twin's state (f32 widened to f64), its constants and the heat pass it asks for.
fn world(scene: &Scene) -> (State, Physics, Heat) {
    let physics = scene.physics().unwrap();
    let (planes, warnings) = scene.planes(scene.world, &physics, &elements()).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let st = State::new(scene.world.width as usize, scene.world.height as usize, planes.iter().map(|&v| f64::from(v)).collect());
    let heat = Heat::new(&physics, &elements(), flags(scene)).unwrap();
    (st, physics, heat)
}

/// An inline scene: heat only, the cold pressure off, one rect of hydrogen at `sigma` and `t`.
fn rect_world(width: u32, height: u32, rect: [u32; 4], sigma: f64, t: f64) -> Scene {
    let [x0, y0, w, h] = rect;
    let text = format!(
        r#"{{"version": 1, "world": {{"width": {width}, "height": {height}, "edge": "bounce"}}, "slowdown": false,
 "switches": {{"gravity": false, "hydro": false, "heat": true, "reactions": false, "neutrinos": false, "sinks": false}},
 "objects": [{{"kind": "rect", "x0": {x0}, "y0": {y0}, "w": {w}, "h": {h}, "sigma": {sigma}, "t": {t}, "composition": {{"H": 1.0}}, "v": [0, 0]}}],
 "overrides": {{"k1e": 0, "k2e": 0, "k1n": 0, "k2n": 0}}}}"#
    );
    Scene::parse(&text, &elements()).unwrap_or_else(|e| panic!("{e}"))
}

/// T of every cell from E: ε_th = E/Σ (at rest, no cold energy), T = ε_th/(1/μ).
fn temps(st: &State) -> Vec<f64> {
    let n = st.cells();
    (0..n).map(|i| st.planes[3 * n + i] / st.planes[i] / INV_MU).collect()
}

/// Sets the temperatures of `st` (E = Σ · (1/μ) · T).
fn set_temps(st: &mut State, t: &[f64]) {
    let n = st.cells();
    for i in 0..n {
        st.planes[3 * n + i] = st.planes[i] * INV_MU * t[i];
    }
}

/// The heat_pulse.json medium with a Gaussian of `peak` over its background: T = T_bg + peak · exp(−r²/2σ₀²) about (64, 64).
fn gaussian(scene: &Scene, sigma0: f64, peak: f64) -> (State, Heat, f64) {
    gaussian_on(scene, None, sigma0, peak)
}

/// [`gaussian`] on a background of `bg` instead of T_floor (a background above the floor loses heat through the edge).
fn gaussian_on(scene: &Scene, bg: Option<f64>, sigma0: f64, peak: f64) -> (State, Heat, f64) {
    let (mut st, physics, heat) = world(scene);
    let w = st.width;
    // The scene's f32 planes hold T = 0.05 to 1e-8; the background is set here at T_floor to the last bit, so the vacuum
    // beyond the edge takes exactly nothing from it.
    let bg = bg.unwrap_or_else(|| physics.get("t_floor").unwrap());
    let mut t = vec![bg; st.cells()];
    for (i, ti) in t.iter_mut().enumerate() {
        let (dx, dy) = ((i % w) as f64 - 64.0, (i / w) as f64 - 64.0);
        *ti += peak * (-(dx * dx + dy * dy) / (2.0 * sigma0 * sigma0)).exp();
    }
    set_temps(&mut st, &t);
    (st, heat, bg)
}

/// The heat above `bg` (energy per area), and its variance along x and y about its own centre.
fn moments(st: &State, bg: f64) -> (f64, f64, f64) {
    let (w, n) = (st.width, st.cells());
    let t = temps(st);
    let (mut total, mut mx, mut my, mut mxx, mut myy) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for i in 0..n {
        let wgt = st.planes[i] * INV_MU * (t[i] - bg);
        let (x, y) = ((i % w) as f64, (i / w) as f64);
        total += wgt;
        mx += wgt * x;
        my += wgt * y;
        mxx += wgt * x * x;
        myy += wgt * y * y;
    }
    (total, mxx / total - (mx / total).powi(2), myy / total - (my / total).powi(2))
}

/// The sum of E over the world.
fn energy(st: &State) -> f64 {
    let n = st.cells();
    st.planes[3 * n..4 * n].iter().sum()
}

/// `steps` explicit sub-steps of dT/dt = χ ∇²T (five points, Δx = 1, the edges reflecting): the independent stepping G-HEAT2
/// compares the implicit step with. Stable for χ dt ≤ 1/4, but at 1/4 exactly a one-cell pulse keeps its checkerboard component for ever
/// (the Nyquist mode's factor is −1): the tests step it at half that.
fn explicit(t: &[f64], w: usize, h: usize, chi: f64, dt: f64, steps: usize) -> Vec<f64> {
    assert!(chi * dt <= 0.25 + 1e-12, "the explicit step must be stable");
    let mut t = t.to_vec();
    let mut next = t.clone();
    for _ in 0..steps {
        for y in 0..h {
            for x in 0..w {
                let at = |xx: usize, yy: usize| t[yy * w + xx];
                let c = at(x, y);
                let (l, r) = (at(x.saturating_sub(1), y), at((x + 1).min(w - 1), y));
                let (u, d) = (at(x, y.saturating_sub(1)), at(x, (y + 1).min(h - 1)));
                next[y * w + x] = c + chi * dt * (l + r + u + d - 4.0 * c);
            }
        }
        std::mem::swap(&mut t, &mut next);
    }
    t
}

/// The heat above `bg` of a plain temperature field of Σ = 1 (per unit C), and its variance along x and y, over the cells
/// within `half` of the pulse's centre (64, 64) on each axis — `usize::MAX` for the whole world.
fn stats(t: &[f64], w: usize, bg: f64, half: usize) -> (f64, f64, f64) {
    let (mut total, mut mx, mut my, mut mxx, mut myy) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for (i, ti) in t.iter().enumerate() {
        let (x, y) = (i % w, i / w);
        if x.abs_diff(64) > half || y.abs_diff(64) > half {
            continue;
        }
        let wgt = ti - bg;
        let (x, y) = (x as f64, y as f64);
        total += wgt;
        mx += wgt * x;
        my += wgt * y;
        mxx += wgt * x * x;
        myy += wgt * y * y;
    }
    (total, mxx / total - (mx / total).powi(2), myy / total - (my / total).powi(2))
}

/// The scene's pulse is far from every edge: nothing may leave, and what leaves is a failure of the set-up.
fn assert_inside(p: &Pass) {
    assert!(p.radiated.abs() < 1e-12, "radiated {}", p.radiated);
}

/// `st` as the GPU holds it: each plane rounded to f32 (the twin then starts where the GPU does).
fn rounded(st: &State) -> State {
    State::new(st.width, st.height, st.planes.iter().map(|&v| f64::from(v as f32)).collect())
}

/// The largest |a − b| over two fields.
fn largest_gap(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| (x - y).abs()).fold(0.0, f64::max)
}

/// A whole buffer read back as f32 (COPY_SRC buffers: the side fields).
fn read(gpu: &Gpu, buffer: &wgpu::Buffer) -> Vec<f32> {
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("heat.test.read"),
        size: buffer.size(),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    encoder.copy_buffer_to_buffer(buffer, 0, &staging, 0, buffer.size());
    gpu.queue.submit([encoder.finish()]);
    staging.map_async(wgpu::MapMode::Read, .., |r| r.expect("mapped"));
    gpu.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let view = staging.get_mapped_range(..).unwrap();
    view.as_chunks::<4>().0.iter().map(|b| f32::from_le_bytes(*b)).collect()
}

/// P5 alone on the physics adapter (M0-T42): gravity and hydro off, the box fit, P1, P8 and P9 around it, from `first` under
/// the test flags `test`. Each [`OnGpu::pass`] is one step at a Δt the test picks, written as the Δt record's Δt_{n+1}
/// (reduce.wgsl `DtState`'s second word) between submissions — P1 makes it the step's Δt. Step 0 takes the initial
/// reduction's Δt instead, so it runs first and `first` is uploaded again (the box, fit to Σ alone, is the same).
struct OnGpu {
    gpu: Gpu,
    state: GpuState,
    booking: Booking,
    side: SideFields,
    step: Step,
    first: State,
}

impl OnGpu {
    fn new(physics: &Physics, test: TestFlags, first: &State) -> OnGpu {
        let gpu = super::device();
        let world = WorldConfig::new(first.width as u32, first.height as u32).unwrap();
        let state = GpuState::new(&gpu.device, world).unwrap();
        let booking = Booking::new(&gpu.device, world).unwrap();
        let side = SideFields::new(&gpu.device, world).unwrap();
        let eos = EosGpu::new(&gpu.device, physics, &elements());
        let test = TestSettings { pin_temperature: test.pin_temperature, constant_diffusivity: test.constant_diffusivity, flux_limiter: test.flux_limiter };
        let passes = Passes { mode: BoxMode::Fit, gravity: None, hydro: None };
        let step = Step::with_heat(&gpu.device, &state, &booking, &eos, physics, &passes, Some(&HeatSetup { side: &side, test })).unwrap();
        let run = OnGpu { gpu, state, booking, side, step, first: rounded(first) };
        run.restart();
        run.step.run(&run.gpu.device, &run.gpu.queue, 1);
        run.restart();
        run
    }

    /// `first` uploaded again, the booking cleared.
    fn restart(&self) {
        self.upload(&self.first);
    }

    /// `st` uploaded (its planes rounded to f32), the booking cleared.
    fn upload(&self, st: &State) {
        let planes: Vec<f32> = st.planes.iter().map(|&v| v as f32).collect();
        self.state.upload(&self.gpu.queue, &planes).unwrap();
        let mut encoder = self.gpu.device.create_command_encoder(&Default::default());
        self.booking.clear(&mut encoder);
        self.gpu.queue.submit([encoder.finish()]);
    }

    /// One step whose P5 runs over `dt`; its record.
    fn pass(&self, dt: f64) -> HeatReport {
        self.pass_in(dt, None).0
    }

    /// [`OnGpu::pass`] with each submission recording `rounds` of P5's slots (the engine's choice when none, §0.5 [M0-TJ4]
    /// (6)); its record and the submissions the step took.
    fn pass_in(&self, dt: f64, rounds: Option<usize>) -> (HeatReport, u32) {
        self.gpu.queue.write_buffer(self.step.dt_record(), 4, &(dt as f32).to_le_bytes());
        self.step.set_heat_rounds(rounds);
        let submissions = self.step.run_counted(&self.gpu.device, &self.gpu.queue, 1);
        assert_eq!(self.step.read_dt(&self.gpu.device, &self.gpu.queue).dt, dt as f32, "P1 took the Δt written");
        (self.step.read_heat(&self.gpu.device, &self.gpu.queue).expect("the step runs P5"), submissions)
    }

    fn planes(&self) -> State {
        let planes = self.state.readback(&self.gpu.device, &self.gpu.queue).iter().map(|&v| f64::from(v)).collect();
        State::new(self.first.width, self.first.height, planes)
    }

    /// `radiated` per cell since the last restart (booking.heat's plane).
    fn radiated(&self) -> Vec<f64> {
        let book = self.booking.readback(&self.gpu.device, &self.gpu.queue);
        let world = self.state.world();
        let group = BOOK_GROUPS.iter().position(|g| TERMS[g.terms.start].name == "radiated").unwrap();
        let base: usize = BOOK_GROUPS[..group].iter().map(|g| g.terms.len() * g.domain.slots(world)).sum();
        book[base..base + world.cells()].iter().map(|&v| f64::from(v)).collect()
    }

    /// The last P5's face fluxes, the twin's `Flux` layout.
    fn faces(&self) -> (Vec<f64>, Vec<f64>) {
        let (x, y) = self.step.read_heat_flux(&self.gpu.device, &self.gpu.queue).unwrap();
        (x.iter().map(|&v| f64::from(v)).collect(), y.iter().map(|&v| f64::from(v)).collect())
    }
}

/// M0-D83 — `case` with every P5 it builds starting on `precond` (`set_default_precond`), the default before it put back
/// after (M0-D84: the V-cycle), a panic included.
pub(crate) fn on_precond(precond: HeatPrecond, case: fn()) {
    struct Back(HeatPrecond);
    impl Drop for Back {
        fn drop(&mut self) {
            set_default_precond(self.0);
        }
    }
    let _back = Back(set_default_precond(precond));
    println!("SR-PRECOND {precond:?}");
    case();
}

/// Each named GPU case on both of P5's preconditioners (§0.5 [M0-D82], M0-D83), a case parameter: `<case>::jacobi` and
/// `<case>::multigrid`, every bound the case's own.
macro_rules! on_both_preconditioners {
    ($($case:ident),* $(,)?) => {$(
        mod $case {
            #[test]
            fn jacobi() {
                $crate::heat::on_precond(sr_engine::step::heat::HeatPrecond::Jacobi, super::$case);
            }

            #[test]
            fn multigrid() {
                $crate::heat::on_precond(sr_engine::step::heat::HeatPrecond::Multigrid, super::$case);
            }
        }
    )*};
}
pub(crate) use on_both_preconditioners;

/// G-HEAT1: σ²(t) = σ₀² + 2χt per axis within 1 %, total heat constant to 1e-5, F at a face.
mod gauss {
    use super::*;

    on_both_preconditioners!(
        gpu_the_variance_grows_as_sigma0_squared_plus_two_chi_t_and_the_heat_is_kept,
        gpu_the_flux_kept_is_the_twins_and_side_flux_is_its_cell_mean,
        gpu_a_hot_slab_books_radiated_equal_to_the_heat_the_grid_lost,
        gpu_pinned_temperature_leaves_the_state_and_keeps_the_flux,
    );

    #[test]
    fn the_scene_files_carry_what_the_oracles_assume() {
        let pulse = scene("heat_pulse.json");
        let (st, physics, _) = world(&pulse);
        assert_eq!((pulse.world.width, pulse.world.height), (128, 128));
        assert_eq!(pulse.edge, sr_engine::scene::EdgeMode::Bounce);
        let s = pulse.switches;
        assert!(s.heat && !s.gravity && !s.hydro && !s.reactions && !s.neutrinos && !s.sinks, "{s:?}");
        let t = pulse.test;
        assert_eq!((t.pin_temperature, t.constant_diffusivity, t.flux_limiter), (false, Some(CHI), true));
        for key in ["k1e", "k2e", "k1n", "k2n"] {
            assert_eq!(physics.get(key), Some(0.0), "cold pressure off: {key}");
        }
        // The background sits at the floor temperature, so the vacuum beyond the edge takes nothing from it.
        let bg = temps(&st);
        assert!(bg.iter().all(|&t| (t - physics.get("t_floor").unwrap()).abs() < 1e-6), "{:?}", &bg[..3]);
        assert!(st.plane(0).iter().all(|&s| s == 1.0));

        let spot = scene("heat_spot.json");
        let (st, physics, _) = world(&spot);
        assert_eq!(spot.edge, sr_engine::scene::EdgeMode::Leave);
        assert!(spot.switches.heat && !spot.switches.hydro && !spot.switches.gravity);
        assert_eq!(spot.test.constant_diffusivity, None);
        assert!(spot.test.flux_limiter && !spot.test.pin_temperature);
        // Thin gas, far above the vacuum threshold, with one hot 8 × 8 spot at T = 2.
        let sigma_vac = physics.get("sigma_vac").unwrap();
        assert!(st.plane(0).iter().all(|&s| (s - 1e-4).abs() < 1e-4 * 1e-6 && s >= 10.0 * sigma_vac), "thin gas everywhere");
        let t = temps(&st);
        let hot = t.iter().filter(|&&t| t > 1.0).count();
        assert_eq!(hot, 64);
        assert!(t.iter().all(|&t| (t - 2.0).abs() < 1e-6 || (t - 0.05).abs() < 1e-6));
    }

    /// σ_x² and σ_y² at t = 8, 16, 24 against σ₀² + 2χt: 24 passes of Δt = 1 — the law linear, so two Newton passes each
    /// (the second moves nothing).
    #[test]
    fn the_variance_grows_as_sigma0_squared_plus_two_chi_t_per_axis() {
        let (mut st, heat, bg) = gaussian(&scene("heat_pulse.json"), SIGMA0, PEAK);
        let (_, vx0, vy0) = moments(&st, bg);
        assert!((vx0 - SIGMA0 * SIGMA0).abs() < 1e-6 && (vy0 - SIGMA0 * SIGMA0).abs() < 1e-6, "{vx0} {vy0}");
        let mut worst: f64 = 0.0;
        for pass in 1..=24 {
            let p = heat.step(&mut st, 1.0);
            assert!(p.converged && p.passes == 2 && p.rounds > 0, "{} passes, {} rounds", p.passes, p.rounds);
            assert_inside(&p);
            if pass % 8 == 0 {
                let t = f64::from(pass);
                let want = SIGMA0 * SIGMA0 + 2.0 * CHI * t;
                let (_, vx, vy) = moments(&st, bg);
                let err = ((vx - want).abs() / want).max((vy - want).abs() / want);
                println!("SR-HEAT1 t = {t}: sigma2 x {vx:.6}, y {vy:.6}, expected {want}, error {err:.2e}");
                assert!(err <= 0.01, "t = {t}: σ² x {vx}, y {vy}, expected {want}");
                worst = worst.max(err);
            }
        }
        println!("SR-HEAT1 worst sigma2 error {worst:.2e}");
    }

    #[test]
    fn the_total_heat_is_constant() {
        let (mut st, heat, bg) = gaussian(&scene("heat_pulse.json"), SIGMA0, PEAK);
        let (h0, _, _) = moments(&st, bg);
        let e0 = energy(&st);
        let mut worst: f64 = 0.0;
        for _ in 0..24 {
            let p = heat.step(&mut st, 1.0);
            assert_inside(&p);
            let (h, _, _) = moments(&st, bg);
            worst = worst.max((h - h0).abs() / h0);
        }
        println!("SR-HEAT1 heat {h0:.6}, worst change {worst:.2e} of the pulse, E {:.2e} of the world", (energy(&st) - e0).abs() / e0);
        assert!(worst <= 1e-5, "{worst}");
        assert!(h0 > 100.0, "a real pulse: {h0}");
    }

    /// F is −χ·C·ΔT at a face (C = Σ/μ = 2) of the solved temperatures — the flux the step moved — positive toward +x, +y;
    /// the cell field is the mean of a cell's two faces.
    #[test]
    fn the_flux_kept_is_chi_c_times_the_temperature_difference() {
        let (mut st, heat, _) = gaussian(&scene("heat_pulse.json"), SIGMA0, PEAK);
        let (w, h) = (st.width, st.height);
        let p = heat.step(&mut st, 1.0);
        let t = temps(&st);
        let c = 1.0 * INV_MU;
        for &(x, y) in &[(70usize, 64usize), (64, 64), (60, 61), (1, 1), (127, 40)] {
            let want = -CHI * c * (t[y * w + x] - t[y * w + x - 1]);
            let got = p.flux.x[y * (w + 1) + x];
            assert!((got - want).abs() <= 1e-12 * (1.0 + want.abs()), "x face ({x}, {y}): {got} vs {want}");
            let want = -CHI * c * (t[y * w + x] - t[(y - 1) * w + x]);
            let got = p.flux.y[y * w + x];
            assert!((got - want).abs() <= 1e-12 * (1.0 + want.abs()), "y face ({x}, {y}): {got} vs {want}");
        }
        // Heat flows away from the peak: at the centre's right face F > 0 would mean toward +x from a cooler cell.
        assert!(p.flux.x[64 * (w + 1) + 70] > 0.0 && p.flux.x[64 * (w + 1) + 58] < 0.0);
        assert!(p.flux.y[70 * w + 64] > 0.0 && p.flux.y[58 * w + 64] < 0.0);
        let (fx, fy) = p.flux.cells();
        let i = 64 * w + 70;
        assert!((fx[i] - 0.5 * (p.flux.x[64 * (w + 1) + 70] + p.flux.x[64 * (w + 1) + 71])).abs() < 1e-15);
        assert!((fy[i] - 0.5 * (p.flux.y[64 * w + 70] + p.flux.y[65 * w + 70])).abs() < 1e-15);
        // The world's faces: x = 0 and x = w are the edges, and the background is at the floor, so nothing flows there.
        assert_eq!(p.flux.x.len(), (w + 1) * h);
        assert_eq!(p.flux.y.len(), w * (h + 1));
    }

    /// A hot slab at the world's left edge and beside vacuum (§2.4.4): what the pass books `radiated` equals what the grid
    /// lost, to 1e-6, and each cell's share sits where the light left.
    #[test]
    fn a_hot_slab_beside_vacuum_books_radiated_equal_to_the_heat_the_grid_lost() {
        let scene = rect_world(64, 64, [0, 16, 32, 32], 1.0, 30.0);
        let (mut st, physics, heat) = world(&scene);
        let sigma_vac = physics.get("sigma_vac").unwrap();
        let n = st.cells();
        let before = st.planes.clone();
        let e_before = energy(&st);
        let slab_energy: f64 = (0..n).filter(|&i| st.planes[i] >= sigma_vac).map(|i| st.planes[3 * n + i]).sum();
        let (mut radiated, mut emitted) = (0.0, vec![0.0; n]);
        let (mut max_passes, mut max_delta) = (0, 0.0f64);
        for _ in 0..12 {
            let p = heat.step(&mut st, 2.0);
            assert!(p.converged, "{} passes, {} rounds", p.passes, p.rounds);
            radiated += p.radiated;
            for (e, q) in emitted.iter_mut().zip(&p.emitted) {
                *e += q;
            }
            max_passes = max_passes.max(p.passes);
            max_delta = max_delta.max(p.delta.abs());
        }
        let lost = e_before - energy(&st);
        println!("SR-HEAT-RAD slab {slab_energy:.4}: lost {lost:.9}, radiated {radiated:.9}, Newton passes up to {max_passes}, |delta| up to {max_delta:.2e}");
        assert!(radiated / slab_energy > 0.01, "the slab must lose something real: {}", radiated / slab_energy);
        assert!(max_passes >= 3, "the radiative law is nonlinear: {max_passes}");
        assert!((lost - radiated).abs() <= 1e-6 * radiated, "lost {lost}, radiated {radiated}");
        assert!((emitted.iter().sum::<f64>() - radiated).abs() <= 1e-12 * radiated);
        // The emitting cells are the slab's surface: the left edge (x = 0), its right face (x = 31) and its top and bottom rows
        // (y = 16, 47); a cell inside books nothing, a vacuum cell neither, and no vacuum cell was touched.
        let mut surface = 0;
        for i in 0..n {
            let (x, y) = (i % 64, i / 64);
            let in_slab = x < 32 && (16..48).contains(&y);
            let on_surface = in_slab && (x == 0 || x == 31 || y == 16 || y == 47);
            if on_surface {
                assert!(emitted[i] > 0.0, "surface cell ({x}, {y}) emitted {}", emitted[i]);
                surface += 1;
            } else {
                assert_eq!(emitted[i], 0.0, "cell ({x}, {y}) emitted {}", emitted[i]);
            }
            if !in_slab {
                for c in 0..14 {
                    assert_eq!(st.planes[c * n + i].to_bits(), before[c * n + i].to_bits(), "vacuum cell ({x}, {y}) channel {c}");
                }
            }
        }
        assert_eq!(surface, 32 + 32 + 32 + 32 - 4, "the slab's ring: two columns and two rows of 32, four corners counted twice");
    }

    /// `test.pin_temperature` (§2.12.1): the pass leaves the state alone, books nothing, and keeps the law's F on the start
    /// state (−χ·C·ΔT of the start temperatures, the free pass's law) — not the flux a free pass solves.
    #[test]
    fn pinned_temperature_leaves_the_state_and_keeps_the_flux() {
        let (mut st, _, _) = gaussian(&scene("heat_pulse.json"), SIGMA0, PEAK);
        let (_, physics, _) = world(&scene("heat_pulse.json"));
        let pinned = Heat::new(&physics, &elements(), TestFlags { pin_temperature: true, constant_diffusivity: Some(CHI), flux_limiter: true }).unwrap();
        let free = Heat::new(&physics, &elements(), TestFlags { pin_temperature: false, constant_diffusivity: Some(CHI), flux_limiter: true }).unwrap();
        let before = st.clone();
        let t = temps(&st);
        let w = st.width;
        let p = pinned.step(&mut st, 1.0);
        assert_eq!(st, before, "the planes are bit for bit what they were");
        assert_eq!((p.passes, p.rounds, p.radiated), (0, 0, 0.0));
        assert_eq!(p.flux, free.law(&st), "F is the law on the start state");
        let (x, y) = (70usize, 64usize);
        let want = -CHI * INV_MU * (t[y * w + x] - t[y * w + x - 1]);
        assert!((p.flux.x[y * (w + 1) + x] - want).abs() <= 1e-12 * want.abs(), "{} vs {want}", p.flux.x[y * (w + 1) + x]);
        let mut moved = before.clone();
        let q = free.step(&mut moved, 1.0);
        assert_ne!(moved, before, "the free pass moves heat");
        assert_ne!(p.flux, q.flux, "the free pass keeps the flux it solved");
    }

    /// G-HEAT1 on the GPU's implicit step (M0-T118): 24 passes of Δt = 1, each converged in two Newton passes or more (the
    /// second the linear law's check), σ² at t = 8, 16, 24 within 1 %, the heat to 1e-5, and the field the twin's
    /// (`Heat::step`) within 1e-4 of the peak.
    fn gpu_the_variance_grows_as_sigma0_squared_plus_two_chi_t_and_the_heat_is_kept() {
        let scene = scene("heat_pulse.json");
        let (st, heat, bg) = gaussian(&scene, SIGMA0, PEAK);
        let gpu = OnGpu::new(&scene.physics().unwrap(), flags(&scene), &st);
        let mut twin = gpu.first.clone();
        let (h0, _, _) = moments(&gpu.first, bg);
        let mut worst: f64 = 0.0;
        let (mut passes, mut rounds, mut delta) = (0, 0, 0.0f32);
        for pass in 1..=24 {
            let r = gpu.pass(1.0);
            assert!(r.converged && r.passes >= 2 && r.rounds > 0 && r.dt == 1.0, "{r:?}");
            assert!(heat.step(&mut twin, 1.0).converged);
            (passes, rounds, delta) = (passes.max(r.passes), rounds.max(r.rounds), delta.max(r.delta.abs()));
            if pass % 8 == 0 {
                let got = gpu.planes();
                let t = f64::from(pass);
                let want = SIGMA0 * SIGMA0 + 2.0 * CHI * t;
                let (h, vx, vy) = moments(&got, bg);
                let err = ((vx - want).abs() / want).max((vy - want).abs() / want);
                let gap = largest_gap(&temps(&got), &temps(&twin));
                println!("SR-HEAT1 GPU t = {t}: sigma2 x {vx:.6}, y {vy:.6}, expected {want}, error {err:.2e}; heat {:.2e}; |T - twin| {gap:.2e}", (h - h0).abs() / h0);
                assert!(err <= 0.01, "t = {t}: σ² x {vx}, y {vy}, expected {want}");
                assert!((h - h0).abs() <= 1e-5 * h0, "heat {h} against {h0}");
                assert!(gap <= 1e-4 * PEAK, "T against the twin: {gap}");
                worst = worst.max(err);
            }
        }
        // Nothing reaches the edges but f32's round-off: the solve's noise at the floor, kept where it lands above u_vac,
        // books ~2e-9 an ulp at an edge face a pass (M0-D38: 3e-9 of the pulse on the RTX 4080 Laptop) — so the edges' sum is
        // held to 1e-7 of the pulse's heat, and a cell off the edge books nothing (the lead, 2026-10-14, M0-D38). On the V-cycle,
        // to 1e-5 of it, (4)'s own tolerance: from pass 15 its solves meet (4)'s measures in 2 rounds with the closure δ ~4e-7
        // (Jacobi's ~1e-9), and δ lifts every cell, the floor's at the edges too, ~2e-8 a pass (5.9e-7 of the pulse on the RTX
        // 4090; the lead, 2026-10-22, M0-D83: «V-cycle at 1e-5 (Recommended)»).
        let band = if gpu.step.heat_precond() == Some(HeatPrecond::Multigrid) { 1e-5 } else { 1e-7 };
        let rad = gpu.radiated();
        let w = gpu.first.width;
        let h = gpu.first.height;
        let leak: f64 = rad.iter().map(|r| r.abs()).sum();
        println!("SR-HEAT1 GPU radiated {leak:.2e}, {:.2e} of the pulse (band {band:e})", leak / h0);
        assert!(leak <= band * h0, "the edges radiated {leak} of a pulse of {h0}");
        for (i, &r) in rad.iter().enumerate() {
            let (x, y) = (i % w, i / w);
            assert!(r == 0.0 || x == 0 || y == 0 || x == w - 1 || y == h - 1, "cell ({x}, {y}) off the edge booked {r}");
        }
        println!("SR-HEAT1 GPU worst sigma2 error {worst:.2e}; up to {passes} Newton passes, {rounds} rounds, |delta| {delta:.1e}");
    }

    /// F kept on the GPU — the flux the implicit step moved: every face the twin's within 1e-4 of the largest, −χ·C·ΔT signs;
    /// side.flux the mean of a cell's two faces (the field step n + 1's P4 reads).
    fn gpu_the_flux_kept_is_the_twins_and_side_flux_is_its_cell_mean() {
        let scene = scene("heat_pulse.json");
        let (st, heat, _) = gaussian(&scene, SIGMA0, PEAK);
        let gpu = OnGpu::new(&scene.physics().unwrap(), flags(&scene), &st);
        gpu.pass(1.0);
        let p = heat.step(&mut gpu.first.clone(), 1.0);
        let (fx, fy) = gpu.faces();
        let scale = p.flux.x.iter().chain(&p.flux.y).fold(0.0f64, |m, f| m.max(f.abs()));
        let gap = largest_gap(&fx, &p.flux.x).max(largest_gap(&fy, &p.flux.y));
        println!("SR-HEAT1 GPU flux: largest |F - twin| {gap:.2e} of a largest |F| {scale:.4}");
        assert!(scale > 0.1 && gap <= 1e-4 * scale, "{gap} of {scale}");
        let w = gpu.first.width;
        assert!(fx[64 * (w + 1) + 70] > 0.0 && fx[64 * (w + 1) + 58] < 0.0 && fy[70 * w + 64] > 0.0 && fy[58 * w + 64] < 0.0);
        let side = read(&gpu.gpu, gpu.side.flux());
        let n = gpu.first.cells();
        let h = gpu.first.height;
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                let (cx, cy) = (0.5 * (fx[y * (w + 1) + x] + fx[y * (w + 1) + x + 1]), 0.5 * (fy[y * w + x] + fy[(y + 1) * w + x]));
                assert!((f64::from(side[i]) - cx).abs() <= 1e-6 * (scale + cx.abs()), "F_x at ({x}, {y}): {} vs {cx}", side[i]);
                assert!((f64::from(side[n + i]) - cy).abs() <= 1e-6 * (scale + cy.abs()), "F_y at ({x}, {y}): {} vs {cy}", side[n + i]);
            }
        }
    }

    /// The hot slab on the GPU's implicit step, the radiative law (u = a_r T⁴, Newton passes): each pass converged,
    /// `radiated` equals the heat the grid lost within 1e-5 and the twin's (`Heat::step`) within 1e-4, booked exactly on the
    /// slab's ring, and T the twin's within 1e-4 of the slab's 30.
    fn gpu_a_hot_slab_books_radiated_equal_to_the_heat_the_grid_lost() {
        let scene = rect_world(64, 64, [0, 16, 32, 32], 1.0, 30.0);
        let (st, physics, heat) = world(&scene);
        let gpu = OnGpu::new(&physics, flags(&scene), &st);
        let mut twin = gpu.first.clone();
        let mut twin_radiated = 0.0;
        let (mut passes, mut rounds, mut delta, mut gap) = (0, 0, 0.0f32, 0.0f64);
        for _ in 0..12 {
            let r = gpu.pass(2.0);
            let p = heat.step(&mut twin, 2.0);
            assert!(r.converged && r.passes >= 2 && p.converged, "{r:?}");
            (passes, rounds, delta) = (passes.max(r.passes), rounds.max(r.rounds), delta.max(r.delta.abs()));
            twin_radiated += p.radiated;
            gap = gap.max(largest_gap(&temps(&gpu.planes()), &temps(&twin)));
        }
        let emitted = gpu.radiated();
        let radiated: f64 = emitted.iter().sum();
        let lost = energy(&gpu.first) - energy(&gpu.planes());
        println!("SR-HEAT-RAD GPU: lost {lost:.6}, radiated {radiated:.6}, twin {twin_radiated:.6}; |T - twin| {gap:.2e}; up to {passes} Newton passes, {rounds} rounds, |delta| {delta:.1e}");
        assert!(gap <= 1e-4 * 30.0, "T against the twin: {gap}");
        assert!((lost - radiated).abs() <= 1e-5 * radiated, "lost {lost}, radiated {radiated}");
        assert!((radiated - twin_radiated).abs() <= 1e-4 * twin_radiated, "{radiated} vs the twin's {twin_radiated}");
        for (i, &e) in emitted.iter().enumerate() {
            let (x, y) = (i % 64, i / 64);
            let ring = x < 32 && (16..48).contains(&y) && (x == 0 || x == 31 || y == 16 || y == 47);
            assert!(if ring { e > 0.0 } else { e == 0.0 }, "cell ({x}, {y}) emitted {e}");
        }
    }

    /// `test.pin_temperature` on the GPU's implicit step: no pass runs, nothing is booked, the planes are the start's bit for
    /// bit, F is the law on the start state (the twin's `Heat::law` within 1e-4 of the largest face) — not the flux a free
    /// pass solves.
    fn gpu_pinned_temperature_leaves_the_state_and_keeps_the_flux() {
        let scene = scene("heat_pulse.json");
        let (st, heat, _) = gaussian(&scene, SIGMA0, PEAK);
        let physics = scene.physics().unwrap();
        let pinned = OnGpu::new(&physics, TestFlags { pin_temperature: true, ..flags(&scene) }, &st);
        let r = pinned.pass(1.0);
        assert_eq!((r.passes, r.rounds, r.dt, r.delta), (0, 0, 0.0, 0.0), "{r:?}");
        let after = pinned.planes();
        assert!(after.planes.iter().zip(&pinned.first.planes).all(|(a, b)| a.to_bits() == b.to_bits()), "the planes moved");
        assert!(pinned.radiated().iter().all(|&e| e == 0.0));
        let law = heat.law(&pinned.first);
        let (fx, fy) = pinned.faces();
        let scale = law.x.iter().chain(&law.y).fold(0.0f64, |m, f| m.max(f.abs()));
        let gap = largest_gap(&fx, &law.x).max(largest_gap(&fy, &law.y));
        assert!(scale > 0.1 && gap <= 1e-4 * scale, "F against the law: {gap} of {scale}");
        let free = OnGpu::new(&physics, flags(&scene), &st);
        assert!(free.pass(1.0).passes >= 2);
        assert_ne!(pinned.faces(), free.faces(), "the free pass keeps the flux it solved");
    }
}

/// G-HEAT2 (contract §0.5 [M0-TJ4]): the implicit step against its own system's solve and against explicit sub-steps; the
/// radiative step solved cell by cell; the GPU's implicit step on the linear law against the twin (M0-T118).
mod implicit {
    use super::*;

    on_both_preconditioners!(
        gpu_one_pass_is_the_twins_and_runs_bit_for_bit_twice,
        gpu_a_sharp_pulse_stays_non_negative_and_spreads_as_two_chi_t,
    );

    /// One backward-Euler step of `dt` on a constant-diffusivity medium, solved here by Gauss–Seidel until no cell moves by
    /// 1e-14: C_i (T_i − T_i⁰)/Δt = Σ_f χ C_f (T_nb − T_i), C = Σ/μ, C_f the mean of the face's two C; beyond the world's edge
    /// the vacuum state (C = Σ_floor/μ, T = T_floor) — the contract's law written from §0.5 [M0-TJ4] (1)–(2), not the scheme.
    fn backward_euler(st: &State, t0: &[f64], physics: &Physics, chi: f64, dt: f64) -> Vec<f64> {
        let (w, h) = (st.width, st.height);
        let (c_vac, t_vac) = (physics.get("sigma_floor").unwrap() * INV_MU, physics.get("t_floor").unwrap());
        let c: Vec<f64> = (0..w * h).map(|i| st.planes[i] * INV_MU).collect();
        let mut t = t0.to_vec();
        for sweep in 0..100_000 {
            let mut moved: f64 = 0.0;
            for y in 0..h {
                for x in 0..w {
                    let i = y * w + x;
                    let nb = [(x > 0).then(|| i - 1), (x + 1 < w).then(|| i + 1), (y > 0).then(|| i - w), (y + 1 < h).then(|| i + w)];
                    let (mut diag, mut rhs) = (c[i] / dt, c[i] / dt * t0[i]);
                    for j in nb {
                        let (cj, tj) = j.map_or((c_vac, t_vac), |j| (c[j], t[j]));
                        let d = chi * 0.5 * (c[i] + cj);
                        diag += d;
                        rhs += d * tj;
                    }
                    let next = rhs / diag;
                    moved = moved.max((next - t[i]).abs());
                    t[i] = next;
                }
            }
            if moved <= 1e-14 {
                println!("SR-HEAT2 Gauss-Seidel: {sweep} sweeps");
                return t;
            }
        }
        panic!("Gauss-Seidel did not settle");
    }

    /// G-HEAT2 (a): one pass of Δt = 24 (48 Δt_p) is its own system's solution — within 1e-10 of the peak of the
    /// Gauss–Seidel solve — and σ² is within 1 % of σ₀² + 2χt (exact by the moment property).
    #[test]
    fn one_pass_solves_its_own_system_and_keeps_sigma_squared() {
        let scene = scene("heat_pulse.json");
        let (mut st, heat, bg) = gaussian(&scene, SIGMA0, PEAK);
        let tight = backward_euler(&st, &temps(&st), &scene.physics().unwrap(), CHI, 24.0);
        let p = heat.step(&mut st, 24.0);
        let gap = largest_gap(&temps(&st), &tight);
        let (total, vx, vy) = moments(&st, bg);
        let want = SIGMA0 * SIGMA0 + 2.0 * CHI * 24.0;
        println!("SR-HEAT2 one pass of 24: {} Newton passes, {} rounds, delta {:.1e}; |T - tight solve| {gap:.2e}; sigma2 x {vx:.6} y {vy:.6}, expected {want}", p.passes, p.rounds, p.delta);
        assert!(p.converged && p.passes == 2, "{} passes", p.passes);
        // One step of 24 reaches the edges: backward Euler's response falls off as exp(−r/√(χΔt)) (3.5 cells), so the light
        // leaving there (the Gauss–Seidel solve has it too) is ~1e-7 of the pulse — measured 1.1e-5 of 200 on linux-pc.
        assert!(p.radiated >= 0.0 && p.radiated <= 1e-6 * total, "radiated {} of {total}", p.radiated);
        assert!(gap <= 1e-10 * PEAK, "{gap}");
        assert!((vx - want).abs() / want <= 0.01 && (vy - want).abs() / want <= 0.01, "{vx} {vy} vs {want}");
        assert!(total > 100.0);
    }

    /// G-HEAT2 (b): backward Euler's order — the pulse to t = 24 in 6, 12 and 24 passes against 1,536 explicit sub-steps of
    /// Δt = 1/64: the largest |ΔT| e(6) ≤ 0.03 of the peak, e(6)/e(12) and e(12)/e(24) each within [1.8, 2.2].
    #[test]
    fn backward_euler_is_first_order_in_the_step() {
        let (st, heat, _) = gaussian(&scene("heat_pulse.json"), SIGMA0, PEAK);
        let (w, h) = (st.width, st.height);
        let reference = explicit(&temps(&st), w, h, CHI, 1.0 / 64.0, 1536);
        let e: Vec<f64> = [6u32, 12, 24]
            .iter()
            .map(|&k| {
                let mut s = st.clone();
                for _ in 0..k {
                    assert!(heat.step(&mut s, 24.0 / f64::from(k)).converged);
                }
                largest_gap(&temps(&s), &reference)
            })
            .collect();
        let (r1, r2) = (e[0] / e[1], e[1] / e[2]);
        println!("SR-HEAT2 order: e(6) {:.4e}, e(12) {:.4e}, e(24) {:.4e}; ratios {r1:.3}, {r2:.3}", e[0], e[1], e[2]);
        assert!(e[0] <= 0.03 * PEAK, "{}", e[0]);
        assert!((1.8..=2.2).contains(&r1) && (1.8..=2.2).contains(&r2), "{r1} {r2}");
    }

    /// The adversarial case: a one-cell pulse (every wavenumber present) in 15 passes of Δt = 3.2, read as the difference of
    /// two runs on a background of 2.0 (the law linear, so the difference is the pulse's own response with the edges' cooling
    /// of the background taken out): σ² within 1 % of an explicit run's and of 2χt, the heat to 1e-6, and never negative —
    /// backward Euler's maximum principle, where RKL2 rang 2–4 % below zero (M0-T41).
    #[test]
    fn a_sharp_pulse_stays_non_negative_and_spreads_as_two_chi_t() {
        let scene = scene("heat_pulse.json");
        let (mut with, heat, bg) = gaussian_on(&scene, Some(2.0), 0.01, PEAK);
        let (mut without, _, _) = gaussian_on(&scene, Some(2.0), 0.01, 0.0);
        let (t0, w, h) = (temps(&with), with.width, with.height);
        let mut ring: f64 = 0.0;
        for _ in 0..15 {
            assert!(heat.step(&mut with, 3.2).converged && heat.step(&mut without, 3.2).converged);
            let low = temps(&with).iter().zip(temps(&without)).map(|(a, b)| a - b).fold(f64::MAX, f64::min);
            ring = ring.min(low);
        }
        let response: Vec<f64> = temps(&with).iter().zip(temps(&without)).map(|(a, b)| a - b).collect();
        let pulse: Vec<f64> = t0.iter().map(|t| t - bg).collect();
        let (_, ex, ey) = stats(&explicit(&pulse, w, h, CHI, 0.2, 240), w, 0.0, usize::MAX);
        let (total, vx, vy) = stats(&response, w, 0.0, usize::MAX);
        let highest = response.iter().cloned().fold(f64::MIN, f64::max);
        println!("SR-HEAT2 sharp pulse, 15 passes: sigma2 x {vx:.5} y {vy:.5}, explicit x {ex:.5} y {ey:.5}, lowest {ring:.3e}, highest {highest:.4e}, heat {total:.9}");
        assert!((vx - ex).abs() / ex <= 0.01 && (vy - ey).abs() / ey <= 0.01, "{vx} {vy} vs {ex} {ey}");
        assert!((vx - 48.0).abs() / 48.0 <= 0.01, "{vx} vs 2 chi t = 48");
        assert!((total - PEAK).abs() / PEAK <= 1e-6, "heat {total}");
        assert!(ring >= -1e-9 && highest <= 0.01, "lowest {ring}, highest {highest}");
    }

    /// The radiative law (u = a_r T⁴, the limiter on): a hot slab (T = 30, Σ = 1) at the world's left edge beside vacuum, one
    /// pass of Δt = 2 — every gas cell solves backward Euler's nonlinear balance with the flux the pass kept,
    /// |C_i (T_i − T_i⁰) + Δt Σ_f F_f| ≤ 1e-8 (C_i T_i⁰ + Δt Σ_f |F_f|) (G-STIFF (c)'s measure), in more Newton passes than the
    /// linear law's two, and the grid loses what it books.
    #[test]
    fn the_radiative_step_solves_its_nonlinear_balance_cell_by_cell() {
        let scene = rect_world(64, 64, [0, 16, 32, 32], 1.0, 30.0);
        let (mut st, physics, heat) = world(&scene);
        let sigma_vac = physics.get("sigma_vac").unwrap();
        let (w, n) = (st.width, st.cells());
        let (t0, e0) = (temps(&st), energy(&st));
        let p = heat.step(&mut st, 2.0);
        let t1 = temps(&st);
        let f = &p.flux;
        let mut worst: f64 = 0.0;
        for i in (0..n).filter(|&i| st.planes[i] >= sigma_vac) {
            let (x, y) = (i % w, i / w);
            let faces = [f.x[y * (w + 1) + x + 1], -f.x[y * (w + 1) + x], f.y[(y + 1) * w + x], -f.y[y * w + x]];
            let c = st.planes[i] * INV_MU;
            let balance = c * (t1[i] - t0[i]) + 2.0 * faces.iter().sum::<f64>();
            let scale = c * t0[i] + 2.0 * faces.iter().map(|q| q.abs()).sum::<f64>();
            worst = worst.max(balance.abs() / scale);
        }
        let lost = e0 - energy(&st);
        println!("SR-HEAT2 radiative slab: {} Newton passes, {} rounds, delta {:.1e}; worst balance {worst:.2e}; lost {lost:.6} of radiated {:.6}", p.passes, p.rounds, p.delta, p.radiated);
        assert!(p.converged && p.passes >= 3, "{} passes", p.passes);
        assert!(worst <= 1e-8, "{worst}");
        assert!(p.radiated > 0.0 && (lost - p.radiated).abs() <= 1e-9 * p.radiated, "lost {lost}, radiated {}", p.radiated);
    }

    /// G-HEAT2 (a) on the GPU (contract §0.5 [M0-TJ4]; M0-T118): one pass of Δt = 24 (48 Δt_p) from the same f32 start as the
    /// twin — the committed T within 1e-4 of the peak of the twin's `Heat::step` (the solved field, never the solve's own
    /// residual), σ² within 1 % of σ₀² + 2χt — and the same pass run again bit for bit: the planes, the record, the faces
    /// (every reduction a fixed-order tree, §1.3.4).
    fn gpu_one_pass_is_the_twins_and_runs_bit_for_bit_twice() {
        let scene = scene("heat_pulse.json");
        let (st, heat, bg) = gaussian(&scene, SIGMA0, PEAK);
        let gpu = OnGpu::new(&scene.physics().unwrap(), flags(&scene), &st);
        let mut twin = gpu.first.clone();
        let p = heat.step(&mut twin, 24.0);
        let r = gpu.pass(24.0);
        let got = gpu.planes();
        let gap = largest_gap(&temps(&got), &temps(&twin));
        let (_, vx, vy) = moments(&got, bg);
        let want = SIGMA0 * SIGMA0 + 2.0 * CHI * 24.0;
        println!("SR-HEAT2 GPU one pass of 24: {r:?}; the twin {} passes, {} rounds; |T - twin| {gap:.2e} of the peak; sigma2 x {vx:.6} y {vy:.6}, expected {want}", p.passes, p.rounds);
        assert!(r.converged && r.passes >= 2 && r.rounds > 0 && r.dt == 24.0, "{r:?}");
        assert!(gap <= 1e-4 * PEAK, "T against the twin: {gap}");
        assert!((vx - want).abs() / want <= 0.01 && (vy - want).abs() / want <= 0.01, "{vx} {vy} vs {want}");
        let faces = gpu.faces();
        gpu.restart();
        let again = gpu.pass(24.0);
        assert_eq!(again, r, "the record");
        assert!(gpu.planes().planes.iter().zip(&got.planes).all(|(a, b)| a.to_bits() == b.to_bits()), "the planes, bit for bit");
        assert_eq!(gpu.faces(), faces, "the faces, bit for bit");
    }

    /// The adversarial case on the GPU's implicit step (M0-T118): a one-cell pulse (every wavenumber present) of 100 over the
    /// floor background — the law is linear, and at 100 the pulse holds most of the field's heat, the scale the solve's
    /// tolerance is relative to (§0.5 [M0-TJ4] (4)) — 15 passes of Δt = 3.2, each converged: the field within 1e-4 of the
    /// peak of the twin's from the same f32 start, σ² within 1 % of an explicit run's and of 2χt, the pulse's heat to 1e-5,
    /// never below −1e-5 of the peak (backward Euler's maximum principle, to the solve's own tolerance in f32).
    fn gpu_a_sharp_pulse_stays_non_negative_and_spreads_as_two_chi_t() {
        const SHARP: f64 = 100.0;
        let scene = scene("heat_pulse.json");
        let (st, heat, _) = gaussian(&scene, 0.01, SHARP);
        let gpu = OnGpu::new(&scene.physics().unwrap(), flags(&scene), &st);
        let mut twin = gpu.first.clone();
        let (w, h) = (st.width, st.height);
        // The background as the f32 planes hold it (T_floor's f32 E over Σ), so the pulse is the only heat counted.
        let bg = temps(&gpu.first)[0];
        let pulse: Vec<f64> = temps(&gpu.first).iter().map(|t| t - bg).collect();
        let (mut low, mut gap): (f64, f64) = (0.0, 0.0);
        let (mut passes, mut rounds) = (0, 0);
        for _ in 0..15 {
            let r = gpu.pass(3.2);
            assert!(r.converged && heat.step(&mut twin, 3.2).converged, "{r:?}");
            (passes, rounds) = (passes.max(r.passes), rounds.max(r.rounds));
            let t = temps(&gpu.planes());
            low = low.min(t.iter().map(|t| t - bg).fold(f64::MAX, f64::min));
            gap = gap.max(largest_gap(&t, &temps(&twin)));
        }
        let response: Vec<f64> = temps(&gpu.planes()).iter().map(|t| t - bg).collect();
        let (_, ex, ey) = stats(&explicit(&pulse, w, h, CHI, 0.2, 240), w, 0.0, usize::MAX);
        let (total, vx, vy) = stats(&response, w, 0.0, usize::MAX);
        let (p0, _, _) = stats(&pulse, w, 0.0, usize::MAX);
        let highest = response.iter().cloned().fold(f64::MIN, f64::max);
        println!("SR-HEAT2 GPU sharp pulse of {SHARP}, 15 passes: |T - twin| {:.2e} of the peak; sigma2 x {vx:.5} y {vy:.5}, explicit x {ex:.5} y {ey:.5}; lowest {:.3e} of the peak, highest {highest:.4e}; heat {total:.7} of {p0:.7}; up to {passes} Newton passes, {rounds} rounds", gap / SHARP, low / SHARP);
        assert!(gap <= 1e-4 * SHARP, "T against the twin: {gap}");
        assert!((vx - ex).abs() / ex <= 0.01 && (vy - ey).abs() / ey <= 0.01, "{vx} {vy} vs {ex} {ey}");
        assert!((vx - 48.0).abs() / 48.0 <= 0.01, "{vx} vs 2 chi t = 48");
        assert!((total - p0).abs() <= 1e-5 * p0, "heat {total} of {p0}");
        assert!(low >= -1e-5 * SHARP && highest <= 0.01 * SHARP, "lowest {low}, highest {highest}");
    }
}

/// G-FLD: |F| ≤ c_sb a_r T⁴ in thin gas; the radiative conductivity in thick gas; λ.
mod limiter {
    use super::*;

    on_both_preconditioners!(
        gpu_in_thin_gas_the_flux_never_passes_c_sb_a_r_t4,
        gpu_thick_gas_carries_the_radiative_conductivity_and_the_flag_turns_the_limiter_off,
    );

    fn constants(physics: &Physics) -> (f64, f64) {
        (physics.get("a_r").unwrap(), physics.get("c_sb").unwrap())
    }

    /// The largest ratio |F|/(c_sb a_r T⁴) over every face of `f` for the state `t` (the hotter side's T of the face; the
    /// world's edges face the vacuum at T_floor).
    fn worst_ratio(f: &Flux, t: &[f64], w: usize, h: usize, a_r: f64, c_sb: f64) -> f64 {
        let t_floor = 0.05;
        let bound = |ta: f64, tb: f64| c_sb * a_r * ta.max(tb).powi(4);
        let mut worst: f64 = 0.0;
        for y in 0..h {
            for x in 0..=w {
                let ta = if x > 0 { t[y * w + x - 1] } else { t_floor };
                let tb = if x < w { t[y * w + x] } else { t_floor };
                worst = worst.max(f.x[y * (w + 1) + x].abs() / bound(ta, tb));
            }
        }
        for y in 0..=h {
            for x in 0..w {
                let ta = if y > 0 { t[(y - 1) * w + x] } else { t_floor };
                let tb = if y < h { t[y * w + x] } else { t_floor };
                worst = worst.max(f.y[y * w + x].abs() / bound(ta, tb));
            }
        }
        worst
    }

    /// The hot spot of heat_spot.json over 20 passes of Δt = 0.05: the law each pass freezes (its start state's flux,
    /// contract §0.5 [M0-TJ4]'s G-FLD) under c_sb a_r T⁴ of the hotter side at every face; the limiter binds.
    #[test]
    fn in_thin_gas_the_flux_never_passes_c_sb_a_r_t4() {
        let (mut st, physics, heat) = world(&scene("heat_spot.json"));
        let (a_r, c_sb) = constants(&physics);
        let (w, h) = (st.width, st.height);
        let mut worst: f64 = 0.0;
        let (mut passes, mut rounds) = (0, 0);
        for pass in 0..20 {
            let t = temps(&st);
            let r = worst_ratio(&heat.law(&st), &t, w, h, a_r, c_sb);
            let p = heat.step(&mut st, 0.05);
            assert!(p.converged, "pass {pass}: {} passes, {} rounds", p.passes, p.rounds);
            (passes, rounds) = (passes.max(p.passes), rounds.max(p.rounds));
            assert!(r <= 1.0 + 1e-6, "pass {pass}: |F| = {r} c_sb a_r T^4");
            worst = worst.max(r);
        }
        println!("SR-FLD worst |F| / (c_sb a_r T^4) {worst:.6} over 20 passes of 64x64 thin gas, up to {passes} Newton passes and {rounds} rounds");
        // The oracle is not vacuous: the limiter is the thing holding the flux, within a factor of 3 of the bound.
        assert!(worst > 0.3, "the limiter binds: {worst}");
        // And it can fail: the same spot without the limiter (λ ≡ 1/3) puts the flux orders of magnitude over it.
        let (st, physics, _) = world(&scene("heat_spot.json"));
        let off = Heat::new(&physics, &elements(), TestFlags { pin_temperature: false, constant_diffusivity: None, flux_limiter: false }).unwrap();
        let t = temps(&st);
        let r = worst_ratio(&off.law(&st), &t, w, h, a_r, c_sb);
        println!("SR-FLD limiter off: |F| = {r:.1} c_sb a_r T^4");
        assert!(r > 100.0, "{r}");
    }

    /// Thick gas (κΣ = 200), a gentle gradient: λ = 1/3 and the solved F is the radiative conductivity (4/3) a_r c_sb
    /// T³/(κΣ) times −∇T of the solved temperatures (SM-E20) to 1 % — the limiter must not touch gas it has no business in.
    #[test]
    fn in_thick_gas_the_flux_is_the_radiative_conductivity() {
        let scene = rect_world(64, 64, [0, 0, 64, 64], 100.0, 10.0);
        let (mut st, physics, heat) = world(&scene);
        let (a_r, c_sb) = constants(&physics);
        let (w, h) = (st.width, st.height);
        let t: Vec<f64> = (0..w * h).map(|i| 10.0 + 0.02 * (i % w) as f64).collect();
        set_temps(&mut st, &t);
        let p = heat.step(&mut st, 0.001);
        assert!(p.converged && p.rounds > 0, "{} passes, {} rounds", p.passes, p.rounds);
        let t = temps(&st);
        let ks = KAPPA_H * 100.0;
        let mut worst: f64 = 0.0;
        for y in 1..h - 1 {
            for x in 1..w {
                let mid = 0.5 * (t[y * w + x - 1] + t[y * w + x]);
                let want = -(4.0 / 3.0) * a_r * c_sb * mid.powi(3) / ks * (t[y * w + x] - t[y * w + x - 1]);
                let got = p.flux.x[y * (w + 1) + x];
                worst = worst.max((got - want).abs() / want.abs());
            }
        }
        println!("SR-FLD thick gas: worst |F - (4/3) a c T^3/(kappa Sigma) (-dT)| / |F| = {worst:.2e}");
        assert!(worst <= 0.01, "{worst}");
        // R is ~10^-4 here, so λ is 1/3 to 10^-4.
        assert!((lambda(1e-4) * 3.0 - 1.0).abs() < 1e-3);
    }

    #[test]
    fn lambda_is_levermore_pomraning() {
        // λ(R) = (2 + R)/(6 + 3R + R²): 1/3 at 0, 3/10 at 1, 4/16 at 2, and λR < 1 → 1.
        for (r, want) in [(0.0, 1.0 / 3.0), (1.0, 0.3), (2.0, 0.25), (3.0, 5.0 / 24.0), (10.0, 12.0 / 136.0)] {
            assert!((lambda(r) - want).abs() < 1e-15, "R = {r}: {}", lambda(r));
        }
        for r in [0.01, 1.0, 100.0, 1e6, 1e12] {
            assert!(lambda(r) * r < 1.0, "R = {r}");
        }
        assert!(lambda(1e12) * 1e12 > 1.0 - 1e-9);
        // λ falls monotonically, so the flux (c λ ∇E/κΣ) grows more slowly than the gradient.
        assert!((0..60).map(|k| lambda(f64::from(k))).collect::<Vec<_>>().windows(2).all(|w| w[1] < w[0]));
    }

    /// `test.flux_limiter` = false (§2.12.1) is λ ≡ 1/3 even at R = 10⁴: the law's F = c_sb ΔE/(3 κΣ_face) across a face of
    /// heat_spot.json.
    #[test]
    fn the_scene_flag_turns_the_limiter_into_one_third() {
        let (st, physics, _) = world(&scene("heat_spot.json"));
        let (a_r, c_sb) = constants(&physics);
        let w = st.width;
        let off = Heat::new(&physics, &elements(), TestFlags { pin_temperature: false, constant_diffusivity: None, flux_limiter: false }).unwrap();
        let on = Heat::new(&physics, &elements(), TestFlags::default()).unwrap();
        let t = temps(&st);
        // The face between a hot cell (T = 2, x = 28) and its cooler neighbour (T = 0.05, x = 27) on row 30.
        let (x, y) = (28usize, 30usize);
        let (ta, tb) = (t[y * w + x - 1], t[y * w + x]);
        assert!((ta - 0.05).abs() < 1e-6 && (tb - 2.0).abs() < 1e-6, "{ta} {tb}");
        let ks = KAPPA_H * 0.5 * (st.get(0, y * w + x - 1) + st.get(0, y * w + x));
        let f_off = off.law(&st).x[y * (w + 1) + x];
        let f_on = on.law(&st).x[y * (w + 1) + x];
        let thick = -c_sb / (3.0 * ks) * a_r * (tb.powi(4) - ta.powi(4));
        let bound = c_sb * a_r * tb.powi(4);
        println!("SR-FLD face of the spot: unlimited {f_off:.4e} (thick form {thick:.4e}), limited {f_on:.4e}, bound {bound:.4e}");
        assert!((f_off - thick).abs() <= 1e-9 * thick.abs(), "{f_off} vs {thick}");
        assert!(f_off.abs() > 1000.0 * bound);
        assert!(f_on.abs() <= bound * (1.0 + 1e-6) && f_on.abs() > 0.1 * bound, "{f_on}");
        assert!(f_on * f_off > 0.0, "the same direction");
    }

    /// G-FLD on the GPU (§0.5 [M0-TJ4]'s G-FLD): heat_spot.json over 20 passes of Δt = 0.05, each converged; the law each
    /// pass freezes — read from a pinned run on the pass's start state — under c_sb a_r T⁴ of the hotter side (1 + 1e-6) at
    /// every face, the limiter binding; on the first pass, from the same f32 start as the twin, the law within 1e-4 of the
    /// largest face of `Heat::law`, T within 1e-4 of the spot's 2 of `Heat::step`, and the solved F (the flux the step moved)
    /// its by the energy it moves — Δt |F − F_twin| ≤ 1e-5 of the heat Σ C T⁰ at every face (the lead, M0-T119) — the
    /// adversarial case: u = a_r T⁴ spans 2.5 × 10⁻¹² (T_floor) to 6.4 × 10⁻⁶ (T = 2) here.
    fn gpu_in_thin_gas_the_flux_never_passes_c_sb_a_r_t4() {
        let (st, physics, heat) = world(&scene("heat_spot.json"));
        let (a_r, c_sb) = constants(&physics);
        let gpu = OnGpu::new(&physics, TestFlags::default(), &st);
        let law = OnGpu::new(&physics, TestFlags { pin_temperature: true, ..TestFlags::default() }, &st);
        let (w, h) = (st.width, st.height);
        let dt = 0.05;
        let (mut worst, mut passes, mut rounds): (f64, u32, u32) = (0.0, 0, 0);
        for pass in 0..20 {
            let start = gpu.planes();
            law.upload(&start);
            law.pass(dt);
            let (x, y) = law.faces();
            let frozen = Flux { width: w, height: h, x, y };
            let r = gpu.pass(dt);
            assert!(r.converged && r.passes >= 2, "pass {pass}: {r:?}");
            (passes, rounds) = (passes.max(r.passes), rounds.max(r.rounds));
            if pass == 0 {
                let mut twin = gpu.first.clone();
                let want = heat.law(&twin);
                let scale = want.x.iter().chain(&want.y).fold(0.0f64, |m, f| m.max(f.abs()));
                let gap = largest_gap(&frozen.x, &want.x).max(largest_gap(&frozen.y, &want.y));
                let p = heat.step(&mut twin, f64::from(dt as f32));
                let (fx, fy) = gpu.faces();
                let moved = p.flux.x.iter().chain(&p.flux.y).fold(0.0f64, |m, f| m.max(f.abs()));
                let solved = largest_gap(&fx, &p.flux.x).max(largest_gap(&fy, &p.flux.y));
                let t_gap = largest_gap(&temps(&gpu.planes()), &temps(&twin));
                println!("SR-FLD GPU first pass: law |F - twin| {gap:.2e} of {scale:.4e}; solved |F - twin| {solved:.2e} of {moved:.4e}; |T - twin| {t_gap:.2e}; {r:?}, the twin {} passes, {} rounds", p.passes, p.rounds);
                assert!(gap <= 1e-4 * scale, "the law against the twin: {gap} of {scale}");
                // The solved F graded by the energy it moves (the lead, 2026-10-14, M0-T119: «Grade by energy»): inside the
                // spot neighbours differ by ~3e-7 of T and D_f ≈ 3.3e5, so one f32 step of u moves F by ~2e-4 of the largest
                // face; the step's heat error is what the solve's tolerance holds (§0.5 [M0-TJ4] (4)).
                let first = temps(&gpu.first);
                let heat0: f64 = (0..w * h).map(|i| gpu.first.get(0, i) * INV_MU * first[i]).sum();
                let moved_err = f64::from(dt as f32) * solved / heat0;
                println!("SR-FLD GPU solved F: Δt |F - twin| {moved_err:.2e} of the heat {heat0:.4e} ({:.2e} of the largest face)", solved / moved);
                assert!(moved_err <= 1e-5, "the solved F against the twin: Δt {solved} of the heat {heat0}");
                assert!(t_gap <= 1e-4 * 2.0, "T against the twin: {t_gap}");
            }
            let ratio = worst_ratio(&frozen, &temps(&start), w, h, a_r, c_sb);
            assert!(ratio <= 1.0 + 1e-6, "pass {pass}: |F| = {ratio} c_sb a_r T^4");
            worst = worst.max(ratio);
        }
        println!("SR-FLD GPU worst |F| / (c_sb a_r T^4) {worst:.6} over 20 passes, up to {passes} Newton passes and {rounds} rounds");
        assert!(worst > 0.3, "the limiter binds: {worst}");
    }

    /// Thick gas on the GPU carries (4/3) a_r c_sb T³/(κΣ) times −∇T of the solved temperatures within 1 % in the solved F;
    /// with `flux_limiter` off the law on a face of the spot (a pinned run) is c_sb ΔE/(3 κΣ_face) within 1e-5, a thousand
    /// times the bound.
    fn gpu_thick_gas_carries_the_radiative_conductivity_and_the_flag_turns_the_limiter_off() {
        let thick = rect_world(64, 64, [0, 0, 64, 64], 100.0, 10.0);
        let (mut st, physics, _) = world(&thick);
        let (a_r, c_sb) = constants(&physics);
        let w = st.width;
        let t: Vec<f64> = (0..w * st.height).map(|i| 10.0 + 0.02 * (i % w) as f64).collect();
        set_temps(&mut st, &t);
        let gpu = OnGpu::new(&physics, flags(&thick), &st);
        let r = gpu.pass(0.001);
        assert!(r.converged, "{r:?}");
        let (fx, _) = gpu.faces();
        let t = temps(&gpu.planes());
        let mut worst: f64 = 0.0;
        for y in 1..st.height - 1 {
            for x in 1..w {
                let mid = 0.5 * (t[y * w + x - 1] + t[y * w + x]);
                let want = -(4.0 / 3.0) * a_r * c_sb * mid.powi(3) / (KAPPA_H * 100.0) * (t[y * w + x] - t[y * w + x - 1]);
                worst = worst.max((fx[y * (w + 1) + x] - want).abs() / want.abs());
            }
        }
        println!("SR-FLD GPU thick gas: worst relative error {worst:.2e}; {r:?}");
        assert!(worst <= 0.01, "{worst}");

        let (st, physics, _) = world(&scene("heat_spot.json"));
        let off = OnGpu::new(&physics, TestFlags { flux_limiter: false, pin_temperature: true, ..TestFlags::default() }, &st);
        off.pass(0.05);
        let t = temps(&off.first);
        let (x, y, w) = (28usize, 30usize, st.width);
        let (ta, tb) = (t[y * w + x - 1], t[y * w + x]);
        let ks = KAPPA_H * 0.5 * (off.first.get(0, y * w + x - 1) + off.first.get(0, y * w + x));
        let thick = -c_sb / (3.0 * ks) * a_r * (tb.powi(4) - ta.powi(4));
        let got = off.faces().0[y * (w + 1) + x];
        println!("SR-FLD GPU limiter off: {got:.4e} against {thick:.4e}, bound {:.4e}", c_sb * a_r * tb.powi(4));
        assert!((got - thick).abs() <= 1e-5 * thick.abs(), "{got} vs {thick}");
        assert!(got.abs() > 1000.0 * c_sb * a_r * tb.powi(4));
    }
}

/// G-STIFF (contract §5.1; §0.5 [M0-TJ4]; M0-T120): one pass of the implicit step on two scenes whose cells RKL2 would
/// have needed ≥ 500 stages for — scenes/heat_hot.json (a burnt helium field) and scenes/heat_rim.json (a hydrogen disk in a
/// thin cold ring). (a) T_floor ≤ T ≤ (1 + 1e-5) max T⁰ in every gas cell; (b) the heat the grid lost equals the booked
/// `radiated` (1e-6 on the twin, 1e-5 on the GPU); (c) the twin's T solves the step, cell by cell, from the flux it kept;
/// (d) the GPU's T against the twin's from the same f32 start, C-weighted relative L1 ≤ 1e-4 and every cell with Σ ≥ 1e-4
/// within 1e-3; (e) heat_hot's heat left within [1.0, 2.2] × the twin's after 64 passes of Δt/64. The bands are the
/// contract's, fixed before the first run (§0.4): a red is a D or a question, never a looser band. The oracles are written
/// here from the contract: the stage count from §2.4.3's definition and §2.4.1's law, the temperatures from E (the cold
/// pressure off, the gas at rest), the balance from the flux the twin kept.
mod stiff {
    use super::*;

    on_both_preconditioners!(
        gpu_the_burnt_field_is_the_twins_and_books_the_heat_it_lost,
        gpu_the_hot_disk_in_a_thin_ring_is_the_twins_and_books_the_heat_it_lost,
        gpu_a_step_at_the_ceiling_moves_no_more_heat_than_its_cells_hold,
    );

    /// A scene, its gas and the Δt of its one pass.
    pub(super) struct Case {
        pub(super) file: &'static str,
        /// The gas's 1/μ and κ = κ₀ (1 + X_H) (§1.5, §2.4.2): helium 3/4 and 1, hydrogen 2 and 2.
        inv_mu: f64,
        kappa: f64,
        pub(super) dt: f64,
    }

    pub(super) const HOT: Case = Case { file: "heat_hot.json", inv_mu: 0.75, kappa: 1.0, dt: 0.02 };
    pub(super) const RIM: Case = Case { file: "heat_rim.json", inv_mu: INV_MU, kappa: KAPPA_H, dt: 0.01 };

    /// The constants a case's oracles read.
    pub(super) struct Ctx {
        w: usize,
        h: usize,
        inv_mu: f64,
        kappa: f64,
        sigma_vac: f64,
        sigma_floor: f64,
        pub(super) t_floor: f64,
        pub(super) a_r: f64,
        c_sb: f64,
    }

    impl Ctx {
        fn new(case: &Case, scene: &Scene) -> Ctx {
            let physics = scene.physics().unwrap();
            let get = |k: &str| physics.get(k).unwrap();
            assert_eq!(get("k_cond"), 0.0, "the stage count below leaves out the conduction");
            assert!(scene.test.constant_diffusivity.is_none() && scene.test.flux_limiter && !scene.test.pin_temperature);
            Ctx {
                w: scene.world.width as usize,
                h: scene.world.height as usize,
                inv_mu: case.inv_mu,
                kappa: case.kappa,
                sigma_vac: get("sigma_vac"),
                sigma_floor: get("sigma_floor"),
                t_floor: get("t_floor"),
                a_r: get("a_r"),
                c_sb: get("c_sb"),
            }
        }

        /// A cell is gas from Σ_vac (§2.3.5).
        fn gas(&self, st: &State, i: usize) -> bool {
            st.planes[i] >= self.sigma_vac
        }

        /// T of every cell from E (the floor below Σ_vac), as [`temps`] with the gas's own 1/μ.
        pub(super) fn temps(&self, st: &State) -> Vec<f64> {
            let n = st.cells();
            (0..n).map(|i| if self.gas(st, i) { st.planes[3 * n + i] / st.planes[i] / self.inv_mu } else { self.t_floor }).collect()
        }

        /// C = Σ/μ of every gas cell, 0 elsewhere.
        pub(super) fn c(&self, st: &State) -> Vec<f64> {
            (0..st.cells()).map(|i| if self.gas(st, i) { st.planes[i] * self.inv_mu } else { 0.0 }).collect()
        }

        /// §2.4.3's RKL2 stage count on `st`, computed from the definition: Δt_p = 1/max_cells (Σ_f K_f / C), K_f = dF/dT of
        /// the face's law (D_f a_r (T_a + T_b)(T_a² + T_b²), D_f = c_sb λ(R)/(κΣ)_f with R = |Δu|/((κΣ)_f ū), the vacuum
        /// state beyond a gas cell's vacuum or edge face), s the smallest integer ≥ 2 with Δt_p (s² + s − 2)/4 ≥ Δt.
        fn rkl2_stages(&self, st: &State, dt: f64) -> usize {
            let (w, h) = (self.w, self.h);
            let t = self.temps(st);
            let side = |x: isize, y: isize| -> (f64, f64) {
                let inside = x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h;
                match inside.then(|| y as usize * w + x as usize) {
                    Some(i) if self.gas(st, i) => (t[i], st.planes[i]),
                    _ => (self.t_floor, self.sigma_floor),
                }
            };
            let k_face = |a: (f64, f64), b: (f64, f64)| -> f64 {
                let ks = 0.5 * self.kappa * (a.1 + b.1);
                let (ua, ub) = (self.a_r * a.0.powi(4), self.a_r * b.0.powi(4));
                let r = (ub - ua).abs() / (ks * 0.5 * (ua + ub));
                self.c_sb * lambda(r) / ks * self.a_r * (a.0 + b.0) * (a.0 * a.0 + b.0 * b.0)
            };
            let mut max_rate: f64 = 0.0;
            for y in 0..h as isize {
                for x in 0..w as isize {
                    let i = y as usize * w + x as usize;
                    if !self.gas(st, i) {
                        continue;
                    }
                    let me = side(x, y);
                    let k: f64 = [(-1, 0), (1, 0), (0, -1), (0, 1)].iter().map(|&(dx, dy)| k_face(me, side(x + dx, y + dy))).sum();
                    max_rate = max_rate.max(k / (st.planes[i] * self.inv_mu));
                }
            }
            let dt_p = 1.0 / max_rate;
            (2..).find(|&s| dt_p * ((s * s + s - 2) as f64) / 4.0 >= dt).unwrap()
        }

        /// (a) in every gas cell of `before`: T after in [T_floor, (1 + 1e-5) max T⁰]. Returns the range of T after and the
        /// ceiling; [`Ctx::assert_bounds`] asserts it.
        fn bounds(&self, before: &State, after: &State) -> (f64, f64, f64) {
            let (t0, t1) = (self.temps(before), self.temps(after));
            let gas = (0..t0.len()).filter(|&i| self.gas(before, i));
            let top = gas.clone().map(|i| t0[i]).fold(f64::MIN, f64::max);
            (gas.clone().map(|i| t1[i]).fold(f64::MAX, f64::min), gas.map(|i| t1[i]).fold(f64::MIN, f64::max), (1.0 + 1e-5) * top)
        }

        fn assert_bounds(&self, (low, high, ceiling): (f64, f64, f64)) {
            assert!(low >= self.t_floor, "(a) T {low} below the floor {}", self.t_floor);
            assert!(high <= ceiling, "(a) T {high} above (1 + 1e-5) max T0 = {ceiling}");
        }
    }

    /// The scene of `case`, its start as the twin holds it, its constants and its twin.
    pub(super) fn load(case: &Case) -> (Scene, Ctx, State, Heat) {
        let scene = scene(case.file);
        let ctx = Ctx::new(case, &scene);
        let (st, _, heat) = world(&scene);
        (scene, ctx, st, heat)
    }

    /// The scenes carry what the oracles assume: their first temperatures and the stage count the contract names (≥ 500,
    /// §5.1) — a scene that needed fewer would test nothing stiff.
    #[test]
    fn the_scenes_are_stiff_for_rkl2_and_carry_what_the_oracles_assume() {
        let (_, hot, st, _) = load(&HOT);
        let t = hot.temps(&st);
        assert!(t.iter().all(|&t| (t - 18_800.0).abs() <= 1e-4 * 18_800.0), "heat_hot: a helium field at T 18,800");
        assert!((0..st.cells()).all(|i| (st.planes[i] - 30.0).abs() < 1e-5), "heat_hot: Σ 30");
        let hot_stages = hot.rkl2_stages(&st, HOT.dt);

        let (_, rim, st, _) = load(&RIM);
        let t = rim.temps(&st);
        let gas = (0..st.cells()).filter(|&i| rim.gas(&st, i)).count();
        let at = |x: usize, y: usize| y * 64 + x;
        assert_eq!(gas, st.cells(), "heat_rim: the ring is gas everywhere in the world");
        assert!((t[at(32, 32)] - 100.0).abs() < 0.01 && (st.planes[at(32, 32)] - 1.0).abs() < 1e-3, "the disk's centre: Σ 1, T 100");
        assert!((t[at(0, 0)] - 5.0).abs() < 1e-3 && (st.planes[at(0, 0)] - 2e-6).abs() < 1e-11, "the ring: Σ 2e-6, T 5");
        assert!((0..12).all(|x| st.planes[at(x, 32)] < 1e-5), "twelve cells of ring at the west edge of the world");
        let rim_stages = rim.rkl2_stages(&st, RIM.dt);
        println!("SR-STIFF RKL2 would need {hot_stages} stages on heat_hot (Δt {}) and {rim_stages} on heat_rim (Δt {})", HOT.dt, RIM.dt);
        assert!(hot_stages >= 500, "heat_hot needs {hot_stages} stages");
        assert!(rim_stages >= 500, "heat_rim needs {rim_stages} stages");
    }

    /// The twin's one pass on `case`: (a) and (b) to 1e-6, and (c) every gas cell solves its own balance from the flux the
    /// step kept, |C (T − T⁰) + Δt Σ_f F_f| ≤ 1e-8 (C T⁰ + Δt Σ_f |F_f|) with outflow positive. Returns the pass.
    fn twin_one_pass(case: &Case) -> Pass {
        let (_, ctx, mut st, heat) = load(case);
        let before = st.clone();
        let (t0, c) = (ctx.temps(&before), ctx.c(&before));
        let p = heat.step(&mut st, case.dt);
        let t1 = ctx.temps(&st);
        let range = ctx.bounds(&before, &st);
        let lost = energy(&before) - energy(&st);
        let (w, f) = (ctx.w, &p.flux);
        let mut worst: f64 = 0.0;
        for i in (0..w * ctx.h).filter(|&i| ctx.gas(&before, i)) {
            let (x, y) = (i % w, i / w);
            let faces = [f.x[y * (w + 1) + x + 1], -f.x[y * (w + 1) + x], f.y[(y + 1) * w + x], -f.y[y * w + x]];
            let balance = c[i] * (t1[i] - t0[i]) + case.dt * faces.iter().sum::<f64>();
            let scale = c[i] * t0[i] + case.dt * faces.iter().map(|q| q.abs()).sum::<f64>();
            worst = worst.max(balance.abs() / scale);
        }
        println!(
            "SR-STIFF twin {}: {} Newton passes, {} rounds, converged {}, delta {:.1e}; T in [{:.4}, {:.4}]; lost {lost:.9e} of radiated {:.9e}; worst balance {worst:.2e}",
            case.file, p.passes, p.rounds, p.converged, p.delta, range.0, range.1, p.radiated
        );
        assert!(p.converged, "{p:?}");
        assert!(p.radiated > 0.0 && (lost - p.radiated).abs() <= 1e-6 * p.radiated, "(b) lost {lost}, radiated {}", p.radiated);
        assert!(worst <= 1e-8, "(c) the balance of a cell: {worst}");
        ctx.assert_bounds(range);
        p
    }

    /// (a)–(c) on the burnt field. Adversarial: the pass takes Newton passes (the factor-2 damping at work), not one.
    #[test]
    fn the_twin_solves_the_burnt_field_with_the_maximum_principle_and_a_closed_balance() {
        let p = twin_one_pass(&HOT);
        assert!(p.passes >= 3, "a nonlinear step in {} passes", p.passes);
    }

    /// (a)–(c) on the thin ring. Adversarial: its thin cells (Σ 2e-6) solve their balance as exactly as the disk's.
    #[test]
    fn the_twin_solves_the_hot_disk_in_a_thin_ring_with_the_maximum_principle_and_a_closed_balance() {
        let p = twin_one_pass(&RIM);
        assert!(p.passes >= 2, "{} passes", p.passes);
    }

    /// (e): backward Euler cools no faster than its refinement — heat_hot's heat left after the one pass within [1.0, 2.2]
    /// × the heat left after 64 passes of Δt/64 (computed 1.69 on a scratch model of the scene). Adversarial: the scene
    /// loses at least half its heat in the refined run (it does not barely move), the band is read before the result.
    /// The band is the contract's at the scene's ~580 stages, a_r 4 × 10⁻⁷ (M0-TC's), pinned here: the step reads a_r only
    /// through a_r Δt, so the ratio is the stiffness's — measured on the twin (M0-D55): 1.686 at 580 stages (a_r 4e-7 at Δt
    /// 0.02, and a_r 4e-5 at 2e-4), 2.138 at 1,833 (4e-5, 0.002), 2.657 at 5,797 (the shipped 4e-5 at 0.02), every solve
    /// converged; (a)–(d) run on the shipped a_r.
    #[test]
    fn the_burnt_field_keeps_no_less_heat_than_its_refinement_and_under_a_factor_of_two_point_two_more() {
        let (scene, ctx, st, _) = load(&HOT);
        let physics = scene.physics().unwrap().with(&[("a_r", 4e-7)]).unwrap();
        let heat = Heat::new(&physics, &elements(), flags(&scene)).unwrap();
        let held = |st: &State| -> f64 { ctx.c(st).iter().zip(ctx.temps(st)).map(|(c, t)| c * t).sum() };
        let before = held(&st);
        let (mut one, mut fine) = (st.clone(), st.clone());
        let p = heat.step(&mut one, HOT.dt);
        assert!(p.converged);
        let (mut passes, mut rounds) = (0, 0);
        for _ in 0..64 {
            let q = heat.step(&mut fine, HOT.dt / 64.0);
            assert!(q.converged, "{q:?}");
            (passes, rounds) = (passes.max(q.passes), rounds.max(q.rounds));
        }
        let (left_one, left_fine) = (held(&one) / before, held(&fine) / before);
        let ratio = left_one / left_fine;
        println!("SR-STIFF heat_hot heat left: one pass {left_one:.4}, 64 passes of Δt/64 {left_fine:.4} (up to {passes} Newton passes, {rounds} rounds a pass), ratio {ratio:.3}");
        assert!(left_fine <= 0.5, "the refined run loses at least half the heat: it kept {left_fine}");
        assert!(left_one < 0.9, "the pass moved heat: {left_one}");
        assert!((1.0..=2.2).contains(&ratio), "(e) one pass keeps {ratio} × the refinement's heat");
    }

    /// (a), (b) and (d) on the GPU: one step in one submission from the twin's f32 start, the committed T against the
    /// twin's one pass — C-weighted relative L1 ≤ 1e-4, every cell with Σ ≥ 1e-4 within 1e-3 relative — then (d)'s carried
    /// case (§0.5 [M0-TJ4] (6), M0-T121): the same step from the same start over submissions of 8 rounds, the fewest a
    /// submission carries, bit for bit the one-submission step — the state, the light booked, F at every face, side.flux
    /// and the record — in exactly ⌈slots / 8⌉ submissions (adversarial: a round counted twice, or a partial sum read
    /// before it was written, moves a bit or the count).
    fn gpu_one_pass(case: &Case) {
        let (scene, ctx, st, heat) = load(case);
        let physics = scene.physics().unwrap();
        let gpu = OnGpu::new(&physics, flags(&scene), &st);
        let before = gpu.first.clone();
        let dt = f64::from(case.dt as f32);
        let mut twin = before.clone();
        let p = heat.step(&mut twin, dt);
        let (r, one) = gpu.pass_in(case.dt, Some(SLOTS));
        assert_eq!(one, 1, "every slot recorded: one submission");
        let got = gpu.planes();
        let range = ctx.bounds(&before, &got);
        let lost = energy(&before) - energy(&got);
        let radiated: f64 = gpu.radiated().iter().sum();
        let (tg, tt, c) = (ctx.temps(&got), ctx.temps(&twin), ctx.c(&before));
        let n = before.cells();
        let l1 = (0..n).map(|i| c[i] * (tg[i] - tt[i]).abs()).sum::<f64>() / (0..n).map(|i| c[i] * tt[i]).sum::<f64>();
        let cell = (0..n).filter(|&i| before.planes[i] >= 1e-4).map(|i| (tg[i] - tt[i]).abs() / tt[i]).fold(0.0, f64::max);
        // Where the gap sits: C-weighted L1 by Σ band, and the signed heat the GPU holds over the twin (relative to the twin's).
        let held: f64 = (0..n).map(|i| c[i] * tt[i]).sum();
        let band = |lo: f64, hi: f64| (0..n).filter(|&i| before.planes[i] >= lo && before.planes[i] < hi).map(|i| c[i] * (tg[i] - tt[i]).abs()).sum::<f64>() / held;
        let signed = (0..n).map(|i| c[i] * (tg[i] - tt[i])).sum::<f64>() / held;
        println!("SR-STIFF GPU {} gap by Σ: < 1e-4 {:.2e}, [1e-4, 0.1) {:.2e}, ≥ 0.1 {:.2e}; signed heat over the twin's {signed:.2e}", case.file, band(0.0, 1e-4), band(1e-4, 0.1), band(0.1, f64::MAX));
        // The light each cell emitted, GPU against twin, by the same bands (relative to the twin's radiated).
        let booked = gpu.radiated();
        let light = |lo: f64, hi: f64| (0..n).filter(|&i| before.planes[i] >= lo && before.planes[i] < hi).map(|i| (booked[i] - p.emitted[i]).abs()).sum::<f64>() / p.radiated;
        let light_signed = |lo: f64, hi: f64| (0..n).filter(|&i| before.planes[i] >= lo && before.planes[i] < hi).map(|i| booked[i] - p.emitted[i]).sum::<f64>() / p.radiated;
        println!("SR-STIFF GPU {} radiated gap by Σ (|·| and signed, of the twin's radiated): < 1e-4 {:.2e} {:.2e}, [1e-4, 0.1) {:.2e} {:.2e}, ≥ 0.1 {:.2e} {:.2e}", case.file, light(0.0, 1e-4), light_signed(0.0, 1e-4), light(1e-4, 0.1), light_signed(1e-4, 0.1), light(0.1, f64::MAX), light_signed(0.1, f64::MAX));
        println!(
            "SR-STIFF GPU {}: {r:?}, the twin {} passes, {} rounds; T in [{:.4}, {:.4}]; lost {lost:.9e} of radiated {radiated:.9e} (twin {:.9e}); C-weighted L1 against the twin {l1:.2e}, worst cell with Σ ≥ 1e-4 {cell:.2e}",
            case.file, p.passes, p.rounds, range.0, range.1, p.radiated
        );
        ctx.assert_bounds(range);
        assert!(r.converged && r.passes >= 2 && r.rounds > 0 && r.dt == dt as f32, "{r:?}");
        assert!(radiated > 0.0 && (lost - radiated).abs() <= 1e-5 * radiated, "(b) lost {lost}, radiated {radiated}");
        assert!(l1 <= 1e-4, "(d) C-weighted L1 {l1}");
        assert!(cell <= 1e-3, "(d) worst cell with Σ ≥ 1e-4: {cell}");

        let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<u64>>();
        let face_bits = |(x, y): (Vec<f64>, Vec<f64>)| (bits(&x), bits(&y));
        let want = (bits(&got.planes), bits(&booked), face_bits(gpu.faces()), read(&gpu.gpu, gpu.side.flux()));
        gpu.restart();
        let (carried, submissions) = gpu.pass_in(case.dt, Some(MIN_ROUNDS));
        let slots = 2 + 3 * r.passes + r.rounds;
        let flux_bits = |f: Vec<f32>| f.iter().map(|x| x.to_bits()).collect::<Vec<u32>>();
        println!("SR-STIFF GPU {} carried: {submissions} submissions of {MIN_ROUNDS} rounds for the step's {slots} slots, {carried:?}", case.file);
        assert_eq!(MIN_ROUNDS, 8, "the contract's smallest carry");
        assert_eq!(carried, r, "(d) carried: the record");
        assert_eq!(submissions, slots.div_ceil(8), "(d) carried: one submission per 8 slots the step used");
        assert!(bits(&gpu.planes().planes) == want.0, "(d) carried: the state is not the one-submission step's, bit for bit");
        assert!(bits(&gpu.radiated()) == want.1, "(d) carried: the light booked differs");
        assert!(face_bits(gpu.faces()) == want.2, "(d) carried: F at a face differs");
        assert!(flux_bits(read(&gpu.gpu, gpu.side.flux())) == flux_bits(want.3), "(d) carried: side.flux differs");
    }

    fn gpu_the_burnt_field_is_the_twins_and_books_the_heat_it_lost() {
        gpu_one_pass(&HOT);
    }

    fn gpu_the_hot_disk_in_a_thin_ring_is_the_twins_and_books_the_heat_it_lost() {
        gpu_one_pass(&RIM);
    }

    /// M0-D89's cut (scenes/heat_ceiling.bin, a §2.12.3 dump): P5's start in step 2,517 of the 40 M₀ life (`calibrate --only
    /// m_bh_sb` at 134,041.287, RTX 4090, 2026-10-23: s_2517 through P1–P4 and P8 with P5 pinned), cells x 184–247, y 43–106
    /// of its 600 × 400 world — the thin hot cell (Σ 2.1e-5, T 3.4e5) at (32, 32) beside a vacuum cell — and the step's Δt
    /// (the header's `resume.dt_next`). The shipped constants; every test flag off.
    fn ceiling_cut() -> (State, Physics, f64) {
        let dump = Dump::read(&scenes_dir().join("heat_ceiling.bin")).unwrap();
        let (w, h) = (dump.world.width as usize, dump.world.height as usize);
        let st = State::new(w, h, dump.planes.iter().map(|&v| f64::from(v)).collect());
        (st, Physics::shipped().unwrap(), f64::from(dump.resume.dt_next))
    }

    /// Each cell's heat above its floor as P5 reads its start (§0.5 [M0-TJ4] (1)): C (T⁰ − T_floor), C = Σ/μ, T⁰ from the
    /// EOS's ε_th (floored at T_floor/μ); 0 off the gas (Σ < Σ_vac).
    fn held_above_floor(physics: &Physics, st: &State) -> Vec<f64> {
        let eos = Eos::from_physics(physics).unwrap();
        let elements = elements();
        let sigma_vac = physics.get("sigma_vac").unwrap();
        let n = st.cells();
        let at = |c: usize, i: usize| st.planes[c * n + i];
        (0..n)
            .map(|i| {
                if at(0, i) < sigma_vac {
                    return 0.0;
                }
                let x: [f64; N_SPECIES] = std::array::from_fn(|s| at(4 + s, i));
                let comp = elements.composition(&x);
                let mu = 1.0 / comp.inv_mu;
                let th = eos.thermal(at(3, i), at(0, i), [at(1, i), at(2, i)], comp.y_e, x[N_SPECIES - 1], mu);
                at(0, i) * comp.inv_mu * (Eos::temperature(th.eps_th, mu) - eos.t_floor())
            })
            .collect()
    }

    /// M0-D89's bounds on a step that reached the ceiling, `before` to `after` with `radiated` booked: (a) no gas cell loses
    /// more than its heat above the floor, E⁰ − E ≤ held (1 + `rel`) + `abs` |E⁰|; (b) the grid loses the heat it books,
    /// |ΔE + R| ≤ `rel` Σ held; (c) R ≤ Σ held (1 + `rel`). Returns the worst cell's (index, lost, held).
    fn assert_bounded(physics: &Physics, before: &State, after: &State, radiated: f64, rel: f64, abs: f64) -> (usize, f64, f64) {
        let held = held_above_floor(physics, before);
        let n = before.cells();
        let lost: Vec<f64> = (0..n).map(|i| before.planes[3 * n + i] - after.planes[3 * n + i]).collect();
        let worst = (0..n).max_by(|&a, &b| (lost[a] - held[a]).total_cmp(&(lost[b] - held[b]))).unwrap();
        let all: f64 = held.iter().sum();
        let grid: f64 = lost.iter().sum();
        println!(
            "SR-STIFF ceiling: heat above the floor {all:.6e}, the grid lost {grid:.6e}, radiated {radiated:.6e}; the worst cell ({}, {}) lost {:.6e} of {:.6e} held",
            worst % before.width, worst / before.width, lost[worst], held[worst]
        );
        for i in (0..n).filter(|&i| before.planes[i] >= physics.get("sigma_vac").unwrap()) {
            let e0 = before.planes[3 * n + i];
            assert!(lost[i] <= held[i] * (1.0 + rel) + abs * e0.abs(), "(a) cell ({}, {}) lost {} of {} held", i % before.width, i / before.width, lost[i], held[i]);
        }
        assert!((grid - radiated).abs() <= rel * all, "(b) the grid lost {grid}, radiated {radiated}");
        assert!(radiated <= all * (1.0 + rel), "(c) radiated {radiated} over the heat above the floor {all}");
        (worst, lost[worst], held[worst])
    }

    /// M0-D89 on the twin: the cut's step reaches the ceiling (11 passes, 4,096 rounds; the case is moot otherwise), and its
    /// commit by limited fluxes (§0.5 [M0-D89]) holds the bounds to 1e-9 with δ = 0 — before it, the last iterate's T and (5)'s
    /// closure gave δ −1.7e-2.
    #[test]
    fn the_twin_at_the_ceiling_moves_no_more_heat_than_its_cells_hold() {
        let (before, physics, dt) = ceiling_cut();
        let heat = Heat::new(&physics, &elements(), TestFlags::default()).unwrap();
        let mut after = before.clone();
        let p = heat.step(&mut after, dt);
        println!("SR-STIFF twin heat_ceiling.bin: {} passes, {} rounds, converged {}, delta {:e}, radiated {:e}", p.passes, p.rounds, p.converged, p.delta, p.radiated);
        assert!(!p.converged, "the cut's step must reach the ceiling: {} passes, {} rounds", p.passes, p.rounds);
        assert_eq!(p.delta, 0.0, "past the ceiling the fluxes are booked on both sides: no closure");
        assert_bounded(&physics, &before, &after, p.radiated, 1e-9, 1e-12);
    }

    /// M0-D89 on the GPU: the cut's step reaches the ceiling (1 pass, 4,096 rounds on the RTX 4090 on either preconditioner: in
    /// f32 the thin hot cell's residual never meets (4)'s u-measure), and the commit holds the bounds to f32's 1e-5 with δ 0 —
    /// where the last iterate with (5)'s closure booked 4.2e16 radiated against ~1e5 of heat above the floor, δ −2.9e11. P8
    /// runs after P5 here and would floor a cell that gave more than it held, so its `floor_added` is held to the same 1e-5 of
    /// the heat above the floor: (a) on the state P5 left.
    fn gpu_a_step_at_the_ceiling_moves_no_more_heat_than_its_cells_hold() {
        let (st, physics, dt) = ceiling_cut();
        let gpu = OnGpu::new(&physics, TestFlags::default(), &st);
        let before = gpu.first.clone();
        let r = gpu.pass(dt);
        let after = gpu.planes();
        let radiated: f64 = gpu.radiated().iter().sum();
        let floored = book_totals(gpu.state.world(), &gpu.booking.readback(&gpu.gpu.device, &gpu.gpu.queue))
            [TERMS.iter().position(|t| t.name == "energy.floor_added").unwrap()];
        println!("SR-STIFF GPU heat_ceiling.bin: {r:?}, radiated booked {radiated:e}, P8's floor_added {floored:e}");
        assert!(!r.converged, "the cut's step must reach the ceiling: {r:?}");
        assert_eq!(r.delta, 0.0, "past the ceiling the fluxes are booked on both sides: no closure");
        let all: f64 = held_above_floor(&physics, &before).iter().sum();
        assert!(floored.abs() <= 1e-5 * all, "(a) P8 floored {floored} after P5: a cell gave more than it held");
        assert_bounded(&physics, &before, &after, radiated, 1e-5, 2.0 * f64::from(f32::EPSILON));
    }
}

/// M0-D82 — P5's multigrid preconditioner (§0.5 [M0-D82]; reports/heat_multigrid.md): the V-cycle's levels built from G-STIFF's
/// two scenes' first Newton pass on the physics adapter (`Step::heat_first_system`), M⁻¹ applied there in f32
/// (`Step::apply_heat_precond`). It is symmetric, |⟨Mx, y⟩ − ⟨x, My⟩| ≤ 1e-7 ‖Mx‖ ‖y‖ over the pairs of 8 random vectors
/// (normwise: a nearly orthogonal pair's ⟨Mx, y⟩ is no scale — the lead, M0-D82, 2026-10-22: «Normwise ≤ 1e-7
/// (Recommended)»; measured ≤ 4.8e-9, and ≥ 4.4e-6 with the sweeps going up red first, not the adjoint), and positive,
/// ⟨Mx, x⟩ > 0; as the preconditioner of that pass's CG — run here in f64 from the pass's start to §0.5
/// [M0-TJ4] (4)'s two measures — it converges in ≤ 2 × the f64 model's rounds (M6: heat_hot 14, heat_rim 17 in pass 1).
/// Jacobi's rounds on the same pass are printed beside them.
mod multigrid {
    use super::stiff::{load, Case, Ctx, HOT, RIM};
    use super::*;
    use sr_engine::step::heat::{HeatPrecond, HeatSystem};

    /// The model's first-pass rounds, the tile-smoother V(1,1) (reports/heat_multigrid.md M6).
    const MODEL_HOT: usize = 14;
    const MODEL_RIM: usize = 17;

    /// The case's first step on the GPU and its first pass's system, with the case's constants.
    fn first_pass(case: &Case) -> (OnGpu, HeatSystem, Ctx) {
        let (scene, ctx, st, _) = load(case);
        let gpu = OnGpu::new(&scene.physics().unwrap(), flags(&scene), &st);
        gpu.pass(case.dt);
        let s = gpu.step.heat_first_system(&gpu.gpu.device, &gpu.gpu.queue).unwrap();
        println!("SR-MG {}: box {} × {} at ({}, {}), {} levels, mg_coarse from level {}", case.file, s.w, s.h, s.x0, s.y0, s.levels, s.coarse);
        (gpu, s, ctx)
    }

    /// The four faces (D_f, the neighbour) of box cell k; a face of D_f 0 has no gas cell beyond it.
    fn faces(s: &HeatSystem, k: usize) -> [(f64, usize); 4] {
        let (i, j, w) = (k % s.w, k / s.w, s.w);
        let d = [s.fx[j * (w + 1) + i], s.fx[j * (w + 1) + i + 1], s.fy[j * w + i], s.fy[(j + 1) * w + i]].map(f64::from);
        [(d[0], k.wrapping_sub(1)), (d[1], k + 1), (d[2], k.wrapping_sub(w)), (d[3], k + w)]
    }

    /// A v over the box, face by face, in f64.
    fn apply(s: &HeatSystem, v: &[f64]) -> Vec<f64> {
        let at = |k: usize| faces(s, k).iter().filter(|f| f.0 != 0.0).fold(f64::from(s.own[k]) * v[k], |a, &(d, o)| a + d * (v[k] - v[o]));
        (0..v.len()).map(|k| if s.own[k] < 0.0 { 0.0 } else { at(k) }).collect()
    }

    fn dot(a: &[f64], b: &[f64]) -> f64 {
        a.iter().zip(b).map(|(x, y)| x * y).sum()
    }

    /// M⁻¹ r on the GPU, by the preconditioner chosen.
    fn precond(gpu: &OnGpu, r: &[f64]) -> Vec<f64> {
        let r: Vec<f32> = r.iter().map(|&v| v as f32).collect();
        gpu.step.apply_heat_precond(&gpu.gpu.device, &gpu.gpu.queue, &r).iter().map(|&v| f64::from(v)).collect()
    }

    fn symmetric_and_positive(case: &Case) {
        let (gpu, s, _) = first_pass(case);
        gpu.step.set_heat_precond(HeatPrecond::Multigrid);
        let mut seed = 0x9e37_79b9_7f4a_7c15u64;
        let mut random = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            (seed >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
        };
        let xs: Vec<Vec<f64>> = (0..8).map(|_| s.own.iter().map(|&o| if o >= 0.0 { random() } else { 0.0 }).collect()).collect();
        let ms: Vec<Vec<f64>> = xs.iter().map(|x| precond(&gpu, x)).collect();
        let mut worst: f64 = 0.0;
        for i in 0..8 {
            for j in i + 1..8 {
                let (a, b) = (dot(&ms[i], &xs[j]), dot(&xs[i], &ms[j]));
                worst = worst.max((a - b).abs() / (dot(&ms[i], &ms[i]) * dot(&xs[j], &xs[j])).sqrt());
            }
        }
        let least = (0..8).map(|i| dot(&ms[i], &xs[i])).fold(f64::MAX, f64::min);
        println!("SR-MG {}: the worst |<Mx, y> - <x, My>| / (|Mx| |y|) over 28 pairs {worst:.2e}; the least <Mx, x> {least:.3e}", case.file);
        assert!(worst <= 1e-7, "M⁻¹ is not symmetric: {worst:.2e}");
        assert!(least > 0.0, "M⁻¹ is not positive: {least:e}");
    }

    /// The first pass's CG from its start under the preconditioner chosen, until both of (4)'s measures hold (§0.5 [M0-TJ4]):
    /// x = u⁰, b − A x = −Σ_vac D_f (u_i − u_vac) − Σ_f D_f (u_i − u_f); its rounds (none past the ceiling's 4,096).
    fn rounds(case: &Case, gpu: &OnGpu, s: &HeatSystem, ctx: &Ctx, precond_by: HeatPrecond) -> usize {
        gpu.step.set_heat_precond(precond_by);
        let (t_all, c_all) = (ctx.temps(&gpu.first), ctx.c(&gpu.first));
        let world = |k: usize| (s.y0 + k / s.w) * gpu.first.width + s.x0 + k % s.w;
        let gas: Vec<bool> = s.own.iter().map(|&o| o >= 0.0).collect();
        let n = s.own.len();
        let t: Vec<f64> = (0..n).map(|k| t_all[world(k)]).collect();
        let heat: Vec<f64> = (0..n).map(|k| if gas[k] { c_all[world(k)] / f64::from(case.dt as f32) } else { 0.0 }).collect();
        let slope: Vec<f64> = t.iter().map(|&t| 1.0 / (4.0 * ctx.a_r * t.powi(3))).collect();
        let u: Vec<f64> = (0..n).map(|k| if gas[k] { ctx.a_r * t[k].powi(4) } else { 0.0 }).collect();
        let diag: Vec<f64> = (0..n).map(|k| f64::from(s.own[k]) + faces(s, k).iter().map(|f| f.0).sum::<f64>()).collect();
        let u_vac = ctx.a_r * ctx.t_floor.powi(4);
        let au = apply(s, &u);
        let mut r: Vec<f64> =
            (0..n).map(|k| if gas[k] { -f64::from(s.vac[k]) * (u[k] - u_vac) - (au[k] - f64::from(s.own[k]) * u[k]) } else { 0.0 }).collect();
        let tol2 = 1e-10;
        let u_bound = tol2 * (0..n).filter(|&k| gas[k]).map(|k| heat[k] * slope[k] * u[k] * u[k]).sum::<f64>();
        let t_bound = tol2 * (0..n).filter(|&k| gas[k]).map(|k| heat[k] * t[k] * t[k]).sum::<f64>();
        let held = |r: &[f64]| {
            let gamma: f64 = (0..n).filter(|&k| gas[k]).map(|k| r[k] * r[k] / diag[k]).sum();
            let e: f64 = (0..n).filter(|&k| gas[k]).map(|k| heat[k] * (r[k] * slope[k] / diag[k]).powi(2)).sum();
            gamma <= u_bound && e <= t_bound
        };
        let mut z = precond(gpu, &r);
        let (mut p, mut rz) = (z.clone(), dot(&r, &z));
        let mut k = 0;
        while !held(&r) && k < 4096 {
            let q = apply(s, &p);
            let alpha = rz / dot(&p, &q);
            r.iter_mut().zip(&q).for_each(|(r, q)| *r -= alpha * q);
            z = precond(gpu, &r);
            let rz1 = dot(&r, &z);
            p = z.iter().zip(&p).map(|(z, p)| z + rz1 / rz * p).collect();
            rz = rz1;
            k += 1;
        }
        k
    }

    fn converges_within_twice_the_model(case: &Case, model: usize) {
        let (gpu, s, ctx) = first_pass(case);
        let jacobi = rounds(case, &gpu, &s, &ctx, HeatPrecond::Jacobi);
        let multigrid = rounds(case, &gpu, &s, &ctx, HeatPrecond::Multigrid);
        println!("SR-MG {}: the first pass converges in {multigrid} V-cycle rounds (the model {model}, the bound {}), {jacobi} Jacobi rounds", case.file, 2 * model);
        assert!(multigrid <= 2 * model, "{multigrid} V-cycle rounds, over 2 × the model's {model}");
    }

    #[test]
    fn gpu_the_v_cycle_is_symmetric_and_positive_on_the_burnt_field() {
        symmetric_and_positive(&HOT);
    }

    #[test]
    fn gpu_the_v_cycle_is_symmetric_and_positive_on_the_hot_disk_in_a_thin_ring() {
        symmetric_and_positive(&RIM);
    }

    #[test]
    fn gpu_the_burnt_fields_first_pass_converges_within_twice_the_models_rounds() {
        converges_within_twice_the_model(&HOT, MODEL_HOT);
    }

    #[test]
    fn gpu_the_hot_disks_first_pass_converges_within_twice_the_models_rounds() {
        converges_within_twice_the_model(&RIM, MODEL_RIM);
    }
}
