//! M0-D154 scratch — never in the tree. Dumps → a window's cells: held mark, Σ, T (CPU EOS, as heat_cells forms it), |v|,
//! P5's rate. Args: <x0> <y0> <half> <dump>...
use sr_engine::dump::Dump;
use sr_physics::eos::Eos;
use sr_physics::registry::{Elements, Physics, N_SPECIES};
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (x0, y0, half): (usize, usize, usize) = (args[1].parse().unwrap(), args[2].parse().unwrap(), args[3].parse().unwrap());
    let (physics, elements) = (Physics::shipped().unwrap(), Elements::shipped().unwrap());
    let eos = Eos::from_physics(&physics).unwrap();
    for path in &args[4..] {
        let dump = Dump::read(Path::new(path)).expect("dump");
        let (w, h) = (dump.world.width as usize, dump.world.height as usize);
        let n = w * h;
        let p = &dump.planes;
        let held = |c: usize| dump.held.as_ref().is_some_and(|m| m[c] != 0);
        let heldlist: Vec<(usize, usize)> = (0..n).filter(|&c| held(c)).map(|c| (c % w, c / w)).collect();
        println!("== step {} dt {:e} held {:?}", dump.step, dump.resume.dt, heldlist);
        for y in y0 - half..=y0 + half {
            let mut line = format!("y{y:3}");
            for x in x0 - half..=x0 + half {
                let c = y * w + x;
                let s = f64::from(p[c]);
                let xs: [f64; N_SPECIES] = std::array::from_fn(|i| f64::from(p[(4 + i) * n + c]));
                let comp = elements.composition(&xs);
                let (mx, my, e) = (f64::from(p[n + c]), f64::from(p[2 * n + c]), f64::from(p[3 * n + c]));
                let v = (mx * mx + my * my).sqrt() / s.max(1e-30);
                let raw = e / s.max(1e-30) - 0.5 * v * v - eos.eps_cold(s.max(1e-30), comp.y_e, xs[9]);
                let t = raw.max(eos.t_floor() * comp.inv_mu) / comp.inv_mu;
                line += &format!(" |{}{:8.2e} T{:8.2e} v{:7.1e}", if held(c) { "H" } else { " " }, s, t, v);
            }
            println!("{line}");
        }
        let _ = h;
    }
}
