//! Time control (m0_contrat.md §1.8, §3.2): the rungs, how many steps a frame runs, the cap and its readout, pause and
//! step, the automatic slow-down and the top rung's 30-frames switch. Pure logic, no GPU: the app hands it each frame's
//! wall time and each read-back frame, and passes its plan to the engine's `encode_frame` (M0-T22). Time never touches
//! the physics (§1.8.7) — nothing here reaches Δt, only how many steps a frame runs.

use std::collections::VecDeque;

/// §1.8.2's ladder, each rung with its label as written (§4.12).
pub const LADDER: [(f64, &str); 9] = [
    (0.1, "0.1"),
    (0.3, "0.3"),
    (1.0, "1"),
    (3.0, "3"),
    (10.0, "10"),
    (30.0, "30"),
    (100.0, "100"),
    (300.0, "300"),
    (1000.0, "1000"),
];
/// The frame rate every rung aims at, and the top rung's once its switch engages (§1.8.6, Q2 — FROZEN).
pub const F_TARGET: u32 = 60;
pub const F_TARGET_TOP: u32 = 30;
/// R_reserve until calibration's `render_reserve_ms` exists (§1.8.3).
pub const RENDER_RESERVE_MS: f64 = 3.0;
/// c_step's moving average, in frames (§1.8.3).
const COST_FRAMES: usize = 16;
/// The cap's window and how many of its frames must bind (§1.8.4); the switch reads N_max at 60 the same way (§1.8.6).
const CAP_WINDOW: usize = 60;
const CAP_BINDS: usize = 30;
/// Consecutive frames needing fewer than half of N_max at 60 that return the top rung to 60 frames a second (§1.8.6).
const QUIET_FRAMES: u32 = 120;
/// A Δt at or above this is not a CFL step: an all-vacuum world's box is empty and its Δt reduces to the largest finite
/// f32 (M0-T21). Such a frame carries no estimate and owes nothing.
const DT_EMPTY: f64 = f32::MAX as f64;

/// A speed setting: sandbox time units per wall second, and the label `time.rung` shows (§4.3, §4.12).
#[derive(Clone, Debug, PartialEq)]
pub struct Rung {
    pub value: f64,
    pub label: String,
}

/// The events the slow-down answers (§1.8.5) and the banner each one shows (§4.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Ignition,
    CoreCollapse,
    Supernova,
}

impl Event {
    /// The rung the slow-down sets: ×1 for ignition, ×0.1 for core collapse and supernova.
    pub fn rung_value(self) -> f64 {
        match self {
            Event::Ignition => 1.0,
            Event::CoreCollapse | Event::Supernova => 0.1,
        }
    }

    pub fn banner_key(self) -> &'static str {
        match self {
            Event::Ignition => "banner.ignition",
            Event::CoreCollapse => "banner.core_collapse",
            Event::Supernova => "banner.supernova",
        }
    }
}

/// What the app measured since the previous plan: the wall time of the frame just presented, in seconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Estimates {
    pub wall_s: f64,
}

/// A frame's plan: the engine's `FramePlan {steps, slow_down}` and the frame rate to pace it at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Plan {
    pub steps: u32,
    pub f_target: u32,
    pub slow_down: bool,
}

/// A frame as read back (the engine's `FrameReport`, ≤ 2 frames late): steps run, the sim time they advanced, the next
/// step's Δt, the mean cost per timed step (none while timing is off) and whether the latch stopped the frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Report {
    pub steps_run: u32,
    pub advanced: f64,
    pub dt_next: f64,
    pub step_ms: Option<f64>,
    pub latch_fired: bool,
}

/// What the speed readout shows (§4.3): the rung's label, the running speed while the cap binds, and paused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpeedText {
    pub rung: String,
    pub running: Option<String>,
    pub paused: bool,
}

/// §1.8.2: TOP = life_sun_sb ÷ 10 s, rounded to two significant digits.
pub fn top_rung_from_life(life_sun_sb: f64) -> f64 {
    round_sig2(life_sun_sb / 10.0)
}

fn round_sig2(x: f64) -> f64 {
    if !x.is_finite() || x <= 0.0 {
        return x;
    }
    let scale = 10f64.powi(x.log10().floor() as i32 - 1);
    (x / scale).round() * scale
}

/// Two significant digits, as TOP and `{actual}` are shown (§4.12): "42", "0.83", "1.0", "1200".
pub fn sig2(x: f64) -> String {
    let r = round_sig2(x);
    if !r.is_finite() || r <= 0.0 {
        return "0".to_string();
    }
    let decimals = (1 - r.log10().floor() as i32).max(0) as usize;
    format!("{r:.decimals$}")
}

/// Steps that pay `owed` at `dt` each: ⌈owed / dt⌉, 0 when nothing is owed.
fn steps_for(owed: f64, dt: f64) -> u32 {
    if owed <= 0.0 {
        0
    } else {
        (owed / dt).ceil().min(u32::MAX as f64) as u32
    }
}

fn push<T>(window: &mut VecDeque<T>, value: T, len: usize) {
    if window.len() == len {
        window.pop_front();
    }
    window.push_back(value);
}

/// A real sim-time quantity, as opposed to the empty world's Δt or a non-finite sum.
fn real(t: f64) -> bool {
    t.is_finite() && t > 0.0 && t < DT_EMPTY
}

#[derive(Debug)]
pub struct TimeControl {
    rungs: Vec<Rung>,
    rung: usize,
    paused: bool,
    slow_down: bool,
    f_target: u32,
    /// Sim time owed, net of what the frames in flight planned (§1.8.3); bounded to [−Δt_est, rung / f_target].
    owed: f64,
    reserve_ms: f64,
    /// The last Δt read back; none before the first report and while the world is empty.
    dt_est: Option<f64>,
    /// Single steps asked while paused, one per frame.
    pending_steps: u32,
    banner: Option<Event>,
    /// Sim time each submitted frame planned, oldest first; none for a single step, which owes nothing.
    in_flight: VecDeque<Option<f64>>,
    costs: VecDeque<f64>,
    binds: VecDeque<bool>,
    walls: VecDeque<f64>,
    advanced: VecDeque<f64>,
    top_binds: VecDeque<bool>,
    quiet: u32,
}

impl TimeControl {
    /// `top_rung` and `render_reserve_ms` from calibration (§2.11); until it exists the nine ladder rungs and 3.0 ms.
    /// The game starts at ×1, running, the slow-down on.
    pub fn new(top_rung: Option<f64>, render_reserve_ms: Option<f64>) -> Result<TimeControl, String> {
        let rungs: Vec<Rung> = match top_rung {
            None => LADDER.iter().map(|&(value, label)| Rung { value, label: label.to_string() }).collect(),
            Some(top) if top.is_finite() && top > 0.0 => LADDER
                .iter()
                .filter(|&&(value, _)| value < top / 1.5)
                .map(|&(value, label)| Rung { value, label: label.to_string() })
                .chain(std::iter::once(Rung { value: top, label: sig2(top) }))
                .collect(),
            Some(top) => return Err(format!("top_rung: {top} is not a positive finite number")),
        };
        let reserve_ms = render_reserve_ms.unwrap_or(RENDER_RESERVE_MS);
        if !reserve_ms.is_finite() || reserve_ms < 0.0 {
            return Err(format!("render_reserve_ms: {reserve_ms} is not a finite number ≥ 0"));
        }
        let rung = rungs.iter().rposition(|r| r.value <= 1.0).unwrap_or(0);
        Ok(TimeControl {
            rungs,
            rung,
            paused: false,
            slow_down: true,
            f_target: F_TARGET,
            owed: 0.0,
            reserve_ms,
            dt_est: None,
            pending_steps: 0,
            banner: None,
            in_flight: VecDeque::new(),
            costs: VecDeque::new(),
            binds: VecDeque::new(),
            walls: VecDeque::new(),
            advanced: VecDeque::new(),
            top_binds: VecDeque::new(),
            quiet: 0,
        })
    }

    pub fn rungs(&self) -> &[Rung] {
        &self.rungs
    }

    pub fn rung(&self) -> usize {
        self.rung
    }

    pub fn paused(&self) -> bool {
        self.paused
    }

    pub fn f_target(&self) -> u32 {
        self.f_target
    }

    pub fn banner(&self) -> Option<Event> {
        self.banner
    }

    pub fn slow_down(&self) -> bool {
        self.slow_down
    }

    pub fn set_slow_down(&mut self, on: bool) {
        self.slow_down = on;
    }

    /// Selects rung `k` of `rungs()` (keys 1…n are k = 0…n−1); out of range is ignored.
    pub fn select(&mut self, k: usize) {
        if k < self.rungs.len() {
            self.set_rung(k, None);
        }
    }

    pub fn faster(&mut self) {
        self.select((self.rung + 1).min(self.rungs.len() - 1));
    }

    pub fn slower(&mut self) {
        self.select(self.rung.saturating_sub(1));
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        self.pending_steps = 0;
    }

    /// While paused, exactly one step (run by the next plan); while running, it pauses.
    pub fn step_once(&mut self) {
        if self.paused {
            self.pending_steps += 1;
        } else {
            self.paused = true;
        }
    }

    /// One frame (§1.8.3): owed += rung / f_target, N = min(N_max, ⌈owed / Δt_est⌉), what N planned taken off owed
    /// until its report books what it advanced. No estimate yet (the first frames, an empty world): one step.
    pub fn plan_frame(&mut self, est: Estimates) -> Plan {
        let (f_target, slow_down) = (self.f_target, self.slow_down);
        let plan = move |steps| Plan { steps, f_target, slow_down };
        if self.paused {
            if self.pending_steps == 0 {
                return plan(0);
            }
            self.pending_steps -= 1;
            self.in_flight.push_back(None);
            return plan(1);
        }
        push(&mut self.walls, est.wall_s, CAP_WINDOW);
        self.owed += self.rungs[self.rung].value / self.f_target as f64;
        let n_max = self.n_max(self.f_target);
        let (steps, planned, bind) = match self.dt_est {
            Some(dt) => {
                let need = steps_for(self.owed, dt);
                (need.min(n_max), need.min(n_max) as f64 * dt, need > n_max)
            }
            None => (u32::from(self.owed > 0.0), 0.0, false),
        };
        self.owed -= planned;
        push(&mut self.binds, bind, CAP_WINDOW);
        let out = plan(steps);
        self.update_switch();
        self.bound_owed();
        if steps > 0 {
            self.in_flight.push_back(Some(planned));
        }
        out
    }

    /// A frame read back: owed −= the sim time it advanced (its plan's estimate given back), owed ≥ −Δt_est; the latch's
    /// stopped steps are never owed. Call it once per frame read back, in order.
    pub fn on_report(&mut self, report: &Report) {
        self.dt_est = real(report.dt_next).then_some(report.dt_next);
        if report.steps_run == 0 {
            return;
        }
        let planned = self.in_flight.pop_front().flatten();
        if !real(report.advanced) {
            // The empty world's frame: its sim time is not time anything owes.
            self.owed = 0.0;
        } else {
            push(&mut self.advanced, report.advanced, CAP_WINDOW);
            if let Some(planned) = planned {
                self.owed += planned - report.advanced;
            }
        }
        if report.latch_fired {
            self.owed = self.owed.min(0.0);
        } else if let Some(ms) = report.step_ms.filter(|ms| ms.is_finite() && *ms > 0.0) {
            push(&mut self.costs, ms, COST_FRAMES);
        }
        self.bound_owed();
    }

    /// The slow-down (§1.8.5): only ever slower, the banner until the player changes the rung; off, nothing.
    pub fn on_event(&mut self, event: Event) {
        if !self.slow_down {
            return;
        }
        let target = self.rungs.iter().rposition(|r| r.value <= event.rung_value()).unwrap_or(0);
        if target < self.rung {
            self.set_rung(target, Some(event));
        }
    }

    /// The readout (§4.3): `{rung}`, `{actual}` when N_max bound in ≥ 30 of the last 60 frames (§1.8.4), paused.
    pub fn speed_text(&self) -> SpeedText {
        let capped = self.binds.iter().filter(|&&b| b).count() >= CAP_BINDS;
        let wall: f64 = self.walls.iter().sum();
        let running = (capped && wall > 0.0).then(|| sig2(self.advanced.iter().sum::<f64>() / wall));
        SpeedText { rung: self.rungs[self.rung].label.clone(), running, paused: self.paused }
    }

    /// N_max = max(1, ⌊(1000 / f − R_reserve) / c_step⌋), c_step the mean of the last 16 frames' cost per step; 1 until a
    /// cost is read back.
    fn n_max(&self, f: u32) -> u32 {
        if self.costs.is_empty() {
            return 1;
        }
        let c_step = self.costs.iter().sum::<f64>() / self.costs.len() as f64;
        let n = ((1000.0 / f as f64 - self.reserve_ms) / c_step).floor();
        n.clamp(1.0, u32::MAX as f64) as u32
    }

    fn at_top(&self) -> bool {
        self.rung == self.rungs.len() - 1
    }

    /// §1.8.6: at TOP, 30 frames a second once N_max at 60 binds in ≥ 30 of the last 60 frames; back to 60 after 120
    /// consecutive frames needing fewer than half of N_max at 60. owed per frame follows f_target, so the sim time per
    /// wall second does not change.
    fn update_switch(&mut self) {
        let Some(dt) = self.dt_est.filter(|_| self.at_top()) else { return };
        let n60 = self.n_max(F_TARGET) as u64;
        let need60 = steps_for(self.rungs[self.rung].value / F_TARGET as f64, dt) as u64;
        push(&mut self.top_binds, need60 > n60, CAP_WINDOW);
        self.quiet = if 2 * need60 < n60 { self.quiet + 1 } else { 0 };
        if self.f_target == F_TARGET && self.top_binds.iter().filter(|&&b| b).count() >= CAP_BINDS {
            self.f_target = F_TARGET_TOP;
            self.quiet = 0;
        } else if self.f_target == F_TARGET_TOP && self.quiet >= QUIET_FRAMES {
            self.f_target = F_TARGET;
            self.top_binds.clear();
        }
    }

    fn set_rung(&mut self, k: usize, banner: Option<Event>) {
        if k == self.rung {
            return;
        }
        self.rung = k;
        self.banner = banner;
        self.f_target = F_TARGET;
        self.binds.clear();
        self.walls.clear();
        self.advanced.clear();
        self.top_binds.clear();
        self.quiet = 0;
        self.bound_owed();
    }

    /// owed ≥ −Δt_est (§1.8.3), and never more than one frame's sim time: what the cap or the latch left unrun is shown by
    /// the readout, never repaid later at another rung.
    fn bound_owed(&mut self) {
        let hi = self.rungs[self.rung].value / self.f_target as f64;
        self.owed = self.owed.clamp(-self.dt_est.unwrap_or(0.0), hi);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plays frames without latency — frame i's report arrives before frame i + 1's plan — every step costing `ms`.
    struct Driver {
        tc: TimeControl,
        last: Option<(u32, f64)>,
        ms: f64,
    }

    impl Driver {
        /// c_step filled at 1 ms (20 frames at TOP), then a rung change clears the windows and lands on rung `k`.
        fn primed(top: Option<f64>, k: usize) -> Driver {
            let mut d = Driver { tc: TimeControl::new(top, None).unwrap(), last: None, ms: 1.0 };
            d.tc.select(d.tc.rungs().len() - 1);
            for _ in 0..20 {
                d.frame(1.5);
            }
            d.tc.select(0);
            d.tc.select(k);
            d
        }

        /// One frame, planned with Δt_est = `dt`; returns its steps.
        fn frame(&mut self, dt: f64) -> u32 {
            let (n, prev) = self.last.take().unwrap_or((0, dt));
            let advanced = n as f64 * prev;
            self.tc.on_report(&Report { steps_run: n, advanced, dt_next: dt, step_ms: Some(self.ms), latch_fired: false });
            let plan = self.tc.plan_frame(Estimates { wall_s: 1.0 / self.tc.f_target() as f64 });
            self.last = Some((plan.steps, dt));
            plan.steps
        }

        fn value(&self) -> f64 {
            self.tc.rungs()[self.tc.rung()].value
        }
    }

    /// A report no plan is waiting for: it feeds Δt_est and c_step and books nothing.
    fn cost(tc: &mut TimeControl, ms: f64, latch_fired: bool) {
        tc.on_report(&Report { steps_run: 1, advanced: 0.004, dt_next: 0.004, step_ms: Some(ms), latch_fired });
    }

    #[test]
    fn starts_at_x1_running() {
        for top in [None, Some(42.0)] {
            let mut tc = TimeControl::new(top, None).unwrap();
            assert_eq!((tc.rungs()[tc.rung()].value, tc.paused(), tc.slow_down(), tc.f_target()), (1.0, false, true, 60));
            assert_eq!(tc.speed_text(), SpeedText { rung: "1".into(), running: None, paused: false });
            assert_eq!(tc.plan_frame(Estimates { wall_s: 1.0 / 60.0 }).steps, 1, "no estimate yet: one step");
        }
    }

    #[test]
    fn nine_ladder_rungs_until_calibration() {
        let tc = TimeControl::new(None, None).unwrap();
        let labels: Vec<&str> = tc.rungs().iter().map(|r| r.label.as_str()).collect();
        assert_eq!(labels, LADDER.map(|(_, l)| l));
    }

    #[test]
    fn rungs_below_top_over_1_5_then_top() {
        let values = |top| TimeControl::new(Some(top), None).unwrap().rungs().iter().map(|r| r.value).collect::<Vec<_>>();
        assert_eq!(values(45.0), [0.1, 0.3, 1.0, 3.0, 10.0, 45.0], "30 = 45 / 1.5 is not below it");
        assert_eq!(values(45.1), [0.1, 0.3, 1.0, 3.0, 10.0, 30.0, 45.1]);
        assert_eq!(values(4200.0).len(), 10);
        assert_eq!(TimeControl::new(Some(42.0), None).unwrap().rungs().last().unwrap().label, "42");
        assert!(TimeControl::new(Some(f64::NAN), None).unwrap_err().starts_with("top_rung"));
        assert_eq!([423.0, 425.0, 4249.0].map(top_rung_from_life), [42.0, 43.0, 420.0]);
    }

    #[test]
    fn select_faster_slower_stop_at_the_ends() {
        let mut tc = TimeControl::new(None, None).unwrap();
        tc.select(9);
        assert_eq!(tc.rung(), 2, "out of range is ignored");
        tc.select(8);
        tc.faster();
        assert_eq!(tc.rung(), 8);
        tc.select(0);
        tc.slower();
        assert_eq!(tc.rung(), 0);
        tc.faster();
        assert_eq!(tc.rung(), 1);
    }

    #[test]
    fn a_frame_runs_ceil_owed_over_dt() {
        let mut d = Driver::primed(None, 2);
        let steps: Vec<u32> = (0..600).map(|_| d.frame(0.004)).collect();
        assert!(steps[1..].iter().all(|&n| n == 4 || n == 5), "⌈(1/60) / 0.004⌉ with the carry: {steps:?}");
        let total: u32 = steps.iter().sum();
        assert!((total as f64 - 2500.0).abs() <= 6.0, "10 s at ×1 is 2500 steps of 0.004: {total}");
    }

    #[test]
    fn slow_motion_owed_floored_at_minus_dt() {
        let mut d = Driver::primed(None, 0);
        let mut at = Vec::new();
        for i in 0..3000 {
            let n = d.frame(0.447);
            assert!(n <= 1 && d.tc.owed >= -0.447 - 1e-12, "frame {i}: {n} steps, owed {}", d.tc.owed);
            if n == 1 {
                at.push(i);
            }
        }
        // 0.447 / (0.1 / 60) = 268.2 frames per step.
        assert!(at.len() >= 11 && at.windows(2).all(|w| w[1] - w[0] == 268 || w[1] - w[0] == 269), "{at:?}");
    }

    #[test]
    fn n_max_from_the_16_frame_cost_average() {
        let mut tc = TimeControl::new(None, None).unwrap();
        assert_eq!(tc.n_max(60), 1, "no cost read back yet");
        (0..16).for_each(|_| cost(&mut tc, 1.0, false));
        assert_eq!(tc.n_max(60), 13, "⌊(16.67 − 3) / 1⌋");
        cost(&mut tc, 0.01, true);
        assert_eq!(tc.n_max(60), 13, "a frame the latch stopped is skipped");
        (0..8).for_each(|_| cost(&mut tc, 0.5, false));
        assert_eq!(tc.n_max(60), 18, "the oldest 8 dropped: mean 0.75");
        (0..8).for_each(|_| cost(&mut tc, 0.5, false));
        assert_eq!((tc.n_max(60), tc.n_max(30)), (27, 60));
        cost(&mut tc, 1e6, false);
        assert_eq!(tc.n_max(60), 1);
        let mut tight = TimeControl::new(None, Some(20.0)).unwrap();
        cost(&mut tight, 1.0, false);
        assert_eq!(tight.n_max(60), 1, "the reserve over the frame: still one step");
    }

    #[test]
    fn cap_readout_at_30_of_the_last_60_frames() {
        let mut d = Driver::primed(None, 2);
        (0..31).for_each(|_| _ = d.frame(0.01));
        (0..29).for_each(|_| _ = d.frame(0.001));
        assert_eq!(d.tc.speed_text().running, None, "29 of 60");
        d.frame(0.001);
        assert!(d.tc.speed_text().running.is_some(), "30 of 60");
        (0..60).for_each(|_| _ = d.frame(0.001));
        assert_eq!(d.tc.speed_text().running.as_deref(), Some("0.78"), "13 steps × 0.001 per 1/60 s");
    }

    #[test]
    fn capped_10000_frames_then_slow_motion_holds() {
        let mut d = Driver::primed(None, 8);
        for i in 0..10_000 {
            d.frame(0.001);
            assert!((-0.001..=1000.0 / 30.0 + 1e-9).contains(&d.tc.owed), "frame {i}: owed {}", d.tc.owed);
        }
        d.tc.select(0);
        let steps: u32 = (0..200).map(|_| d.frame(0.447)).sum();
        assert!(steps <= 1, "no backlog repaid at ×0.1: {steps} steps in 200 frames");
    }

    #[test]
    fn pause_and_step_once() {
        let mut d = Driver::primed(None, 2);
        d.frame(0.447);
        d.tc.toggle_pause();
        let owed = d.tc.owed;
        assert_eq!((0..100).map(|_| d.frame(0.447)).sum::<u32>(), 0);
        d.tc.step_once();
        d.tc.step_once();
        assert_eq!([0; 4].map(|_| d.frame(0.447)), [1, 1, 0, 0], "exactly one step per press");
        assert_eq!(d.tc.owed, owed, "a single step owes nothing");
        d.tc.toggle_pause();
        d.tc.step_once();
        assert!(d.tc.paused(), "running, Step pauses");
        assert_eq!(d.frame(0.447), 0);
    }

    #[test]
    fn slow_down_only_ever_slower() {
        let mut d = Driver::primed(None, 8);
        d.tc.on_event(Event::Ignition);
        assert_eq!((d.value(), d.tc.banner()), (1.0, Some(Event::Ignition)));
        d.tc.on_event(Event::CoreCollapse);
        assert_eq!((d.value(), d.tc.banner()), (0.1, Some(Event::CoreCollapse)));
        d.tc.on_event(Event::Ignition);
        assert_eq!((d.value(), d.tc.banner()), (0.1, Some(Event::CoreCollapse)), "never faster");
        d.tc.select(1);
        d.tc.on_event(Event::Ignition);
        assert_eq!((d.value(), d.tc.banner()), (0.3, None), "×0.3 is already slower than ×1");
        d.tc.select(2);
        d.tc.on_event(Event::Supernova);
        assert_eq!((d.value(), d.tc.banner().map(Event::banner_key)), (0.1, Some("banner.supernova")));
    }

    #[test]
    fn no_auto_return_after_the_banner() {
        let mut d = Driver::primed(None, 8);
        d.tc.on_event(Event::CoreCollapse);
        (0..10_000).for_each(|_| _ = d.frame(0.447));
        assert_eq!((d.value(), d.tc.banner()), (0.1, Some(Event::CoreCollapse)));
        d.tc.faster();
        assert_eq!((d.value(), d.tc.banner()), (0.3, None), "the player's + ends it");
    }

    #[test]
    fn slow_down_off_runs_through_events() {
        let mut d = Driver::primed(None, 8);
        d.tc.set_slow_down(false);
        d.tc.on_event(Event::CoreCollapse);
        assert_eq!((d.value(), d.tc.banner()), (1000.0, None));
        assert!(!d.tc.plan_frame(Estimates { wall_s: 1.0 / 60.0 }).slow_down);
    }

    #[test]
    fn latch_stopped_steps_are_not_owed() {
        let mut d = Driver::primed(None, 8);
        d.ms = 0.1;
        (0..20).for_each(|_| _ = d.frame(0.447));
        let (planned, _) = d.last.take().unwrap();
        assert!(planned > 3);
        d.tc.on_report(&Report { steps_run: 3, advanced: 3.0 * 0.447, dt_next: 0.447, step_ms: None, latch_fired: true });
        assert!(d.tc.owed <= 0.0);
        d.tc.on_event(Event::CoreCollapse);
        assert!(d.tc.plan_frame(Estimates { wall_s: 1.0 / 60.0 }).steps <= 1);
    }

    #[test]
    fn top_switches_to_30_at_30_of_60_and_back_after_120_quiet() {
        let mut d = Driver::primed(None, 8);
        (0..31).for_each(|_| _ = d.frame(1.5)); // need 12 of N_max 13: neither binding nor quiet
        (0..29).for_each(|_| _ = d.frame(0.447)); // need 38: binds
        assert_eq!(d.tc.f_target(), 60, "29 of 60");
        d.frame(0.447);
        assert_eq!(d.tc.f_target(), 30, "30 of 60");
        (0..119).for_each(|_| _ = d.frame(4.0)); // need 5, under half of 13
        assert_eq!(d.tc.f_target(), 30, "119 quiet frames");
        d.frame(4.0);
        assert_eq!(d.tc.f_target(), 60, "120 quiet frames");
        let mut below = Driver::primed(None, 7);
        (0..100).for_each(|_| _ = below.frame(0.001));
        assert_eq!(below.tc.f_target(), 60, "below TOP, never");
    }

    #[test]
    fn the_switch_keeps_sim_time_per_wall_second() {
        let per_second = |f_target| {
            let mut tc = TimeControl::new(None, None).unwrap();
            tc.select(8);
            cost(&mut tc, 0.01, false);
            tc.dt_est = Some(1.0);
            tc.f_target = f_target;
            tc.plan_frame(Estimates { wall_s: 1.0 / f_target as f64 }).steps * f_target
        };
        assert_eq!(per_second(30), per_second(60), "34 × 30 = 17 × 60 steps of 1.0 at ×1000");
    }

    #[test]
    fn an_empty_world_dt_is_survived() {
        let mut d = Driver::primed(None, 2);
        for _ in 0..100 {
            assert_eq!(d.frame(DT_EMPTY), 1);
            assert!(d.tc.owed.is_finite() && d.tc.owed.abs() <= 1.0 / 60.0);
        }
        d.tc.on_report(&Report { steps_run: 1, advanced: f64::INFINITY, dt_next: DT_EMPTY, step_ms: None, latch_fired: false });
        let steps: Vec<u32> = (0..600).map(|_| d.frame(0.447)).collect();
        let total: u32 = steps.iter().sum();
        assert!(steps[..30].contains(&1) && (21..=24).contains(&total), "600 frames at ×1, Δt 0.447: {total} steps");
    }

    #[test]
    fn speed_text_formats() {
        assert_eq!([0.834, 42.0, 1.0, 1234.0, 9.96].map(sig2), ["0.83", "42", "1.0", "1200", "10"]);
        let mut tc = TimeControl::new(Some(42.0), None).unwrap();
        tc.select(5);
        tc.toggle_pause();
        assert_eq!(tc.speed_text(), SpeedText { rung: "42".into(), running: None, paused: true });
    }
}
