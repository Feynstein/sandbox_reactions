//! M0-T130 scratch — never in the tree. Dumps → the max-Σ object's mass (§1.9.2's form: 4×4 blocks holding a cell with
//! Σ ≥ σ_obj, 8-connected, plus sinks inside), P3's M_core (disk r ≤ 4 about the max-Σ cell), Σ_c, R₉₀(r ≤ 20), held count,
//! X_n,c, X_Fe,c, and how many Σ ≥ 10 cells have X_n ≥ 0.5.
use sr_engine::dump::Dump;
use sr_physics::reference::sinks::{disk, R_CORE};
use std::collections::VecDeque;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    for path in &args[1..] {
        let d = Dump::read(Path::new(path)).expect("dump");
        let (w, h) = (d.world.width as usize, d.world.height as usize);
        let n = w * h;
        let p = &d.planes;
        let s = |c: usize| f64::from(p[c]);
        let c0 = (0..n).max_by(|&a, &b| s(a).total_cmp(&s(b))).unwrap();
        let (cx, cy) = (c0 % w, c0 / w);
        let (bw, bh) = (w / 4, h / 4);
        let mut bm = vec![0.0f64; bw * bh];
        let mut on = vec![false; bw * bh];
        for c in 0..n {
            let b = (c / w / 4) * bw + (c % w) / 4;
            bm[b] += s(c);
            if s(c) >= 1e-3 {
                on[b] = true;
            }
        }
        let start = (cy / 4) * bw + cx / 4;
        let mut seen = vec![false; bw * bh];
        let mut q = VecDeque::from([start]);
        seen[start] = true;
        let mut m_obj = 0.0;
        let mut members = vec![];
        while let Some(b) = q.pop_front() {
            m_obj += bm[b];
            members.push(b);
            let (bx, by) = ((b % bw) as i64, (b / bw) as i64);
            for dy in -1..=1i64 {
                for dx in -1..=1i64 {
                    let (nx, ny) = (bx + dx, by + dy);
                    if nx < 0 || ny < 0 || nx >= bw as i64 || ny >= bh as i64 {
                        continue;
                    }
                    let nb = ny as usize * bw + nx as usize;
                    if on[nb] && !seen[nb] {
                        seen[nb] = true;
                        q.push_back(nb);
                    }
                }
            }
        }
        let mut m_sink = 0.0;
        for sk in &d.sinks {
            let (sx, sy) = (sk.x as usize / 4, sk.y as usize / 4);
            if sx < bw && sy < bh && seen[sy * bw + sx] {
                m_sink += sk.m;
            }
        }
        let m_core: f64 = disk(cx, cy, R_CORE, w, h).map(s).sum();
        let mut rm: Vec<(f64, f64)> = (0..n)
            .filter_map(|c| {
                let r = (((c % w) as f64 - cx as f64).powi(2) + ((c / w) as f64 - cy as f64).powi(2)).sqrt();
                (r <= 20.0 && s(c) >= 1e-3).then_some((r, s(c)))
            })
            .collect();
        rm.sort_by(|a, b| a.0.total_cmp(&b.0));
        let tot: f64 = rm.iter().map(|q| q.1).sum();
        let (mut acc, mut r90) = (0.0, 0.0);
        for (r, m) in &rm {
            acc += m;
            if acc >= 0.9 * tot {
                r90 = *r;
                break;
            }
        }
        let xn = |c: usize| f64::from(p[(4 + 9) * n + c]);
        let xfe = |c: usize| f64::from(p[(4 + 8) * n + c]);
        let n10 = (0..n).filter(|&c| s(c) >= 10.0).count();
        let n10n = (0..n).filter(|&c| s(c) >= 10.0 && xn(c) >= 0.5).count();
        let held = d.held.as_ref().map_or(0, |m| m.iter().filter(|&&b| b != 0).count());
        let m_world: f64 = (0..n).map(s).sum();

        // The 8-connected clusters of cells with Σ ≥ σ_hold (10): each cluster's mass, and those holding a neutron-rich cell.
        let mut lab = vec![usize::MAX; n];
        let mut clusters: Vec<(f64, usize, bool, bool)> = vec![]; // (mass, cells, holds X_n ≥ 0.5, contains c0)
        for c in 0..n {
            if s(c) < 10.0 || lab[c] != usize::MAX { continue; }
            let id = clusters.len();
            let (mut m, mut k, mut nr, mut has0) = (0.0, 0usize, false, false);
            let mut q = VecDeque::from([c]);
            lab[c] = id;
            while let Some(a) = q.pop_front() {
                m += s(a); k += 1; nr |= xn(a) >= 0.5; has0 |= a == c0;
                let (ax, ay) = ((a % w) as i64, (a / w) as i64);
                for dy in -1..=1i64 { for dx in -1..=1i64 {
                    let (nx, ny) = (ax + dx, ay + dy);
                    if nx < 0 || ny < 0 || nx >= w as i64 || ny >= h as i64 { continue; }
                    let b = ny as usize * w + nx as usize;
                    if s(b) >= 10.0 && lab[b] == usize::MAX { lab[b] = id; q.push_back(b); }
                }}
            }
            clusters.push((m, k, nr, has0));
        }
        let c0m = clusters.iter().find(|c| c.3).map_or(0.0, |c| c.0);
        let nrc: Vec<String> = clusters.iter().filter(|c| c.2).map(|c| format!("{:.0}/{}", c.0, c.1)).collect();
        let under = clusters.iter().filter(|c| c.2 && c.0 < 4853.17).map(|c| c.1).sum::<usize>();
        println!("   clusters Σ≥10: {} · at c0 {:.1} · neutron-rich (mass/cells) [{}] · cells in neutron-rich clusters under m_tov {}",
            clusters.len(), c0m, nrc.join(" "), under);
        println!(
            "step {} t {:.3} M_obj {:.1} (sink {:.1}, blocks {}) M_world {:.1} M_core4 {:.1} Sc {:.1} R90 {:.2} Xn_c {:.3} XFe_c {:.3} n10 {} n10_Xn {} held {} sinks {}",
            d.step, d.sim_time, m_obj + m_sink, m_sink, members.len(), m_world, m_core, s(c0), r90, xn(c0), xfe(c0), n10, n10n, held, d.sinks.len()
        );
    }
}
