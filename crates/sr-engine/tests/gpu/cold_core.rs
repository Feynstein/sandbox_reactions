//! M0-T67 — the cold ceiling holds (contract §5.1's G-CORE [M0-R4], §1.4.1's K9, §2.3.2, §2.11 items 1–2, §5.5; M0-R4,
//! reports/cold_ceiling.md M3, M4, M6). The sheet's ceiling is a continuum limit (R → 0 as Σ_c → ∞): no grid run grades it,
//! so (a)–(c) are graded on `sr_physics::reference::sheet` and on what `calibrate` writes, and (d) on one resolved disk on the
//! grid, which grades the grid's statics and its hydro — *never the ceiling*.
//!
//! - **(a) the sequence** — M(Σ_c) rises at every decade 10⁰ … 10⁸ with each equilibrium's |2∫Π dA/W + 1| ≤ 10⁻³; the ring
//!   kernel gives the Maclaurin disk's potential within 10⁻⁴; `calibrate`'s m_ch_sb (read from the file it writes) is within
//!   0.5 % of c_sheet (K₂ₑ Y_e^{3/2})²/G², c_sheet the pure Π = KΣ^{3/2} sheet's eigen-mass computed *here* by Hachisu's
//!   iteration with Σ_max and the edge fixed (`Sheet::polytrope`, a second solver the sequence never uses) and within 10⁻³
//!   of 1.4331; the same for neutron matter (m_tov_sb, K₂ₙ), K9's order and band.
//! - **(b) the ceiling** — a fresh scan (half decades, 100 annuli, the test's own bisection): an equilibrium exists at
//!   0.95 m_ch_sb and none at 1.1 m_ch_sb, which is above every mass the scan reaches.
//! - **(c) a pure Σ² law** (the relativistic branch removed) — R₉₀(M) flat within 3 % over M ∈ {¼, ½, 1} × m_ch_sb, and
//!   visibly not flat with the branch on (the check would pass on a solver that ignored M).
//! - **(d) the resolved hold** — scenes/cold_co_disk.json: the grid's own SCF equilibrium (`cold_disk_scene`) of M = 100 C/O
//!   at T_floor with k1e × 6 (R₉₀ 18.8 cells), 64 × 64, bounce, gravity and hydro only; the test rebuilds it and refuses a
//!   stale file (`SR_WRITE_SCENES=1 bash tests/gpu.sh cold_core physics` rewrites it). R₉₀ within 3 % of its start at every
//!   dump over 10 τ_dyn (τ_dyn = √(R₉₀³/GM), the tracker's) on the GPU (the physics adapter, R12) and on the CPU twin run for
//!   the same number of steps, and the GPU's final R₉₀ within 1 % of the twin's. Measured by M0-R4: 1.62 % GPU, 1.65 % twin.
//!
//! Plants: `cold-exponent` (the EoS's relativistic exponent 3/2 → 1.6: no ceiling) and `cold-gpu-pressure` (the GPU's cold
//! pressure × 0.9: the (d) disk is no longer in balance there).

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde_json::Value;
use sr_engine::calibrate::{cold_disk_scene, radius_holding, run_with, Matter};
use sr_engine::dump::Dump;
use sr_engine::headless;
use sr_engine::scene::{EdgeMode, Gas, Heat, Object, Scene};
use sr_physics::eos::Eos;
use sr_physics::reference::sheet::{self, ColdMatter, Equilibrium, Fluid, Sheet};
use sr_physics::registry::{Elements, Physics, N_SPECIES};

/// c_sheet, the pure sheet's mass constant M G²/K² for Π = KΣ^{3/2} (reports/cold_ceiling.md M3).
const C_SHEET: f64 = 1.4331;
/// G = 1 (§2.1).
const G: f64 = 1.0;

fn physics() -> Physics {
    Physics::shipped().unwrap()
}

fn elements() -> Elements {
    Elements::shipped().unwrap()
}

fn scenes_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenes")
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sr-cold-core-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    dir
}

/// C 0.5 + O 0.5 (item 1) and neutron matter (item 2), through the registry's own composition.
fn matters() -> (ColdMatter, ColdMatter) {
    let elements = elements();
    let comp = |parts: &[(&str, f64)]| {
        let mut x = [0.0; N_SPECIES];
        for &(key, share) in parts {
            x[elements.index_of(key).unwrap_or_else(|| panic!("elements.json has no {key}"))] = share;
        }
        elements.composition(&x)
    };
    let (co, n) = (comp(&[("C", 0.5), ("O", 0.5)]), comp(&[("n", 1.0)]));
    (ColdMatter { y_e: co.y_e, x_n: 0.0, inv_mu: co.inv_mu }, ColdMatter { y_e: n.y_e, x_n: 1.0, inv_mu: n.inv_mu })
}

/// Π(Σ) of a cold disk at T_floor, from the EoS's own pressure.
fn cold_fluid(matter: ColdMatter) -> Fluid {
    let (physics, eos) = (physics(), Eos::from_physics(&physics()).unwrap());
    let eps_th = eos.t_floor() * matter.inv_mu;
    Fluid::new(move |s| eos.pi(s, eps_th, matter.y_e, matter.x_n), physics.get("sigma_vac").unwrap())
}

/// K₂′ of a matter: K₂ₑ Y_e^{3/2} for electrons, K₂ₙ X_n^{3/2} for neutron matter (§2.3.2), from physics.json's own numbers.
fn k2_prime(matter: ColdMatter) -> f64 {
    let physics = physics();
    if matter.x_n > 0.0 {
        physics.get("k2n").unwrap() * matter.x_n.powf(1.5)
    } else {
        physics.get("k2e").unwrap() * matter.y_e.powf(1.5)
    }
}

/// The test's own c_sheet: Hachisu's iteration on the pure critical sheet with Σ_max and the edge fixed, `annuli` annuli.
fn c_sheet(annuli: usize) -> f64 {
    let p = Sheet::new(annuli).polytrope(1.5).expect("the critical sheet settles");
    assert!(p.virial().abs() < 1e-4, "{annuli} annuli: virial {}", p.virial());
    p.mass_constant()
}

/// What `calibrate` writes for items 1–2 on the shipped constants (nothing mocked), read from the file.
fn calibrated() -> &'static Value {
    static FILE: OnceLock<Value> = OnceLock::new();
    FILE.get_or_init(|| {
        let out = scratch("calibration").join("calibration.json");
        let mut lines = Vec::new();
        let code = run_with(&["--out".to_string(), out.display().to_string(), "--only".to_string(), "m_ch_sb,m_tov_sb".to_string()], &[], &mut |l| lines.push(l.to_string()));
        assert_eq!(code, 0, "calibrate items 1–2 exit 0: {lines:?}");
        serde_json::from_str(&std::fs::read_to_string(&out).expect("the calibration file")).expect("JSON")
    })
}

fn item(name: &str) -> f64 {
    calibrated()[name].as_f64().unwrap_or_else(|| panic!("{name} is not a number in {}", calibrated()))
}

fn near(value: f64, expected: f64, share: f64, what: &str) {
    assert!((value / expected - 1.0).abs() <= share, "{what}: {value} is not within {share} of {expected}");
}

// ---- (a) the sequence ----

#[test]
fn the_ring_kernel_gives_the_maclaurin_disks_potential() {
    // Σ = √(1 − r²) (a = 1, Σ_c = 1, G = 1) has φ(r) = −(π²/2)(1 − r²/2) inside r < 1 (SM-E6; S7): a closed form.
    let sheet = Sheet::new(sheet::ANNULI);
    let sigma: Vec<f64> = sheet.nodes().iter().map(|r| (1.0 - r * r).sqrt()).collect();
    let phi = sheet.potential(&sigma);
    let phi0 = std::f64::consts::PI.powi(2) / 2.0;
    let worst = sheet.nodes().iter().zip(&phi).filter(|(r, _)| **r <= 0.95).map(|(r, p)| (p + phi0 * (1.0 - r * r / 2.0)).abs() / phi0).fold(0.0, f64::max);
    println!("SR-CORE (a) Maclaurin potential: worst error {worst:.2e} of |φ_c| (bound 1e-4)");
    assert!(worst < 1e-4, "the ring kernel is off the Maclaurin disk by {worst:e}");
}

#[test]
fn c_sheet_from_the_pure_law_is_1_4331_at_two_resolutions() {
    for n in [100, 200] {
        let c = c_sheet(n);
        println!("SR-CORE (a) c_sheet at {n} annuli: {c:.5}");
        near(c, C_SHEET, 1e-3, &format!("c_sheet at {n} annuli"));
        assert!(c < 9.0 * std::f64::consts::PI / 16.0 / 1.2, "not the uniform disk's 9π/16 = 1.767");
    }
}

/// The sequence M_k at Σ_c = 10^k, k = 0 … 8, of one matter: rising at every decade, each virial inside 10⁻³, below the limit.
fn check_sequence(name: &str, matter: ColdMatter, written: f64) {
    let c = sheet::ceiling(&Eos::from_physics(&physics()).unwrap(), matter, physics().get("sigma_vac").unwrap()).expect("the shipped EoS has a ceiling");
    assert_eq!(c.sequence.len(), 9, "{name}: Σ_c = 10⁰ … 10⁸");
    for (k, pair) in c.sequence.windows(2).enumerate() {
        assert!(pair[1].mass > pair[0].mass, "{name}: M does not rise from decade {k} to {}: {} → {}", k + 1, pair[0].mass, pair[1].mass);
    }
    for (k, eq) in c.sequence.iter().enumerate() {
        assert!(eq.virial().abs() <= 1e-3, "{name}: decade {k}: |2∫Π dA/W + 1| = {:e}", eq.virial().abs());
    }
    let top = c.sequence[8].mass;
    println!("SR-CORE (a) {name}: M(10⁰) {:.2} … M(10⁸) {top:.2}, limit {:.2}, written {written:.2}", c.sequence[0].mass, c.limit);
    assert!(top < written, "{name}: the ceiling {written} lies above the last equilibrium {top}");
    near(top, written, 0.01, &format!("{name}: M(10⁸) against the ceiling"));
}

#[test]
fn carbon_oxygen_m_ch_sb_is_c_sheet_k2_squared() {
    let (co, _) = matters();
    let m_ch = item("m_ch_sb");
    // The expectation is built here, from the pure-law solver and physics.json's own K₂ₑ, Y_e^{3/2}: not from the sequence.
    let want = c_sheet(100) * k2_prime(co).powi(2) / (G * G);
    println!("SR-CORE (a) C/O: m_ch_sb {m_ch:.3}, c_sheet K₂′²/G² {want:.3} (K₂′ {:.4})", k2_prime(co));
    near(m_ch, want, 0.005, "m_ch_sb against c_sheet (K₂ₑ Y_e^{3/2})²/G²");
    check_sequence("C/O", co, m_ch);
}

#[test]
fn neutron_m_tov_sb_is_c_sheet_k2_squared_and_k9_holds() {
    let (_, n) = matters();
    let (m_ch, m_tov) = (item("m_ch_sb"), item("m_tov_sb"));
    let want = c_sheet(100) * k2_prime(n).powi(2) / (G * G);
    println!("SR-CORE (a) neutron: m_tov_sb {m_tov:.3}, c_sheet K₂ₙ²/G² {want:.3}, m_tov/m_ch {:.4}", m_tov / m_ch);
    near(m_tov, want, 0.005, "m_tov_sb against c_sheet K₂ₙ²/G²");
    assert_eq!(item("ns_sigma_max_sb"), m_tov, "ns_sigma_max_sb is m_tov_sb per cell (area 1)");
    assert!(m_ch < m_tov && (1.4..=2.0).contains(&(m_tov / m_ch)), "K9: m_ch < m_tov, ratio {} in [1.4, 2.0]", m_tov / m_ch);
    assert!(physics().get("sigma_bh").unwrap() >= m_tov, "§2.6: Σ_BH reaches ns_sigma_max_sb");
    check_sequence("neutron", n, m_tov);
}

// ---- (b) the ceiling ----

/// M(Σ_c) of `fluid` on the sheet, the disk of central Σ `sigma_c`.
fn disk(sheet: &Sheet, fluid: &Fluid, sigma_c: f64) -> Equilibrium {
    sheet.fixed_central(fluid, sigma_c).unwrap_or_else(|e| panic!("Σ_c = {sigma_c:e}: {e}"))
}

/// The equilibrium of mass `target` (within 10⁻⁴), the central Σ bisected in ln over `[lo, hi]`; `Err(m)` when `target` is
/// above the heaviest disk in range (`m`) or under the lightest.
fn disk_of_mass(sheet: &Sheet, fluid: &Fluid, target: f64, lo: f64, hi: f64) -> Result<Equilibrium, f64> {
    let (mut a, mut b) = (lo.ln(), hi.ln());
    let (top, bottom) = (disk(sheet, fluid, hi).mass, disk(sheet, fluid, lo).mass);
    if top < target {
        return Err(top);
    }
    if bottom > target {
        return Err(bottom);
    }
    for _ in 0..80 {
        let eq = disk(sheet, fluid, (0.5 * (a + b)).exp());
        if (eq.mass / target - 1.0).abs() < 1e-4 {
            return Ok(eq);
        }
        if eq.mass < target {
            a = 0.5 * (a + b);
        } else {
            b = 0.5 * (a + b);
        }
    }
    panic!("the bisection for M = {target} did not close");
}

#[test]
fn an_equilibrium_exists_at_095_m_ch_sb_and_none_at_11() {
    let (co, _) = matters();
    let m_ch = item("m_ch_sb");
    let (sheet, fluid) = (Sheet::new(100), cold_fluid(co));
    // A fresh scan at half decades Σ_c = 10⁰ … 10⁸ (not the sequence's decades, not its annuli): M rises and stays under 1.1 m_ch.
    let scan: Vec<f64> = (0..=16).map(|k| disk(&sheet, &fluid, 10f64.powf(k as f64 / 2.0)).mass).collect();
    let top = scan.iter().copied().fold(0.0, f64::max);
    println!("SR-CORE (b) scan of 17 half decades: M from {:.2} to {top:.2}; m_ch_sb {m_ch:.2}", scan[0]);
    assert!(scan.windows(2).all(|w| w[1] > w[0]), "M rises at every half decade: {scan:?}");
    assert!(top < 1.1 * m_ch, "the scan reaches {top}, above 1.1 m_ch_sb = {}", 1.1 * m_ch);
    assert!(top <= 1.005 * m_ch, "a ceiling is a ceiling: the scan reaches {top} over m_ch_sb {m_ch}");

    let eq = disk_of_mass(&sheet, &fluid, 0.95 * m_ch, 1.0, 1e8).unwrap_or_else(|m| panic!("no equilibrium at 0.95 m_ch_sb = {}: the range holds {m}", 0.95 * m_ch));
    println!("SR-CORE (b) 0.95 m_ch_sb = {:.2}: Σ_c {:.4e}, R₉₀ {:.4} cells, virial {:.1e}", 0.95 * m_ch, eq.sigma_c, eq.r90, eq.virial());
    assert!(eq.virial().abs() <= 1e-3 && eq.r90 > 0.0);
    match disk_of_mass(&sheet, &fluid, 1.1 * m_ch, 1.0, 1e8) {
        Err(heaviest) => {
            println!("SR-CORE (b) 1.1 m_ch_sb = {:.2}: none; the heaviest disk to Σ_c 10⁸ is {heaviest:.2}", 1.1 * m_ch);
            assert!(heaviest < 1.1 * m_ch);
        }
        Ok(eq) => panic!("an equilibrium of 1.1 m_ch_sb exists: M = {} at Σ_c = {:e}", eq.mass, eq.sigma_c),
    }
}

// ---- (c) the pure Σ² law ----

#[test]
fn without_the_relativistic_branch_r90_is_flat_in_mass() {
    let (co, _) = matters();
    let m_ch = item("m_ch_sb");
    let vac = physics().get("sigma_vac").unwrap();
    // K₁′ = K₁ₑ Y_e²: the non-relativistic branch alone, Π = K₁′Σ² (SM-E4: for D = 2 the radius does not depend on mass).
    let k1 = physics().get("k1e").unwrap() * co.y_e * co.y_e;
    let sheet = Sheet::new(sheet::ANNULI);
    let pure = Fluid::new(move |s| k1 * s * s, vac);
    let r90: Vec<f64> = [0.25, 0.5, 1.0]
        .iter()
        .map(|&share| {
            let eq = disk_of_mass(&sheet, &pure, share * m_ch, 1e-2, 1e6).unwrap_or_else(|m| panic!("no pure-Σ² disk of {share} m_ch_sb: range ends at {m}"));
            near(eq.mass, share * m_ch, 1e-3, "the pure-law disk's mass");
            eq.r90
        })
        .collect();
    println!("SR-CORE (c) pure Σ²: R₉₀ at ¼, ½, 1 × m_ch_sb = {r90:?} cells");
    assert!(r90.iter().all(|r| (r / r90[2] - 1.0).abs() < 0.03), "R₉₀ is not flat within 3 %: {r90:?}");
    // The same masses on the full law: the radius falls by a large factor, so a flat reading is the law's, not the check's.
    let full = cold_fluid(co);
    let small = disk_of_mass(&sheet, &full, 0.25 * m_ch, 1.0, 1e8).expect("¼ m_ch_sb is in range");
    let large = disk_of_mass(&sheet, &full, 0.95 * m_ch, 1.0, 1e8).expect("0.95 m_ch_sb is in range");
    println!("SR-CORE (c) full law: R₉₀ {:.3} cells at ¼ m_ch_sb, {:.3} at 0.95", small.r90, large.r90);
    assert!(small.r90 > 3.0 * large.r90, "with the relativistic branch R₉₀ must fall: {} → {}", small.r90, large.r90);
}

// ---- (d) the resolved hold ----

/// M = 100 C/O, k1e × 6 — contract §5.1 (d); R₉₀ 18.8 cells.
const DISK_MASS: f64 = 100.0;
const K1E_FACTOR: f64 = 6.0;
/// A dump every this many steps.
const DUMP_EVERY: u64 = 50;
const WORLD: usize = 64;

fn overrides() -> Vec<(String, f64)> {
    vec![("k1e".to_string(), K1E_FACTOR * physics().get("k1e").unwrap())]
}

/// The grid's SCF disk, rebuilt: the scene and its planes.
fn rebuilt() -> &'static (Scene, Vec<f32>) {
    static DISK: OnceLock<(Scene, Vec<f32>)> = OnceLock::new();
    DISK.get_or_init(|| {
        cold_disk_scene(DISK_MASS, Matter::CarbonOxygen, &physics(), &elements(), &overrides())
            .expect("the SCF runs")
            .expect("a disk of 100 settles")
    })
}

/// The scene file's text: one 1 × 1 rect per cell, as `cold_disk_scene` lays the disk down.
fn scene_text(scene: &Scene) -> String {
    let t_floor = physics().get("t_floor").unwrap();
    let mut lines = Vec::new();
    for object in &scene.objects {
        let Object::Rect { x0, y0, w: 1, h: 1, sigma, .. } = object else { panic!("cold_disk_scene lays down 1 × 1 rects: {object:?}") };
        lines.push(format!(
            "   {{\"kind\": \"rect\", \"x0\": {x0}, \"y0\": {y0}, \"w\": 1, \"h\": 1, \"sigma\": {sigma:?}, \"t\": {t_floor:?}, \"composition\": {{\"C\": 0.5, \"O\": 0.5}}, \"v\": [0, 0]}}"
        ));
    }
    let k1e = scene.overrides[0].1;
    format!(
        "{{\"version\": 1,\n \"world\": {{\"width\": {WORLD}, \"height\": {WORLD}, \"edge\": \"bounce\"}},\n \"slowdown\": false,\n \"switches\": {{\"gravity\": true, \"hydro\": true, \"heat\": false, \"reactions\": false, \"neutrinos\": false, \"sinks\": false}},\n \"objects\": [\n{}\n ],\n \"overrides\": {{\"k1e\": {k1e:?}}}}}\n",
        lines.join(",\n")
    )
}

/// Whether two scenes are the same disk: everything equal but the floats (σ, T, the overrides), which JSON may return a last digit off.
fn same_disk(a: &Scene, b: &Scene) -> Result<(), String> {
    if (a.world, a.edge, a.slowdown, a.switches, a.test) != (b.world, b.edge, b.slowdown, b.switches, b.test) {
        return Err("the world, edge or switches differ".to_string());
    }
    let same_overrides = a.overrides.len() == b.overrides.len() && a.overrides.iter().zip(&b.overrides).all(|(p, q)| p.0 == q.0 && (p.1 / q.1 - 1.0).abs() < 1e-12);
    if !same_overrides {
        return Err(format!("the overrides differ: {:?} against {:?}", a.overrides, b.overrides));
    }
    if a.objects.len() != b.objects.len() {
        return Err(format!("{} objects against {}", a.objects.len(), b.objects.len()));
    }
    for (i, (p, q)) in a.objects.iter().zip(&b.objects).enumerate() {
        match (p, q) {
            (Object::Rect { x0, y0, w, h, sigma, gas }, Object::Rect { x0: x1, y0: y1, w: w1, h: h1, sigma: s1, gas: g1 }) => {
                let same_gas = |a: &Gas, b: &Gas| match (a.heat, b.heat) {
                    (Heat::T(t), Heat::T(u)) => (t / u - 1.0).abs() < 1e-12 && a.composition == b.composition && a.v == b.v,
                    _ => false,
                };
                if (x0, y0, w, h) != (x1, y1, w1, h1) || (sigma / s1 - 1.0).abs() > 1e-12 || !same_gas(gas, g1) {
                    return Err(format!("object {i}: {p:?} against {q:?}"));
                }
            }
            _ => return Err(format!("object {i} is not a rect on both sides")),
        }
    }
    Ok(())
}

/// scenes/cold_co_disk.json, checked against the rebuilt disk: stale is a failure, never a silent regeneration
/// (`SR_WRITE_SCENES=1` rewrites it first, the one way to refresh it).
fn scene_file() -> &'static PathBuf {
    static FILE: OnceLock<PathBuf> = OnceLock::new();
    FILE.get_or_init(|| {
        let path = scenes_dir().join("cold_co_disk.json");
        let (scene, _) = rebuilt();
        if std::env::var_os("SR_WRITE_SCENES").is_some() {
            std::fs::write(&path, scene_text(scene)).expect("write scenes/cold_co_disk.json");
        }
        let loaded = Scene::load(&path).unwrap_or_else(|e| panic!("{e} — SR_WRITE_SCENES=1 bash tests/gpu.sh cold_core physics writes it"));
        if let Err(why) = same_disk(&loaded, scene) {
            panic!("scenes/cold_co_disk.json is stale ({why}) — SR_WRITE_SCENES=1 bash tests/gpu.sh cold_core physics rewrites it");
        }
        path
    })
}

#[test]
fn the_scene_file_is_the_current_grid_disk() {
    let (scene, planes) = rebuilt();
    let path = scene_file();
    let loaded = Scene::load(path).unwrap();
    // The file's planes are the rebuilt ones to the last f32 bit, or the hold below starts from some other disk.
    let (physics, elements) = (loaded.physics().unwrap(), elements());
    let (from_file, warnings) = loaded.planes(loaded.world, &physics, &elements).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let differing = from_file.iter().zip(planes).filter(|(a, b)| a.to_bits() != b.to_bits()).count();
    let worst = from_file.iter().zip(planes).map(|(a, b)| (f64::from(*a) - f64::from(*b)).abs() / (f64::from(*b).abs() + 1e-30)).fold(0.0, f64::max);
    println!("SR-CORE (d) scene file: {} rects, planes differ in {differing} of {} values (worst relative {worst:.1e})", loaded.objects.len(), planes.len());
    assert!(worst < 1e-6, "the file's planes are not the rebuilt disk's: {worst:e}");
    let sigma = &planes[..WORLD * WORLD];
    let mass: f64 = sigma.iter().map(|&s| f64::from(s)).sum();
    near(mass, DISK_MASS, 1e-6, "the disk's mass");
    let r90 = radius_holding(sigma, WORLD, physics.get("sigma_vac").unwrap(), 0.9);
    assert!((18.0..19.6).contains(&r90), "R₉₀ {r90} cells (contract §5.1 reads 18.8)");
    assert_eq!(scene.edge, EdgeMode::Bounce);
    assert!(scene.switches.gravity && scene.switches.hydro && !scene.switches.heat && !scene.switches.reactions && !scene.switches.sinks);
    assert_eq!(scene.overrides.len(), 1);
}

/// R₉₀ and the mass of a dump's Σ plane.
fn r90_of(planes: &[f32]) -> (f64, f64) {
    let sigma = &planes[..WORLD * WORLD];
    (radius_holding(sigma, WORLD, physics().get("sigma_vac").unwrap(), 0.9), sigma.iter().map(|&s| f64::from(s)).sum())
}

/// The run's dumps in step order: (step, sim time, R₉₀, mass).
fn series(dir: &Path) -> Vec<(u64, f64, f64, f64)> {
    let mut dumps: Vec<(u64, f64, f64, f64)> = std::fs::read_dir(dir.join("dumps"))
        .expect("the run's dumps")
        // Each dump's observers' snapshot sits beside it (M0-D76).
        .filter(|e| e.as_ref().unwrap().path().extension().is_some_and(|x| x == "bin"))
        .map(|e| {
            let d = Dump::read(&e.unwrap().path()).expect("a dump");
            let (r90, mass) = r90_of(&d.planes);
            (d.step, d.sim_time, r90, mass)
        })
        .collect();
    dumps.sort_by_key(|d| d.0);
    dumps
}

fn run_headless(args: &[String]) {
    let code = headless::run(args);
    assert_eq!(code, 0, "headless {args:?}");
}

#[test]
fn the_grid_holds_a_resolved_cold_disk_on_the_gpu_and_the_twin() {
    // What this grades is the grid's statics (gravity's source term against the pressure) and its hydro on a disk 18.8
    // cells wide. It does NOT grade the ceiling: that is a continuum limit (R → 0) no grid run reaches (M0-R4, M6).
    let scene = scene_file().display().to_string();
    let (_, start) = rebuilt();
    let (r0, mass0) = r90_of(start);
    let tau_dyn = (r0.powi(3) / (G * mass0)).sqrt();
    let horizon = 10.0 * tau_dyn;
    println!("SR-CORE (d) start: R₉₀ {r0:.4} cells, M {mass0:.4}, τ_dyn {tau_dyn:.4}, 10 τ_dyn = {horizon:.3}");

    let gpu_dir = scratch("gpu");
    let mut args: Vec<String> = ["--scene", &scene, "--until", &format!("sim_time:{horizon}"), "--dump-every", &DUMP_EVERY.to_string(), "--out", &gpu_dir.display().to_string()]
        .iter()
        .map(|s| s.to_string())
        .collect();
    if let Some(adapter) = std::env::var("SR_TEST_ADAPTER").ok().filter(|s| !s.trim().is_empty()) {
        args.extend(["--adapter".to_string(), adapter]);
    }
    run_headless(&args);
    let summary: Value = serde_json::from_slice(&std::fs::read(gpu_dir.join("summary.json")).unwrap()).unwrap();
    let steps = summary["steps"].as_u64().expect("the GPU run's steps");
    println!("SR-CORE (d) GPU {}: {steps} steps to sim time {:.3}", summary["adapter"]["name"], summary["sim_time"].as_f64().unwrap());

    // The twin takes the same number of steps: it has no --until.
    let twin_dir = scratch("twin");
    run_headless(&["--scene", &scene, "--steps", &steps.to_string(), "--dump-every", &DUMP_EVERY.to_string(), "--out", &twin_dir.display().to_string(), "--cpu-reference"].map(String::from));

    let (gpu, twin) = (series(&gpu_dir), series(&twin_dir));
    assert_eq!(gpu.len(), twin.len(), "the two runs dump at the same steps");
    assert!(gpu.len() >= 5 && gpu.last().unwrap().0 == steps, "dumps from step {DUMP_EVERY} to the end, {} of them", gpu.len());
    let mut worst = [0.0f64; 2];
    for (g, t) in gpu.iter().zip(&twin) {
        assert_eq!(g.0, t.0, "dump steps");
        worst[0] = worst[0].max((g.2 / r0 - 1.0).abs());
        worst[1] = worst[1].max((t.2 / r0 - 1.0).abs());
        println!("SR-CORE (d) step {:>6}: R₉₀ GPU {:.4} (t {:.3}), twin {:.4} (t {:.3}); mass {:.4} / {:.4}", g.0, g.2, g.1, t.2, t.1, g.3, t.3);
        near(g.3, mass0, 1e-4, "the GPU's mass (bounce edges: none leaves)");
        near(t.3, mass0, 1e-4, "the twin's mass (bounce edges: none leaves)");
    }
    let (g_end, t_end) = (gpu.last().unwrap(), twin.last().unwrap());
    println!("SR-CORE (d) largest |R₉₀/R₀ − 1|: GPU {:.4}, twin {:.4} (M0-R4: 0.0162, 0.0165; bound 0.03); final GPU/twin {:.5}", worst[0], worst[1], g_end.2 / t_end.2);
    assert!(g_end.1 >= horizon, "the GPU run covers 10 τ_dyn: sim time {} of {horizon}", g_end.1);
    assert!(worst[0] <= 0.03, "the GPU disk's R₉₀ moved {:.4} of its start", worst[0]);
    assert!(worst[1] <= 0.03, "the twin disk's R₉₀ moved {:.4} of its start", worst[1]);
    near(g_end.2, t_end.2, 0.01, "the GPU's final R₉₀ against the twin's");
    let _ = (std::fs::remove_dir_all(gpu_dir), std::fs::remove_dir_all(twin_dir));
}
