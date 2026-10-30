//! The app's panels as pure view models (m0_contrat.md §4.2): each module computes what a panel shows — its texts, its
//! states, the command each control sends — from the simulation's own types, and the egui layer in desktop.rs only draws
//! it and hands the commands back. That keeps the panels testable without a window (scopes `ui-*`).

pub mod banner;
pub mod theme;
pub mod topbar;
