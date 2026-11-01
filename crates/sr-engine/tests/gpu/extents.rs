//! M0-T127a — «Fit to view»'s histogram (contract §1.11 [M0-R12]; crates/sr-engine/src/render.rs `Renderer::fit_range`,
//! `extent_from_bins`, shaders/render.wgsl `extent_bins`) on the physics adapter (R12), graded against a CPU histogram of the
//! same worlds written out here from the contract's text: the quantity of every cell with Σ ≥ Σ_vac (log₁₀ T_K through the EOS and
//! `TempScale` in f64 from the f32 planes the GPU holds, or log₁₀ Σ), the 0.5th percentile read by sorting them, the bins' edges
//! `lo + w · b` — never the shader's arithmetic. The worlds are hand-made, 72 × 64 (§2.8 asks multiples of 8; not square, so a
//! transposed index shows) — `NOT PROVEN (synthetic)` for a star's own extent.
//!
//! The hand-built worlds put each cell at the *centre* of a chosen bin, so a range read from centres instead of edges, or one bin
//! off, differs by half a bin or a whole one; the disk's decisive cells are checked to sit clear of a bin edge, where the f32
//! shader and the f64 oracle could part.

use sr_engine::gpu::Gpu;
use sr_engine::render::{extent_from_bins, Renderer, View, DENSITY_SPAN, EXTENT_BINS, HEAT_SPAN};
use sr_engine::state::{State as GpuState, WorldConfig};
use sr_engine::step::EosGpu;
use sr_physics::calibration::{physics_hash, Calibration};
use sr_physics::eos::Eos;
use sr_physics::observe::translate::TempScale;
use sr_physics::registry::{Elements, Physics, N_SPECIES};

const W: usize = 72;
const H: usize = 64;
const CELLS: usize = W * H;
const TOL: f64 = 1e-9;

#[derive(Clone)]
struct Spec {
    sigma: f64,
    t: f64,
}

fn gas(sigma: f64, t: f64) -> Spec {
    Spec { sigma, t }
}

struct Rig {
    gpu: Gpu,
    renderer: Renderer,
    eos: Eos,
    elements: Elements,
    scale: TempScale,
    sigma_vac: f64,
    planes: Vec<f32>,
}

fn idx(x: usize, y: usize) -> usize {
    y * W + x
}

/// The planes of a world holding `cells` over the floor state: hydrogen at rest.
fn planes(physics: &Physics, elements: &Elements, cells: &[(usize, Spec)]) -> Vec<f32> {
    let eos = Eos::from_physics(physics).unwrap();
    let mut p = vec![0f32; 14 * CELLS];
    let mut put = |c: usize, s: &Spec| {
        let mut x = [0.0; N_SPECIES];
        x[0] = 1.0;
        let comp = elements.composition(&x);
        let e = s.sigma * (s.t * comp.inv_mu + eos.eps_cold(s.sigma, comp.y_e, x[9]));
        for (k, v) in [s.sigma, 0.0, 0.0, e].into_iter().chain(x).enumerate() {
            p[k * CELLS + c] = v as f32;
        }
    };
    let floor = gas(physics.get("sigma_floor").unwrap(), physics.get("t_floor").unwrap());
    for c in 0..CELLS {
        put(c, &floor);
    }
    for (c, s) in cells {
        put(*c, s);
    }
    p
}

fn setup(cells: &[(usize, Spec)]) -> Rig {
    let (physics, elements) = (Physics::shipped().unwrap(), Elements::shipped().unwrap());
    let p = planes(&physics, &elements, cells);
    let gpu = super::device();
    let world = WorldConfig::new(W as u32, H as u32).unwrap();
    let (cs, ps) = (GpuState::new(&gpu.device, world).unwrap(), GpuState::new(&gpu.device, world).unwrap());
    cs.upload(&gpu.queue, &p).unwrap();
    ps.upload(&gpu.queue, &p).unwrap();
    let eos_gpu = EosGpu::new(&gpu.device, &physics, &elements);
    let cal = Calibration::test_calibration(&physics_hash().expect("the shipped assets are JSON"));
    let k = |key: &str| physics.get(key).unwrap();
    let anchors = [k("t_floor"), cal.t_eff_sun_sb, k("t_h"), k("t_he"), k("t_c"), k("t_ne"), k("t_o"), k("t_si")];
    let renderer = Renderer::new(&gpu.device, &cs, &ps, &eos_gpu, anchors, &elements);
    Rig {
        eos: Eos::from_physics(&physics).unwrap(),
        scale: TempScale::new(anchors).unwrap(),
        sigma_vac: physics.get("sigma_vac").unwrap(),
        gpu,
        renderer,
        elements,
        planes: p,
    }
}

fn span(view: View) -> [f64; 2] {
    match view {
        View::Heat => HEAT_SPAN,
        View::Density => DENSITY_SPAN,
        _ => unreachable!("only Heat and Density have a range"),
    }
}

fn bin_width(view: View) -> f64 {
    let s = span(view);
    (s[1] - s[0]) / EXTENT_BINS as f64
}

/// The lower edge of bin `b`, in the view's log₁₀ units.
fn edge(view: View, b: usize) -> f64 {
    span(view)[0] + bin_width(view) * b as f64
}

/// The middle of bin `b`.
fn centre(view: View, b: usize) -> f64 {
    edge(view, b) + 0.5 * bin_width(view)
}

/// A cell whose quantity sits at the centre of bin `b` of `view`'s histogram: gas at Σ = 0.01 and the temperature that
/// translates to the centre (Heat), or the Σ of the centre at T = 1 (Density).
fn at_bin(rig_probe: &Rig, view: View, b: usize) -> Spec {
    match view {
        View::Heat => gas(0.01, rig_probe.scale.to_sb(10f64.powf(centre(view, b)))),
        _ => gas(10f64.powf(centre(view, b)), 1.0),
    }
}

impl Rig {
    fn fit(&self, view: View) -> Option<[f64; 2]> {
        self.renderer.fit_range(&self.gpu.queue, view)
    }

    fn cell_quantity(&self, view: View, c: usize) -> (f64, f64) {
        let v = |k: usize| f64::from(self.planes[k * CELLS + c]);
        let (sigma, energy) = (v(0), v(3));
        let x: [f64; N_SPECIES] = std::array::from_fn(|i| v(4 + i));
        let q = match view {
            View::Density => sigma.log10(),
            _ => {
                let comp = self.elements.composition(&x);
                let eth = energy / sigma - self.eos.eps_cold(sigma, comp.y_e, x[9]);
                self.scale.to_kelvin((eth / comp.inv_mu).max(self.eos.t_floor())).log10()
            }
        };
        (q, sigma)
    }

    /// The quantity of every cell with Σ ≥ Σ_vac, ascending.
    fn counted(&self, view: View) -> Vec<f64> {
        let mut q: Vec<f64> = (0..CELLS)
            .map(|c| self.cell_quantity(view, c))
            .filter(|&(_, sigma)| sigma >= self.sigma_vac)
            .map(|(q, _)| q)
            .collect();
        q.sort_by(f64::total_cmp);
        q
    }
}

/// The contract's range from the sorted quantities: the bin of the ⌈0.005 n⌉-th from below to the top bin's upper edge, 0.1 wide at
/// least about their middle, slid inside the span.
fn oracle(view: View, sorted: &[f64]) -> Option<[f64; 2]> {
    let n = sorted.len();
    if n == 0 {
        return None;
    }
    let s = span(view);
    let w = bin_width(view);
    let bin = |q: f64| (((q - s[0]) / w).floor().max(0.0) as usize).min(EXTENT_BINS - 1);
    let k = ((5 * n + 999) / 1000).max(1);
    let (mut lo, mut hi) = (edge(view, bin(sorted[k - 1])), edge(view, bin(sorted[n - 1]) + 1));
    if hi - lo < 0.1 {
        let m = 0.5 * (lo + hi);
        (lo, hi) = (m - 0.05, m + 0.05);
    }
    let shift = if lo < s[0] {
        s[0] - lo
    } else if hi > s[1] {
        s[1] - hi
    } else {
        0.0
    };
    Some([lo + shift, hi + shift])
}

fn assert_range(what: &str, got: Option<[f64; 2]>, want: Option<[f64; 2]>) {
    match (got, want) {
        (None, None) => {}
        (Some(g), Some(w)) => assert!((g[0] - w[0]).abs() < TOL && (g[1] - w[1]).abs() < TOL, "{what}: got {g:?}, want {w:?}"),
        _ => panic!("{what}: got {got:?}, want {want:?}"),
    }
}

/// A star-like disk: matter of radius 20 about the middle, T_K from 10⁷ at the centre to 10³ at the rim and Σ from 10^−0.9 to 10^−4.9,
/// log-linear in r, over vacuum; plus a ring of cells between Σ_floor and Σ_vac (Σ = 10⁻⁷) that must not count.
fn disk() -> Vec<(usize, Spec)> {
    let probe = setup(&[]);
    let (cx, cy) = (36.0, 32.0);
    let mut cells = Vec::new();
    for y in 0..H {
        for x in 0..W {
            let r = ((x as f64 - cx).powi(2) + (y as f64 - cy).powi(2)).sqrt();
            if r <= 20.0 {
                cells.push((idx(x, y), gas(10f64.powf(-0.9 - 4.0 * r / 20.0), probe.scale.to_sb(10f64.powf(7.0 - 4.0 * r / 20.0)))));
            } else if r <= 22.0 {
                cells.push((idx(x, y), gas(1e-7, probe.scale.to_sb(1e6))));
            }
        }
    }
    cells
}

/// Whether `q` sits at least 2 % of a bin from every bin edge (or lies past the span, where it is clamped to the end bin).
fn clear_of_edges(view: View, q: f64) -> bool {
    let s = span(view);
    let u = (q - s[0]) / bin_width(view);
    u < 0.0 || u >= EXTENT_BINS as f64 || (0.02..=0.98).contains(&u.fract())
}

/// The disk against the CPU histogram, Heat and Density — and the same answer asked twice (the counters start at zero each time).
#[test]
fn a_star_like_disk_fits_the_cpu_histogram() {
    let rig = setup(&disk());
    for view in [View::Heat, View::Density] {
        let sorted = rig.counted(view);
        let n = sorted.len();
        assert!(n > 1000 && n < CELLS / 2, "{view:?}: {n} cells count — the disk, not the vacuum");
        let k = ((5 * n + 999) / 1000).max(1);
        for r in (k.saturating_sub(3)..(k + 2).min(n)).chain(n - 3..n) {
            assert!(clear_of_edges(view, sorted[r]), "{view:?}: the decisive cell of rank {r} sits on a bin edge ({}) — reseed the world", sorted[r]);
        }
        let want = oracle(view, &sorted);
        assert_range(&format!("{view:?}"), rig.fit(view), want);
        assert_range(&format!("{view:?} again"), rig.fit(view), want);
        let [lo, hi] = want.unwrap();
        assert!(hi - lo > 1.0, "{view:?}: the disk's range {lo:.3}–{hi:.3} is not a trivial one");
    }
}

/// A one-cell world: the cell's bin edges, widened to 0.1 about their middle; a cell at the temperature floor (the span's lower
/// end) gives [1.0, 1.1], the window slid in rather than cut short.
#[test]
fn a_one_cell_world_widens_to_a_tenth_about_its_bin() {
    let probe = setup(&[]);
    for (view, b) in [(View::Heat, 300), (View::Density, 700)] {
        let rig = setup(&[(idx(17, 9), at_bin(&probe, view, b))]);
        let c = centre(view, b);
        assert_range(&format!("{view:?} bin {b}"), rig.fit(view), Some([c - 0.05, c + 0.05]));
    }
    let rig = setup(&[(idx(5, 5), gas(0.01, 0.05))]);
    assert_eq!(rig.counted(View::Heat).len(), 1);
    assert_range("Heat at T_floor", rig.fit(View::Heat), Some([1.0, 1.1]));
}

/// No matter leaves no range: a world of floor cells, a world of cells below Σ_vac only, and the views that have no range.
#[test]
fn an_empty_world_has_no_range() {
    let probe = setup(&[]);
    let vacuum: Vec<(usize, Spec)> = (0..CELLS).step_by(7).map(|c| (c, gas(5e-7, probe.scale.to_sb(1e8)))).collect();
    for rig in [setup(&[]), setup(&vacuum)] {
        assert_eq!(rig.fit(View::Heat), None);
        assert_eq!(rig.fit(View::Density), None);
    }
    let rig = setup(&[(idx(3, 3), gas(0.01, 1.0))]);
    assert_eq!(rig.fit(View::Glow), None, "Glow has no range");
    assert_eq!(rig.fit(View::Element), None, "Element has no range");
    assert!(rig.fit(View::Heat).is_some());
}

/// Matter past the span counts in the end bin: past 10¹⁰ K the range is [9.9, 10], and Σ past 10⁴ gives [3.9, 4]; with some
/// matter inside, H is the span's end and L still the 0.5th percentile's bin.
#[test]
fn matter_past_the_span_counts_in_the_end_bin() {
    let probe = setup(&[]);
    let hot: Vec<(usize, Spec)> = (0..5).map(|i| (idx(2 + i, 3), gas(0.01, probe.scale.to_sb(1e12)))).collect();
    let rig = setup(&hot);
    assert!(rig.counted(View::Heat).iter().all(|&q| q > 10.0), "the cells are past 10¹⁰ K");
    assert_range("past 10¹⁰ K", rig.fit(View::Heat), Some([9.9, 10.0]));
    let dense: Vec<(usize, Spec)> = (0..5).map(|i| (idx(2 + i, 3), gas(1e6, 1.0))).collect();
    assert_range("Σ past 10⁴", setup(&dense).fit(View::Density), Some([3.9, 4.0]));
    // Mixed: most matter at a bin 200 centre, five cells past the top.
    let mut mixed: Vec<(usize, Spec)> = (0..CELLS - 5).map(|c| (c, at_bin(&probe, View::Heat, 200))).collect();
    mixed.extend((0..5).map(|i| (CELLS - 5 + i, gas(0.01, probe.scale.to_sb(1e12)))));
    assert_range("mixed", setup(&mixed).fit(View::Heat), Some([edge(View::Heat, 200), 10.0]));
}

/// The extreme cell in the world's last row and last column (the corner, and the two edges' far ends — cells of the last
/// workgroups) sets H: a reduction that misses the last tile, or a transposed index, reads the main bin instead.
#[test]
fn the_extreme_cell_in_the_last_row_and_column_sets_the_top() {
    let probe = setup(&[]);
    for view in [View::Heat, View::Density] {
        for (x, y) in [(W - 1, H - 1), (W - 1, 0), (0, H - 1)] {
            let mut cells: Vec<(usize, Spec)> = (0..CELLS).map(|c| (c, at_bin(&probe, view, 400))).collect();
            cells[idx(x, y)] = (idx(x, y), at_bin(&probe, view, 700));
            assert_range(&format!("{view:?} hot cell at ({x}, {y})"), setup(&cells).fit(view), Some([edge(view, 400), edge(view, 701)]));
        }
    }
}

/// The 0.5th percentile's rank: of 4,608 counted cells ⌈23.04⌉ = 24, so 23 cold cells (first row) are ignored and 24 set L.
#[test]
fn the_low_end_is_the_half_percentile_not_the_minimum() {
    let probe = setup(&[]);
    for view in [View::Heat, View::Density] {
        let c = centre(view, 400);
        // 23 cold cells: only the main bin counts, a bin narrower than 0.1, so the window about its middle.
        for (cold, want) in [(23, [c - 0.05, c + 0.05]), (24, [edge(view, 100), edge(view, 401)])] {
            let mut cells: Vec<(usize, Spec)> = (0..CELLS).map(|c| (c, at_bin(&probe, view, 400))).collect();
            for x in 0..cold {
                cells[idx(x, 0)] = (idx(x, 0), at_bin(&probe, view, 100));
            }
            assert_range(&format!("{view:?}, {cold} cold cells"), setup(&cells).fit(view), Some(want));
        }
    }
}

/// `extent_from_bins` on bins written by hand: the percentile's rank, edges not centres, the widening and the slide.
#[test]
fn the_range_from_bins_follows_the_contract() {
    let span = HEAT_SPAN;
    let w = (span[1] - span[0]) / EXTENT_BINS as f64;
    let e = |b: usize| span[0] + w * b as f64;
    let mut bins = vec![0u32; EXTENT_BINS];
    assert_eq!(extent_from_bins(&bins, span), None);
    // One cell in the first bin: [1, 1 + w] is narrower than 0.1, so the window [1, 1.1] — slid in, not cut.
    bins[0] = 1;
    assert_range("first bin", extent_from_bins(&bins, span), Some([1.0, 1.1]));
    // One cell in the last bin: [9.9, 10].
    bins = vec![0; EXTENT_BINS];
    bins[EXTENT_BINS - 1] = 1;
    assert_range("last bin", extent_from_bins(&bins, span), Some([9.9, 10.0]));
    // Wide: the edges, not the centres — bins 100 and 900.
    bins = vec![0; EXTENT_BINS];
    bins[100] = 3;
    bins[900] = 2;
    assert_range("wide", extent_from_bins(&bins, span), Some([e(100), e(901)]));
    // Rank: 1 cell in bin 30, 150 in bin 50 — n = 151, rank 1 → bin 30; with 200 in bin 50 — n = 201, rank 2 → bin 50.
    bins = vec![0; EXTENT_BINS];
    bins[30] = 1;
    bins[50] = 150;
    bins[400] = 1;
    assert_range("rank 1", extent_from_bins(&bins, span), Some([e(30), e(401)]));
    bins[50] = 200;
    assert_range("rank 2", extent_from_bins(&bins, span), Some([e(50), e(401)]));
    // Widened about the middle, inside the span: one cell in bin 500.
    bins = vec![0; EXTENT_BINS];
    bins[500] = 1;
    let mid = e(500) + 0.5 * w;
    assert_range("widened", extent_from_bins(&bins, span), Some([mid - 0.05, mid + 0.05]));
    // The Density span too.
    let dspan = DENSITY_SPAN;
    let dw = (dspan[1] - dspan[0]) / EXTENT_BINS as f64;
    assert_range("density wide", extent_from_bins(&{ let mut b = vec![0; EXTENT_BINS]; b[10] = 1; b[20] = 1; b }, dspan), Some([dspan[0] + 10.0 * dw, dspan[0] + 21.0 * dw]));
}
