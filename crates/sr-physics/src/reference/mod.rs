//! The CPU f64 twin (m0_contrat.md §1.1, R13): each pass's reference, written in f64 on the CPU before its GPU shader.
//! `fft` is gravity's machinery (§1.4.2, the Hockney–Eastwood convolution) and the neutrino-heating kernel's (§2.5.4);
//! `grav` is P2 on it (§1.4).

pub mod fft;
pub mod grav;
