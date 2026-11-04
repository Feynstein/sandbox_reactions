# M0-D156 scratch probe (never in the tree): /tmp/d156/probe on laserax-ai-server = M0-D91's probe (logs/M0-D91.scratch.probe.diff: D91_TRACE dumps P5's planes after each 2-slot submission) + these test modules appended to crates/sr-engine/tests/gpu/heat.rs; /tmp/d156/probe_old the same on the pre-fix heat.wgsl (M0-D91's).
/// M0-D156's scratch probe (never in the tree): the heat-free START shift, by cell and by connected heat-free component, at every
/// RESID of a traced headless step (D156_DIR, D156_W, D156_H; today's physics).
mod d156 {
    use super::*;

    fn find(p: &mut Vec<usize>, mut i: usize) -> usize {
        while p[i] != i {
            p[i] = p[p[i]];
            i = p[i];
        }
        i
    }

    #[test]
    fn d156_shift() {
        let Ok(dir) = std::env::var("D156_DIR") else { return };
        let get = |k: &str| std::env::var(k).unwrap().parse::<usize>().unwrap();
        let (w, h) = (get("D156_W"), get("D156_H"));
        let physics = Physics::shipped().unwrap();
        let a_r = physics.get("a_r").unwrap();
        let t_floor = Eos::from_physics(&physics).unwrap().t_floor();
        let u_vac = a_r * t_floor.powi(4);
        let n = w * h;
        let nxf = (w + 1) * h;
        let mut names: Vec<_> = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).map(|e| e.file_name().into_string().unwrap()).filter(|s| s.starts_with("sub_")).collect();
        names.sort();
        let mut prev_mode = u32::MAX;
        let mut prev_x: Option<(Vec<f64>, Vec<bool>, Vec<f32>)> = None;
        let show: usize = std::env::var("D156_SHOW").ok().and_then(|s| s.parse().ok()).unwrap_or(6);
        for name in names {
            let bytes = std::fs::read(format!("{dir}/{name}")).unwrap();
            let v: Vec<f32> = bytes.chunks_exact(4).map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect();
            let faces_len = 2 * ((w + 1) * h + w * (h + 1));
            let m_len = v.len() - 16 * n - faces_len;
            let (frozen, rest) = v.split_at(5 * n);
            let (solve, rest) = rest.split_at(5 * n);
            let (krylov, rest) = rest.split_at(6 * n);
            let (mach, faces) = rest.split_at(m_len);
            let rec = |k: usize| &mach[16 * k..16 * k + 16];
            let p = if rec(0)[0].to_bits() >= rec(1)[0].to_bits() { 0 } else { 1 };
            let mode = rec(p)[1].to_bits();
            let passes = rec(p)[2].to_bits();
            let rounds = rec(p)[3].to_bits();
            let dt = f64::from(rec(p)[8]);
            let cap = &frozen[n..2 * n];
            let t0 = &frozen[0..n];
            let gas: Vec<bool> = (0..n).map(|i| cap[i] > 0.0).collect();
            let tt = &solve[n..2 * n];
            let x: Vec<f64> = (0..n).map(|i| f64::from(solve[i]) + f64::from(solve[4 * n + i])).collect();
            let nb = |i: usize| -> [(f64, Option<usize>); 4] {
                let (cx, cy) = (i % w, i / w);
                let at = |dx: isize, dy: isize| -> Option<usize> {
                    let (ox, oy) = (cx as isize + dx, cy as isize + dy);
                    if ox < 0 || oy < 0 || ox >= w as isize || oy >= h as isize {
                        return None;
                    }
                    let o = oy as usize * w + ox as usize;
                    gas[o].then_some(o)
                };
                [
                    (f64::from(faces[cy * (w + 1) + cx]), at(-1, 0)),
                    (f64::from(faces[cy * (w + 1) + cx + 1]), at(1, 0)),
                    (f64::from(faces[nxf + cy * w + cx]), at(0, -1)),
                    (f64::from(faces[nxf + (cy + 1) * w + cx]), at(0, 1)),
                ]
            };
            let heat = |i: usize| f64::from(cap[i]) / dt;
            let c = |i: usize| heat(i) / (4.0 * a_r * f64::from(tt[i]).powi(3));
            let free: Vec<bool> = (0..n).map(|i| gas[i] && c(i) <= 1e-3 * nb(i).iter().map(|f| f.0).sum::<f64>()).collect();
            // The START just applied: x moved along z by the shift.
            if mode == 3 && prev_mode == 2 {
                if let Some((px, pfree, _)) = &prev_x {
                    let moved: Vec<f64> = (0..n).filter(|&i| pfree[i]).map(|i| x[i] - px[i]).collect();
                    let (lo, hi) = moved.iter().fold((f64::MAX, f64::MIN), |a, &m| (a.0.min(m), a.1.max(m)));
                    println!("D156 START pass {passes}: free cells' x moved by [{lo:.4e}, {hi:.4e}]");
                }
            }
            // A RESID just ran (the machine in DO_START): r and zᵀr, zᵀA z by cell.
            if mode == 2 && prev_mode != 2 {
                let mut par: Vec<usize> = (0..n).collect();
                for i in (0..n).filter(|&i| free[i]) {
                    for (_, o) in nb(i) {
                        if let Some(o) = o {
                            if free[o] {
                                let (a, b) = (find(&mut par, i), find(&mut par, o));
                                if a != b {
                                    par[a.max(b)] = a.min(b);
                                }
                            }
                        }
                    }
                }
                let mut zr = vec![0.0; n];
                let mut az = vec![0.0; n];
                let mut own_r = vec![0.0; n];
                for i in (0..n).filter(|&i| free[i]) {
                    let kept = -heat(i) * (f64::from(tt[i]) - f64::from(t0[i]));
                    let mut z = kept;
                    let mut a = c(i);
                    let mut r = kept;
                    for (d, o) in nb(i) {
                        let far = o.map_or(u_vac, |o| x[o]);
                        r -= d * (x[i] - far);
                        if !o.is_some_and(|o| free[o]) {
                            z -= d * (x[i] - far);
                            a += d;
                        }
                    }
                    zr[i] = z;
                    az[i] = a;
                    own_r[i] = r;
                }
                use std::collections::BTreeMap;
                let mut comp: BTreeMap<usize, (usize, f64, f64, [usize; 4], f64, usize)> = BTreeMap::new();
                for i in (0..n).filter(|&i| free[i]) {
                    let root = find(&mut par, i);
                    let e = comp.entry(root).or_insert((0, 0.0, 0.0, [usize::MAX, usize::MAX, 0, 0], f64::MAX, 0));
                    e.0 += 1;
                    e.1 += zr[i];
                    e.2 += az[i];
                    let (cx, cy) = (i % w, i / w);
                    e.3 = [e.3[0].min(cx), e.3[1].min(cy), e.3[2].max(cx), e.3[3].max(cy)];
                    e.4 = e.4.min(f64::from(tt[i]));
                    if (x[i] - u_vac) < 2e-9 {
                        e.5 += 1;
                    }
                }
                let sz: f64 = comp.values().map(|c| c.1).sum();
                let sa: f64 = comp.values().map(|c| c.2).sum();
                let gpu_p: f64 = (0..n).filter(|&i| free[i]).map(|i| f64::from(solve[2 * n + i])).sum();
                let gpu_r: f64 = (0..n).filter(|&i| free[i]).map(|i| f64::from(krylov[p * n + i])).sum();
                let nfree = free.iter().filter(|&&f| f).count();
                println!(
                    "D156 RESID pass {} (rounds so far {rounds}): {nfree} free cells in {} components | zᵀr {sz:.4e} zᵀAz {sa:.4e} shift {:.4e} | GPU Σ_free p {gpu_p:.4e} Σ_free r {gpu_r:.4e}",
                    passes + 1,
                    comp.len(),
                    sz / sa
                );
                let mut list: Vec<_> = comp.iter().collect();
                list.sort_by(|a, b| b.1 .1.abs().total_cmp(&a.1 .1.abs()));
                for (root, c) in list.iter().take(show) {
                    println!(
                        "D156   comp@({},{}) cells {} bbox x {}–{} y {}–{} Tmin {:.3} near-floor {} | Σzr {:+.4e} ({:.1} % of zᵀr) Σaz {:.4e} own shift {:+.4e}",
                        *root % w, *root / w, c.0, c.3[0], c.3[2], c.3[1], c.3[3], c.4, c.5, c.1, 100.0 * c.1 / sz, c.2, c.1 / c.2
                    );
                }
                {
                    let aux = std::fs::read(format!("{dir}/{}", name.replace("sub_", "aux_"))).unwrap();
                    let a: Vec<f32> = aux.chunks_exact(4).map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect();
                    let (tx, ty) = (rec(p)[6].to_bits() as usize, rec(p)[7].to_bits() as usize);
                    let tiles = w.div_ceil(16) * h.div_ceil(16);
                    let mut gz = vec![0.0f64; tx * ty];
                    let mut ga = vec![0.0f64; tx * ty];
                    for t in 0..tx * ty {
                        gz[t] = f64::from(a[4 * (p * tiles + t)]);
                        ga[t] = f64::from(a[4 * (p * tiles + t) + 1]);
                    }
                    let mut hz = vec![0.0f64; tx * ty];
                    let mut ha = vec![0.0f64; tx * ty];
                    let mut habs = vec![0.0f64; tx * ty];
                    for i in (0..n).filter(|&i| free[i]) {
                        let t = (i / w / 16) * tx + (i % w) / 16;
                        if t < tx * ty {
                            hz[t] += zr[i];
                            ha[t] += az[i];
                            habs[t] += zr[i].abs();
                        }
                    }
                    let gsum: f64 = gz.iter().sum();
                    let gasum: f64 = ga.iter().sum();
                    println!("D156   GPU partials (tx {tx} ty {ty}): Σ zᵀr {gsum:.4e} Σ zᵀAz {gasum:.4e} → shift {:.4e} | host f64 {sz:.4e} / {sa:.4e}", gsum / gasum);
                    let mut d: Vec<(f64, usize)> = (0..tx * ty).map(|t| ((gz[t] - hz[t]).abs(), t)).collect();
                    d.sort_by(|a, b| b.0.total_cmp(&a.0));
                    for &(e, t) in d.iter().take(5) {
                        println!("D156     tile ({},{}) GPU zr {:+.4e} host {:+.4e} diff {e:.3e} Σ|zr_i| {:.3e} | az GPU {:.4e} host {:.4e}", t % tx, t / tx, gz[t], hz[t], habs[t], ga[t], ha[t]);
                    }
                    let mut big: Vec<(f64, usize)> = (0..n).filter(|&i| free[i]).map(|i| (zr[i].abs(), i)).collect();
                    big.sort_by(|a, b| b.0.total_cmp(&a.0));
                    for &(v, i) in big.iter().take(5) {
                        let terms: f64 = nb(i).iter().filter(|f| !f.1.is_some_and(|o| free[o])).map(|f| (f.0 * (x[i] - f.1.map_or(u_vac, |o| x[o]))).abs()).sum();
                        println!("D156     big zr cell ({},{}) zr {:+.4e} Σ|face terms| {:.3e} kept {:+.3e} T {:.3e}", i % w, i / w, zr[i], terms, -heat(i) * (f64::from(tt[i]) - f64::from(t0[i])), tt[i]);
                    }
                }
                {
                    let mut dif: Vec<(f64, usize)> = (0..n).filter(|&i| free[i]).map(|i| ((own_r[i] - f64::from(krylov[p * n + i])).abs(), i)).collect();
                    dif.sort_by(|a, b| b.0.total_cmp(&a.0));
                    let tot: f64 = dif.iter().map(|d| own_r[d.1] - f64::from(krylov[p * n + d.1])).sum();
                    println!("D156   host r − GPU r over free cells: Σ {tot:+.4e}");
                    for &(_, i) in dif.iter().take(6) {
                        let f = nb(i);
                        let fs: Vec<String> = f.iter().map(|(d, o)| format!("D {:.3e} {} far {:.4e}", d, o.map_or("vac".to_string(), |o| format!("{}{}", if free[o] { "F" } else { "N" }, if gas[o] { "" } else { "?" })), o.map_or(u_vac, |o| x[o]))).collect();
                        println!("D156     ({},{}) host r {:+.5e} GPU r {:+.5e} x {:.5e} lo {:.3e} T {:.4e} T0 {:.4e} cap {:.3e} | {}", i % w, i / w, own_r[i], krylov[p * n + i], x[i], solve[4 * n + i], tt[i], t0[i], cap[i], fs.join(" ; "));
                    }
                }
                let against = (0..n).filter(|&i| free[i] && own_r[i] * sz < 0.0).count();
                println!("D156   free cells whose own residual opposes the shift: {against} of {nfree}");
                if let Ok(cells) = std::env::var("D156_CELLS") {
                    for name in cells.split(';') {
                        let xy: Vec<usize> = name.split(',').map(|v| v.parse().unwrap()).collect();
                        let i = xy[1] * w + xy[0];
                        let root = find(&mut par, i);
                        println!(
                            "D156   cell ({},{}) free {} comp@({},{}) x {:.4e} T {:.4e} T0 {:.4e} own r {:+.4e} zr {:+.4e} az {:.4e} ΣD {:.4e} c {:.3e}",
                            xy[0], xy[1], free[i], root % w, root / w, x[i], tt[i], t0[i], own_r[i], zr[i], az[i], nb(i).iter().map(|f| f.0).sum::<f64>(), c(i)
                        );
                    }
                }
                prev_x = Some((x.clone(), free.clone(), tt.to_vec()));
            }
            prev_mode = mode;
        }
    }
}

/// M0-D156 probe 2: per pass, the GPU's own sums of the start (both pairs), the pinned and the halved cells.
mod d156b {
    use super::*;

    #[test]
    fn d156_passes() {
        let Ok(dir) = std::env::var("D156_DIR") else { return };
        let get = |k: &str| std::env::var(k).unwrap().parse::<usize>().unwrap();
        let (w, h) = (get("D156_W"), get("D156_H"));
        let physics = Physics::shipped().unwrap();
        let a_r = physics.get("a_r").unwrap();
        let t_floor = Eos::from_physics(&physics).unwrap().t_floor() as f32;
        let n = w * h;
        let tiles = w.div_ceil(16) * h.div_ceil(16);
        let mut names: Vec<_> = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).map(|e| e.file_name().into_string().unwrap()).filter(|s| s.starts_with("sub_")).collect();
        names.sort();
        let mut t_last: Option<Vec<f32>> = None;
        for name in names {
            let bytes = std::fs::read(format!("{dir}/{name}")).unwrap();
            let v: Vec<f32> = bytes.chunks_exact(4).map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect();
            let faces_len = 2 * ((w + 1) * h + w * (h + 1));
            let m_len = v.len() - 16 * n - faces_len;
            let (frozen, rest) = v.split_at(5 * n);
            let (solve, rest) = rest.split_at(5 * n);
            let (_krylov, rest) = rest.split_at(6 * n);
            let (mach, faces) = rest.split_at(m_len);
            let rec = |k: usize| &mach[16 * k..16 * k + 16];
            let mode = |k: usize| rec(k)[1].to_bits();
            // The RESID's parity: the last slot (parity 1) if the machine waits on a START, else parity 0 when parity 1 ran it.
            let q = if mode(1) == 2 { 1 } else if mode(0) == 2 && mode(1) == 3 { 0 } else { continue };
            let r = rec(q);
            let (passes, rounds, dt) = (r[2].to_bits(), r[3].to_bits(), f64::from(r[8]));
            let aux = std::fs::read(format!("{dir}/{}", name.replace("sub_", "aux_"))).unwrap();
            let a: Vec<f32> = aux.chunks_exact(4).map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect();
            let (tx, ty) = (r[6].to_bits() as usize, r[7].to_bits() as usize);
            let mut s = [0.0f64; 4];
            for t in 0..tx * ty {
                for k in 0..4 {
                    s[k] += f64::from(a[4 * (q * tiles + t) + k]);
                }
            }
            let cap = &frozen[n..2 * n];
            let tt = &solve[n..2 * n];
            let nxf = (w + 1) * h;
            let free = |i: usize| -> bool {
                if cap[i] <= 0.0 {
                    return false;
                }
                let (cx, cy) = (i % w, i / w);
                let sd = f64::from(faces[cy * (w + 1) + cx]) + f64::from(faces[cy * (w + 1) + cx + 1]) + f64::from(faces[nxf + cy * w + cx]) + f64::from(faces[nxf + (cy + 1) * w + cx]);
                f64::from(cap[i]) / dt / (4.0 * a_r * f64::from(tt[i]).powi(3)) <= 1e-3 * sd
            };
            let fr: Vec<usize> = (0..n).filter(|&i| free(i)).collect();
            let pinned = fr.iter().filter(|&&i| tt[i] <= t_floor * (1.0 + 1e-6)).count();
            let (mut halved, mut doubled, mut over) = (0, 0, 0);
            let mut tmin_halved = f32::MAX;
            if let Some(tl) = &t_last {
                for &i in &fr {
                    let q = tt[i] / tl[i];
                    if q <= 0.500001 {
                        halved += 1;
                        tmin_halved = tmin_halved.min(tl[i]);
                    }
                    if q >= 1.99999 {
                        doubled += 1;
                    }
                    if (tt[i] - tl[i]).abs() > 1e-3 * tl[i] {
                        over += 1;
                    }
                }
            }
            let reach = r[14];
            let xs: Vec<f32> = fr.iter().map(|&i| solve[i] + solve[4 * n + i]).collect();
            let umin = xs.iter().cloned().fold(f32::MAX, f32::min);
            let k4 = xs.iter().filter(|&&u| reach >= 4e-3 * u).count();
            let k25 = xs.iter().filter(|&&u| reach >= 0.25 * u).count();
            println!("D156b   reach {reach:.3e} umin {umin:.3e} | free cells out at 4e-3 u: {k4}, at u/4: {k25}");
            let s1 = s[0] / s[1];
            let sd = if s[3] > 0.0 { s[2] / s[3] } else { 0.0 };
            println!(
                "D156b pass {:>2} rounds {:>3} | free {} pinned {} | last update: halved {} (Tmin {:.3}) doubled {} over 1e-3 {} (free) | s_all {:+.3e} (zr {:+.3e} az {:.3e}) s_down {:+.3e} (zr {:+.3e} az {:.3e})",
                passes + 1, rounds, fr.len(), pinned, halved, tmin_halved, doubled, over, s1, s[0], s[1], sd, s[2], s[3]
            );
            t_last = Some(tt.to_vec());
        }
    }
}

/// M0-D156 probe 3: P5 alone (OnGpu) on a dump or a window of it (D156_CUT, D156_WIN="x0,y0,w,h"), shipped physics, the
/// dump's Δt (× D156_DTX), on both preconditioners.
mod d156c {
    use super::*;
    use sr_engine::step::heat::HeatPrecond;

    #[test]
    fn d156_cut() {
        if std::env::var("D156_CUT").is_err() { return }
        on_precond(HeatPrecond::Multigrid, || run("multigrid"));
        on_precond(HeatPrecond::Jacobi, || run("jacobi"));
    }

    fn run(precond: &str) {
        let file = std::env::var("D156_CUT").unwrap();
        let dump = Dump::read(std::path::Path::new(&file)).unwrap();
        let (w0, h0) = (dump.world.width as usize, dump.world.height as usize);
        let n0 = w0 * h0;
        let planes: Vec<f64> = dump.planes.iter().map(|&v| f64::from(v)).collect();
        let k = planes.len() / n0;
        let (x0, y0, w, h) = match std::env::var("D156_WIN") {
            Ok(s) => {
                let v: Vec<usize> = s.split(',').map(|t| t.parse().unwrap()).collect();
                (v[0], v[1], v[2], v[3])
            }
            Err(_) => (0, 0, w0, h0),
        };
        let mut cut = Vec::with_capacity(k * w * h);
        for p in 0..k {
            for y in y0..y0 + h {
                for x in x0..x0 + w {
                    cut.push(planes[p * n0 + y * w0 + x]);
                }
            }
        }
        let st = State::new(w, h, cut);
        let mut physics = Physics::shipped().unwrap();
        if std::env::var("D156_OLDPHYS").is_ok() {
            physics = physics.with(&[("a_r", 4e-5), ("c_sb", 200.0)]).unwrap();
        }
        let dtx: f64 = std::env::var("D156_DTX").ok().and_then(|s| s.parse().ok()).unwrap_or(1.0);
        let dt = f64::from(dump.resume.dt_next) * dtx;
        let gpu = OnGpu::new(&physics, TestFlags::default(), &st);
        if let Ok(rf) = std::env::var("D156_RATE") {
            let rd = Dump::read(std::path::Path::new(&rf)).unwrap();
            let src = rd.rate.expect("a rate plane");
            let mut rate = Vec::with_capacity(w * h);
            for y in y0..y0 + h {
                for x in x0..x0 + w {
                    rate.push(src[y * w0 + x]);
                }
            }
            gpu.step.set_heat_rate(&gpu.gpu.queue, Some(&rate));
        }
        let r = gpu.pass(dt);
        println!("D156c rate {} win ({x0},{y0},{w},{h}) dt {dt:e} {precond}: {r:?}", std::env::var("D156_RATE").is_ok());
    }
}

/// M0-D156 probe 5: P5 alone (OnGpu) on the scenes in D156_SCENES (a directory), at D156_DT, both preconditioners.
mod d156e {
    use super::*;
    use sr_engine::step::heat::HeatPrecond;

    #[test]
    fn d156_scene() {
        if std::env::var("D156_SCENES").is_err() {
            return;
        }
        on_precond(HeatPrecond::Multigrid, || run("multigrid"));
        on_precond(HeatPrecond::Jacobi, || run("jacobi"));
    }

    fn run(precond: &str) {
        let dir = std::env::var("D156_SCENES").unwrap();
        let dt: f64 = std::env::var("D156_DT").unwrap().parse().unwrap();
        let mut names: Vec<_> = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).collect();
        names.sort();
        for path in names {
            let scene = Scene::load(&path).unwrap();
            let (st, physics, _) = world(&scene);
            let gpu = OnGpu::new(&physics, flags(&scene), &st);
            let r = gpu.pass(dt);
            println!("D156e {} {precond}: passes {} rounds {} converged {} δ {:.2e}", path.file_name().unwrap().to_string_lossy(), r.passes, r.rounds, r.converged, r.delta);
        }
    }
}
