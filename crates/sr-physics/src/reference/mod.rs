//! The CPU f64 twin (m0_contrat.md §1.1, R13): each pass's reference, written in f64 on the CPU before its GPU shader.
//! `fft` is gravity's machinery (§1.4.2, the Hockney–Eastwood convolution) and the neutrino-heating kernel's (§2.5.4);
//! `grav` is P2 on it (§1.4); `riemann` is the exact solution of the γ = 2 gas's shock tube (§2.3.1, G-SOD §5.1);
//! `hydro` is P4, the gas flow, graded against it (§2.3.4, G-SOD).

pub mod fft;
pub mod grav;
pub mod hydro;
pub mod riemann;
