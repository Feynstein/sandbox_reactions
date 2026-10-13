//! The exact Riemann solution for an ideal γ = 2 gas (m0_contrat.md §2.3.1, G-SOD §5.1, SM-O5): the oracle the CPU twin's
//! gas flow (M0-T33b) and the GPU's (M0-T34) are graded against — written first, in its own block, never fitted to a scheme.
//!
//! One-dimensional Euler equations for (ρ, u, p), the gas at rest where it is uniform, a discontinuity at x = 0 and
//! ξ = x / t. After Toro, *Riemann Solvers and Numerical Methods for Fluid Dynamics*, §4.2–§4.5 (SM-E14 for Sod's states):
//! the star-region pressure p* is the root of f_L(p) + f_R(p) + (u_R − u_L) = 0, f_K the shock branch (Rankine–Hugoniot)
//! for p > p_K and the rarefaction branch (isentrope and Riemann invariant) below; u* = ½(u_L + u_R) + ½(f_R − f_L).
//! The 2D gas the game runs is the plane's γ = 2 (§2.3.1); the strip of G-SOD is one-dimensional, so the solution is exact
//! there. Pressure is the total Π of the contract with the cold terms off (G-SOD's `overrides`).

/// The adiabatic index of the plane's monatomic gas (§2.3.1).
pub const GAMMA: f64 = 2.0;

/// A uniform gas state: density, velocity along x, pressure.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State {
    pub rho: f64,
    pub u: f64,
    pub p: f64,
}

impl State {
    pub fn new(rho: f64, u: f64, p: f64) -> State {
        State { rho, u, p }
    }

    /// The speed of sound, √(γ p / ρ).
    pub fn sound(&self) -> f64 {
        (GAMMA * self.p / self.rho).sqrt()
    }

    /// The state seen in a mirror: x → −x.
    pub fn mirrored(&self) -> State {
        State { rho: self.rho, u: -self.u, p: self.p }
    }
}

/// Why a problem has no solution this oracle returns.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RiemannError {
    /// A density or pressure that is not finite and positive.
    BadState,
    /// The two sides recede fast enough that a vacuum opens between them (2 (c_L + c_R) / (γ − 1) ≤ u_R − u_L).
    Vacuum,
    /// The root-find did not converge (never seen for a valid state; kept as a refusal, not a guess).
    NoConvergence,
}

/// One wave of the pattern: a shock (one speed) or a rarefaction fan (head and tail speeds, head first).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Wave {
    Shock { speed: f64 },
    Rarefaction { head: f64, tail: f64 },
}

/// The solved problem: the star region and the wave pattern left wave · contact (at u*) · right wave.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Solution {
    pub left: State,
    pub right: State,
    /// The pressure and velocity both sides share between the waves.
    pub p_star: f64,
    pub u_star: f64,
    /// The density left and right of the contact.
    pub rho_star_left: f64,
    pub rho_star_right: f64,
    pub left_wave: Wave,
    pub right_wave: Wave,
}

/// Toro's f_K and its derivative: the velocity change across the K wave for the star pressure `p`.
fn side_function(p: f64, k: &State, gamma: f64) -> (f64, f64) {
    let c = (gamma * k.p / k.rho).sqrt();
    if p > k.p {
        let a = 2.0 / ((gamma + 1.0) * k.rho);
        let b = (gamma - 1.0) / (gamma + 1.0) * k.p;
        let q = (a / (p + b)).sqrt();
        ((p - k.p) * q, q * (1.0 - (p - k.p) / (2.0 * (b + p))))
    } else {
        let e = (gamma - 1.0) / (2.0 * gamma);
        let ratio = p / k.p;
        (2.0 * c / (gamma - 1.0) * (ratio.powf(e) - 1.0), ratio.powf(-(gamma + 1.0) / (2.0 * gamma)) / (k.rho * c))
    }
}

/// The star pressure: the root of f_L + f_R + (u_R − u_L), a safeguarded Newton inside a bracket that bisection keeps.
fn star_pressure(l: &State, r: &State) -> Result<f64, RiemannError> {
    let gamma = GAMMA;
    let du = r.u - l.u;
    let total = |p: f64| side_function(p, l, gamma).0 + side_function(p, r, gamma).0 + du;
    if total(0.0) >= 0.0 {
        return Err(RiemannError::Vacuum);
    }
    let mut lo = 0.0_f64;
    let mut hi = 0.5 * (l.p + r.p);
    while total(hi) < 0.0 {
        lo = hi;
        hi *= 2.0;
        if !hi.is_finite() {
            return Err(RiemannError::NoConvergence);
        }
    }
    let mut p = 0.5 * (lo + hi);
    for _ in 0..200 {
        let (fl, dl) = side_function(p, l, gamma);
        let (fr, dr) = side_function(p, r, gamma);
        let f = fl + fr + du;
        if f < 0.0 {
            lo = p;
        } else {
            hi = p;
        }
        let newton = p - f / (dl + dr);
        let next = if newton > lo && newton < hi { newton } else { 0.5 * (lo + hi) };
        if (next - p).abs() <= 1e-15 * next {
            return Ok(next);
        }
        p = next;
    }
    Err(RiemannError::NoConvergence)
}

impl Solution {
    /// Solve the Riemann problem for `left` | `right`.
    pub fn solve(left: State, right: State) -> Result<Solution, RiemannError> {
        for s in [&left, &right] {
            if !(s.rho.is_finite() && s.rho > 0.0 && s.p.is_finite() && s.p > 0.0 && s.u.is_finite()) {
                return Err(RiemannError::BadState);
            }
        }
        let p_star = star_pressure(&left, &right)?;
        let u_star = 0.5 * (left.u + right.u)
            + 0.5 * (side_function(p_star, &right, GAMMA).0 - side_function(p_star, &left, GAMMA).0);
        let mu = (GAMMA - 1.0) / (GAMMA + 1.0);
        let e = (GAMMA - 1.0) / (2.0 * GAMMA);
        let shock_ratio = |k: &State| (p_star / k.p + mu) / (mu * p_star / k.p + 1.0);
        let (cl, cr) = (left.sound(), right.sound());
        let (left_wave, rho_star_left) = if p_star > left.p {
            let s = left.u - cl * ((GAMMA + 1.0) / (2.0 * GAMMA) * p_star / left.p + e).sqrt();
            (Wave::Shock { speed: s }, left.rho * shock_ratio(&left))
        } else {
            let c_star = cl * (p_star / left.p).powf(e);
            (
                Wave::Rarefaction { head: left.u - cl, tail: u_star - c_star },
                left.rho * (p_star / left.p).powf(1.0 / GAMMA),
            )
        };
        let (right_wave, rho_star_right) = if p_star > right.p {
            let s = right.u + cr * ((GAMMA + 1.0) / (2.0 * GAMMA) * p_star / right.p + e).sqrt();
            (Wave::Shock { speed: s }, right.rho * shock_ratio(&right))
        } else {
            let c_star = cr * (p_star / right.p).powf(e);
            (
                Wave::Rarefaction { head: right.u + cr, tail: u_star + c_star },
                right.rho * (p_star / right.p).powf(1.0 / GAMMA),
            )
        };
        Ok(Solution { left, right, p_star, u_star, rho_star_left, rho_star_right, left_wave, right_wave })
    }

    /// The solution at ξ = x / t (the discontinuity at x = 0).
    pub fn sample(&self, xi: f64) -> State {
        let g = GAMMA;
        if xi <= self.u_star {
            let (l, c) = (&self.left, self.left.sound());
            match self.left_wave {
                Wave::Shock { speed } => {
                    if xi <= speed { *l } else { State::new(self.rho_star_left, self.u_star, self.p_star) }
                }
                Wave::Rarefaction { head, tail } => {
                    if xi <= head {
                        *l
                    } else if xi > tail {
                        State::new(self.rho_star_left, self.u_star, self.p_star)
                    } else {
                        let u = 2.0 / (g + 1.0) * (c + (g - 1.0) / 2.0 * l.u + xi);
                        let a = 2.0 / (g + 1.0) * (c + (g - 1.0) / 2.0 * (l.u - xi));
                        State::new(l.rho * (a / c).powf(2.0 / (g - 1.0)), u, l.p * (a / c).powf(2.0 * g / (g - 1.0)))
                    }
                }
            }
        } else {
            let (r, c) = (&self.right, self.right.sound());
            match self.right_wave {
                Wave::Shock { speed } => {
                    if xi >= speed { *r } else { State::new(self.rho_star_right, self.u_star, self.p_star) }
                }
                Wave::Rarefaction { head, tail } => {
                    if xi >= head {
                        *r
                    } else if xi < tail {
                        State::new(self.rho_star_right, self.u_star, self.p_star)
                    } else {
                        let u = 2.0 / (g + 1.0) * (-c + (g - 1.0) / 2.0 * r.u + xi);
                        let a = 2.0 / (g + 1.0) * (c - (g - 1.0) / 2.0 * (r.u - xi));
                        State::new(r.rho * (a / c).powf(2.0 / (g - 1.0)), u, r.p * (a / c).powf(2.0 * g / (g - 1.0)))
                    }
                }
            }
        }
    }

    /// The solution at `x` and `t > 0` for a discontinuity at `x0`.
    pub fn at(&self, x: f64, x0: f64, t: f64) -> State {
        self.sample((x - x0) / t)
    }

    /// The wave speeds that bound every piece of the solution that is smooth, ascending: the left wave's head (or the
    /// shock) and tail, the contact, the right wave's tail and head (or the shock).
    pub fn breakpoints(&self) -> Vec<f64> {
        let mut v = Vec::new();
        match self.left_wave {
            Wave::Shock { speed } => v.push(speed),
            Wave::Rarefaction { head, tail } => v.extend([head, tail]),
        }
        v.push(self.u_star);
        match self.right_wave {
            Wave::Shock { speed } => v.push(speed),
            Wave::Rarefaction { head, tail } => v.extend([tail, head]),
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOD: (State, State) = (State { rho: 1.0, u: 0.0, p: 1.0 }, State { rho: 0.125, u: 0.0, p: 0.1 });

    /// Problems that exercise every wave pair, Toro's test layouts at γ = 2 (their values are for γ = 1.4; the patterns —
    /// rarefaction | shock, two rarefactions, two shocks, a moving contact — are what they pick).
    fn problems() -> Vec<(&'static str, State, State)> {
        vec![
            ("sod", SOD.0, SOD.1),
            ("two rarefactions", State::new(1.0, -0.5, 1.0), State::new(1.0, 0.5, 1.0)),
            ("two shocks", State::new(1.0, 1.0, 1.0), State::new(1.0, -1.0, 1.0)),
            ("strong shock right", State::new(1.0, 0.0, 1000.0), State::new(1.0, 0.0, 0.01)),
            ("shock left, rarefaction right", State::new(0.125, 0.0, 0.1), State::new(1.0, 0.0, 1.0)),
            ("moving contact", State::new(1.0, 0.75, 1.0), State::new(0.125, 0.75, 1.0)),
            ("slow collision", State::new(5.99924, 19.5975, 460.894), State::new(5.99242, -6.19633, 46.0950)),
        ]
    }

    /// An independent pressure function: the shock branch from the Rankine–Hugoniot jump written through the mass flux
    /// (m² = Δp / Δ(1/ρ), Δu = Δp / m), the rarefaction branch from the Riemann invariant and the isentrope — not
    /// Toro's closed forms the solver uses. Returns (velocity of the star side relative to the K state's), K = −1 left
    /// and +1 right, signed so that the left and right sum to u*.
    fn du_across(p: f64, k: &State, side: f64) -> f64 {
        if p > k.p {
            let mu = (GAMMA - 1.0) / (GAMMA + 1.0);
            let rho_star = k.rho * (p / k.p + mu) / (mu * p / k.p + 1.0);
            let m = ((p - k.p) / (1.0 / k.rho - 1.0 / rho_star)).sqrt();
            side * (p - k.p) / m
        } else {
            let c = k.sound();
            let c_star = c * (p / k.p).powf((GAMMA - 1.0) / (2.0 * GAMMA));
            side * 2.0 / (GAMMA - 1.0) * (c_star - c)
        }
    }

    /// Bisection on u*_from_left(p) − u*_from_right(p): left gives u_L + du(−1 side → −f_L), right u_R + f_R.
    fn bisect_star(l: &State, r: &State) -> f64 {
        let g = |p: f64| (l.u + du_across(p, l, -1.0)) - (r.u + du_across(p, r, 1.0));
        let (mut lo, mut hi) = (1e-12, 1.0);
        while g(hi) > 0.0 {
            hi *= 2.0;
        }
        for _ in 0..300 {
            let mid = 0.5 * (lo + hi);
            if g(mid) > 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    }

    fn rel(a: f64, b: f64) -> f64 {
        (a - b).abs() / b.abs().max(f64::MIN_POSITIVE)
    }

    #[test]
    fn star_pressure_matches_an_independent_bisection() {
        for (name, l, r) in problems() {
            let s = Solution::solve(l, r).unwrap();
            let p = bisect_star(&l, &r);
            assert!(rel(s.p_star, p) < 1e-10, "{name}: p* {} vs bisection {p}", s.p_star);
            // the velocity both ways: u* from the left wave and from the right wave agree
            let from_left = l.u + du_across(p, &l, -1.0);
            let from_right = r.u + du_across(p, &r, 1.0);
            assert!((s.u_star - from_left).abs() < 1e-9 * (1.0 + from_left.abs()), "{name}: u* {} vs {from_left}", s.u_star);
            assert!((from_left - from_right).abs() < 1e-9 * (1.0 + from_left.abs()), "{name}: {from_left} vs {from_right}");
        }
    }

    #[test]
    fn sods_pattern_is_a_left_rarefaction_and_a_right_shock() {
        let s = Solution::solve(SOD.0, SOD.1).unwrap();
        assert!(matches!(s.left_wave, Wave::Rarefaction { .. }), "{:?}", s.left_wave);
        assert!(matches!(s.right_wave, Wave::Shock { .. }), "{:?}", s.right_wave);
        assert!(s.p_star > SOD.1.p && s.p_star < SOD.0.p, "p* {}", s.p_star);
        assert!(s.u_star > 0.0 && s.rho_star_left > s.rho_star_right, "u* {}, ρ* {} | {}", s.u_star, s.rho_star_left, s.rho_star_right);
        // the pattern is ordered: head < tail < contact < shock
        let b = s.breakpoints();
        assert!(b.windows(2).all(|w| w[0] < w[1]), "{b:?}");
    }

    /// In the frame of a shock moving at `s`, mass, momentum and energy fluxes are continuous (Rankine–Hugoniot).
    fn assert_jump(pre: &State, post: &State, s: f64, what: &str) {
        let flux = |q: &State| {
            let w = q.u - s;
            let e = q.p / (GAMMA - 1.0) + 0.5 * q.rho * q.u * q.u;
            [q.rho * w, q.rho * q.u * w + q.p, e * w + q.p * q.u]
        };
        let (a, b) = (flux(pre), flux(post));
        for i in 0..3 {
            assert!((a[i] - b[i]).abs() < 1e-10 * (a[i].abs() + b[i].abs() + 1.0), "{what}: flux {i} {} vs {}", a[i], b[i]);
        }
    }

    #[test]
    fn sod_satisfies_the_jumps_and_the_isentrope() {
        let sol = Solution::solve(SOD.0, SOD.1).unwrap();
        let (Wave::Rarefaction { head, tail }, Wave::Shock { speed }) = (sol.left_wave, sol.right_wave) else { panic!() };
        // the shock: Rankine–Hugoniot between the undisturbed right state and the star state behind it
        assert_jump(&SOD.1, &State::new(sol.rho_star_right, sol.u_star, sol.p_star), speed, "Sod's shock");
        // the contact: p and u continuous, ρ jumps
        let (a, b) = (sol.sample(sol.u_star - 1e-9), sol.sample(sol.u_star + 1e-9));
        assert!(rel(a.p, b.p) < 1e-8 && (a.u - b.u).abs() < 1e-8, "{a:?} vs {b:?}");
        assert!(a.rho > b.rho * 1.1, "{a:?} vs {b:?}");
        // the rarefaction: the isentrope p / ρ^γ and the Riemann invariant u + 2c/(γ−1) are the left state's everywhere in the fan
        let k0 = SOD.0.p / SOD.0.rho.powf(GAMMA);
        let j0 = SOD.0.u + 2.0 * SOD.0.sound() / (GAMMA - 1.0);
        for i in 0..=50 {
            let xi = head + (tail - head) * i as f64 / 50.0;
            let q = sol.sample(xi);
            assert!(rel(q.p / q.rho.powf(GAMMA), k0) < 1e-10, "ξ {xi}: isentrope {}", q.p / q.rho.powf(GAMMA));
            assert!(rel(q.u + 2.0 * q.sound() / (GAMMA - 1.0), j0) < 1e-10, "ξ {xi}: invariant");
            // and the fan is the characteristic ξ = u − c
            assert!((xi - (q.u - q.sound())).abs() < 1e-10, "ξ {xi}: u − c = {}", q.u - q.sound());
        }
        let end = sol.sample(tail + 1e-12);
        assert!(rel(end.p, sol.p_star) < 1e-9 && rel(end.rho, sol.rho_star_left) < 1e-9, "{end:?}");
    }

    #[test]
    fn every_shock_and_fan_in_the_problems_satisfies_its_jump_or_its_isentrope() {
        for (name, l, r) in problems() {
            let s = Solution::solve(l, r).unwrap();
            let star_l = State::new(s.rho_star_left, s.u_star, s.p_star);
            let star_r = State::new(s.rho_star_right, s.u_star, s.p_star);
            match s.left_wave {
                Wave::Shock { speed } => assert_jump(&l, &star_l, speed, name),
                Wave::Rarefaction { .. } => {
                    assert!(rel(star_l.p / star_l.rho.powf(GAMMA), l.p / l.rho.powf(GAMMA)) < 1e-10, "{name}: left isentrope");
                    let inv = |q: &State| q.u + 2.0 * q.sound() / (GAMMA - 1.0);
                    assert!((inv(&star_l) - inv(&l)).abs() < 1e-9 * (1.0 + inv(&l).abs()), "{name}: left invariant");
                }
            }
            match s.right_wave {
                Wave::Shock { speed } => assert_jump(&r, &star_r, speed, name),
                Wave::Rarefaction { .. } => {
                    assert!(rel(star_r.p / star_r.rho.powf(GAMMA), r.p / r.rho.powf(GAMMA)) < 1e-10, "{name}: right isentrope");
                    let inv = |q: &State| q.u - 2.0 * q.sound() / (GAMMA - 1.0);
                    assert!((inv(&star_r) - inv(&r)).abs() < 1e-9 * (1.0 + inv(&r).abs()), "{name}: right invariant");
                }
            }
        }
    }

    #[test]
    fn mirrored_states_give_the_mirrored_solution() {
        for (name, l, r) in problems() {
            let a = Solution::solve(l, r).unwrap();
            let b = Solution::solve(r.mirrored(), l.mirrored()).unwrap();
            assert!(rel(a.p_star, b.p_star) < 1e-10, "{name}: p* {} vs {}", a.p_star, b.p_star);
            assert!((a.u_star + b.u_star).abs() < 1e-9 * (1.0 + a.u_star.abs()), "{name}: u*");
            for i in -300..=300 {
                let xi = i as f64 * 0.05;
                let (q, m) = (a.sample(xi), b.sample(-xi));
                assert!(rel(q.rho, m.rho) < 1e-9 && rel(q.p, m.p) < 1e-9 && (q.u + m.u).abs() < 1e-9 * (1.0 + q.u.abs()), "{name}: ξ {xi}: {q:?} vs {m:?}");
            }
        }
    }

    /// ∫ q dξ over [−Ξ, Ξ] at t = 1 equals the initial integral plus the flux in minus the flux out — mass, momentum and
    /// energy — by Simpson between the wave positions (smooth inside each piece).
    #[test]
    fn the_solution_conserves_mass_momentum_and_energy() {
        let cons = |q: &State| {
            [q.rho, q.rho * q.u, q.p / (GAMMA - 1.0) + 0.5 * q.rho * q.u * q.u]
        };
        let flux = |q: &State| {
            let e = q.p / (GAMMA - 1.0) + 0.5 * q.rho * q.u * q.u;
            [q.rho * q.u, q.rho * q.u * q.u + q.p, (e + q.p) * q.u]
        };
        for (name, l, r) in problems() {
            let s = Solution::solve(l, r).unwrap();
            let b = s.breakpoints();
            let big = 2.0 * b.iter().fold(1.0_f64, |m, v| m.max(v.abs()));
            let mut edges = vec![-big];
            edges.extend(b.iter().copied());
            edges.push(big);
            let mut integral = [0.0; 3];
            for w in edges.windows(2) {
                let (lo, hi) = (w[0] + 1e-13, w[1] - 1e-13);
                let n = 4000;
                let h = (hi - lo) / n as f64;
                for k in 0..=n {
                    let weight = if k == 0 || k == n { 1.0 } else if k % 2 == 1 { 4.0 } else { 2.0 };
                    let c = cons(&s.sample(lo + h * k as f64));
                    for i in 0..3 {
                        integral[i] += weight * c[i] * h / 3.0;
                    }
                }
            }
            let (cl, cr) = (cons(&l), cons(&r));
            let (fl, fr) = (flux(&l), flux(&r));
            for i in 0..3 {
                let expected = (cl[i] + cr[i]) * big + fl[i] - fr[i];
                assert!(
                    (integral[i] - expected).abs() < 1e-8 * (expected.abs() + (cl[i].abs() + cr[i].abs()) * big),
                    "{name}: conserved {i}: {} vs {expected}", integral[i]
                );
            }
        }
    }

    #[test]
    fn far_from_the_waves_the_solution_is_the_initial_states() {
        for (name, l, r) in problems() {
            let s = Solution::solve(l, r).unwrap();
            let b = s.breakpoints();
            let (first, last) = (b[0], b[b.len() - 1]);
            assert_eq!(s.sample(first - 1.0), l, "{name}");
            assert_eq!(s.sample(last + 1.0), r, "{name}");
            // x and t: Sod at t = 80 of 400 cells sampled at the discontinuity's cell
            assert_eq!(s.at(200.0, 200.0, 80.0), s.sample(0.0), "{name}");
        }
    }

    #[test]
    fn a_state_with_no_solution_is_refused() {
        let ok = State::new(1.0, 0.0, 1.0);
        for bad in [State::new(0.0, 0.0, 1.0), State::new(1.0, 0.0, -1.0), State::new(f64::NAN, 0.0, 1.0), State::new(1.0, f64::INFINITY, 1.0)] {
            assert_eq!(Solution::solve(bad, ok), Err(RiemannError::BadState));
            assert_eq!(Solution::solve(ok, bad), Err(RiemannError::BadState));
        }
        // c = √2 each side: the sides recede at 2 (2√2) / (γ − 1) = 5.66 or more and a vacuum opens
        assert_eq!(Solution::solve(State::new(1.0, -3.0, 1.0), State::new(1.0, 3.0, 1.0)), Err(RiemannError::Vacuum));
        assert!(Solution::solve(State::new(1.0, -2.0, 1.0), State::new(1.0, 2.0, 1.0)).is_ok());
    }
}
