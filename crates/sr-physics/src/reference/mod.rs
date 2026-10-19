//! The CPU f64 twin (m0_contrat.md §1.1, R13): each pass's reference, written in f64 on the CPU before its GPU shader.
//! `fft` is gravity's machinery (§1.4.2, the Hockney–Eastwood convolution) and the neutrino-heating kernel's (§2.5.4);
//! `grav` is P2 on it (§1.4); `riemann` is the exact solution of the γ = 2 gas's shock tube (§2.3.1, G-SOD §5.1);
//! `hydro` is P4, the gas flow, graded against it (§2.3.4, G-SOD); `heat` is P5, flux-limited diffusion stepped implicitly
//! (§2.4, G-HEAT1, G-HEAT2, G-FLD); `react` is P6, burning by linearised backward-Euler sub-cycles (§2.5, G-BURN, G-ORDER); `floors` is P8 (§2.7); `sinks` is P3, the black holes' formation, accretion and motion (§2.6, G-SINK), not yet in the twin's step; `nuheat` is P7, the neutron star's neutrinos deposited in the gas around it (§2.5.4), not yet in the twin's step either. `sheet` is no pass: the thin disk's cold equilibria in the continuum, the ceiling `calibrate` computes (§2.3.2, §2.11 items 1–2, G-CORE).
//!
//! [`Twin`] is the step driver (M0-T37): the passes built so far, in §1.3.2's order, on the state's f64 planes, the scene's
//! switches honoured — P1 Δt, P2 gravity, P4 hydro, P5 heat, P6 reactions, P8 floors, P9 Δt_{n+1} — each pass joining with its twin. It steps the
//! GPU's way: Δt_n is the reduction P9 made at the end of step n − 1 (step 0's, the initial state's after P2 on it, so it
//! has its gravity term), and P9's gravity term reads g from the same step's P2 — the field of the state the step started
//! from — not from the state P9 reduces. The twin runs the whole world, no active box (§1.3.3); outside the GPU's box the
//! vacuum is left alone where the twin's P8 resets it to the floor, a difference of Σ_floor per cell (G-BOX's, §5.3).

pub mod fft;
pub mod floors;
pub mod grav;
pub mod heat;
pub mod hydro;
pub mod nuheat;
pub mod react;
pub mod riemann;
pub mod sheet;
pub mod sinks;

use std::fmt;

use crate::registry::{Elements, Physics, RegistryError, Reactions, N_SPECIES};
use floors::{Booked, Floors};
use grav::{Field, Gravity};
use heat::Heat;
use hydro::{Edge, Hydro, State, COMPONENTS};
use react::React;

/// The largest world the twin runs on each axis (§3.3's `--cpu-reference`): it is a reference, not a player.
pub const MAX_WORLD: usize = 128;
/// G = 1 in the sandbox's units (§2.1).
const G: f64 = 1.0;
/// The guard looks at step indices ≡ 0 (mod `GUARD_EVERY`), as P9's does (§3.3, exit 5).
pub const GUARD_EVERY: u64 = 64;

/// The passes the twin has, as scene switches (§2.12.1): the others have no twin yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Switches {
    pub gravity: bool,
    pub hydro: bool,
    /// P5 (M0-T48): on needs [`Burning`] at [`Twin::new`].
    pub heat: bool,
    /// P6 (M0-T48): on needs [`Burning`] at [`Twin::new`].
    pub reactions: bool,
}

/// What P5 and P6 read beyond the switches (§2.12.1's `test`, the registry's records, the scene's `neutrinos`): given to
/// [`Twin::new`] whenever `heat` or `reactions` is on, and ignored otherwise.
pub struct Burning<'a> {
    pub heat: heat::TestFlags,
    pub react: react::TestFlags,
    pub reactions: &'a Reactions,
    pub neutrinos: bool,
}

#[derive(Debug)]
pub enum TwinError {
    /// A world axis over [`MAX_WORLD`].
    TooLarge { width: usize, height: usize },
    /// P5 or P6 switched on without the [`Burning`] they read.
    NoBurning,
    Constants(RegistryError),
}

impl fmt::Display for TwinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TwinError::TooLarge { width, height } => {
                write!(f, "a {width}x{height} world: the CPU twin runs up to {MAX_WORLD}x{MAX_WORLD}")
            }
            TwinError::NoBurning => write!(f, "heat or reactions switched on without the burning setup (Burning)"),
            TwinError::Constants(e) => write!(f, "{e}"),
        }
    }
}

impl From<RegistryError> for TwinError {
    fn from(e: RegistryError) -> Self {
        TwinError::Constants(e)
    }
}

/// The twin of a run: the state in f64 and the records of the step.
pub struct Twin {
    /// §2.2's channels in f64, channel-major ([`hydro::State`]), and what has left through the edges.
    pub state: State,
    /// What P8 has booked since the twin was made.
    pub booked: Booked,
    switches: Switches,
    hydro: Hydro,
    floors: Floors,
    gravity: Option<Gravity>,
    /// P5 and P6, when their switches are on.
    heat: Option<Heat>,
    react: Option<React>,
    /// F of the last P5 pass, the next step's P4 radiation force (`None` before step 0: zero, as on the GPU).
    flux: Option<(Vec<f64>, Vec<f64>)>,
    sigma_vac: f64,
    /// P2's g for the state the last step started from (the initial state's before step 0).
    g: Option<Field>,
    step: u64,
    sim_time: f64,
    dt: f64,
    dt_next: f64,
    /// The passes the last step ran, by the GPU's labels (`Pass::label`).
    ran: Vec<&'static str>,
}

impl Twin {
    /// A twin of `planes` (§2.2's canonical f64 planes of a `width` × `height` world) under `physics`: the step index 0,
    /// P2 run on the state and Δt_0 reduced from it.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        physics: &Physics,
        elements: &Elements,
        edge: Edge,
        switches: Switches,
        burning: Option<Burning<'_>>,
        width: usize,
        height: usize,
        planes: Vec<f64>,
    ) -> Result<Twin, TwinError> {
        if width > MAX_WORLD || height > MAX_WORLD {
            return Err(TwinError::TooLarge { width, height });
        }
        if (switches.heat || switches.reactions) && burning.is_none() {
            return Err(TwinError::NoBurning);
        }
        let (heat, react) = match burning {
            Some(b) if switches.heat || switches.reactions => (
                switches.heat.then(|| Heat::new(physics, elements, b.heat)).transpose()?,
                switches.reactions.then(|| React::new(physics, elements, b.reactions, b.react).map(|r| r.with_neutrinos(b.neutrinos))).transpose()?,
            ),
            _ => (None, None),
        };
        let mut twin = Twin {
            state: State::new(width, height, planes),
            booked: Booked::default(),
            switches,
            hydro: Hydro::new(physics, elements, edge)?,
            floors: Floors::new(physics, elements)?,
            gravity: switches.gravity.then(|| Gravity::new(G)),
            heat,
            react,
            flux: None,
            sigma_vac: physics.get("sigma_vac").expect("physics.json key sigma_vac"),
            g: None,
            step: 0,
            sim_time: 0.0,
            dt: 0.0,
            dt_next: 0.0,
            ran: Vec::new(),
        };
        twin.solve_gravity();
        twin.dt_next = twin.reduce_dt();
        Ok(twin)
    }

    /// The passes a step runs with these switches, by the GPU's labels (`Pass::label`).
    pub fn pass_list(switches: Switches) -> Vec<&'static str> {
        let mut list = vec!["P1 dt"];
        if switches.gravity {
            list.push("P2 gravity");
        }
        if switches.hydro {
            list.push("P4 hydro");
        }
        if switches.heat {
            list.push("P5 heat");
        }
        if switches.reactions {
            list.push("P6 reactions");
        }
        list.extend(["P8 floors", "P9 reductions"]);
        list
    }

    /// The passes the last step ran, in the order it ran them; empty before the first step.
    pub fn passes_run(&self) -> &[&'static str] {
        &self.ran
    }

    /// The index of the next step (the steps run since the scene).
    pub fn step_index(&self) -> u64 {
        self.step
    }

    /// The sim time of the steps run: each Δt added in step order.
    pub fn sim_time(&self) -> f64 {
        self.sim_time
    }

    /// Δt_n of the last step run (0 before the first).
    pub fn dt(&self) -> f64 {
        self.dt
    }

    /// Δt_{n+1}: the next step's, as P9 reduced it.
    pub fn dt_next(&self) -> f64 {
        self.dt_next
    }

    /// P2 on the state as it stands (a no-op without gravity).
    fn solve_gravity(&mut self) {
        let (w, h) = (self.state.width, self.state.height);
        if let Some(gravity) = &mut self.gravity {
            let field = gravity.field(self.state.plane(0), self.sigma_vac, w, h).expect("a world within MAX_WORLD pads within the FFT");
            self.g = Some(field);
        }
    }

    /// Δt (§2.3.5) of the state as it stands, the gravity term from the field P2 last solved.
    fn reduce_dt(&self) -> f64 {
        let g = self.g.as_ref().map(|f| (&f.gx[..], &f.gy[..]));
        self.hydro.dt(&self.state, g)
    }

    /// One step, §1.3.2's order.
    pub fn step(&mut self) {
        self.ran.clear();
        // P1: this step runs on the Δt P9 made at the end of the last, and adds it to the sim time.
        self.ran.push("P1 dt");
        let dt = self.dt_next;
        self.dt = dt;
        self.sim_time += dt;
        if self.switches.gravity {
            self.ran.push("P2 gravity");
            self.solve_gravity();
        }
        if self.switches.hydro {
            self.ran.push("P4 hydro");
            let g = self.g.as_ref().map(|f| (&f.gx[..], &f.gy[..]));
            let radiation = self.flux.as_ref().map(|(x, y)| (&x[..], &y[..]));
            self.hydro.step_forced(&mut self.state, dt, g, radiation);
        }
        if let Some(heat) = &self.heat {
            self.ran.push("P5 heat");
            let pass = heat.step(&mut self.state, dt);
            self.flux = Some(pass.flux.cells());
        }
        if let Some(react) = &self.react {
            self.ran.push("P6 reactions");
            react.step(&mut self.state, dt);
        }
        self.ran.push("P8 floors");
        self.booked += self.floors.apply(&mut self.state);
        self.ran.push("P9 reductions");
        self.dt_next = self.reduce_dt();
        self.step += 1;
    }

    /// The first non-finite value in the state, by cell and then channel as P9's guard finds it: (cell, channel).
    pub fn first_non_finite(&self) -> Option<(usize, usize)> {
        let n = self.state.cells();
        (0..n).find_map(|i| (0..COMPONENTS).find(|&c| !self.state.planes[c * n + i].is_finite()).map(|c| (i, c)))
    }
}

const _: () = assert!(COMPONENTS == 4 + N_SPECIES, "the twin's state is §2.2's fourteen channels");
