//! M0-D154 scratch — never in the tree. Dumps → P5's two pass bounds per cell as heat.wgsl's `build` forms them in f32
//! (c u², (C/Δt) T²), from the dump's state (the step's start; P1–P4 move free cells before P5, never a held one).
//! Args: <dump>... — one line per dump: Δt, the f32 sums, the non-finite cells, the top cells by c u² (held or free).
use sr_engine::dump::Dump;
use sr_physics::eos::Eos;
use sr_physics::registry::{Elements, Physics, N_SPECIES};
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (physics, elements) = (Physics::shipped().unwrap(), Elements::shipped().unwrap());
    let k = |key: &str| physics.get(key).unwrap();
    let eos = Eos::from_physics(&physics).unwrap();
    let (a_r, t_floor, sigma_vac) = (k("a_r") as f32, k("t_floor") as f32, k("sigma_floor"));
    println!("# a_r {a_r} t_floor {t_floor} sigma_floor {sigma_vac} f32::MAX {:e}", f32::MAX);
    for path in &args[1..] {
        let dump = Dump::read(Path::new(path)).expect("dump");
        let (w, h) = (dump.world.width as usize, dump.world.height as usize);
        let n = w * h;
        let p = &dump.planes;
        let dt = dump.resume.dt;
        let held = |c: usize| dump.held.as_ref().is_some_and(|m| m[c] != 0);
        let (mut s_cu2, mut s_ht2) = (0f32, 0f32);
        let (mut d_cu2, mut d_ht2) = (0f64, 0f64);
        let mut bad = Vec::new();
        let mut rows = Vec::new();
        for c in 0..n {
            let sigma = p[c];
            if f64::from(sigma) < sigma_vac {
                continue;
            }
            let x: [f64; N_SPECIES] = std::array::from_fn(|i| f64::from(p[(4 + i) * n + c]));
            let comp = elements.composition(&x);
            let (s64, mx, my, e) = (f64::from(sigma), f64::from(p[n + c]), f64::from(p[2 * n + c]), f64::from(p[3 * n + c]));
            let raw = e / s64 - 0.5 * (mx * mx + my * my) / (s64 * s64) - eos.eps_cold(s64, comp.y_e, x[9]);
            let eps = raw.max(eos.t_floor() * comp.inv_mu);
            let t = (eps / comp.inv_mu) as f32;
            let cap = (s64 * comp.inv_mu) as f32;
            // heat.wgsl build(): heat = C/Δt; c = heat · 1/(4 a_r t t t); u = a_r (t t)(t t); (c u u, heat t t)
            let heat = cap / dt;
            let slope = 1.0 / (4.0 * a_r * t * t * t);
            let cc = heat * slope;
            let q = t * t;
            let u = a_r * q * q;
            let cu2 = cc * u * u;
            let ht2 = heat * t * t;
            s_cu2 += cu2;
            s_ht2 += ht2;
            let (t64, c64) = (f64::from(t), f64::from(cap) / f64::from(dt));
            d_cu2 += c64 * f64::from(a_r) * t64.powi(5) / 4.0;
            d_ht2 += c64 * t64 * t64;
            if !(cu2.is_finite() && ht2.is_finite()) {
                bad.push(format!("({},{}) {} Σ {:.4e} T {:.4e} C {:.4e} u {:.3e} c {:.3e} cu² {:e} hT² {:e}", c % w, c / w,
                    if held(c) { "HELD" } else { "free" }, sigma, t, cap, u, cc, cu2, ht2));
            }
            rows.push((f64::from(cap) / f64::from(dt) * f64::from(a_r) * t64.powi(5) / 4.0, c, sigma, t, held(c)));
        }
        rows.sort_by(|a, b| b.0.total_cmp(&a.0));
        let nheld = dump.held.as_ref().map_or(0, |m| m.iter().filter(|&&b| b != 0).count());
        println!("step {} dt {:e} held {} · f32 Σcu² {:e} Σ(C/Δt)T² {:e} · f64 {:e} {:e} · non-finite cells {}", dump.step, dt,
            nheld, s_cu2, s_ht2, d_cu2, d_ht2, bad.len());
        for b in bad.iter().take(6) {
            println!("  bad {b}");
        }
        for (v, c, sigma, t, hd) in rows.iter().take(4) {
            println!("  top ({},{}) {} cu²(f64) {v:e} Σ {sigma:.4e} T {t:.4e}", c % w, c / w, if *hd { "HELD" } else { "free" });
        }
        let nf = |v: &[f32]| v.iter().filter(|x| !x.is_finite()).count();
        let planes = p.len() / n;
        let per: Vec<usize> = (0..planes).map(|ch| nf(&p[ch * n..(ch + 1) * n])).collect();
        println!("  non-finite: planes {:?} flux {} rate {}", per, dump.flux.as_ref().map_or(0, |f| nf(f)),
            dump.rate.as_ref().map_or(0, |r| nf(r)));
        let mut ts = rows.clone();
        ts.sort_by(|a, b| b.3.total_cmp(&a.3));
        for (_, c, sigma, t, hd) in ts.iter().take(3) {
            let rate = dump.rate.as_ref().map_or(f32::NAN, |r| r[*c]);
            println!("  hottest ({},{}) {} Σ {sigma:.4e} T {t:.4e} P5-rate {rate:e}", c % w, c / w, if *hd { "HELD" } else { "free" });
        }
        if let Some(r) = dump.rate.as_ref() {
            let mut rr: Vec<(usize, f32)> = r.iter().copied().enumerate().filter(|(_, v)| v.is_finite()).collect();
            rr.sort_by(|a, b| b.1.abs().total_cmp(&a.1.abs()));
            for (c, v) in rr.iter().take(3) {
                println!("  |rate| ({},{}) {} rate {v:e} Σ {:.4e}", c % w, c / w, if held(*c) { "HELD" } else { "free" }, p[*c]);
            }
        }
    }
}
