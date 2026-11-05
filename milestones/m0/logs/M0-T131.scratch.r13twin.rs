//! M0-T131 scratch (M0-R13's r13twin.rs, never in the tree) — adds: <cx> <cy> = -1 centres on the max-Σ cell; a 6th arg q_ratio for N_Fe (the
//! reactions.json text with -1.14 replaced, parsed by Reactions::parse). The twin carries M0-D153's hold. M0-R13's original header follows:
//! A 128 × 128 crop of a GPU dump centred on (cx, cy), the CPU twin with every pass of
//! the Massive preset, the core's heat (Σ ≥ 10 at each step's start) tallied by pass. Args: <dump> <cx> <cy> <steps> <every>.
use sr_engine::dump::Dump;
use sr_engine::scene::Scene;
use sr_engine::state::WorldConfig;
use sr_physics::eos::Eos;
use sr_physics::reference::hydro::Edge;
use sr_physics::reference::{heat, react, Burning, Switches, Twin, R13};
use sr_physics::registry::{Elements, Reactions};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let dump = Dump::read(std::path::Path::new(&a[1])).unwrap();
    let (mut cx, mut cy): (i64, i64) = (a[2].parse().unwrap(), a[3].parse().unwrap());
    if cx < 0 {
        let n0 = dump.world.width as usize * dump.world.height as usize;
        let best = (0..n0).max_by(|&i, &j| dump.planes[i].partial_cmp(&dump.planes[j]).unwrap()).unwrap();
        (cx, cy) = ((best % dump.world.width as usize) as i64, (best / dump.world.width as usize) as i64);
    }
    let q: Option<f64> = a.get(6).map(|s| s.parse().unwrap());
    let steps: u64 = a[4].parse().unwrap();
    let every: u64 = a[5].parse().unwrap();
    let (w0, h0) = (dump.world.width as usize, dump.world.height as usize);
    let scene = Scene::preset("massive", WorldConfig::new(w0 as u32, h0 as u32).unwrap()).unwrap();
    let physics = scene.physics().unwrap();
    let elements = Elements::shipped().unwrap();
    let n = 128usize;
    let ch = dump.planes.len() / (w0 * h0);
    let mut p = vec![0.0f64; 14 * n * n];
    for c in 0..14.min(ch) {
        for y in 0..n {
            for x in 0..n {
                let (sx, sy) = (x as i64 - 64 + cx, y as i64 - 64 + cy);
                let src = if sx >= 0 && sy >= 0 && (sx as usize) < w0 && (sy as usize) < h0 { sy as usize * w0 + sx as usize } else { 0 };
                p[c * n * n + y * n + x] = f64::from(dump.planes[c * w0 * h0 + src]);
            }
        }
    }
    let sw = Switches { gravity: true, hydro: true, heat: scene.switches.heat, reactions: scene.switches.reactions };
    let text = include_str!("../../../assets/reactions.json");
    let text = match q { Some(q) => text.replace("\"q_ratio\": -1.14,", &format!("\"q_ratio\": {q:e},")), None => text.to_string() };
    assert!(q.is_none() || text.contains(&format!("{:e}", q.unwrap())), "q_ratio not replaced");
    let records = Reactions::parse(&text, &elements, &physics).unwrap();
    println!("N_Fe q_ratio {}", records.get("N_Fe").unwrap().q_ratio);
    let burning = Burning { heat: heat::TestFlags::default(), react: react::TestFlags { pin_temperature: false }, reactions: &records, neutrinos: scene.switches.neutrinos };
    let mut twin = Twin::new(&physics, &elements, Edge::Leave, sw, Some(burning), n, n, p).unwrap();
    twin.r13 = Some(R13::new(Eos::from_physics(&physics).unwrap(), elements.clone(), 10.0));
    println!("dump {} step {} crop centre ({cx},{cy}) switches {:?} neutrinos {}", a[1], dump.step, sw, scene.switches.neutrinos);
    let smax = |t: &Twin| (0..t.state.cells()).map(|i| t.state.planes[i]).fold(0.0f64, f64::max);
    let mut last = ([0.0f64; 4], 0.0f64, 0.0f64);
    for k in 1..=steps {
        twin.step();
        if k % every == 0 || k == steps {
            let r = twin.r13.as_ref().unwrap();
            let h = r.heat(&twin.state);
            let b = r.by_pass;
            let held = twin.held_mask().iter().filter(|&&m| m).count();
            println!(
                "+{k:>5} t {:.3} Sc {:.1} core n {} heat {h:.4e} | window: P4 {:+.3e} P5 {:+.3e} P6 {:+.3e} (nu {:.3e}) P8 {:+.3e} | floor_added(all) {:+.3e} held {held}",
                twin.sim_time(),
                smax(&twin),
                r.core.iter().filter(|&&c| c).count(),
                b[0] - last.0[0], b[1] - last.0[1], b[2] - last.0[2], r.nu_core - last.1, b[3] - last.0[3],
                twin.booked.floor_added - last.2
            );
            last = (b, r.nu_core, twin.booked.floor_added);
        }
        if twin.first_non_finite().is_some() {
            println!("NON-FINITE at +{k}");
            break;
        }
    }
    let r = twin.r13.as_ref().unwrap();
    println!("TOTAL P4 {:+.4e} P5 {:+.4e} P6 {:+.4e} (nu {:.4e}) P8 {:+.4e} floor_added(all) {:+.4e}", r.by_pass[0], r.by_pass[1], r.by_pass[2], r.nu_core, r.by_pass[3], twin.booked.floor_added);
}
