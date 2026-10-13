//! M0-T35 — the world's two edges (contract §2.8, §2.9's escaped, §5.1's G-EDGE), on the CPU twin
//! (`sr_physics::reference::hydro`) and on the GPU's P4 (`sr_engine::step::hydro`, shaders/hydro.wgsl), the GPU on the
//! physics adapter (R12, R13).
//!
//! Bounce: scenes/edge_pulse.json — a 256 × 64 world of gas at rest (Σ 1, Π 1, cold pressure off, so c² = 2Π/Σ is the
//! sound speed) holding a right-going acoustic pulse 32 cells wide (u = v₀ = 0.002, Π + Σc·v₀, Σ + Σv₀/c: one simple
//! wave, Mach 1.4e-3, so the reflection is linear acoustics' within O(Mach)). It reaches the right wall at t ≈ 113 and
//! has left it by t ≈ 140; at t = 200 the pulse's mass-weighted normal velocity — the world's x momentum over the pulse
//! slab's mass, v₀ at t = 0 — is within 2 % of −v₀ (G-EDGE), with nothing booked and the mass kept (the wall's mass flux
//! exactly 0). That oracle cannot see the ghosts: the wall's flux alone sets the impulse that turns the pulse (measured
//! on the twin, copy ghosts in place of mirrors: −2.000013e-3 both ways). So `wall-copy` — the shader's bounce ghosts
//! copying instead of mirroring — is graded against the twin's mirrors: the wind below run under bounce to t = 20 on the
//! GPU and on the twin to the GPU's time, Σ, Σu, Σv and E each within 1e-4 relative L1 (fixed before the first GPU run;
//! copy ghosts move the twin's by 2.6e-4, 6.4e-4, 2.6e-3, 3.5e-4).
//!
//! Leave: a wind — the 128 × 64 world moving at (0.6, 0.4), out through the right and bottom sides, away from the left
//! and top ones, with a helium block riding it (Σ + 2, Π + 0.02) so species and gradients cross the edge — to t = 40,
//! by then over a third of the mass gone. The acoustic pulse leaving would book ~1e-4 of its grid's mass, below what an
//! f32 grid's sum resolves at G-EDGE's 1e-6, hence the wind. Graded: the grid's mass loss equals the booked `escaped`
//! to 1e-6 of the escaped mass (G-EDGE; the twin to its f64 reach, every component); inflow exactly 0 — no face's
//! booked mass ever falls in a step (the twin, every step) or is negative in a step's booking (the GPU, read every
//! step), while HLL's diffusion still lets a little out of the left and top sides where the gas lies stagnant against
//! them; the GPU's escaped totals within G-REF's 1e-3 of the twin's. `set_edge_mode` lands at the next step boundary: a
//! run switched from bounce to leave between two submissions is, bit for bit, a leave run started there.

use std::path::{Path, PathBuf};

use sr_engine::gpu::Gpu;
use sr_engine::scene::{EdgeMode, Scene};
use sr_engine::state::{Booking, Domain, State as GpuState, WorldConfig, BOOK_GROUPS, TERMS};
use sr_engine::step::boxfit::BoxMode;
use sr_engine::step::{EosGpu, Passes, Step};
use sr_physics::reference::hydro::{Edge, Hydro, State, COMPONENTS, SIDES};
use sr_physics::registry::{Elements, Physics};

/// The pulse's speed, where its slab lies and when it is read (module doc).
const V0: f64 = 0.002;
const SLAB: (usize, usize) = (64, 96);
const T_PULSE: f64 = 200.0;
/// The wind's read time.
const T_WIND: f64 = 40.0;

const WIND: &str = r#"{"version": 1, "world": {"width": 128, "height": 64, "edge": "leave"},
    "switches": {"gravity": false, "hydro": true, "heat": false, "reactions": false, "neutrinos": false, "sinks": false},
    "objects": [
      {"kind": "rect", "x0": 0, "y0": 0, "w": 128, "h": 64, "sigma": 1.0, "pressure": 1.0, "composition": {"H": 1.0}, "v": [0.6, 0.4]},
      {"kind": "rect", "x0": 88, "y0": 24, "w": 24, "h": 16, "sigma": 2.0, "pressure": 0.02, "composition": {"He": 1.0}, "v": [0.6, 0.4]}],
    "overrides": {"k1e": 0, "k2e": 0, "k1n": 0, "k2n": 0}}"#;

fn elements() -> Elements {
    Elements::shipped().unwrap()
}

fn scenes_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenes")
}

fn pulse_scene() -> Scene {
    Scene::load(&scenes_dir().join("edge_pulse.json")).unwrap_or_else(|e| panic!("{e}"))
}

fn wind_scene() -> Scene {
    Scene::parse(WIND, &elements()).unwrap_or_else(|e| panic!("{e}"))
}

/// The scene's planes as the twin's state, f32 widened to f64 (the GPU starts from the same f32 values).
fn world_of(scene: &Scene, physics: &Physics) -> State {
    let (planes, warnings) = scene.planes(scene.world, physics, &elements()).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    State::new(scene.world.width as usize, scene.world.height as usize, planes.iter().map(|&v| f64::from(v)).collect())
}

fn twin(physics: &Physics, edge: Edge) -> Hydro {
    Hydro::new(physics, &elements(), edge).unwrap()
}

/// The world's x momentum over the pulse slab's mass at t = 0: the pulse's mass-weighted normal velocity.
fn pulse_velocity(st: &State, slab_mass: f64) -> f64 {
    st.totals()[1] / slab_mass
}

/// The mass in the pulse's slab of the first state (v₀ times it is the x momentum).
fn slab_mass(first: &State) -> f64 {
    let w = first.width;
    (0..first.height).flat_map(|y| (SLAB.0..SLAB.1).map(move |x| first.get(0, y * w + x))).sum()
}

fn rel(a: f64, b: f64) -> f64 {
    (a - b).abs() / b.abs()
}

/// Σ|a − b| / Σ|b|.
fn rel_l1(a: &[f64], b: &[f64]) -> f64 {
    let d: f64 = a.iter().zip(b).map(|(x, y)| (x - y).abs()).sum();
    d / b.iter().map(|y| y.abs()).sum::<f64>()
}

// ---- the twin ----

#[test]
fn twin_bounce_returns_the_pulse() {
    // The pulse's rows are all alike: the twin runs four of them (a 256 × 4 strip between the same walls).
    let scene = pulse_scene();
    assert_eq!(scene.edge, EdgeMode::Bounce);
    let physics = scene.physics().unwrap();
    let first = world_of(&scene, &physics).rows(0, 4);
    let m_slab = slab_mass(&first);
    let v_start = pulse_velocity(&first, m_slab);
    let mut st = first.clone();
    let steps = twin(&physics, Edge::Bounce).advance(&mut st, 0.0, T_PULSE);
    let v_end = pulse_velocity(&st, m_slab);
    let (start, end) = (first.totals(), st.totals());
    println!(
        "SR-EDGE-BOUNCE twin 256x4: {steps} steps to t = {T_PULSE}, v at 0 {v_start:.6e} (v0 {V0}), at {T_PULSE} {v_end:.6e}, \
         {:.3} % from -v0; mass {:.1e} relative, escaped {:?}",
        100.0 * rel(v_end, -V0),
        rel(end[0], start[0]),
        st.escaped_total()[0]
    );
    assert!(rel(v_start, V0) <= 1e-6, "the slab moves at v0");
    assert!(rel(v_end, -V0) <= 0.02, "the pulse returns at {v_end:e}, not -v0 = {:e}", -V0);
    assert!(rel(end[0], start[0]) <= 1e-12, "mass: the walls pass none");
    assert!(st.escaped.iter().all(|&e| e == 0.0), "bounce books nothing");
}

#[test]
fn twin_leave_books_what_crosses() {
    let scene = wind_scene();
    assert_eq!(scene.edge, EdgeMode::Leave);
    let physics = scene.physics().unwrap();
    let first = world_of(&scene, &physics);
    // Stepped here, so every face's booked mass is read after every step: it may never fall (inflow exactly 0).
    let hydro = twin(&physics, Edge::Leave);
    let (mut st, mut t, mut steps, mut fell) = (first.clone(), 0.0, 0usize, 0usize);
    let faces = st.faces();
    while t < T_WIND {
        let before = st.escaped[..faces].to_vec();
        let dt = hydro.dt(&st, None).min(T_WIND - t);
        hydro.step(&mut st, dt);
        fell += st.escaped[..faces].iter().zip(&before).filter(|(a, b)| a < b).count();
        t += dt;
        steps += 1;
    }
    let (start, end, esc) = (first.totals(), st.totals(), st.escaped_total());
    // Each component: the grid's loss against the booked term, over the larger of the two (energy and momentum may be
    // small beside the grid's).
    let mut worst = 0.0f64;
    for k in 0..COMPONENTS {
        let loss = start[k] - end[k];
        let d = (loss - esc[k]).abs() / esc[k].abs().max(start[k].abs()).max(1e-300);
        worst = worst.max(d);
    }
    println!(
        "SR-EDGE-LEAVE twin 128x64: {steps} steps to t = {T_WIND}, escaped mass {:.4} of {:.4} ({:.1} %), loss − escaped \
         {:.2e} of escaped, worst component {worst:.2e}; by side {:?}; a face's mass fell {fell} times",
        esc[0],
        start[0],
        100.0 * esc[0] / start[0],
        ((start[0] - end[0]) - esc[0]).abs() / esc[0],
        (0..SIDES).map(|s| st.escaped_side(s)[0]).collect::<Vec<_>>()
    );
    assert!(esc[0] >= start[0] / 3.0, "over a third of the mass leaves: {}", esc[0]);
    assert!(((start[0] - end[0]) - esc[0]).abs() <= 1e-6 * esc[0], "mass lost ≠ escaped");
    assert!(worst <= 1e-12, "a component's loss ≠ its escaped by {worst:e}");
    assert_eq!(fell, 0, "inflow: a face's booked mass fell in a step");
    let (right, bottom) = (st.escaped_side(1)[0], st.escaped_side(3)[0]);
    assert!(right + bottom >= 0.9 * esc[0], "the wind leaves through the right and bottom sides");
    // The helium crossed, and the species' masses add up to the mass — as far as the scene's f32 fractions sum to 1.
    let n = first.cells();
    let off = (0..n).map(|i| ((4..COMPONENTS).map(|c| first.get(c, i)).sum::<f64>() - 1.0).abs()).fold(0.0, f64::max);
    let species: f64 = esc[4..].iter().sum();
    println!("SR-EDGE-SPECIES twin: Σ species escaped {species:.6} for mass {:.6}, the scene's |ΣX − 1| ≤ {off:.1e}", esc[0]);
    assert!(esc[5] > 0.0, "helium escaped");
    assert!((species - esc[0]).abs() <= (off + 1e-12) * esc[0], "species {species} ≠ mass {}", esc[0]);
}

// ---- the GPU's P4 ----

/// A GPU run's end: its planes, the escaped booking summed in f64 per slot (14 planes of 2 (w + h)), its sim time and
/// steps, the adapter.
struct GpuRun {
    state: State,
    escaped: Vec<f64>,
    /// The smallest mass any face booked in one batch.
    least_face: f64,
    sim_time: f64,
    steps: u64,
    adapter: String,
}

/// Where `booking.escaped`'s planes start in a booking readback, and its slots per plane.
fn escaped_span(world: WorldConfig) -> (usize, usize) {
    let mut base = 0;
    for g in &BOOK_GROUPS {
        let slots = g.domain.slots(world);
        if TERMS[g.terms.start].name == "escaped.mass" {
            assert_eq!(g.domain, Domain::EdgeFaces);
            return (base, slots);
        }
        base += (g.terms.end - g.terms.start) * slots;
    }
    panic!("no escaped booking")
}

/// How a GPU run ends and is batched.
#[derive(Clone, Copy, Default)]
struct Run {
    /// Until the first step at this sim time.
    t_end: Option<f64>,
    /// Until this many steps.
    steps: Option<u64>,
    /// This many steps under bounce first, then the run's edge (`set_edge_mode`), between two submissions.
    switch: Option<u64>,
    /// One step a batch (the booking read after every step), else up to 32.
    single: bool,
}

/// A GPU run of `first` with P4 alone under `edge`, the box fit, as `run` says; the booking read and cleared after every
/// batch, so no f32 slot adds more than 32 steps.
fn gpu_run(physics: &Physics, first: &State, edge: EdgeMode, run: Run) -> GpuRun {
    let Run { t_end, steps, switch, single } = run;
    let gpu: Gpu = super::device();
    let world = WorldConfig::new(first.width as u32, first.height as u32).unwrap();
    let state = GpuState::new(&gpu.device, world).unwrap();
    let planes: Vec<f32> = first.planes.iter().map(|&v| v as f32).collect();
    state.upload(&gpu.queue, &planes).unwrap();
    let booking = Booking::new(&gpu.device, world).unwrap();
    let eos = EosGpu::new(&gpu.device, physics, &elements());
    let start = if switch.is_some() { EdgeMode::Bounce } else { edge };
    let passes = Passes { mode: BoxMode::Fit, gravity: None, hydro: Some(start) };
    let step = Step::with_passes(&gpu.device, &state, &booking, &eos, physics, &passes).unwrap();
    let (base, slots) = escaped_span(world);
    let mut escaped = vec![0f64; COMPONENTS * slots];
    let mut least_face = f64::INFINITY;
    loop {
        let before = step.read_dt(&gpu.device, &gpu.queue);
        if switch == Some(before.steps) {
            step.set_edge_mode(&gpu.queue, edge);
        }
        let mut batch = if single { 1 } else { 32u64 };
        if let Some(n) = steps {
            if before.steps >= n {
                break;
            }
            batch = batch.min(n - before.steps);
        }
        if let Some(s) = switch.filter(|&s| s > before.steps) {
            batch = batch.min(s - before.steps);
        }
        if let Some(t) = t_end {
            if before.steps > 0 && before.sim_time >= t {
                break;
            }
            let dt = f64::from(if before.steps == 0 { before.dt_next } else { before.dt_next.max(before.dt) });
            batch = batch.min((((t - before.sim_time) / dt * 0.8).floor() as u64).max(1));
        }
        step.run(&gpu.device, &gpu.queue, batch as u32);
        let after = step.read_dt(&gpu.device, &gpu.queue);
        assert!(after.non_finite.is_none(), "non-finite state: {:?}", after.non_finite);
        let book = booking.readback(&gpu.device, &gpu.queue);
        for (acc, &v) in escaped.iter_mut().zip(&book[base..base + COMPONENTS * slots]) {
            *acc += f64::from(v);
        }
        least_face = book[base..base + slots].iter().fold(least_face, |m, &v| m.min(f64::from(v)));
        let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("clear") });
        booking.clear(&mut encoder);
        gpu.queue.submit([encoder.finish()]);
    }
    let report = step.read_dt(&gpu.device, &gpu.queue);
    let planes = state.readback(&gpu.device, &gpu.queue).iter().map(|&v| f64::from(v)).collect();
    GpuRun {
        state: State::new(first.width, first.height, planes),
        escaped,
        least_face,
        sim_time: report.sim_time,
        steps: report.steps,
        adapter: gpu.info.sr_adapter_line(),
    }
}

/// The escaped booking's total per component, in f64.
fn escaped_totals(run: &GpuRun) -> [f64; COMPONENTS] {
    let slots = run.escaped.len() / COMPONENTS;
    std::array::from_fn(|k| run.escaped[k * slots..(k + 1) * slots].iter().sum())
}

/// The slots of side `side` (state.rs `Domain::EdgeFaces`' order) in plane `k`.
fn side_slots(run: &GpuRun, k: usize, side: usize) -> &[f64] {
    let (w, h) = (run.state.width, run.state.height);
    let slots = run.escaped.len() / COMPONENTS;
    let plane = &run.escaped[k * slots..(k + 1) * slots];
    match side {
        0 => &plane[..h],
        1 => &plane[h..2 * h],
        2 => &plane[2 * h..2 * h + w],
        _ => &plane[2 * h + w..],
    }
}

#[test]
fn gpu_bounce_returns_the_pulse() {
    let scene = pulse_scene();
    let physics = scene.physics().unwrap();
    let first = world_of(&scene, &physics);
    let run = gpu_run(&physics, &first, EdgeMode::Bounce, Run { t_end: Some(T_PULSE), ..Run::default() });
    println!("{}", run.adapter);
    let m_slab = slab_mass(&first);
    let (v_start, v_end) = (pulse_velocity(&first, m_slab), pulse_velocity(&run.state, m_slab));
    let (start, end) = (first.totals(), run.state.totals());
    println!(
        "SR-EDGE-BOUNCE gpu 256x64: {} steps to t = {:.4}, v at 0 {v_start:.6e}, at the end {v_end:.6e}, {:.3} % from -v0; \
         mass {:.1e} relative",
        run.steps,
        run.sim_time,
        100.0 * rel(v_end, -V0),
        rel(end[0], start[0])
    );
    assert!(rel(v_end, -V0) <= 0.02, "the pulse returns at {v_end:e}, not -v0 = {:e}", -V0);
    assert!(rel(end[0], start[0]) <= 1e-6, "mass: the walls pass none");
    assert!(run.escaped.iter().all(|&e| e == 0.0), "bounce books nothing");
}

#[test]
fn gpu_bounce_walls_are_the_twins() {
    // The wind against the right and bottom walls under bounce, to t = 20: the GPU's mirror ghosts and wall faces against
    // the twin's, run to the GPU's own time.
    let scene = wind_scene();
    let physics = scene.physics().unwrap();
    let first = world_of(&scene, &physics);
    let run = gpu_run(&physics, &first, EdgeMode::Bounce, Run { t_end: Some(20.0), ..Run::default() });
    println!("{}", run.adapter);
    let mut st = first.clone();
    twin(&physics, Edge::Bounce).advance(&mut st, 0.0, run.sim_time);
    let diff = [0usize, 1, 2, 3].map(|c| rel_l1(run.state.plane(c), st.plane(c)));
    println!(
        "SR-EDGE-WALLS gpu 128x64 bounce: {} steps to t = {:.4}; against the twin (Σ, Σu, Σv, E) relative L1 {} ≤ 1e-4",
        run.steps,
        run.sim_time,
        diff.map(|d| format!("{d:.2e}")).join(" ")
    );
    assert!(run.escaped.iter().all(|&e| e == 0.0), "bounce books nothing");
    assert!(diff.iter().all(|&d| d <= 1e-4), "the GPU's bounce differs from the twin's: {diff:?}");
}

#[test]
fn gpu_leave_books_what_crosses() {
    let scene = wind_scene();
    let physics = scene.physics().unwrap();
    let first = world_of(&scene, &physics);
    let run = gpu_run(&physics, &first, EdgeMode::Leave, Run { t_end: Some(T_WIND), single: true, ..Run::default() });
    println!("{}", run.adapter);
    let (start, end, esc) = (first.totals(), run.state.totals(), escaped_totals(&run));
    let loss = start[0] - end[0];
    let mut twin_st = first.clone();
    twin(&physics, Edge::Leave).advance(&mut twin_st, 0.0, run.sim_time);
    let twin_esc = twin_st.escaped_total();
    let vs_twin = [0usize, 1, 2, 3, 4, 5].map(|k| rel(esc[k], twin_esc[k]));
    println!(
        "SR-EDGE-LEAVE gpu 128x64: {} steps to t = {:.4}, escaped mass {:.4} of {:.4} ({:.1} %), loss − escaped {:.2e} of \
         escaped; against the twin's escaped (Σ, Σu, Σv, E, H, He) {}; the least mass a face booked in a step {:e}",
        run.steps,
        run.sim_time,
        esc[0],
        start[0],
        100.0 * esc[0] / start[0],
        (loss - esc[0]).abs() / esc[0],
        vs_twin.map(|d| format!("{d:.2e}")).join(" "),
        run.least_face
    );
    assert!(esc[0] >= start[0] / 3.0, "over a third of the mass leaves: {}", esc[0]);
    assert!((loss - esc[0]).abs() <= 1e-6 * esc[0], "mass lost {loss} ≠ escaped {} (G-EDGE 1e-6)", esc[0]);
    assert!(run.least_face >= 0.0, "inflow: a face booked {:e} in a step", run.least_face);
    let (right, bottom) = (side_slots(&run, 0, 1).iter().sum::<f64>(), side_slots(&run, 0, 3).iter().sum::<f64>());
    assert!(right + bottom >= 0.9 * esc[0], "the wind leaves through the right and bottom sides");
    assert!(vs_twin.iter().all(|&d| d <= 1e-3), "escaped {esc:?} vs the twin's {twin_esc:?} (G-REF 1e-3)");
}

#[test]
fn gpu_set_edge_mode_lands_at_the_step_boundary() {
    // 64 steps of the wind under bounce, then leave: against a leave run started from the state after those 64 steps
    // (the box re-fit and the guard fall on the same steps: 64 ≡ 0 mod 16 and 64), bit for bit.
    let scene = wind_scene();
    let physics = scene.physics().unwrap();
    let first = world_of(&scene, &physics);
    let bounced = gpu_run(&physics, &first, EdgeMode::Bounce, Run { steps: Some(64), ..Run::default() });
    assert!(bounced.escaped.iter().all(|&e| e == 0.0), "bounce books nothing");
    let switched =
        gpu_run(&physics, &first, EdgeMode::Leave, Run { steps: Some(128), switch: Some(64), ..Run::default() });
    let fresh = gpu_run(&physics, &bounced.state, EdgeMode::Leave, Run { steps: Some(64), ..Run::default() });
    println!("{}", switched.adapter);
    let same_state = switched.state.planes.iter().zip(&fresh.state.planes).all(|(a, b)| a.to_bits() == b.to_bits());
    let same_book = switched.escaped.iter().zip(&fresh.escaped).all(|(a, b)| a.to_bits() == b.to_bits());
    println!(
        "SR-EDGE-SWITCH gpu: bounce 64 then leave 64 — state {}, escaped {} (mass {:.4}) against a leave run from step 64",
        if same_state { "bit for bit" } else { "DIFFERS" },
        if same_book { "bit for bit" } else { "DIFFERS" },
        escaped_totals(&switched)[0]
    );
    assert!(escaped_totals(&switched)[0] > 0.0, "leave after the switch lets gas out");
    assert!(same_state && same_book, "the switch did not land at the step boundary");
}

#[test]
fn a_new_world_starts_in_leave() {
    // §2.8 (D1): a scene that names no edge is a leave world.
    let text = r#"{"version": 1, "world": {"width": 64, "height": 64}, "objects": []}"#;
    assert_eq!(Scene::parse(text, &elements()).unwrap().edge, EdgeMode::Leave);
}

