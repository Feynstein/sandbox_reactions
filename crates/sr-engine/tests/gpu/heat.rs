//! M0-T41 — heat and light on the CPU twin (contract §2.4, §1.7, §5.1's G-HEAT1, G-HEAT2 and G-FLD, R8, R13): P5's f64
//! reference, `sr_physics::reference::heat`, one-temperature flux-limited diffusion stepped by RKL2; the twin's cases touch
//! no adapter. M0-T42 — the GPU's P5 (`step/heat.rs`, shaders/heat.wgsl), cases `gpu_…`, the same oracles on the physics
//! adapter (R12), P5 alone between P1, P8 and P9 ([`OnGpu`]).
//!
//! Three scopes, one module each. `gauss` — G-HEAT1: a Gaussian heat pulse in a uniform medium whose diffusivity is the
//! scene's `test.constant_diffusivity` (scenes/heat_pulse.json), the variance per axis σ²(t) = σ₀² + 2χt within 1 % (SM-O7,
//! SM-E21) and the total heat constant to 1e-5; the pass's F at a face against −χ·C·ΔT; the light leaving (a hot slab beside
//! vacuum and at the world's edge: what the pass books `radiated` equals the heat the grid lost, to 1e-6, and sits in the
//! cells that emitted it); the scene's `pin_temperature`. `rkl2` — G-HEAT2: the same pulse in one RKL2 pass against explicit
//! sub-steps written here (σ² within 1 %, SM-O8, SM-E18), the stage count against the contract's formula worked by hand,
//! `SR-WARN heat stages` above 32, and a one-cell pulse in many passes at a step just over the stability limit of one
//! stage fewer (the adversarial case: a smooth pulse has no content at the grid's highest wavenumbers, so a missing stage
//! leaves its σ² alone and only a sharp one shows it). `limiter` — G-FLD: thin gas (Σ = 10⁻⁴, scenes/heat_spot.json) around
//! a hot spot, |F| ≤ c_sb a_r T⁴ (1 + 10⁻⁶) at every face (SM-E19); thick gas, where the limiter must not act, carries the
//! radiative conductivity (4/3) a_r c_sb T³/(κΣ) (SM-E20); the scene's `flux_limiter` off gives λ ≡ 1/3.
//!
//! The oracles are written here from the contract, never read from the scheme: temperatures from E with the cold pressure
//! off and the gas at rest and pure hydrogen (μ = 1/2), the explicit stepping, the moments, the closed forms.
//!
//! The GPU's f32 tolerances, fixed before its first run: the contract's own where a row names one (σ² 1 %, total heat 1e-5,
//! |F| ≤ c_sb a_r T⁴ (1 + 1e-6)); against the twin from the same f32 start, T within 1e-4 of the pulse's peak and F within
//! 1e-4 of the largest face flux; the stage count and its warning exact, Δt_p within 1e-6; `radiated` against the heat lost
//! within 1e-5; the one-cell pulse's heat within 1e-4 (two runs' difference, f32 at T = 2). Above 32 stages a cell's faces
//! are capped (contract §0.5 [M0-T42]): the 33-stage step runs 32 over the full Δt.

use std::path::{Path, PathBuf};

use sr_engine::gpu::Gpu;
use sr_engine::scene::{Scene, TestSettings};
use sr_engine::state::{Booking, SideFields, State as GpuState, WorldConfig, BOOK_GROUPS, TERMS};
use sr_engine::step::boxfit::BoxMode;
use sr_engine::step::heat::{HeatReport, HeatSetup};
use sr_engine::step::{EosGpu, Passes, Step};
use sr_physics::reference::heat::{lambda, stages_for, Heat, Pass, TestFlags, MAX_STAGES};
use sr_physics::reference::hydro::State;
use sr_physics::registry::{Elements, Physics};

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
/// compares RKL2 with. Stable for χ dt ≤ 1/4, but at 1/4 exactly a one-cell pulse keeps its checkerboard component for ever
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
        let planes: Vec<f32> = self.first.planes.iter().map(|&v| v as f32).collect();
        self.state.upload(&self.gpu.queue, &planes).unwrap();
        let mut encoder = self.gpu.device.create_command_encoder(&Default::default());
        self.booking.clear(&mut encoder);
        self.gpu.queue.submit([encoder.finish()]);
    }

    /// One step whose P5 runs over `dt`; its record.
    fn pass(&self, dt: f64) -> HeatReport {
        self.gpu.queue.write_buffer(self.step.dt_record(), 4, &(dt as f32).to_le_bytes());
        self.step.run(&self.gpu.device, &self.gpu.queue, 1);
        assert_eq!(self.step.read_dt(&self.gpu.device, &self.gpu.queue).dt, dt as f32, "P1 took the Δt written");
        self.step.read_heat(&self.gpu.device, &self.gpu.queue).expect("the step runs P5")
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

/// G-HEAT1: σ²(t) = σ₀² + 2χt per axis within 1 %, total heat constant to 1e-5, F at a face.
mod gauss {
    use super::*;

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

    /// σ_x² and σ_y² at t = 8, 16, 24 against σ₀² + 2χt: 1 passes of Δt = 1 (three stages each).
    #[test]
    fn the_variance_grows_as_sigma0_squared_plus_two_chi_t_per_axis() {
        let (mut st, heat, bg) = gaussian(&scene("heat_pulse.json"), SIGMA0, PEAK);
        let (_, vx0, vy0) = moments(&st, bg);
        assert!((vx0 - SIGMA0 * SIGMA0).abs() < 1e-6 && (vy0 - SIGMA0 * SIGMA0).abs() < 1e-6, "{vx0} {vy0}");
        let mut worst: f64 = 0.0;
        for pass in 1..=24 {
            let p = heat.step(&mut st, 1.0);
            assert_eq!(p.stages, 3, "dt_p {} (0.5 for χ = 1/2)", p.dt_p);
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

    /// F is −χ·C·ΔT at a face (C = Σ/μ = 2), positive toward +x, +y; the cell field is the mean of a cell's two faces.
    #[test]
    fn the_flux_kept_is_chi_c_times_the_temperature_difference() {
        let (mut st, heat, _) = gaussian(&scene("heat_pulse.json"), SIGMA0, PEAK);
        let t = temps(&st);
        let (w, h) = (st.width, st.height);
        let p = heat.step(&mut st, 1.0);
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
        let mut max_stages = 0;
        for _ in 0..12 {
            let p = heat.step(&mut st, 2.0);
            radiated += p.radiated;
            for (e, q) in emitted.iter_mut().zip(&p.emitted) {
                *e += q;
            }
            max_stages = max_stages.max(p.stages);
        }
        let lost = e_before - energy(&st);
        println!("SR-HEAT-RAD slab {slab_energy:.4}: lost {lost:.9}, radiated {radiated:.9}, stages up to {max_stages}");
        assert!(radiated / slab_energy > 0.01, "the slab must lose something real: {}", radiated / slab_energy);
        assert!(max_stages >= 3, "{max_stages}");
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

    /// `test.pin_temperature` (§2.12.1): the pass leaves the state alone, books nothing, and still keeps F.
    #[test]
    fn pinned_temperature_leaves_the_state_and_keeps_the_flux() {
        let (mut st, _, _) = gaussian(&scene("heat_pulse.json"), SIGMA0, PEAK);
        let (_, physics, _) = world(&scene("heat_pulse.json"));
        let pinned = Heat::new(&physics, &elements(), TestFlags { pin_temperature: true, constant_diffusivity: Some(CHI), flux_limiter: true }).unwrap();
        let free = Heat::new(&physics, &elements(), TestFlags { pin_temperature: false, constant_diffusivity: Some(CHI), flux_limiter: true }).unwrap();
        let before = st.clone();
        let p = pinned.step(&mut st, 1.0);
        assert_eq!(st, before, "the planes are bit for bit what they were");
        assert_eq!((p.stages, p.radiated), (0, 0.0));
        let q = free.step(&mut st.clone(), 1.0);
        assert_eq!(p.flux, q.flux, "F is the same pass's");
        assert!(p.flux.x.iter().any(|&f| f != 0.0));
        let mut moved = before.clone();
        free.step(&mut moved, 1.0);
        assert_ne!(moved, before, "the free pass moves heat");
    }

    /// G-HEAT1 on the GPU: 24 passes of Δt = 1 (3 stages each), σ² at t = 8, 16, 24 within 1 %, the heat to 1e-5, and the
    /// field the twin's within 1e-4 of the peak.
    #[test]
    fn gpu_the_variance_grows_as_sigma0_squared_plus_two_chi_t_and_the_heat_is_kept() {
        let scene = scene("heat_pulse.json");
        let (st, heat, bg) = gaussian(&scene, SIGMA0, PEAK);
        let gpu = OnGpu::new(&scene.physics().unwrap(), flags(&scene), &st);
        let mut twin = gpu.first.clone();
        let (h0, _, _) = moments(&gpu.first, bg);
        let mut worst: f64 = 0.0;
        for pass in 1..=24 {
            let r = gpu.pass(1.0);
            assert_eq!((r.stages, r.need), (3, 3), "{r:?}");
            assert_eq!(heat.step(&mut twin, 1.0).stages, 3);
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
        assert!(gpu.radiated().iter().all(|&r| r.abs() < 1e-9), "nothing reaches the edges");
        println!("SR-HEAT1 GPU worst sigma2 error {worst:.2e}");
    }

    /// F kept on the GPU: every face the twin's within 1e-4 of the largest, −χ·C·ΔT signs; side.flux the mean of a cell's two
    /// faces (the field step n + 1's P4 reads).
    #[test]
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

    /// The hot slab on the GPU: `radiated` equals the heat the grid lost within 1e-5 and the twin's within 1e-4, booked
    /// exactly on the slab's ring.
    #[test]
    fn gpu_a_hot_slab_books_radiated_equal_to_the_heat_the_grid_lost() {
        let scene = rect_world(64, 64, [0, 16, 32, 32], 1.0, 30.0);
        let (st, physics, heat) = world(&scene);
        let gpu = OnGpu::new(&physics, flags(&scene), &st);
        let mut twin = gpu.first.clone();
        let mut twin_radiated = 0.0;
        for _ in 0..12 {
            let r = gpu.pass(2.0);
            let p = heat.step(&mut twin, 2.0);
            assert_eq!(r.stages as usize, p.stages, "{r:?}");
            twin_radiated += p.radiated;
        }
        let emitted = gpu.radiated();
        let radiated: f64 = emitted.iter().sum();
        let lost = energy(&gpu.first) - energy(&gpu.planes());
        println!("SR-HEAT-RAD GPU: lost {lost:.6}, radiated {radiated:.6}, twin {twin_radiated:.6}");
        assert!((lost - radiated).abs() <= 1e-5 * radiated, "lost {lost}, radiated {radiated}");
        assert!((radiated - twin_radiated).abs() <= 1e-4 * twin_radiated, "{radiated} vs the twin's {twin_radiated}");
        for (i, &e) in emitted.iter().enumerate() {
            let (x, y) = (i % 64, i / 64);
            let ring = x < 32 && (16..48).contains(&y) && (x == 0 || x == 31 || y == 16 || y == 47);
            assert!(if ring { e > 0.0 } else { e == 0.0 }, "cell ({x}, {y}) emitted {e}");
        }
    }

    /// `test.pin_temperature` on the GPU: no stage runs, nothing is booked, the planes are the start's bit for bit, F is the
    /// free pass's.
    #[test]
    fn gpu_pinned_temperature_leaves_the_state_and_keeps_the_flux() {
        let scene = scene("heat_pulse.json");
        let (st, _, _) = gaussian(&scene, SIGMA0, PEAK);
        let physics = scene.physics().unwrap();
        let pinned = OnGpu::new(&physics, TestFlags { pin_temperature: true, ..flags(&scene) }, &st);
        let r = pinned.pass(1.0);
        assert_eq!((r.stages, r.need, r.dt_p), (0, 0, Some(0.5)), "{r:?}");
        let after = pinned.planes();
        assert!(after.planes.iter().zip(&pinned.first.planes).all(|(a, b)| a.to_bits() == b.to_bits()), "the planes moved");
        assert!(pinned.radiated().iter().all(|&e| e == 0.0));
        let free = OnGpu::new(&physics, flags(&scene), &st);
        assert_eq!(free.pass(1.0).stages, 3);
        assert_eq!(pinned.faces(), free.faces(), "F is the same pass's");
    }
}

/// G-HEAT2: RKL2 against explicit sub-steps; the stage count; the warning above 32.
mod rkl2 {
    use super::*;

    /// One RKL2 pass over t = 24 (14 stages) against 96 explicit sub-steps of Δt_p/2 = 0.25, the pulse of G-HEAT1: σ² of each
    /// within 1 % of the other and of σ₀² + 2χt.
    #[test]
    fn one_rkl2_pass_matches_explicit_sub_steps_in_sigma_squared() {
        let (mut st, heat, bg) = gaussian(&scene("heat_pulse.json"), SIGMA0, PEAK);
        let t0 = temps(&st);
        let (w, h) = (st.width, st.height);
        let explicit_t = explicit(&t0, w, h, CHI, 0.25, 96);
        let (_, ex, ey) = stats(&explicit_t, w, bg, usize::MAX);
        let p = heat.step(&mut st, 24.0);
        let (total, vx, vy) = moments(&st, bg);
        let want = SIGMA0 * SIGMA0 + 2.0 * CHI * 24.0;
        println!("SR-HEAT2 {} stages for dt/dt_p = {}: RKL2 sigma2 x {vx:.6} y {vy:.6}, explicit x {ex:.6} y {ey:.6}, expected {want}", p.stages, 24.0 / p.dt_p);
        assert_eq!(p.stages, 14);
        assert!((vx - ex).abs() / ex <= 0.01 && (vy - ey).abs() / ey <= 0.01, "RKL2 {vx} {vy}, explicit {ex} {ey}");
        assert!((vx - want).abs() / want <= 0.01 && (vy - want).abs() / want <= 0.01, "{vx} {vy} vs {want}");
        assert!(total > 100.0);
    }

    /// The fields, not only their second moments: six RKL2 passes of Δt = 4 (6 stages each) against 96 explicit sub-steps, the
    /// same t = 24 — second order in the step against the five-point Euler's first, each within 1 % of the pulse's peak.
    #[test]
    fn six_rkl2_passes_match_the_explicit_field() {
        let (mut st, heat, bg) = gaussian(&scene("heat_pulse.json"), SIGMA0, PEAK);
        let t0 = temps(&st);
        let (w, h) = (st.width, st.height);
        let explicit_t = explicit(&t0, w, h, CHI, 0.25, 96);
        for _ in 0..6 {
            let p = heat.step(&mut st, 4.0);
            assert_eq!(p.stages, 6, "dt/dt_p = 8");
        }
        let t1 = temps(&st);
        let diff = t1.iter().zip(&explicit_t).map(|(a, b)| (a - b).abs()).fold(0.0, f64::max);
        let analytic = |i: usize| {
            let (dx, dy) = ((i % w) as f64 - 64.0, (i / w) as f64 - 64.0);
            PEAK * SIGMA0 * SIGMA0 / 40.0 * (-(dx * dx + dy * dy) / (2.0 * 40.0)).exp()
        };
        let exact = t1.iter().enumerate().map(|(i, a)| (a - bg - analytic(i)).abs()).fold(0.0, f64::max);
        println!("SR-HEAT2 six passes: largest |T_rkl2 - T_explicit| {diff:.3e}, |T_rkl2 - analytic| {exact:.3e}, peak {:.4}", t1[64 * w + 64] - bg);
        assert!(diff <= 0.01 * PEAK && exact <= 0.01 * PEAK, "{diff} {exact}");
        assert!(t1.iter().cloned().fold(f64::MAX, f64::min) - bg >= -1e-12, "a smooth pulse does not undershoot");
    }

    /// s from the contract's formula, worked by hand here: Δt_p = 0.5/(χ + χ) = 0.5 and the smallest s ≥ 2 with
    /// Δt_p (s² + s − 2)/4 ≥ Δt.
    #[test]
    fn the_stage_count_is_the_smallest_that_covers_dt() {
        let scene = scene("heat_pulse.json");
        let by_hand = |dt: f64| -> usize {
            let dt_p = 0.5 / (CHI + CHI);
            let mut s = 2usize;
            while dt_p * ((s * s + s) as f64 - 2.0) / 4.0 < dt {
                s += 1;
            }
            s
        };
        for dt in [0.05, 0.4999, 0.5, 0.5001, 1.0, 3.0, 24.0, 100.0] {
            let (mut st, heat, _) = gaussian(&scene, SIGMA0, PEAK);
            let p = heat.step(&mut st, dt);
            let want = by_hand(dt);
            assert!((p.dt_p - 0.5).abs() < 1e-12, "dt_p {}", p.dt_p);
            assert_eq!(p.stages, want, "dt {dt}");
            let dt_h = |s: usize| 0.5 * ((s * s + s) as f64 - 2.0) / 4.0;
            assert!(dt_h(p.stages) >= dt && (p.stages == 2 || dt_h(p.stages - 1) < dt), "dt {dt}: s {}", p.stages);
            assert_eq!(p.stages, stages_for(p.dt_p, dt));
        }
        // And the pass takes it all the way: the variance of the largest step is the analytic one.
        let (mut st, heat, bg) = gaussian(&scene, SIGMA0, PEAK);
        heat.step(&mut st, 100.0);
        let (_, vx, _) = moments(&st, bg);
        let want = SIGMA0 * SIGMA0 + 2.0 * CHI * 100.0;
        assert!((vx - want).abs() / want <= 0.01, "{vx} {want}");
    }

    #[test]
    fn more_than_32_stages_prints_the_warning_and_32_does_not() {
        let scene = scene("heat_pulse.json");
        // s = 32 covers Δt_p (1024 + 32 − 2)/4 = 263.5 Δt_p = 131.75; the next step needs 33.
        let (mut st, heat, _) = gaussian(&scene, SIGMA0, PEAK);
        let p = heat.step(&mut st, 131.75);
        assert_eq!((p.stages, MAX_STAGES), (32, 32));
        assert_eq!(p.warning(), None);
        let (mut st, heat, bg) = gaussian(&scene, SIGMA0, PEAK);
        let p = heat.step(&mut st, 131.76);
        assert_eq!(p.stages, 33);
        assert_eq!(p.warning().as_deref(), Some("SR-WARN heat stages 33"));
        // The pass still runs, and keeps its heat: a warning is not a switch to anything else.
        let (total, _, _) = moments(&st, bg);
        let (t0, _, _) = moments(&gaussian(&scene, SIGMA0, PEAK).0, bg);
        assert!((total - t0).abs() / t0 <= 1e-9, "{total} {t0}");
    }

    /// The adversarial case: a one-cell pulse (every wavenumber present) in 15 passes of Δt = 6.4 Δt_p — s = 5, the step just
    /// over what 4 stages cover (4.5 Δt_p) — stays bounded and keeps σ² equal to an explicit run's.
    ///
    /// The pulse is the difference of two runs on a background of 2.0, with the pulse and without: the scheme is linear
    /// here (constant χ), so the difference is the pulse's own response with the edges' cooling of the background taken out.
    /// The background also keeps the response's ringing clear of T_floor: RKL2 undershoots a one-cell pulse by 2–4 % of it
    /// in the first passes (measured; the ring fades by the 15th), and a pass reads a cell below T_floor as T_floor — P8
    /// books that floor, P5 does not — which would drain the pulse's second moment. σ² is read over the whole world: the
    /// ripples are small but far, and carry second moment.
    #[test]
    fn a_sharp_pulse_stays_stable_at_a_step_just_over_one_stage_fewer() {
        let scene = scene("heat_pulse.json");
        let (mut with, heat, bg) = gaussian_on(&scene, Some(2.0), 0.01, PEAK);
        let (mut without, _, _) = gaussian_on(&scene, Some(2.0), 0.01, 0.0);
        let (t0, w, h) = (temps(&with), with.width, with.height);
        let mut stages = 0;
        let mut ring: f64 = 0.0;
        for _ in 0..15 {
            stages = heat.step(&mut with, 3.2).stages;
            assert_eq!(heat.step(&mut without, 3.2).stages, 5, "dt/dt_p = 6.4");
            let low = temps(&with).iter().zip(temps(&without)).map(|(a, b)| a - b).fold(f64::MAX, f64::min);
            ring = ring.min(low);
        }
        assert_eq!(stages, 5);
        let response: Vec<f64> = temps(&with).iter().zip(temps(&without)).map(|(a, b)| a - b).collect();
        let pulse: Vec<f64> = t0.iter().map(|t| t - bg).collect();
        let explicit_t = explicit(&pulse, w, h, CHI, 0.2, 240);
        let (_, ex, ey) = stats(&explicit_t, w, 0.0, usize::MAX);
        let (total, vx, vy) = stats(&response, w, 0.0, usize::MAX);
        let (lowest, highest) = (response.iter().cloned().fold(f64::MAX, f64::min), response.iter().cloned().fold(f64::MIN, f64::max));
        println!("SR-HEAT2 sharp pulse, {stages} stages x 15: sigma2 x {vx:.5} y {vy:.5}, explicit x {ex:.5} y {ey:.5}, response in [{lowest:.2e}, {highest:.4e}], deepest ring {ring:.3e}, heat {total:.9}");
        assert!((vx - ex).abs() / ex <= 0.01 && (vy - ey).abs() / ey <= 0.01, "{vx} {vy} vs {ex} {ey}");
        assert!((vx - 48.0).abs() / 48.0 <= 0.01, "{vx} vs 2 chi t = 48");
        assert!((total - PEAK).abs() / PEAK <= 1e-6, "heat {total}");
        // Bounded: the ringing is a few percent of the one-cell pulse at most (never growing), the response ends smooth.
        assert!(ring >= -0.05 * PEAK && lowest >= -1e-9 && highest <= 0.01, "ring {ring}, [{lowest}, {highest}]");
    }

    /// G-HEAT2 on the GPU: one pass over t = 24 (14 stages) against 96 explicit sub-steps, σ² within 1 % of them and of
    /// σ₀² + 2χt.
    #[test]
    fn gpu_one_rkl2_pass_matches_explicit_sub_steps_in_sigma_squared() {
        let scene = scene("heat_pulse.json");
        let (st, _, bg) = gaussian(&scene, SIGMA0, PEAK);
        let gpu = OnGpu::new(&scene.physics().unwrap(), flags(&scene), &st);
        let explicit_t = explicit(&temps(&gpu.first), st.width, st.height, CHI, 0.25, 96);
        let (_, ex, ey) = stats(&explicit_t, st.width, bg, usize::MAX);
        let r = gpu.pass(24.0);
        let (_, vx, vy) = moments(&gpu.planes(), bg);
        let want = SIGMA0 * SIGMA0 + 2.0 * CHI * 24.0;
        println!("SR-HEAT2 GPU {r:?}: sigma2 x {vx:.6} y {vy:.6}, explicit x {ex:.6} y {ey:.6}, expected {want}");
        assert_eq!(r.stages, 14);
        assert!((vx - ex).abs() / ex <= 0.01 && (vy - ey).abs() / ey <= 0.01, "GPU {vx} {vy}, explicit {ex} {ey}");
        assert!((vx - want).abs() / want <= 0.01 && (vy - want).abs() / want <= 0.01, "{vx} {vy} vs {want}");
    }

    /// The GPU's s and Δt_p are the twin's for steps from 0.05 to 131.75 (s = 32); at 131.76 the step needs 33: every cell's
    /// faces are capped (contract §0.5 [M0-T42]) and 32 stages run over the whole Δt, the warning says 33, the heat is kept.
    #[test]
    fn gpu_the_stage_count_is_the_twins_and_a_33_stage_step_is_capped_to_32() {
        let scene = scene("heat_pulse.json");
        let (st, heat, bg) = gaussian(&scene, SIGMA0, PEAK);
        let gpu = OnGpu::new(&scene.physics().unwrap(), flags(&scene), &st);
        for dt in [0.05, 0.4999, 0.5, 0.5001, 1.0, 3.0, 24.0, 100.0, 131.75] {
            gpu.restart();
            let r = gpu.pass(dt);
            let p = heat.step(&mut gpu.first.clone(), f64::from(dt as f32));
            let dt_p = f64::from(r.dt_p.unwrap());
            assert_eq!((r.stages as usize, r.need as usize), (p.stages, p.stages), "dt {dt}: {r:?}");
            assert!((dt_p - p.dt_p).abs() <= 1e-6 * p.dt_p && r.dt == dt as f32, "dt {dt}: {r:?}");
            assert_eq!(r.warning(), None);
        }
        gpu.restart();
        let (h0, _, _) = moments(&gpu.first, bg);
        let r = gpu.pass(131.76);
        let (h, vx, _) = moments(&gpu.planes(), bg);
        println!("SR-HEAT2 GPU at 131.76: {r:?}, {:?}, sigma2 x {vx:.3}, heat {:.2e}", r.warning(), (h - h0).abs() / h0);
        assert_eq!((r.stages, r.need, r.worst), (32, 33, 33), "{r:?}");
        assert_eq!(r.warning().as_deref(), Some("SR-WARN heat stages 33"));
        assert_eq!(r.dt, 131.76f32, "the capped faces cover the whole step");
        assert!((h - h0).abs() <= 1e-5 * h0, "heat {h} against {h0}");
        assert!(vx > SIGMA0 * SIGMA0 + 2.0 * CHI * 120.0, "the capped pass still diffuses: {vx}");
    }

    /// The adversarial case on the GPU (the twin's, above): a one-cell pulse, 15 passes of Δt = 6.4 Δt_p (5 stages), read
    /// as the difference of two GPU runs on a background of 2.0 — σ² within 1 % of an explicit run's and of 2χt, bounded.
    /// Each run's heat is graded on itself (the background loses heat through the edges: E lost = `radiated`, 1e-5): the
    /// difference's total measures f32 at T = 2, not P5 — E ≈ 4 in each of 16,384 cells rounds by up to 2.4e-7 a pass, 2.5e-6
    /// of each run's heat over 15 passes, which is 1.75e-3 of a pulse 3 × 10⁻⁵ of the world's (RTX 5090, M0-T42).
    #[test]
    fn gpu_a_sharp_pulse_stays_stable_at_a_step_just_over_one_stage_fewer() {
        let scene = scene("heat_pulse.json");
        let physics = scene.physics().unwrap();
        let (st_with, _, bg) = gaussian_on(&scene, Some(2.0), 0.01, PEAK);
        let (st_without, _, _) = gaussian_on(&scene, Some(2.0), 0.01, 0.0);
        let (with, without) = (OnGpu::new(&physics, flags(&scene), &st_with), OnGpu::new(&physics, flags(&scene), &st_without));
        let (w, h) = (st_with.width, st_with.height);
        let pulse: Vec<f64> = temps(&with.first).iter().zip(temps(&without.first)).map(|(a, b)| a - b).collect();
        let mut ring: f64 = 0.0;
        for _ in 0..15 {
            assert_eq!(with.pass(3.2).stages, 5);
            assert_eq!(without.pass(3.2).stages, 5);
            let low = temps(&with.planes()).iter().zip(temps(&without.planes())).map(|(a, b)| a - b).fold(f64::MAX, f64::min);
            ring = ring.min(low);
        }
        let response: Vec<f64> = temps(&with.planes()).iter().zip(temps(&without.planes())).map(|(a, b)| a - b).collect();
        let (_, ex, ey) = stats(&explicit(&pulse, w, h, CHI, 0.2, 240), w, 0.0, usize::MAX);
        let (total, vx, vy) = stats(&response, w, 0.0, usize::MAX);
        let (p0, _, _) = stats(&pulse, w, 0.0, usize::MAX);
        let highest = response.iter().cloned().fold(f64::MIN, f64::max);
        println!("SR-HEAT2 GPU sharp pulse (bg {bg}): sigma2 x {vx:.5} y {vy:.5}, explicit x {ex:.5} y {ey:.5}, deepest ring {ring:.3e}, highest {highest:.4e}, heat {total:.7} of {p0:.7}");
        assert!((vx - ex).abs() / ex <= 0.01 && (vy - ey).abs() / ey <= 0.01, "{vx} {vy} vs {ex} {ey}");
        assert!((vx - 48.0).abs() / 48.0 <= 0.01, "{vx} vs 2 chi t = 48");
        for run in [&with, &without] {
            let (lost, radiated) = (energy(&run.first) - energy(&run.planes()), run.radiated().iter().sum::<f64>());
            println!("SR-HEAT2 GPU sharp pulse run: lost {lost:.6}, radiated {radiated:.6}");
            assert!(radiated > 1.0 && (lost - radiated).abs() <= 1e-5 * radiated, "lost {lost}, radiated {radiated}");
        }
        assert!(ring >= -0.05 * p0 && highest <= 0.01, "ring {ring}, highest {highest}");
    }
}

/// G-FLD: |F| ≤ c_sb a_r T⁴ in thin gas; the radiative conductivity in thick gas; λ.
mod limiter {
    use super::*;

    fn constants(physics: &Physics) -> (f64, f64) {
        (physics.get("a_r").unwrap(), physics.get("c_sb").unwrap())
    }

    /// The largest ratio |F|/(c_sb a_r T⁴) over every face of `p` for the state `t` (Σ and the hotter side's T of the face;
    /// the world's edges face the vacuum at T_floor), and the faces where the flux is not 0.
    fn worst_ratio(p: &Pass, t: &[f64], w: usize, h: usize, a_r: f64, c_sb: f64) -> f64 {
        let t_floor = 0.05;
        let bound = |ta: f64, tb: f64| c_sb * a_r * ta.max(tb).powi(4);
        let mut worst: f64 = 0.0;
        for y in 0..h {
            for x in 0..=w {
                let ta = if x > 0 { t[y * w + x - 1] } else { t_floor };
                let tb = if x < w { t[y * w + x] } else { t_floor };
                worst = worst.max(p.flux.x[y * (w + 1) + x].abs() / bound(ta, tb));
            }
        }
        for y in 0..=h {
            for x in 0..w {
                let ta = if y > 0 { t[(y - 1) * w + x] } else { t_floor };
                let tb = if y < h { t[y * w + x] } else { t_floor };
                worst = worst.max(p.flux.y[y * w + x].abs() / bound(ta, tb));
            }
        }
        worst
    }

    /// The hot spot of heat_spot.json: every pass's flux, every face, under c_sb a_r T⁴ of its hotter side; the limiter binds.
    #[test]
    fn in_thin_gas_the_flux_never_passes_c_sb_a_r_t4() {
        let (mut st, physics, heat) = world(&scene("heat_spot.json"));
        let (a_r, c_sb) = constants(&physics);
        let (w, h) = (st.width, st.height);
        let mut worst: f64 = 0.0;
        let mut stages = 0;
        for pass in 0..20 {
            let t = temps(&st);
            let p = heat.step(&mut st, 0.05);
            let r = worst_ratio(&p, &t, w, h, a_r, c_sb);
            stages = stages.max(p.stages);
            assert!(r <= 1.0 + 1e-6, "pass {pass}: |F| = {r} c_sb a_r T^4");
            worst = worst.max(r);
        }
        println!("SR-FLD worst |F| / (c_sb a_r T^4) {worst:.6} over 20 passes of 64x64 thin gas, up to {stages} stages");
        // The oracle is not vacuous: the limiter is the thing holding the flux, within a factor of 3 of the bound.
        assert!(worst > 0.3, "the limiter binds: {worst}");
        // And it can fail: the same spot without the limiter (λ ≡ 1/3) puts the flux orders of magnitude over it.
        let (mut st, physics, _) = world(&scene("heat_spot.json"));
        let off = Heat::new(&physics, &elements(), TestFlags { pin_temperature: false, constant_diffusivity: None, flux_limiter: false }).unwrap();
        let t = temps(&st);
        let p = off.step(&mut st, 0.0005);
        let r = worst_ratio(&p, &t, w, h, a_r, c_sb);
        println!("SR-FLD limiter off: |F| = {r:.1} c_sb a_r T^4");
        assert!(r > 100.0, "{r}");
    }

    /// Thick gas (κΣ = 200), a gentle gradient: λ = 1/3 and F is the radiative conductivity (4/3) a_r c_sb T³/(κΣ) times −∇T
    /// (SM-E20) to 1 % — the limiter must not touch gas it has no business in.
    #[test]
    fn in_thick_gas_the_flux_is_the_radiative_conductivity() {
        let scene = rect_world(64, 64, [0, 0, 64, 64], 100.0, 10.0);
        let (mut st, physics, heat) = world(&scene);
        let (a_r, c_sb) = constants(&physics);
        let (w, h) = (st.width, st.height);
        let t: Vec<f64> = (0..w * h).map(|i| 10.0 + 0.02 * (i % w) as f64).collect();
        set_temps(&mut st, &t);
        let p = heat.step(&mut st, 0.001);
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

    /// `test.flux_limiter` = false (§2.12.1) is λ ≡ 1/3 even at R = 10⁴: F = c_sb ΔE/(3 κΣ_face) across a face of heat_spot.json.
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
        let f_off = off.step(&mut st.clone(), 1e-6).flux.x[y * (w + 1) + x];
        let f_on = on.step(&mut st.clone(), 1e-6).flux.x[y * (w + 1) + x];
        let thick = -c_sb / (3.0 * ks) * a_r * (tb.powi(4) - ta.powi(4));
        let bound = c_sb * a_r * tb.powi(4);
        println!("SR-FLD face of the spot: unlimited {f_off:.4e} (thick form {thick:.4e}), limited {f_on:.4e}, bound {bound:.4e}");
        assert!((f_off - thick).abs() <= 1e-9 * thick.abs(), "{f_off} vs {thick}");
        assert!(f_off.abs() > 1000.0 * bound);
        assert!(f_on.abs() <= bound * (1.0 + 1e-6) && f_on.abs() > 0.1 * bound, "{f_on}");
        assert!(f_on * f_off > 0.0, "the same direction");
    }

    /// G-FLD on the GPU: heat_spot.json over 20 passes of Δt = 0.002 (no cap: the twin's 0.05 needs 131 stages), every face
    /// under c_sb a_r T⁴ of its hotter side (1 + 1e-6), the limiter binding, the first pass's F the twin's within 1e-4.
    #[test]
    fn gpu_in_thin_gas_the_flux_never_passes_c_sb_a_r_t4() {
        let (st, physics, heat) = world(&scene("heat_spot.json"));
        let (a_r, c_sb) = constants(&physics);
        let gpu = OnGpu::new(&physics, TestFlags::default(), &st);
        let (w, h) = (st.width, st.height);
        let (mut worst, mut stages): (f64, u32) = (0.0, 0);
        for pass in 0..20 {
            let t = temps(&gpu.planes());
            let r = gpu.pass(0.002);
            assert!(r.need <= 32 && r.stages == r.need, "pass {pass}: {r:?}");
            stages = stages.max(r.stages);
            let (x, y) = gpu.faces();
            let fluxes = Pass { stages: 0, dt_p: 0.0, radiated: 0.0, emitted: Vec::new(), flux: sr_physics::reference::heat::Flux { width: w, height: h, x, y } };
            if pass == 0 {
                let p = heat.step(&mut gpu.first.clone(), f64::from(0.002f32));
                let scale = p.flux.x.iter().chain(&p.flux.y).fold(0.0f64, |m, f| m.max(f.abs()));
                let gap = largest_gap(&fluxes.flux.x, &p.flux.x).max(largest_gap(&fluxes.flux.y, &p.flux.y));
                assert_eq!(r.stages as usize, p.stages);
                assert!(gap <= 1e-4 * scale, "F against the twin: {gap} of {scale}");
            }
            let ratio = worst_ratio(&fluxes, &t, w, h, a_r, c_sb);
            assert!(ratio <= 1.0 + 1e-6, "pass {pass}: |F| = {ratio} c_sb a_r T^4");
            worst = worst.max(ratio);
        }
        println!("SR-FLD GPU worst |F| / (c_sb a_r T^4) {worst:.6} over 20 passes, up to {stages} stages");
        assert!(worst > 0.3, "the limiter binds: {worst}");
    }

    /// Thick gas on the GPU carries (4/3) a_r c_sb T³/(κΣ) within 1 %; with `flux_limiter` off a face of the spot carries
    /// c_sb ΔE/(3 κΣ_face) within 1e-5, a thousand times the bound.
    #[test]
    fn gpu_thick_gas_carries_the_radiative_conductivity_and_the_flag_turns_the_limiter_off() {
        let thick = rect_world(64, 64, [0, 0, 64, 64], 100.0, 10.0);
        let (mut st, physics, _) = world(&thick);
        let (a_r, c_sb) = constants(&physics);
        let w = st.width;
        let t: Vec<f64> = (0..w * st.height).map(|i| 10.0 + 0.02 * (i % w) as f64).collect();
        set_temps(&mut st, &t);
        let gpu = OnGpu::new(&physics, flags(&thick), &st);
        gpu.pass(0.001);
        let (fx, _) = gpu.faces();
        let t = temps(&gpu.first);
        let mut worst: f64 = 0.0;
        for y in 1..st.height - 1 {
            for x in 1..w {
                let mid = 0.5 * (t[y * w + x - 1] + t[y * w + x]);
                let want = -(4.0 / 3.0) * a_r * c_sb * mid.powi(3) / (KAPPA_H * 100.0) * (t[y * w + x] - t[y * w + x - 1]);
                worst = worst.max((fx[y * (w + 1) + x] - want).abs() / want.abs());
            }
        }
        println!("SR-FLD GPU thick gas: worst relative error {worst:.2e}");
        assert!(worst <= 0.01, "{worst}");

        let (st, physics, _) = world(&scene("heat_spot.json"));
        let off = OnGpu::new(&physics, TestFlags { flux_limiter: false, ..TestFlags::default() }, &st);
        off.pass(1e-6);
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
