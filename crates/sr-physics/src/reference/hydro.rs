//! Gas flow on the CPU twin (m0_contrat.md §1.7, §2.3.3–§2.3.5, §2.8, R13): P4's f64 reference, written before its
//! shader (M0-T34 ports it, M0-T33a's exact Riemann solution grades it).
//!
//! Finite volume, unsplit, second order — MUSCL-Hancock (SM-E13): each cell's primitives (Σ, u, v, Π, X_i) are
//! reconstructed with the monotonised-central limiter into four face states; the Hancock predictor advances those by
//! half a step with the flux differences of the cell's own faces (both axes at once: unsplit); HLL fluxes with Davis
//! speeds S_L = min(u_L − c_L, u_R − c_R), S_R = max(u_L + c_L, u_R + c_R) then join the predicted states across every
//! face. Species travel with the mass flux (the upwind side's X_i), so ΣX_i = 1 is kept exactly: the species slopes of a
//! cell share one limiter factor (the smallest any species needs) and, summing to zero before it, still sum to zero.
//!
//! The pressure is the contract's total Π = Π_th + Π_e + Π_n (§2.3.3) with the cold terms of [`Eos`]: a K pair of 0
//! means none (G-SOD's `overrides`), the energy a face state carries is Σ(ε_th + ε_cold) + ½Σ|u|², and the wave speed is
//! the bound c² = 2Π/Σ. Conventions as [`fft`](super::fft): channel-major planes in §2.2's order, row-major cells,
//! `index = y * w + x`, y down.
//!
//! Edges (§2.8): *bounce* — two ghost cells per side mirror the edge cell with the normal velocity negated; the wall
//! face's mass, energy and species fluxes are exactly 0 and its momentum flux is the pressure term alone. *leave* (M0-T35)
//! — both ghosts copy the edge cell with the normal velocity clamped outward, the boundary face's flux is HLL between the
//! edge cell's predicted face and the ghost's, and a face whose mass flux points inward is a wall instead, so inflow is
//! exactly 0; whatever crosses — mass, momentum, energy, each species' mass — is booked into [`State::escaped`] from that
//! face flux, face by face in the GPU booking's layout. Bounce books nothing.
//!
//! Gravity's source (§1.4.2; M0-T36), when a step is given P2's g ([`Hydro::step_with`]): the Hancock predictor kicks
//! each face state by half a step of g — its velocity by ½Δt·g, its energy by the work Σ(u + ¼Δt·g)·g·½Δt, so the kick
//! leaves its thermal energy as it was — beside the flux differences; the update adds Δt·Σg to the momentum and
//! Δt·(Σu)·g to the energy, Σ and Σu the cell centre's half-step values from that same predictor (its flux differences
//! and its half kick), so the source is time-centred on the half-step velocity. A ghost cell's g is its edge cell's.
//!
//! The radiation force (§2.4.4; M0-T43), when a step is given the previous step's flux F ([`Hydro::step_forced`]: P5's
//! cell-centred planes, `heat::Flux::cells`): every gas cell (Σ ≥ Σ_vac) gains Δt·f of momentum, f = κΣF/c_sb from the
//! cell as the step found it (κ = κ₀(1 + X_H) plus κ_dust Z_met where T < T_dust, §2.4.2), added in the update after the
//! flux differences and gravity's source; E is left as it is, so the work comes out of the thermal energy. The predictor
//! is not kicked: F already lags a step, so the force is first order in time either way, and the one-step difference a
//! force makes is Δt·f in each cell, nothing else.
//!
//! A cell below Σ_vac is read as the floor state (Σ_floor, at rest, T_floor), as P9's rates do (M0-T20); a cell and its
//! four neighbours all below Σ_vac are left as they are, so empty space is not filled by the floor.

use crate::eos::Eos;
use crate::registry::{Elements, Physics, RegistryError, N_SPECIES};

/// Ghost cells per side (§2.8).
pub const GHOSTS: usize = 2;
/// Conserved components per cell: Σ, Σu, Σv, E, then Σ X_i — §2.2's order, the species as masses per area here.
pub const COMPONENTS: usize = 4 + N_SPECIES;

const NS: usize = N_SPECIES;

/// §2.8's two edge modes (module doc).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Bounce,
    Leave,
}

/// The world's four sides, in the order of the boundary-face slots ([`State::escaped`], the GPU's `booking.escaped`):
/// x = 0 by y, x = width − 1 by y, y = 0 by x, y = height − 1 by x — 2 (width + height) slots.
pub const SIDES: usize = 4;

/// A world's canonical planes (§2.2): `COMPONENTS` planes of `width × height` f64, channel-major, the species as mass
/// fractions and the energy per area — the order of the GPU's state, in f64 — and what has left through each side.
#[derive(Debug, Clone, PartialEq)]
pub struct State {
    pub width: usize,
    pub height: usize,
    pub planes: Vec<f64>,
    /// Booked `escaped` (§2.8, §2.9) since the state was made: [`State::totals`]' components as `COMPONENTS` planes of
    /// the boundary-face slots ([`SIDES`]' order) — what the grid lost through each face (signed; a mass never negative,
    /// inflow being 0).
    pub escaped: Vec<f64>,
}

impl State {
    /// A state from its planes, nothing escaped yet.
    pub fn new(width: usize, height: usize, planes: Vec<f64>) -> State {
        assert_eq!(planes.len(), COMPONENTS * width * height, "{COMPONENTS} planes of {width} x {height}");
        State { width, height, planes, escaped: vec![0.0; COMPONENTS * 2 * (width + height)] }
    }

    /// The boundary-face slots per escaped plane: 2 (width + height).
    pub fn faces(&self) -> usize {
        2 * (self.width + self.height)
    }

    /// The slot of face `i` on `side` (y on the x sides, x on the y sides).
    pub fn face(&self, side: usize, i: usize) -> usize {
        let (w, h) = (self.width, self.height);
        [i, h + i, 2 * h + i, 2 * h + w + i][side]
    }

    /// The escaped components through `side`'s faces.
    pub fn escaped_side(&self, side: usize) -> [f64; COMPONENTS] {
        let (n, len) = (self.faces(), if side < 2 { self.height } else { self.width });
        let first = self.face(side, 0);
        std::array::from_fn(|k| self.escaped[k * n + first..k * n + first + len].iter().sum())
    }

    /// The escaped components summed over every face.
    pub fn escaped_total(&self) -> [f64; COMPONENTS] {
        let n = self.faces();
        std::array::from_fn(|k| self.escaped[k * n..(k + 1) * n].iter().sum())
    }

    pub fn cells(&self) -> usize {
        self.width * self.height
    }

    /// Channel `c` at cell `i`.
    pub fn get(&self, c: usize, i: usize) -> f64 {
        self.planes[c * self.cells() + i]
    }

    /// The plane of channel `c`.
    pub fn plane(&self, c: usize) -> &[f64] {
        let n = self.cells();
        &self.planes[c * n..(c + 1) * n]
    }

    /// The state of the rows `y0..y1` alone (a strip cut from a taller world), nothing escaped.
    pub fn rows(&self, y0: usize, y1: usize) -> State {
        let (w, n) = (self.width, self.cells());
        let mut planes = Vec::with_capacity(COMPONENTS * w * (y1 - y0));
        for c in 0..COMPONENTS {
            planes.extend_from_slice(&self.planes[c * n + y0 * w..c * n + y1 * w]);
        }
        State::new(w, y1 - y0, planes)
    }

    /// The state with x and y exchanged: cell (x, y) becomes (y, x) and the two momenta swap — what a strip laid along y
    /// is when read along x; nothing escaped.
    pub fn transposed(&self) -> State {
        let (w, h, n) = (self.width, self.height, self.cells());
        let mut planes = vec![0.0; self.planes.len()];
        for c in 0..COMPONENTS {
            let from = match c {
                1 => 2,
                2 => 1,
                c => c,
            };
            for y in 0..h {
                for x in 0..w {
                    planes[c * n + x * h + y] = self.planes[from * n + y * w + x];
                }
            }
        }
        State::new(h, w, planes)
    }

    /// The sums of Σ, Σu, Σv, E and each species' mass over the world (the conserved quantities).
    pub fn totals(&self) -> [f64; COMPONENTS] {
        let n = self.cells();
        let mut t = [0.0; COMPONENTS];
        for i in 0..n {
            let sigma = self.get(0, i);
            t[0] += sigma;
            t[1] += self.get(1, i);
            t[2] += self.get(2, i);
            t[3] += self.get(3, i);
            for s in 0..NS {
                t[4 + s] += sigma * self.get(4 + s, i);
            }
        }
        t
    }
}

/// A cell's primitives: Σ, u, v, the total pressure Π and the mass fractions.
#[derive(Debug, Clone, Copy)]
struct Prim {
    sigma: f64,
    u: f64,
    v: f64,
    pi: f64,
    x: [f64; NS],
}

impl Prim {
    /// `self + f · slope`.
    fn offset(&self, slope: &Prim, f: f64) -> Prim {
        let mut x = self.x;
        for (xi, si) in x.iter_mut().zip(&slope.x) {
            *xi += f * si;
        }
        Prim {
            sigma: self.sigma + f * slope.sigma,
            u: self.u + f * slope.u,
            v: self.v + f * slope.v,
            pi: self.pi + f * slope.pi,
            x,
        }
    }

    /// The ghost across a wall: the same gas with its velocity along `dir` negated.
    fn mirrored(&self, dir: usize) -> Prim {
        let mut m = *self;
        if dir == 0 {
            m.u = -m.u;
        } else {
            m.v = -m.v;
        }
        m
    }

    /// The ghost across a leave edge: the same gas with its velocity along `dir` clamped outward — toward −∞ on the low
    /// side, +∞ on the `high` one.
    fn copied(&self, dir: usize, high: bool) -> Prim {
        let mut m = *self;
        let n = if dir == 0 { &mut m.u } else { &mut m.v };
        *n = if high { n.max(0.0) } else { n.min(0.0) };
        m
    }
}

/// A face state: the conserved vector and the primitives it stands for.
#[derive(Debug, Clone, Copy)]
struct Face {
    u: [f64; COMPONENTS],
    p: Prim,
}

/// The monotonised-central slope of three neighbours: ½(r − l) limited to twice either one-sided difference, 0 at an
/// extremum.
fn mc(l: f64, c: f64, r: f64) -> f64 {
    let (a, b) = (c - l, r - c);
    if a * b <= 0.0 {
        return 0.0;
    }
    (0.5 * (a + b)).abs().min(2.0 * a.abs()).min(2.0 * b.abs()).copysign(a)
}

/// The species' slopes: the central differences, all scaled by the one factor that brings the most limited of them
/// within its MC slope (so the slopes keep summing to zero).
fn species_slopes(l: &[f64; NS], c: &[f64; NS], r: &[f64; NS]) -> [f64; NS] {
    let mut central = [0.0; NS];
    let mut theta = 1.0f64;
    for i in 0..NS {
        central[i] = 0.5 * (r[i] - l[i]);
        if central[i] != 0.0 {
            theta = theta.min(mc(l[i], c[i], r[i]) / central[i]);
        }
    }
    central.map(|d| theta * d)
}

/// The slope of a cell along one axis from its two neighbours on that axis.
fn slope(l: &Prim, c: &Prim, r: &Prim) -> Prim {
    Prim {
        sigma: mc(l.sigma, c.sigma, r.sigma),
        u: mc(l.u, c.u, r.u),
        v: mc(l.v, c.v, r.v),
        pi: mc(l.pi, c.pi, r.pi),
        x: species_slopes(&l.x, &c.x, &r.x),
    }
}

/// P4's CPU f64 twin: the constants of a run and the scheme.
#[derive(Debug, Clone)]
pub struct Hydro {
    eos: Eos,
    elements: Elements,
    sigma_vac: f64,
    sigma_floor: f64,
    cfl: f64,
    eta_g: f64,
    edge: Edge,
    /// The radiation force's constants (§2.4.2, §2.4.4).
    kappa0: f64,
    c_sb: f64,
    kappa_dust: f64,
    t_dust: f64,
}

impl Hydro {
    /// The twin of a run's constants (a scene's `overrides` included).
    pub fn new(physics: &Physics, elements: &Elements, edge: Edge) -> Result<Hydro, RegistryError> {
        let get = |k: &str| physics.get(k).expect("physics.json key");
        Ok(Hydro {
            eos: Eos::from_physics(physics)?,
            elements: elements.clone(),
            sigma_vac: get("sigma_vac"),
            sigma_floor: get("sigma_floor"),
            cfl: get("cfl"),
            eta_g: get("eta_g"),
            edge,
            kappa0: get("kappa0"),
            c_sb: get("c_sb"),
            kappa_dust: get("kappa_dust"),
            t_dust: get("t_dust"),
        })
    }

    /// The radiation force per area f = κΣF/c_sb on cell `i` of `st` under the cell-centred flux (`fx`, `fy`): κ from the
    /// cell's mass fractions and temperature (§2.4.2, as P5's); 0 below Σ_vac.
    fn radiation_force(&self, st: &State, i: usize, fx: f64, fy: f64) -> [f64; 2] {
        let sigma = st.get(0, i);
        if sigma < self.sigma_vac {
            return [0.0, 0.0];
        }
        let x: [f64; NS] = std::array::from_fn(|s| st.get(4 + s, i));
        let mut kappa = self.kappa0 * (1.0 + x[0]);
        if self.kappa_dust != 0.0 {
            let comp = self.elements.composition(&x);
            let mu = 1.0 / comp.inv_mu;
            let th = self.eos.thermal(st.get(3, i), sigma, [st.get(1, i), st.get(2, i)], comp.y_e, Self::x_n(&x), mu);
            if Eos::temperature(th.eps_th, mu) < self.t_dust {
                kappa += self.kappa_dust * comp.z_met;
            }
        }
        [kappa * sigma * fx / self.c_sb, kappa * sigma * fy / self.c_sb]
    }

    /// 1/μ and Y_e of mass fractions `x`.
    fn thermo(&self, x: &[f64; NS]) -> (f64, f64) {
        let comp = self.elements.composition(x);
        (comp.inv_mu, comp.y_e)
    }

    fn x_n(x: &[f64; NS]) -> f64 {
        x[NS - 1]
    }

    /// Cell `i` of `st` as primitives: the floor state below Σ_vac, else the thermal energy recovered from E and floored
    /// at T_floor/μ (§2.3.3).
    fn read(&self, st: &State, i: usize) -> Prim {
        let mut x = [0.0; NS];
        for (s, xs) in x.iter_mut().enumerate() {
            *xs = st.get(4 + s, i);
        }
        let (inv_mu, y_e) = self.thermo(&x);
        let mu = 1.0 / inv_mu;
        let sigma = st.get(0, i);
        if sigma < self.sigma_vac {
            let sigma = self.sigma_floor;
            let pi = self.eos.pi(sigma, self.eos.t_floor() / mu, y_e, Self::x_n(&x));
            return Prim { sigma, u: 0.0, v: 0.0, pi, x };
        }
        let mom = [st.get(1, i), st.get(2, i)];
        let th = self.eos.thermal(st.get(3, i), sigma, mom, y_e, Self::x_n(&x), mu);
        let pi = self.eos.pi(sigma, th.eps_th, y_e, Self::x_n(&x));
        Prim { sigma, u: mom[0] / sigma, v: mom[1] / sigma, pi, x }
    }

    /// The conserved vector of primitives: E = Σ(ε_th + ε_cold) + ½Σ|u|², ε_th = (Π − Π_e − Π_n)/Σ floored at T_floor/μ.
    fn cons(&self, p: &Prim) -> [f64; COMPONENTS] {
        let (inv_mu, y_e) = self.thermo(&p.x);
        let x_n = Self::x_n(&p.x);
        let thermal = (p.pi - self.eos.pi_e(p.sigma, y_e) - self.eos.pi_n(p.sigma, x_n)) / p.sigma;
        let eps_th = thermal.max(self.eos.t_floor() * inv_mu);
        let energy = p.sigma * (eps_th + self.eos.eps_cold(p.sigma, y_e, x_n)) + 0.5 * p.sigma * (p.u * p.u + p.v * p.v);
        let mut u = [0.0; COMPONENTS];
        u[0] = p.sigma;
        u[1] = p.sigma * p.u;
        u[2] = p.sigma * p.v;
        u[3] = energy;
        for s in 0..NS {
            u[4 + s] = p.sigma * p.x[s];
        }
        u
    }

    /// The primitives of a conserved vector; `None` when it is not a state (Σ ≤ 0, no thermal energy left, not finite).
    fn prim(&self, u: &[f64; COMPONENTS]) -> Option<Prim> {
        let sigma = u[0];
        if sigma.is_nan() || sigma <= 0.0 || u.iter().any(|v| !v.is_finite()) {
            return None;
        }
        let mut x = [0.0; NS];
        for s in 0..NS {
            x[s] = u[4 + s] / sigma;
        }
        let (inv_mu, y_e) = self.thermo(&x);
        let x_n = Self::x_n(&x);
        let raw = u[3] / sigma - 0.5 * (u[1] * u[1] + u[2] * u[2]) / (sigma * sigma) - self.eos.eps_cold(sigma, y_e, x_n);
        if raw.is_nan() || raw <= 0.0 {
            return None;
        }
        let eps_th = raw.max(self.eos.t_floor() * inv_mu);
        Some(Prim { sigma, u: u[1] / sigma, v: u[2] / sigma, pi: self.eos.pi(sigma, eps_th, y_e, x_n), x })
    }

    /// The wave-speed bound c = √(2Π/Σ) (§2.3.3).
    fn sound(p: &Prim) -> f64 {
        Eos::wave_speed_sq(p.sigma, p.pi).max(0.0).sqrt()
    }

    /// The flux through a face normal to `dir` (0: x, 1: y) of a state whose total energy per area is `energy`.
    fn flux(p: &Prim, energy: f64, dir: usize) -> [f64; COMPONENTS] {
        let (un, ut) = if dir == 0 { (p.u, p.v) } else { (p.v, p.u) };
        let mass = p.sigma * un;
        let mut f = [0.0; COMPONENTS];
        f[0] = mass;
        f[1 + dir] = mass * un + p.pi;
        f[2 - dir] = mass * ut;
        f[3] = (energy + p.pi) * un;
        for s in 0..NS {
            f[4 + s] = mass * p.x[s];
        }
        f
    }

    /// A face state from the primitives `base` advanced by the Hancock increment `delta`; the unevolved state when the
    /// advanced one is not a state.
    fn evolved(&self, base: &Prim, delta: &[f64; COMPONENTS]) -> Face {
        let u0 = self.cons(base);
        let mut u = u0;
        for k in 0..COMPONENTS {
            u[k] += delta[k];
        }
        match self.prim(&u) {
            Some(p) => Face { u, p },
            None => Face { u: u0, p: *base },
        }
    }

    /// The HLL flux across a face normal to `dir` between `l` and `r`, species by the mass flux's upwind side.
    fn hll(l: &Face, r: &Face, dir: usize) -> [f64; COMPONENTS] {
        let normal = |p: &Prim| if dir == 0 { p.u } else { p.v };
        let (unl, unr) = (normal(&l.p), normal(&r.p));
        let (cl, cr) = (Self::sound(&l.p), Self::sound(&r.p));
        let s_l = (unl - cl).min(unr - cr);
        let s_r = (unl + cl).max(unr + cr);
        let fl = Self::flux(&l.p, l.u[3], dir);
        let fr = Self::flux(&r.p, r.u[3], dir);
        let mut f = if s_l >= 0.0 {
            fl
        } else if s_r <= 0.0 {
            fr
        } else {
            let mut f = [0.0; COMPONENTS];
            for k in 0..4 {
                f[k] = (s_r * fl[k] - s_l * fr[k] + s_l * s_r * (r.u[k] - l.u[k])) / (s_r - s_l);
            }
            f
        };
        let upwind = if f[0] > 0.0 { &l.p } else { &r.p };
        for s in 0..NS {
            f[4 + s] = f[0] * upwind.x[s];
        }
        f
    }

    /// The flux through a wall (§2.8, bounce): mass, energy and species exactly 0, the normal momentum the pressure
    /// alone.
    fn wall(face: &Face, dir: usize) -> [f64; COMPONENTS] {
        let mut f = [0.0; COMPONENTS];
        f[1 + dir] = face.p.pi;
        f
    }

    /// The flux through the boundary face on `side` ([`SIDES`]' order) of an edge cell whose own predicted face there is
    /// `own` (§2.8). Bounce: the wall. Leave: HLL between `own` and the facing predicted face of the ghost beyond it
    /// (`ghost` computes its four), clamped to outflow — a face whose mass flux points into the world is a wall, so inflow
    /// is exactly 0 and nothing else enters with it.
    fn boundary(&self, side: usize, own: &Face, ghost: impl FnOnce() -> [Face; 4]) -> [f64; COMPONENTS] {
        let (dir, high) = (side / 2, side % 2 == 1);
        if self.edge == Edge::Bounce {
            return Self::wall(own, dir);
        }
        let g = ghost();
        let f = if high { Self::hll(own, &g[side - 1], dir) } else { Self::hll(&g[side + 1], own, dir) };
        let inflow = if high { f[0] < 0.0 } else { f[0] > 0.0 };
        if inflow {
            Self::wall(own, dir)
        } else {
            f
        }
    }

    /// The four predicted face states, [x−, x+, y−, y+], of the cell at index `gi` of the ghosted `grid` (rows of `pw`),
    /// and the cell centre's (Σ, Σu, Σv) half a step on: the cell's limited slopes on both axes, then the Hancock half
    /// step with both axes' flux differences and, under the cell's gravity `g`, each face's half kick (module doc).
    fn predicted(&self, grid: &[Prim], pw: usize, gi: usize, half: f64, g: Option<[f64; 2]>) -> ([Face; 4], [f64; 3]) {
        let c = &grid[gi];
        let sx = slope(&grid[gi - 1], c, &grid[gi + 1]);
        let sy = slope(&grid[gi - pw], c, &grid[gi + pw]);
        let w_faces = [c.offset(&sx, -0.5), c.offset(&sx, 0.5), c.offset(&sy, -0.5), c.offset(&sy, 0.5)];
        let u_faces = w_faces.map(|p| self.cons(&p));
        let fx = |k: usize| Self::flux(&w_faces[k], u_faces[k][3], 0);
        let fy = |k: usize| Self::flux(&w_faces[k], u_faces[k][3], 1);
        let (fxl, fxr, fyl, fyr) = (fx(0), fx(1), fy(2), fy(3));
        let mut delta = [0.0; COMPONENTS];
        for k in 0..COMPONENTS {
            delta[k] = half * ((fxl[k] - fxr[k]) + (fyl[k] - fyr[k]));
        }
        let mut mid = [c.sigma + delta[0], c.sigma * c.u + delta[1], c.sigma * c.v + delta[2]];
        let Some([gx, gy]) = g else {
            return (w_faces.map(|p| self.evolved(&p, &delta)), mid);
        };
        mid[1] += half * c.sigma * gx;
        mid[2] += half * c.sigma * gy;
        let kicked = |p: &Prim| {
            let mut d = delta;
            d[1] += half * p.sigma * gx;
            d[2] += half * p.sigma * gy;
            d[3] += half * p.sigma * ((p.u + 0.5 * half * gx) * gx + (p.v + 0.5 * half * gy) * gy);
            self.evolved(p, &d)
        };
        (w_faces.map(|p| kicked(&p)), mid)
    }

    /// Δt = min(C · min_cells [(|u| + c)/Δx + (|v| + c)/Δy]⁻¹, η_g · min_cells √(Δx/|g|)) (§2.3.5); Δx = Δy = 1. `gravity`
    /// is g's two planes when the step runs gravity, else the second term is infinite.
    pub fn dt(&self, st: &State, gravity: Option<(&[f64], &[f64])>) -> f64 {
        let mut rate = 0.0f64;
        let mut g_max = 0.0f64;
        for i in 0..st.cells() {
            let p = self.read(st, i);
            let c = Self::sound(&p);
            rate = rate.max((p.u.abs() + c) + (p.v.abs() + c));
            if let Some((gx, gy)) = gravity {
                g_max = g_max.max(gx[i].hypot(gy[i]));
            }
        }
        let mut dt = self.cfl / rate;
        if g_max > 0.0 {
            dt = dt.min(self.eta_g / g_max.sqrt());
        }
        dt
    }

    /// One step of length `dt`, in place, without gravity.
    pub fn step(&self, st: &mut State, dt: f64) {
        self.step_with(st, dt, None);
    }

    /// One step of length `dt`, in place, with gravity's source when `gravity` holds P2's g (its x plane, then its y
    /// plane, over the world) — module doc.
    pub fn step_with(&self, st: &mut State, dt: f64, gravity: Option<(&[f64], &[f64])>) {
        self.step_forced(st, dt, gravity, None);
    }

    /// As [`Hydro::step_with`], with the radiation force when `radiation` holds the previous step's F (its x plane, then
    /// its y plane, cell-centred, over the world) — module doc.
    pub fn step_forced(&self, st: &mut State, dt: f64, gravity: Option<(&[f64], &[f64])>, radiation: Option<(&[f64], &[f64])>) {
        let (w, h) = (st.width, st.height);
        let n = st.cells();
        let (pw, ph) = (w + 2 * GHOSTS, h + 2 * GHOSTS);
        let at = |x: usize, y: usize| (y + GHOSTS) * pw + x + GHOSTS;

        // The cells and their ghosts (§2.8): the real cells, then the columns' ghosts, then the rows'.
        let blank = Prim { sigma: 1.0, u: 0.0, v: 0.0, pi: 1.0, x: [0.0; NS] };
        let mut grid = vec![blank; pw * ph];
        for y in 0..h {
            for x in 0..w {
                grid[at(x, y)] = self.read(st, y * w + x);
            }
        }
        match self.edge {
            Edge::Bounce => {
                for y in 0..h {
                    for k in 1..=GHOSTS {
                        grid[(y + GHOSTS) * pw + GHOSTS - k] = grid[at(k - 1, y)].mirrored(0);
                        grid[(y + GHOSTS) * pw + GHOSTS + w - 1 + k] = grid[at(w - k, y)].mirrored(0);
                    }
                }
                for px in 0..pw {
                    for k in 1..=GHOSTS {
                        grid[(GHOSTS - k) * pw + px] = grid[(GHOSTS + k - 1) * pw + px].mirrored(1);
                        grid[(GHOSTS + h - 1 + k) * pw + px] = grid[(GHOSTS + h - k) * pw + px].mirrored(1);
                    }
                }
            }
            Edge::Leave => {
                for y in 0..h {
                    for k in 1..=GHOSTS {
                        grid[(y + GHOSTS) * pw + GHOSTS - k] = grid[at(0, y)].copied(0, false);
                        grid[(y + GHOSTS) * pw + GHOSTS + w - 1 + k] = grid[at(w - 1, y)].copied(0, true);
                    }
                }
                for px in 0..pw {
                    for k in 1..=GHOSTS {
                        grid[(GHOSTS - k) * pw + px] = grid[GHOSTS * pw + px].copied(1, false);
                        grid[(GHOSTS + h - 1 + k) * pw + px] = grid[(GHOSTS + h - 1) * pw + px].copied(1, true);
                    }
                }
            }
        }

        // g on the ghosted grid: a ghost takes its edge cell's.
        let accel = |gi: usize| {
            gravity.map(|(gx, gy)| {
                let x = (gi % pw).clamp(GHOSTS, GHOSTS + w - 1) - GHOSTS;
                let y = (gi / pw).clamp(GHOSTS, GHOSTS + h - 1) - GHOSTS;
                [gx[y * w + x], gy[y * w + x]]
            })
        };

        // The reconstruction and the Hancock predictor: every cell's four face states, [x−, x+, y−, y+], and its centre
        // half a step on.
        let half = 0.5 * dt;
        let mut faces: Vec<[Face; 4]> = Vec::with_capacity(n);
        let mut mids: Vec<[f64; 3]> = Vec::with_capacity(n);
        for y in 0..h {
            for x in 0..w {
                let (f, mid) = self.predicted(&grid, pw, at(x, y), half, accel(at(x, y)));
                faces.push(f);
                mids.push(mid);
            }
        }

        // The fluxes: face i of a row lies between cells i − 1 and i; row j of the y fluxes between rows j − 1 and j. A
        // boundary face takes its edge cell's own face and, leaving, the ghost's beyond it (`at` ± a step).
        let boundary = |side: usize, own: &Face, cell: usize| {
            let ghost = match side {
                0 => cell - 1,
                1 => cell + 1,
                2 => cell - pw,
                _ => cell + pw,
            };
            self.boundary(side, own, || self.predicted(&grid, pw, ghost, half, accel(ghost)).0)
        };
        let mut fx = vec![[0.0; COMPONENTS]; (w + 1) * h];
        for y in 0..h {
            for i in 0..=w {
                fx[y * (w + 1) + i] = if i == 0 {
                    boundary(0, &faces[y * w][0], at(0, y))
                } else if i == w {
                    boundary(1, &faces[y * w + w - 1][1], at(w - 1, y))
                } else {
                    Self::hll(&faces[y * w + i - 1][1], &faces[y * w + i][0], 0)
                };
            }
        }
        let mut fy = vec![[0.0; COMPONENTS]; w * (h + 1)];
        for j in 0..=h {
            for x in 0..w {
                fy[j * w + x] = if j == 0 {
                    boundary(2, &faces[x][2], at(x, 0))
                } else if j == h {
                    boundary(3, &faces[(h - 1) * w + x][3], at(x, h - 1))
                } else {
                    Self::hll(&faces[(j - 1) * w + x][3], &faces[j * w + x][2], 1)
                };
            }
        }

        // The update, in place; a cell with no gas around it stays empty.
        let vacuum = |x: usize, y: usize| st.get(0, y * w + x) < self.sigma_vac;
        let mut next = st.clone();
        for y in 0..h {
            for x in 0..w {
                let empty = vacuum(x, y)
                    && (x == 0 || vacuum(x - 1, y))
                    && (x + 1 == w || vacuum(x + 1, y))
                    && (y == 0 || vacuum(x, y - 1))
                    && (y + 1 == h || vacuum(x, y + 1));
                if empty {
                    continue;
                }
                let i = y * w + x;
                // The stored vector where there is gas (bit for bit what the planes hold), the floor state's below Σ_vac.
                let mut u = if vacuum(x, y) {
                    self.cons(&grid[at(x, y)])
                } else {
                    let mut u = [0.0; COMPONENTS];
                    for (k, uk) in u.iter_mut().enumerate().take(4) {
                        *uk = st.get(k, i);
                    }
                    for s in 0..NS {
                        u[4 + s] = u[0] * st.get(4 + s, i);
                    }
                    u
                };
                for k in 0..COMPONENTS {
                    u[k] -= dt * ((fx[y * (w + 1) + x + 1][k] - fx[y * (w + 1) + x][k]) + (fy[(y + 1) * w + x][k] - fy[y * w + x][k]));
                }
                // Gravity's source, time-centred on the predictor's half step (§1.4.2).
                if let Some([gx, gy]) = accel(at(x, y)) {
                    let [sigma, mx, my] = mids[i];
                    u[1] += dt * sigma * gx;
                    u[2] += dt * sigma * gy;
                    u[3] += dt * (mx * gx + my * gy);
                }
                // The radiation force from the previous step's flux (§2.4.4): momentum only, E as it is.
                if let Some((fx, fy)) = radiation {
                    let [ax, ay] = self.radiation_force(st, i, fx[i], fy[i]);
                    u[1] += dt * ax;
                    u[2] += dt * ay;
                }
                // Escaped, leaving (§2.8): each boundary face of a cell this step moved, its flux as the cell took it — the
                // grid loses dt·f through a high face, −dt·f through a low one.
                if self.edge == Edge::Leave {
                    let crossing = [
                        (x == 0, -1.0, &fx[y * (w + 1)], st.face(0, y)),
                        (x + 1 == w, 1.0, &fx[y * (w + 1) + w], st.face(1, y)),
                        (y == 0, -1.0, &fy[x], st.face(2, x)),
                        (y + 1 == h, 1.0, &fy[h * w + x], st.face(3, x)),
                    ];
                    let faces = st.faces();
                    for (edge, sign, f, slot) in crossing {
                        if edge {
                            for (k, fk) in f.iter().enumerate() {
                                next.escaped[k * faces + slot] += sign * dt * fk;
                            }
                        }
                    }
                }
                next.planes[i] = u[0];
                next.planes[n + i] = u[1];
                next.planes[2 * n + i] = u[2];
                next.planes[3 * n + i] = u[3];
                for s in 0..NS {
                    next.planes[(4 + s) * n + i] = u[4 + s] / u[0];
                }
            }
        }
        *st = next;
    }

    /// Steps `st` to sim time `t_end` from `t`, each Δt per §2.3.5 and the last cut to land on `t_end`; returns the
    /// number of steps; panics when Δt collapses (a state no longer finite, or waves without bound).
    pub fn advance(&self, st: &mut State, t: f64, t_end: f64) -> usize {
        let (mut t, mut steps) = (t, 0);
        while t < t_end {
            let dt = self.dt(st, None).min(t_end - t);
            // A broken scheme drives the wave speeds up and Δt down without end: refuse rather than spin.
            assert!(dt.is_finite() && dt > 1e-9, "Δt collapsed to {dt} at t = {t} after {steps} steps");
            self.step(st, dt);
            t += dt;
            steps += 1;
        }
        steps
    }
}
