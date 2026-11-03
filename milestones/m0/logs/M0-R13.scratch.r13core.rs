//! M0-R13 scratch — never in the tree. Dumps → the remnant core's heat, one line per dump (f64 through the CPU EOS).
//! Args: <dump>... ; env R13_SB=<kelvins,...> prints the sandbox T of each reading and the ε_th/ε_cold it means at the core.
use sr_engine::dump::Dump;
use sr_physics::calibration::Calibration;
use sr_physics::eos::Eos;
use sr_physics::observe::translate::TempScale;
use sr_physics::registry::{Elements, Physics, N_SPECIES};
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (physics, elements) = (Physics::shipped().unwrap(), Elements::shipped().unwrap());
    let cal = Calibration::parse(&std::fs::read_to_string("assets/calibration.json").unwrap()).unwrap();
    let k = |key: &str| physics.get(key).unwrap();
    let scale =
        TempScale::new([k("t_floor"), cal.t_eff_sun_sb, k("t_h"), k("t_he"), k("t_c"), k("t_ne"), k("t_o"), k("t_si")]).unwrap();
    let eos = Eos::from_physics(&physics).unwrap();
    println!("# anchors t_floor {} t_eff_sun_sb {} m_ch_sb {} m_tov? k2n {}", k("t_floor"), cal.t_eff_sun_sb, cal.m_ch_sb, k("k2n"));
    for path in &args[1..] {
        let dump = Dump::read(Path::new(path)).expect("dump");
        let (w, h) = (dump.world.width as usize, dump.world.height as usize);
        let n = w * h;
        let p = &dump.planes;
        let v = |ch: usize, c: usize| f64::from(p[ch * n + c]);
        let c0 = (0..n).max_by(|&a, &b| v(0, a).total_cmp(&v(0, b))).unwrap();
        let (cx, cy) = ((c0 % w) as f64, (c0 / w) as f64);
        let cell = |c: usize| {
            let sigma = v(0, c).max(1e-30);
            let x: [f64; N_SPECIES] = std::array::from_fn(|i| v(4 + i, c));
            let comp = elements.composition(&x);
            let u2 = (v(1, c).powi(2) + v(2, c).powi(2)) / (sigma * sigma);
            let cold = eos.eps_cold(sigma, comp.y_e, x[9]);
            let eth = v(3, c) / sigma - 0.5 * u2 - cold;
            let floor = eos.t_floor() / comp.inv_mu;
            (sigma, eth, cold, floor, comp.inv_mu, comp.y_e, x, 0.5 * u2 * sigma)
        };
        let held = dump.held.as_ref().map_or(0, |m| m.iter().filter(|&&b| b != 0).count());
        let (sc, ethc, coldc, _, imu, yec, xc, _) = cell(c0);
        let tc = (ethc * imu).max(eos.t_floor());
        // core: Σ ≥ 10; remnant: r ≤ 8 of the centre, Σ ≥ 1.
        let (mut nc, mut mc, mut hc, mut hcf, mut ccold, mut nfl, mut tkmax) = (0usize, 0.0, 0.0, 0.0, 0.0, 0usize, 0.0f64);
        let (mut mr, mut hr, mut cr, mut ker) = (0.0, 0.0, 0.0, 0.0);
        let mut rm: Vec<(f64, f64)> = Vec::new();
        for c in 0..n {
            let (s, eth, cold, floor, imu, _, _, ke) = cell(c);
            let r = (((c % w) as f64 - cx).powi(2) + ((c / w) as f64 - cy).powi(2)).sqrt();
            if s >= 10.0 {
                nc += 1;
                mc += s;
                hc += s * eth;
                hcf += s * (eth - floor);
                ccold += s * cold;
                if eth <= floor * 1.0001 {
                    nfl += 1;
                }
                tkmax = tkmax.max(scale.to_kelvin((eth * imu).max(eos.t_floor())));
            }
            if r <= 8.0 && s >= 1.0 {
                mr += s;
                hr += s * eth;
                cr += s * cold;
                ker += ke;
            }
            if r <= 20.0 && s >= 1e-3 {
                rm.push((r, s));
            }
        }
        rm.sort_by(|a, b| a.0.total_cmp(&b.0));
        let tot: f64 = rm.iter().map(|q| q.1).sum();
        let (mut acc, mut r90) = (0.0, 0.0);
        for (r, s) in &rm {
            acc += s;
            if acc >= 0.9 * tot {
                r90 = *r;
                break;
            }
        }
        println!(
            "step {} t {:.3} held {held} c ({},{}) Sc {sc:.1} Tc_sb {tc:.3} Tc_K {:.3e} eth/cold_c {:.2e} Ye_c {yec:.3} Xn_c {:.3} XFe_c {:.3} | core n {nc} M {mc:.1} Hth {hc:.4e} Habove {hcf:.4e} Ecold {ccold:.4e} at_floor {nfl} TKmax {tkmax:.3e} | rem M {mr:.1} Hth {hr:.4e} Ecold {cr:.4e} KE {ker:.4e} R90(r<=20) {r90:.2}",
            dump.step, dump.sim_time, cx, cy, scale.to_kelvin(tc), ethc / coldc, xc[9], xc[8]
        );
    }
    if let Ok(list) = std::env::var("R13_SB") {
        for kv in list.split(',') {
            let kelv: f64 = kv.parse().unwrap();
            println!("# {kelv:.3e} K ↔ T_sb {:.4}", scale.to_sb(kelv));
        }
    }
}
