//! M0-T3 — the cell state and the step loop (contract §2.2, §2.8, §1.3.2, §1.3.4): the canonical upload/readback
//! round trip, P0–P9's order, and P8's species renormalisation as the first pass.

use super::device;
use sr_engine::state::{State, StateError, WorldConfig, CHANNELS, N_CHANNELS, SPECIES};
use sr_engine::step::{Pass, Step};

/// A fixed sequence of values in [0, 1) — the same on every run (xorshift64*).
struct Seq(u64);

impl Seq {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        (self.0.wrapping_mul(0x2545_f491_4f6c_dd1d) >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// 64 × 64 cells: channels 0–3 a distinct value per channel and cell; the fractions unnormalised, some negative, and
/// the edge cases on a fixed pattern — all ≤ 0, all zero, only x_n positive.
fn unnormalised_field(world: WorldConfig) -> Vec<f32> {
    let cells = world.cells();
    let mut planes = vec![0f32; N_CHANNELS * cells];
    let mut seq = Seq(0x5eed_0003);
    for c in 0..SPECIES.start {
        for i in 0..cells {
            planes[c * cells + i] = (c * cells + i) as f32 * 0.25 - 1000.0;
        }
    }
    for i in 0..cells {
        for (k, c) in SPECIES.enumerate() {
            let v = match i % 8 {
                0 => 3.0 * seq.next(),               // positive, unnormalised
                1 => 3.0 * seq.next() - 1.0,         // mixed signs
                2 => -seq.next() - 1e-3,             // all negative → pure hydrogen
                3 => 0.0,                            // all zero → pure hydrogen
                4 if k == SPECIES.len() - 1 => 0.7,  // only x_n positive → x_n = 1
                4 => -0.25,
                5 => 1e-3 * seq.next(),              // tiny, sum ≪ 1
                _ => 2.5 * seq.next() - 0.5,
            };
            planes[c * cells + i] = v as f32;
        }
    }
    planes
}

/// The renormalisation in f64, the oracle for one cell's fractions: clamp ≥ 0, divide by the sum; pure hydrogen
/// when every fraction is ≤ 0 (M0-T3's declared default).
fn renormalised(x: &[f32]) -> Vec<f64> {
    let clamped: Vec<f64> = x.iter().map(|&v| (v as f64).max(0.0)).collect();
    let sum: f64 = clamped.iter().sum();
    if sum > 0.0 {
        clamped.iter().map(|v| v / sum).collect()
    } else {
        let mut h = vec![0.0; x.len()];
        h[0] = 1.0;
        h
    }
}

fn bits(v: &[f32]) -> Vec<u32> {
    v.iter().map(|x| x.to_bits()).collect()
}

#[test]
fn world_config_takes_multiples_of_8_within_64_to_2048() {
    assert_eq!(WorldConfig::default(), WorldConfig { width: 600, height: 400 });
    assert!(WorldConfig::default().validate().is_ok());
    for (w, h) in [(64, 64), (2048, 2048), (600, 400), (72, 2040)] {
        assert_eq!(WorldConfig::new(w, h).map(|c| c.cells()), Ok(w as usize * h as usize), "{w} × {h}");
    }
    for (w, h) in [(56, 64), (64, 2056), (601, 400), (600, 402), (0, 0), (4096, 64)] {
        assert_eq!(WorldConfig::new(w, h), Err(StateError::WorldSize { width: w, height: h }), "{w} × {h}");
    }
    assert_eq!(CHANNELS.len(), 14);
    assert_eq!(&CHANNELS[SPECIES], &["x_H", "x_He", "x_C", "x_O", "x_Ne", "x_Mg", "x_Si", "x_S", "x_Fe", "x_n"]);
}

#[test]
fn round_trip_600x400_is_bit_exact_with_a_distinct_value_per_channel_and_cell() {
    let gpu = device();
    let world = WorldConfig::default();
    let cells = world.cells();
    // Every value an exact, distinct f32 (|k| < 2^24), signs mixed: a channel or cell moved anywhere shows.
    let planes: Vec<f32> = (0..N_CHANNELS * cells)
        .map(|k| if k % 3 == 0 { -((k + 1) as f32) } else { (k + 1) as f32 })
        .collect();
    let state = State::new(&gpu.device, world).unwrap();
    state.upload(&gpu.queue, &planes).unwrap();
    let back = state.readback(&gpu.device, &gpu.queue);
    assert_eq!(back.len(), planes.len());
    if let Some(k) = (0..planes.len()).find(|&k| back[k].to_bits() != planes[k].to_bits()) {
        panic!(
            "first difference: channel {} cell {}: uploaded {}, read back {}",
            CHANNELS[k / cells],
            k % cells,
            planes[k],
            back[k]
        );
    }
    println!("round trip {} × {}: {} values bit-exact", world.width, world.height, planes.len());
}

#[test]
fn upload_refuses_anything_but_14_planes_of_the_world() {
    let gpu = device();
    let world = WorldConfig::new(64, 64).unwrap();
    let state = State::new(&gpu.device, world).unwrap();
    let expected = N_CHANNELS * world.cells();
    for got in [0, expected - 1, expected + 1, 13 * world.cells()] {
        assert_eq!(
            state.upload(&gpu.queue, &vec![0.0; got]),
            Err(StateError::PlanesLength { expected, got }),
            "{got} values"
        );
    }
    assert!(State::new(&gpu.device, WorldConfig { width: 60, height: 64 }).is_err(), "a world off §2.8's sizes");
}

#[test]
fn a_step_is_p0_to_p9_with_p8s_renormalisation_its_only_dispatch_today() {
    let labels: Vec<&str> = Pass::ORDER.iter().map(|p| p.label()).collect();
    for (i, label) in labels.iter().enumerate() {
        assert!(label.starts_with(&format!("P{i} ")), "{labels:?}");
    }
    let gpu = device();
    let state = State::new(&gpu.device, WorldConfig::new(64, 64).unwrap()).unwrap();
    let step = Step::new(&gpu.device, &state);
    let mut expected = [0usize; 10];
    expected[Pass::Floors as usize] = 1;
    assert_eq!(step.dispatch_counts(), expected);
}

#[test]
fn one_step_renormalises_the_species_and_leaves_the_other_channels_bit_identical() {
    let gpu = device();
    let world = WorldConfig::new(64, 64).unwrap();
    let cells = world.cells();
    let before = unnormalised_field(world);
    let state = State::new(&gpu.device, world).unwrap();
    state.upload(&gpu.queue, &before).unwrap();
    Step::new(&gpu.device, &state).run(&gpu.device, &gpu.queue, 1);
    let after = state.readback(&gpu.device, &gpu.queue);

    let other = 0..SPECIES.start * cells;
    assert_eq!(bits(&after[other.clone()]), bits(&before[other]), "channels sigma … energy changed");

    let (mut worst_sum, mut worst_x) = (0f64, 0f64);
    for i in 0..cells {
        let x_before: Vec<f32> = SPECIES.map(|c| before[c * cells + i]).collect();
        let x_after: Vec<f32> = SPECIES.map(|c| after[c * cells + i]).collect();
        let oracle = renormalised(&x_before);
        let sum: f64 = x_after.iter().map(|&v| v as f64).sum();
        assert!(x_after.iter().all(|&v| v >= 0.0), "cell {i}: a negative fraction {x_after:?}");
        assert!((sum - 1.0).abs() <= 1e-6, "cell {i}: Σ X = {sum} (from {x_before:?}, now {x_after:?})");
        for (k, (&got, want)) in x_after.iter().zip(&oracle).enumerate() {
            let err = (got as f64 - want).abs();
            assert!(err <= 1e-6, "cell {i}, {}: {got}, the f64 oracle {want}", CHANNELS[SPECIES.start + k]);
            worst_x = worst_x.max(err);
        }
        worst_sum = worst_sum.max((sum - 1.0).abs());
    }
    println!("{cells} cells: worst |Σ X − 1| = {worst_sum:e}, worst |X − oracle| = {worst_x:e} (tolerance 1e-6)");
}

#[test]
fn two_runs_from_the_same_state_are_bit_identical() {
    let gpu = device();
    let world = WorldConfig::new(64, 64).unwrap();
    let start = unnormalised_field(world);
    let run = || {
        let state = State::new(&gpu.device, world).unwrap();
        state.upload(&gpu.queue, &start).unwrap();
        Step::new(&gpu.device, &state).run(&gpu.device, &gpu.queue, 3);
        state.readback(&gpu.device, &gpu.queue)
    };
    let (a, b) = (run(), run());
    assert_eq!(bits(&a), bits(&b), "two runs of 3 steps differ");
    assert_ne!(bits(&a), bits(&start), "the step changed nothing: the case would pass with no pass at all");
}
