//! M0-R12 scratch — never in the tree. A dump → the per-cell quantities every view reads, in f64 through the CPU EOS and
//! TempScale (the path of views.rs's CPU model): planes T_K, Σ, X_0..X_9 as little-endian f32, row-major; and a side JSON with
//! the species colours, Σ_vac and the engine's blackbody table (sRGB).
use sr_engine::dump::Dump;
use sr_engine::render::blackbody_table;
use sr_physics::calibration::Calibration;
use sr_physics::eos::Eos;
use sr_physics::observe::translate::TempScale;
use sr_physics::registry::{Elements, Physics, N_SPECIES};
use std::io::Write;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (dump_path, out) = (Path::new(&args[1]), Path::new(&args[2]));
    let dump = Dump::read(dump_path).expect("dump");
    let (physics, elements) = (Physics::shipped().unwrap(), Elements::shipped().unwrap());
    let cal = Calibration::parse(&std::fs::read_to_string("assets/calibration.json").unwrap()).unwrap();
    let k = |key: &str| physics.get(key).unwrap();
    let scale =
        TempScale::new([k("t_floor"), cal.t_eff_sun_sb, k("t_h"), k("t_he"), k("t_c"), k("t_ne"), k("t_o"), k("t_si")]).unwrap();
    let eos = Eos::from_physics(&physics).unwrap();
    let n = dump.world.cells();
    let p = &dump.planes;
    let mut planes = vec![0f32; (2 + N_SPECIES) * n];
    for c in 0..n {
        let v = |ch: usize| f64::from(p[ch * n + c]);
        let (mut sigma, mut mom, mut energy) = (v(0), [v(1), v(2)], v(3));
        let mut x: [f64; N_SPECIES] = std::array::from_fn(|i| v(4 + i));
        if !(sigma > 0.0) {
            sigma = k("sigma_floor");
            mom = [0.0, 0.0];
            energy = 0.0;
            x = [0.0; N_SPECIES];
            x[0] = 1.0;
        }
        let comp = elements.composition(&x);
        let u2 = (mom[0] * mom[0] + mom[1] * mom[1]) / (sigma * sigma);
        let eth = energy / sigma - 0.5 * u2 - eos.eps_cold(sigma, comp.y_e, x[9]);
        let t = (eth / comp.inv_mu).max(eos.t_floor());
        planes[c] = scale.to_kelvin(t) as f32;
        planes[n + c] = sigma as f32;
        for i in 0..N_SPECIES {
            planes[(2 + i) * n + c] = x[i] as f32;
        }
    }
    let mut f = std::fs::File::create(out.join("cells.f32")).unwrap();
    for v in &planes {
        f.write_all(&v.to_le_bytes()).unwrap();
    }
    let species: Vec<String> = elements.species().iter().map(|s| format!("{:?}", s.color)).collect();
    let table: Vec<String> = blackbody_table().iter().map(|c| format!("[{},{},{}]", c[0], c[1], c[2])).collect();
    let anchors_k: Vec<String> =
        [10.0f64].iter().map(|v| v.to_string()).collect();
    std::fs::write(
        out.join("meta.json"),
        format!(
            "{{\"width\":{},\"height\":{},\"step\":{},\"sigma_vac\":{},\"species\":[{}],\"blackbody\":[{}],\"note\":[{}]}}",
            dump.world.width,
            dump.world.height,
            dump.step,
            k("sigma_vac"),
            species.join(","),
            table.join(","),
            anchors_k.join(",")
        ),
    )
    .unwrap();
}
