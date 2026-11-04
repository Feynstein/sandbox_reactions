//! M0-T130 scratch — never in the tree. Dumps (in step order) → per dump: the held cells, releases and new holds since the
//! previous dump, and the mass within r ≤ 4, 8, 12, 16, 24, 32 of the max-Σ cell and of each held cell's cluster's densest cell.
use sr_engine::dump::Dump;
use std::collections::BTreeSet;
use std::path::Path;

fn main() {
    let mut prev: Option<BTreeSet<usize>> = None;
    let (mut rel_total, mut new_total) = (0usize, 0usize);
    for path in std::env::args().skip(1) {
        let d = Dump::read(Path::new(&path)).expect("dump");
        let (w, h) = (d.world.width as usize, d.world.height as usize);
        let n = w * h;
        let s = |c: usize| f64::from(d.planes[c]);
        let held: BTreeSet<usize> = d.held.as_ref().map_or(BTreeSet::new(), |m| (0..n).filter(|&i| m[i] != 0).collect());
        let (rel, new) = match &prev {
            Some(p) => (p.difference(&held).count(), held.difference(p).count()),
            None => (0, held.len()),
        };
        rel_total += rel;
        new_total += new;
        let mass_r = |c: usize, r: f64| -> f64 {
            let (cx, cy) = ((c % w) as i64, (c / w) as i64);
            let k = r as i64;
            let mut m = 0.0;
            for dy in -k..=k {
                for dx in -k..=k {
                    let (x, y) = (cx + dx, cy + dy);
                    if x >= 0 && y >= 0 && x < w as i64 && y < h as i64 && ((dx * dx + dy * dy) as f64) <= r * r {
                        m += s(y as usize * w + x as usize);
                    }
                }
            }
            m
        };
        let rs = [4.0, 8.0, 12.0, 16.0, 24.0, 32.0];
        let c0 = (0..n).max_by(|&a, &b| s(a).total_cmp(&s(b))).unwrap();
        let fmt = |c: usize| rs.iter().map(|&r| format!("{:.0}", mass_r(c, r))).collect::<Vec<_>>().join("/");
        // Held groups: held cells within 6 of one another, each group's densest.
        let mut groups: Vec<usize> = vec![];
        for &c in &held {
            let near = groups.iter().position(|&g| {
                let (dx, dy) = ((g % w) as i64 - (c % w) as i64, (g / w) as i64 - (c / w) as i64);
                dx * dx + dy * dy <= 36
            });
            match near {
                Some(k) => {
                    if s(c) > s(groups[k]) {
                        groups[k] = c;
                    }
                }
                None => groups.push(c),
            }
        }
        let gl: Vec<String> = groups.iter().map(|&g| format!("({},{}) {}", g % w, g / w, fmt(g))).collect();
        println!(
            "step {} held {} (+{new} −{rel}) · max-Σ ({},{}) Σ {:.1} M(r≤4/8/12/16/24/32) {} · held groups: {}",
            d.step, held.len(), c0 % w, c0 / w, s(c0), fmt(c0), gl.join(" | ")
        );
        prev = Some(held);
    }
    println!("total: {new_total} holds, {rel_total} releases seen between consecutive dumps");
}
