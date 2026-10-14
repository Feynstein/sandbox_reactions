//! M0-T38 — G-REF, the GPU against the CPU twin (contract §5.1, §2.2, §6.7, R13): the GPU's f32 step and the twin's f64 step
//! (`sr_physics::reference::Twin`, M0-T37) run the same scene for the same number of steps and their states are read side
//! by side — the relative L1 difference of Σ and of E at most 10⁻³, the largest absolute difference of each mass fraction X
//! at most 10⁻⁴ (ES-E20: floats differ between GPUs, so the band is the contract's, never tuned to an adapter).
//!
//! §5.1's three 64 × 64 scenes, 200 steps each: scenes/ref_disk.json, a cold Maclaurin disk (a = 24, Σc = 1, the
//! cold pressure off as in scenes/cold_disk.json) falling in under its own gravity with leave edges, scenes/ref_sod.json,
//! Sod's strip (ρ 1 | 0.125, P 1 | 0.1 at rest, γ = 2: the cold pressure off as in scenes/sod.json) with bounce edges, the
//! shock reflected from the far wall inside the 200 steps, and scenes/ref_burn.json (M0-T48), a hot H | He field burning
//! under P5 heat and P6 reactions with no gravity or flow: the twin runs both passes with the scene's test flags, and the
//! field must burn — H's mass in the twin falls by at least 1 % (a field too cool to burn compares nothing).
//!
//! The twin runs the whole world and the GPU its fitted box (M0-T37's note): outside the box the GPU leaves vacuum at Σ = 0
//! where the twin's P8 sets Σ_floor — 10⁻⁸ per cell, invisible in the L1 sums, so X is read only where both states hold
//! matter — Σ at or above Σ_floor / 10⁻⁴ = 10⁻⁴, 100 × Σ_vac, where P8's floor matter (pure hydrogen) cannot move an X by
//! the band: unmasked, five of the disk's 1,608 rim cells at Σ ≈ 3 × 10⁻⁶ (3 × Σ_vac) read |ΔX| up to 1.06 × 10⁻³ in x_H. Both runs take exactly 200 steps, so their sim times differ by the Δt they chose: a GPU
//! whose CFL is 0.39 where the twin's is 0.4 is 2.5 % short in time at step 200, which is what the plant (tests/plants/
//! cfl-gpu.patch) shows these comparisons.

use std::path::{Path, PathBuf};

use sr_engine::scene::{EdgeMode, Scene};
use sr_engine::state::{Booking, SideFields, State as GpuState, CHANNELS, SPECIES};
use sr_engine::step::boxfit::BoxMode;
use sr_engine::step::heat::HeatSetup;
use sr_engine::step::react::ReactSetup;
use sr_engine::step::{EosGpu, Passes, Step};
use sr_physics::reference::hydro::Edge;
use sr_physics::reference::{heat, react, Burning, Switches, Twin};
use sr_physics::registry::{Elements, Reactions};

const STEPS: u64 = 200;
/// §5.1's bands: relative L1 in Σ and E, absolute in each X.
const L1_BAND: f64 = 1e-3;
const X_BAND: f64 = 1e-4;

fn elements() -> Elements {
    Elements::shipped().unwrap()
}

fn scene(file: &str) -> Scene {
    let dir: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenes");
    Scene::load(&dir.join(file)).unwrap_or_else(|e| panic!("{file}: {e}"))
}

/// Relative L1 difference of plane `c`: Σ|a − b| / Σ|b| (b the twin's).
fn l1(gpu: &[f32], twin: &[f64], c: usize, cells: usize) -> f64 {
    let r = c * cells..(c + 1) * cells;
    let diff: f64 = gpu[r.clone()].iter().zip(&twin[r.clone()]).map(|(&a, &b)| (f64::from(a) - b).abs()).sum();
    diff / twin[r].iter().map(|b| b.abs()).sum::<f64>()
}

/// H's mass over the world, Σ · X_H summed over the cells (the twin's f64 planes).
fn hydrogen_mass(planes: &[f64], cells: usize) -> f64 {
    let h = elements().index_of("H").expect("the registry has H");
    (0..cells).map(|i| planes[i] * planes[(4 + h) * cells + i]).sum()
}

/// The GPU against the twin on `file` after `STEPS` steps: asserts G-REF's bands, prints `SR-REF` lines.
fn grade(file: &str) {
    let scene = scene(file);
    let (world, physics) = (scene.world, scene.physics().unwrap());
    let (planes, warnings) = scene.planes(world, &physics, &elements()).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let (w, h, cells) = (world.width as usize, world.height as usize, world.cells());
    let (gravity, hydro) = (scene.switches.gravity, scene.switches.hydro);
    let (switches, test) = (scene.switches, scene.test);
    // P6's records as the registry has them, with the scene's overrides (the physics the scene's run uses).
    let records = Reactions::shipped(&elements(), &physics).unwrap();
    let burning = Burning {
        heat: heat::TestFlags {
            pin_temperature: test.pin_temperature,
            constant_diffusivity: test.constant_diffusivity,
            flux_limiter: test.flux_limiter,
        },
        react: react::TestFlags { pin_temperature: test.pin_temperature },
        reactions: &records,
        neutrinos: switches.neutrinos,
    };

    // The twin, f64 from the same f32 planes.
    let edge = match scene.edge {
        EdgeMode::Leave => Edge::Leave,
        EdgeMode::Bounce => Edge::Bounce,
    };
    let twin_switches = Switches { gravity, hydro, heat: switches.heat, reactions: switches.reactions };
    let mut twin =
        Twin::new(&physics, &elements(), edge, twin_switches, Some(burning), w, h, planes.iter().map(|&v| f64::from(v)).collect()).unwrap();
    let h_before = hydrogen_mass(&twin.state.planes, cells);
    for _ in 0..STEPS {
        twin.step();
    }
    let h_after = hydrogen_mass(&twin.state.planes, cells);

    // The GPU, the same switches and edge, the same number of steps.
    let gpu = super::device();
    let state = GpuState::new(&gpu.device, world).unwrap();
    state.upload(&gpu.queue, &planes).unwrap();
    let booking = Booking::new(&gpu.device, world).unwrap();
    let side = SideFields::new(&gpu.device, world).unwrap();
    let eos = EosGpu::new(&gpu.device, &physics, &elements());
    let passes = Passes { mode: BoxMode::Fit, gravity: gravity.then_some(&side), hydro: hydro.then_some(scene.edge) };
    let heat_setup = switches.heat.then_some(HeatSetup { side: &side, test });
    let react_setup = switches.reactions.then_some(ReactSetup { reactions: &records, test, neutrinos: switches.neutrinos, side: &side });
    let step = Step::with_reactions(&gpu.device, &state, &booking, &eos, &physics, &passes, heat_setup.as_ref(), react_setup.as_ref()).unwrap();
    for _ in 0..STEPS / 50 {
        step.run(&gpu.device, &gpu.queue, 50);
    }
    let report = step.read_dt(&gpu.device, &gpu.queue);
    assert_eq!(report.steps, STEPS, "the GPU ran {} steps: {report:?}", report.steps);
    assert!(report.non_finite.is_none(), "non-finite state: {:?}", report.non_finite);
    let out = state.readback(&gpu.device, &gpu.queue);

    let (sigma, energy) = (l1(&out, &twin.state.planes, 0, cells), l1(&out, &twin.state.planes, 3, cells));
    // X is read where the floor cannot move it by the band: a cell below Σ_floor / 10⁻⁴ may hold more than 10⁻⁴ of its mass
    // as P8's floor matter (pure hydrogen), and whether a rim cell crossed Σ_vac this step is f32 against f64 noise.
    let matter = physics.get("sigma_floor").unwrap() / X_BAND;
    let live = |i: usize| f64::from(out[i]) >= matter && twin.state.planes[i] >= matter;
    let mut worst = (0.0f64, SPECIES.start);
    for c in SPECIES {
        let d = (0..cells).filter(|&i| live(i)).map(|i| (f64::from(out[c * cells + i]) - twin.state.planes[c * cells + i]).abs()).fold(0.0, f64::max);
        if d > worst.0 {
            worst = (d, c);
        }
    }
    let t_gpu = f64::from(report.sim_time);
    println!(
        "SR-REF {file}: {STEPS} steps on {}: sim time GPU {t_gpu:.5} / twin {:.5} ({:+.3} %); relative L1 Σ {sigma:.3e} (band {L1_BAND:e}), \
         E {energy:.3e} (band {L1_BAND:e}); largest |ΔX| {:.3e} in {} (band {X_BAND:e})",
        gpu.info.sr_adapter_line(),
        twin.sim_time(),
        100.0 * (t_gpu / twin.sim_time() - 1.0),
        worst.0,
        CHANNELS[worst.1],
    );
    assert!(sigma <= L1_BAND, "{file}: relative L1 in Σ {sigma:e} > {L1_BAND:e}");
    assert!(energy <= L1_BAND, "{file}: relative L1 in E {energy:e} > {L1_BAND:e}");
    assert!(worst.0 <= X_BAND, "{file}: |ΔX| {:e} in {} > {X_BAND:e}", worst.0, CHANNELS[worst.1]);
    if switches.reactions {
        // The adversarial case: a field too cool to burn in STEPS compares nothing, so the twin must have burnt H by 1 %.
        let fell = 1.0 - h_after / h_before;
        println!("SR-REF {file}: H mass in the twin fell {:.3} % (at least 1 % required)", 100.0 * fell);
        assert!(fell >= 0.01, "{file}: H mass fell {:.3} % in the twin, under 1 %: the field did not burn", 100.0 * fell);
    }
}

#[test]
fn the_collapsing_disk_gpu_against_the_twin() {
    grade("ref_disk.json");
}

#[test]
fn the_sod_strip_gpu_against_the_twin() {
    grade("ref_sod.json");
}

#[test]
fn the_burning_field_gpu_against_the_twin() {
    grade("ref_burn.json");
}
