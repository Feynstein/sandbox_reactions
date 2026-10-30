//! The top bar (m0_contrat.md §4.2, §4.3, §4.10): Pause/Play, Step, the rung buttons, the speed readout and the
//! slow-down checkbox, as a view model computed from `TimeControl`. `view` says what the bar shows; `Command` is what a
//! click or a key asks of the clock and `apply` does it, so the window, the keys and the scripted captures all go through
//! the same door. The egui layer (desktop.rs) reads only the `TopBar` — never the clock — and draws it as given.
//!
//! The world's two actions (§4.2, §4.7) join the clock's: `edge.label` with its two choices, and `world.clear`, which opens
//! a dialog whose yes is the only way a Clear is asked. The clock's state is `TimeControl`'s; the edge mode and the dialog
//! are `WorldBar`'s, whose `apply` hands back the `Effect` the world must carry out (desktop.rs's `Sim::command`).

use eframe::egui;
use sr_engine::scene::EdgeMode;
use sr_physics::time::TimeControl;

use crate::format;
use crate::strings;

/// What a control of the bar (or a key) asks of the clock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// Space and the Pause/Play button.
    TogglePause,
    /// `.` and the Step button.
    Step,
    /// A rung button: its index in `TimeControl::rungs()`, Max last.
    Select(usize),
    /// Keys 1…9: the n-th numbered rung, counting from 1.
    Numbered(usize),
    /// Key 0.
    Max,
    /// `+` or `=`.
    Faster,
    /// `-`.
    Slower,
    /// The slow-down checkbox.
    SetSlowDown(bool),
    /// An edge choice (§2.8): at any time, even while a star lives (D1).
    SetEdge(EdgeMode),
    /// `world.clear` and the Delete key: opens the dialog (§4.7).
    OpenClear,
    /// The dialog's `world.clear_yes`.
    ConfirmClear,
    /// The dialog's `world.clear_no`.
    CancelClear,
}

/// What the world must carry out for a command (§1.12, §2.8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    /// The edge mode, for every step after it (`Step::set_edge_mode`).
    Edge(EdgeMode),
    /// The engine's `Clear` edit, queued for the next step's P0 — settings (rung, view, edge mode, slow-down) kept.
    Clear,
}

/// The world's actions' state: the edge mode the world runs in (a new world starts in *leave*, §2.8) and whether the Clear
/// dialog is open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldBar {
    edge: EdgeMode,
    dialog: bool,
}

impl Default for WorldBar {
    fn default() -> Self {
        WorldBar { edge: EdgeMode::Leave, dialog: false }
    }
}

impl WorldBar {
    pub fn edge(&self) -> EdgeMode {
        self.edge
    }

    /// Does what `command` asks of the edge choice and the dialog; the clock's commands are not its. A Clear comes out of the
    /// dialog's yes alone, and only while the dialog is open.
    pub fn apply(&mut self, command: Command) -> Option<Effect> {
        match command {
            Command::SetEdge(mode) => {
                self.edge = mode;
                Some(Effect::Edge(mode))
            }
            Command::OpenClear => {
                self.dialog = true;
                None
            }
            Command::CancelClear => {
                self.dialog = false;
                None
            }
            Command::ConfirmClear => std::mem::take(&mut self.dialog).then_some(Effect::Clear),
            _ => None,
        }
    }
}

/// A button: the text on it, and what it sends.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Button {
    pub label: String,
    pub command: Command,
}

/// A rung button; the selected one is drawn in `accent` (§4.2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RungButton {
    pub button: Button,
    pub selected: bool,
}

/// One of the edge's two choices; the selected one is the mode the world runs in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice {
    pub button: Button,
    pub selected: bool,
}

/// `edge.label` with its choices (§4.2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EdgeChoice {
    pub label: String,
    pub leave: Choice,
    pub bounce: Choice,
}

/// The Clear dialog (§4.7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dialog {
    pub text: String,
    pub yes: Button,
    pub no: Button,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlowDown {
    pub label: String,
    pub checked: bool,
}

/// The bar, left to right (§4.2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopBar {
    /// `time.pause` while running, `time.play` while paused.
    pub play: Button,
    pub step: Button,
    /// The ladder rungs the clock holds (none above TOP / 1.5, §1.8.2), then Max's.
    pub rungs: Vec<RungButton>,
    pub speed: String,
    /// The clock is paused: the readout (`time.paused`) is drawn in `accent`, so a still world says so (§0.5 [M0-V12]).
    pub paused: bool,
    pub slow_down: SlowDown,
    pub edge: EdgeChoice,
    /// `world.clear`: opens the dialog.
    pub clear: Button,
    /// Open while a Clear awaits its answer.
    pub dialog: Option<Dialog>,
}

/// A template with each `{name}` replaced by its value. Speeds are written as the clock writes them (§4.12): the rung as
/// its label, `{actual}` at two significant digits, neither grouped in thousands (×1000, ×4200).
fn fill(key: &str, args: &[(&str, &str)]) -> String {
    args.iter().fold(strings::get(key).to_string(), |text, (name, value)| text.replace(&format!("{{{name}}}"), value))
}

/// The speed readout (§4.3): `time.paused` while paused, else `time.speed_max` at Max, `time.speed_capped` while the cap
/// binds (§1.8.4), `time.speed`.
fn readout(time: &TimeControl) -> String {
    let speed = time.speed_text();
    let at_max = time.rung() + 1 == time.rungs().len();
    match (speed.paused, at_max, &speed.running) {
        (true, _, _) => strings::get("time.paused").to_string(),
        (false, true, running) => fill("time.speed_max", &[("actual", running.as_deref().unwrap_or("0"))]),
        (false, false, Some(actual)) => fill("time.speed_capped", &[("rung", &speed.rung), ("actual", actual)]),
        (false, false, None) => fill("time.speed", &[("rung", &speed.rung)]),
    }
}

/// The bar as the clock and the world's actions stand.
pub fn view(time: &TimeControl, world: &WorldBar) -> TopBar {
    let rungs = time
        .rungs()
        .iter()
        .enumerate()
        .map(|(i, rung)| RungButton {
            button: Button {
                label: if rung.value.is_finite() { fill("time.rung", &[("rung", &rung.label)]) } else { format::rung_max() },
                command: Command::Select(i),
            },
            selected: i == time.rung(),
        })
        .collect();
    let play_key = if time.paused() { "time.play" } else { "time.pause" };
    TopBar {
        play: Button { label: strings::get(play_key).to_string(), command: Command::TogglePause },
        step: Button { label: strings::get("time.step").to_string(), command: Command::Step },
        rungs,
        speed: readout(time),
        paused: time.paused(),
        slow_down: SlowDown { label: strings::get("time.slowdown").to_string(), checked: time.slow_down() },
        edge: EdgeChoice {
            label: strings::get("edge.label").to_string(),
            leave: choice("edge.leave", EdgeMode::Leave, world.edge),
            bounce: choice("edge.bounce", EdgeMode::Bounce, world.edge),
        },
        clear: Button { label: strings::get("world.clear").to_string(), command: Command::OpenClear },
        dialog: world.dialog.then(|| Dialog {
            text: strings::get("world.clear_confirm").to_string(),
            yes: Button { label: strings::get("world.clear_yes").to_string(), command: Command::ConfirmClear },
            no: Button { label: strings::get("world.clear_no").to_string(), command: Command::CancelClear },
        }),
    }
}

fn choice(key: &str, mode: EdgeMode, current: EdgeMode) -> Choice {
    Choice { button: Button { label: strings::get(key).to_string(), command: Command::SetEdge(mode) }, selected: mode == current }
}

/// The command a key sends (§4.10), none for a key the bar does not bind.
pub fn key_command(key: egui::Key) -> Option<Command> {
    use egui::Key;
    Some(match key {
        Key::Space => Command::TogglePause,
        Key::Period => Command::Step,
        Key::Num1 => Command::Numbered(1),
        Key::Num2 => Command::Numbered(2),
        Key::Num3 => Command::Numbered(3),
        Key::Num4 => Command::Numbered(4),
        Key::Num5 => Command::Numbered(5),
        Key::Num6 => Command::Numbered(6),
        Key::Num7 => Command::Numbered(7),
        Key::Num8 => Command::Numbered(8),
        Key::Num9 => Command::Numbered(9),
        Key::Num0 => Command::Max,
        Key::Plus | Key::Equals => Command::Faster,
        Key::Minus => Command::Slower,
        Key::Delete => Command::OpenClear,
        _ => return None,
    })
}

/// Does what `command` asks of the clock (the world's commands are `WorldBar::apply`'s). A numbered key past the last numbered rung (the ladder is cut at TOP / 1.5)
/// does nothing — it never lands on Max, which only 0 and the button select.
pub fn apply(time: &mut TimeControl, command: Command) {
    let numbered = time.rungs().len() - 1;
    match command {
        Command::TogglePause => time.toggle_pause(),
        Command::Step => time.step_once(),
        Command::Select(k) => time.select(k),
        Command::Numbered(n) if (1..=numbered).contains(&n) => time.select(n - 1),
        Command::Numbered(_) => {}
        Command::Max => time.select(numbered),
        Command::Faster => time.faster(),
        Command::Slower => time.slower(),
        Command::SetSlowDown(on) => time.set_slow_down(on),
        Command::SetEdge(_) | Command::OpenClear | Command::ConfirmClear | Command::CancelClear => {}
    }
}

#[cfg(test)]
mod tests {
    use sr_physics::time::{Estimates, Report};

    use super::*;

    fn clock(top: Option<f64>) -> TimeControl {
        TimeControl::new(top, None).unwrap()
    }

    fn bar_of(time: &TimeControl) -> TopBar {
        view(time, &WorldBar::default())
    }

    fn labels(bar: &TopBar) -> Vec<&str> {
        bar.rungs.iter().map(|r| r.button.label.as_str()).collect()
    }

    /// Plays 60 frames with every step costing 10 ms and each frame advancing `advanced` sim time at the rung `k`: the cap
    /// binds at ×1000 (one step fits the frame's budget), and at Max the readout reads what the steps advanced.
    fn run_capped(time: &mut TimeControl, k: usize, advanced: f64) {
        time.select(k);
        for _ in 0..70 {
            time.on_report(&Report { steps_run: 1, advanced, dt_next: 0.001, step_ms: Some(10.0), latch_fired: false });
            time.plan_frame(Estimates { wall_s: 1.0 / 60.0 });
        }
        time.on_report(&Report { steps_run: 1, advanced, dt_next: 0.001, step_ms: Some(10.0), latch_fired: false });
    }

    #[test]
    fn labels_are_byte_exact() {
        let bar = bar_of(&clock(None));
        assert_eq!(bar.play.label, "Pause");
        assert_eq!(bar.step.label, "Step");
        assert_eq!(
            labels(&bar),
            ["×0.1", "×0.3", "×1", "×3", "×10", "×30", "×100", "×300", "×1000", "Max"]
        );
        assert_eq!(bar.slow_down.label, "Slow down for big events");
    }

    #[test]
    fn rungs_above_top_are_never_shown() {
        // TOP 300 caps the numbered rungs below 300 / 1.5 = 200 (§1.8.2): ×300 and ×1000 are gone.
        let bar = bar_of(&clock(Some(300.0)));
        assert_eq!(labels(&bar), ["×0.1", "×0.3", "×1", "×3", "×10", "×30", "×100", "Max"]);
        let bar = bar_of(&clock(Some(12.0)));
        assert_eq!(labels(&bar), ["×0.1", "×0.3", "×1", "×3", "Max"]);
    }

    #[test]
    fn max_is_the_last_button_and_the_only_one_not_a_number() {
        for top in [None, Some(300.0), Some(12.0)] {
            let bar = bar_of(&clock(top));
            let (last, numbered) = bar.rungs.split_last().unwrap();
            assert_eq!(last.button.label, "Max");
            assert_eq!(last.button.command, Command::Select(numbered.len()));
            assert!(numbered.iter().all(|r| r.button.label.starts_with('×')));
        }
    }

    #[test]
    fn the_selected_rung_is_one_and_follows_the_clock() {
        let mut time = clock(None);
        let selected = |time: &TimeControl| -> Vec<String> {
            bar_of(time).rungs.iter().filter(|r| r.selected).map(|r| r.button.label.clone()).collect()
        };
        assert_eq!(selected(&time), ["×1"]);
        apply(&mut time, Command::Select(0));
        assert_eq!(selected(&time), ["×0.1"]);
        apply(&mut time, Command::Max);
        assert_eq!(selected(&time), ["Max"]);
    }

    #[test]
    fn the_readout_at_a_rung_that_runs_as_asked() {
        let mut time = clock(None);
        assert_eq!(bar_of(&time).speed, "Speed ×1");
        apply(&mut time, Command::Select(0));
        assert_eq!(bar_of(&time).speed, "Speed ×0.1");
        apply(&mut time, Command::Select(8));
        assert_eq!(bar_of(&time).speed, "Speed ×1000");
    }

    #[test]
    fn the_readout_names_the_cap() {
        let mut time = clock(None);
        run_capped(&mut time, 8, 0.5);
        // 60 frames of 0.5 sim time in 60 / 60 s of wall: running ×30, two significant digits.
        assert_eq!(bar_of(&time).speed, "Speed ×1000 (running ×30)");
    }

    #[test]
    fn the_readout_at_max_always_shows_the_running_speed() {
        let mut time = clock(None);
        run_capped(&mut time, 9, 0.5);
        assert_eq!(bar_of(&time).speed, "Speed max (running ×30)");
        // Not grouped in thousands, as the clock writes it.
        run_capped(&mut time, 8, 0.5);
        run_capped(&mut time, 9, 70.0);
        assert_eq!(bar_of(&time).speed, "Speed max (running ×4200)");
    }

    #[test]
    fn paused_reads_paused_and_the_button_reads_play() {
        let mut time = clock(None);
        assert!(!bar_of(&time).paused, "running: the readout is not marked");
        apply(&mut time, Command::TogglePause);
        let bar = bar_of(&time);
        assert_eq!((bar.speed.as_str(), bar.play.label.as_str(), bar.paused), ("Paused", "Play", true));
        // At Max, too: the pause wins over the running speed.
        apply(&mut time, Command::Max);
        assert_eq!(bar_of(&time).speed, "Paused");
        apply(&mut time, Command::TogglePause);
        assert_eq!((bar_of(&time).play.label.as_str(), bar_of(&time).paused), ("Pause", false));
    }

    /// Nothing in the bar moves when the player pauses, plays, changes the rung or the cap shows (the lead at M0-V12: «I dont
    /// like when an interaction in a bar moves the whole bar»): the window's own drawing of the bar keeps one width.
    #[test]
    fn the_bar_keeps_its_width_through_pause_rungs_and_the_cap() {
        let width = |bar: &TopBar| {
            let mut w = 0.0;
            let _ = egui::Context::default().run_ui(egui::RawInput::default(), |ui| {
                crate::desktop::draw_top_bar(ui, bar);
                w = ui.min_rect().width();
            });
            w
        };
        let mut time = clock(None);
        let running = width(&bar_of(&time));
        apply(&mut time, Command::TogglePause);
        let paused = width(&bar_of(&time));
        apply(&mut time, Command::TogglePause);
        run_capped(&mut time, 8, 0.5);
        let capped = width(&bar_of(&time));
        assert_eq!((paused, capped), (running, running), "running {running}, paused {paused}, capped {capped}");
    }

    #[test]
    fn the_slow_down_checkbox_both_ways() {
        let mut time = clock(None);
        assert!(bar_of(&time).slow_down.checked);
        apply(&mut time, Command::SetSlowDown(false));
        assert!(!bar_of(&time).slow_down.checked && !time.slow_down());
        apply(&mut time, Command::SetSlowDown(true));
        assert!(bar_of(&time).slow_down.checked && time.slow_down());
    }

    #[test]
    fn every_key_is_mapped() {
        use egui::Key;
        let expected = [
            (Key::Space, Command::TogglePause),
            (Key::Period, Command::Step),
            (Key::Num1, Command::Numbered(1)),
            (Key::Num5, Command::Numbered(5)),
            (Key::Num9, Command::Numbered(9)),
            (Key::Num0, Command::Max),
            (Key::Plus, Command::Faster),
            (Key::Equals, Command::Faster),
            (Key::Minus, Command::Slower),
            (Key::Delete, Command::OpenClear),
        ];
        for (key, command) in expected {
            assert_eq!(key_command(key), Some(command), "{key:?}");
        }
        for key in [Key::B, Key::Escape, Key::F1, Key::OpenBracket] {
            assert_eq!(key_command(key), None, "{key:?}");
        }
    }

    #[test]
    fn numbered_keys_pick_the_numbered_rungs_in_order_and_zero_picks_max() {
        let mut time = clock(None);
        for n in 1..=9 {
            apply(&mut time, key_command(egui::Key::from_name(&n.to_string()).unwrap()).unwrap());
            assert_eq!(bar_of(&time).rungs.iter().position(|r| r.selected), Some(n - 1), "key {n}");
        }
        apply(&mut time, Command::Numbered(1));
        apply(&mut time, Command::Max);
        assert_eq!(bar_of(&time).rungs.last().map(|r| r.selected), Some(true));
    }

    #[test]
    fn a_numbered_key_past_the_ladder_does_nothing_and_never_lands_on_max() {
        let mut time = clock(Some(12.0)); // ×0.1 ×0.3 ×1 ×3 and Max
        apply(&mut time, Command::Numbered(4));
        assert_eq!(bar_of(&time).speed, "Speed ×3");
        for n in 5..=9 {
            apply(&mut time, Command::Numbered(n));
            assert_eq!(bar_of(&time).speed, "Speed ×3", "key {n}");
        }
    }

    #[test]
    fn plus_and_minus_walk_the_ladder_and_stop_at_its_ends() {
        let mut time = clock(Some(12.0));
        for _ in 0..10 {
            apply(&mut time, Command::Faster);
        }
        assert_eq!(bar_of(&time).rungs.last().map(|r| r.selected), Some(true));
        for _ in 0..10 {
            apply(&mut time, Command::Slower);
        }
        assert_eq!(bar_of(&time).rungs.first().map(|r| r.selected), Some(true));
    }

    #[test]
    fn step_pauses_a_running_clock_and_steps_a_paused_one() {
        let mut time = clock(None);
        apply(&mut time, Command::Step);
        assert!(time.paused());
        apply(&mut time, Command::Step);
        assert_eq!(time.plan_frame(Estimates { wall_s: 1.0 / 60.0 }).steps, 1);
        assert_eq!(time.plan_frame(Estimates { wall_s: 1.0 / 60.0 }).steps, 0);
    }
}

/// The world's actions: the edge choice and the Clear dialog (M0-T87, scope ui_world).
#[cfg(test)]
mod world_tests {
    use super::*;

    fn clock(top: Option<f64>) -> TimeControl {
        TimeControl::new(top, None).unwrap()
    }

    /// Applies a command to the clock and the world's actions both, as `Sim::command` does.
    fn send(time: &mut TimeControl, world: &mut WorldBar, command: Command) -> Option<Effect> {
        apply(time, command);
        world.apply(command)
    }

    #[test]
    fn the_edge_choice_reads_leave_to_start_and_both_ways() {
        let (time, mut world) = (clock(None), WorldBar::default());
        let edge = view(&time, &world).edge;
        assert_eq!((edge.label.as_str(), edge.leave.button.label.as_str(), edge.bounce.button.label.as_str()), ("World edge:", "Leave for good", "Bounce back"));
        assert_eq!((edge.leave.selected, edge.bounce.selected), (true, false));
        assert_eq!((edge.leave.button.command, edge.bounce.button.command), (Command::SetEdge(EdgeMode::Leave), Command::SetEdge(EdgeMode::Bounce)));
        assert_eq!(world.apply(Command::SetEdge(EdgeMode::Bounce)), Some(Effect::Edge(EdgeMode::Bounce)));
        let edge = view(&time, &world).edge;
        assert_eq!((edge.leave.selected, edge.bounce.selected, world.edge()), (false, true, EdgeMode::Bounce));
        assert_eq!(world.apply(Command::SetEdge(EdgeMode::Leave)), Some(Effect::Edge(EdgeMode::Leave)));
        assert_eq!(world.edge(), EdgeMode::Leave);
    }

    #[test]
    fn the_edge_can_change_at_any_time_while_paused_running_or_the_dialog_open() {
        let (mut time, mut world) = (clock(None), WorldBar::default());
        for command in [Command::TogglePause, Command::Max, Command::OpenClear] {
            send(&mut time, &mut world, command);
            let mode = if world.edge() == EdgeMode::Leave { EdgeMode::Bounce } else { EdgeMode::Leave };
            assert_eq!(send(&mut time, &mut world, Command::SetEdge(mode)), Some(Effect::Edge(mode)), "{command:?}");
            assert_eq!(world.edge(), mode);
        }
    }

    #[test]
    fn the_clear_button_opens_the_dialog_and_asks_nothing_yet() {
        let (time, mut world) = (clock(None), WorldBar::default());
        let closed = view(&time, &world);
        assert_eq!((closed.clear.label.as_str(), closed.dialog), ("Clear world", None));
        assert_eq!(world.apply(closed.clear.command), None);
        let dialog = view(&time, &world).dialog.expect("the dialog is open");
        assert_eq!(dialog.text, "Clear the world? Everything in it will be lost.");
        assert_eq!((dialog.yes.label.as_str(), dialog.no.label.as_str()), ("Clear", "Cancel"));
    }

    #[test]
    fn delete_opens_the_dialog_as_the_button_does() {
        let (time, mut world) = (clock(None), WorldBar::default());
        assert_eq!(world.apply(key_command(egui::Key::Delete).unwrap()), None);
        assert!(view(&time, &world).dialog.is_some());
    }

    #[test]
    fn cancel_closes_the_dialog_and_queues_no_clear() {
        let (time, mut world) = (clock(None), WorldBar::default());
        world.apply(Command::OpenClear);
        assert_eq!(world.apply(Command::CancelClear), None);
        assert_eq!(view(&time, &world).dialog, None);
        // A yes after the cancel has no dialog to answer.
        assert_eq!(world.apply(Command::ConfirmClear), None);
    }

    #[test]
    fn clear_is_queued_by_yes_alone_and_once() {
        let (time, mut world) = (clock(None), WorldBar::default());
        assert_eq!(world.apply(Command::ConfirmClear), None, "yes with no dialog open");
        for command in [Command::OpenClear, Command::OpenClear, Command::CancelClear, Command::SetEdge(EdgeMode::Bounce)] {
            assert_ne!(world.apply(command), Some(Effect::Clear), "{command:?}");
        }
        world.apply(Command::OpenClear);
        let yes = view(&time, &world).dialog.unwrap().yes.command;
        assert_eq!(world.apply(yes), Some(Effect::Clear));
        assert_eq!(view(&time, &world).dialog, None, "the dialog closes on yes");
        assert_eq!(world.apply(yes), None, "one yes, one Clear");
    }

    #[test]
    fn a_clear_keeps_the_rung_the_slow_down_and_the_edge_mode() {
        // §1.12: "settings (rung, view, edge mode, slow-down) are kept". The view is the left panel's (M0-T9x), not this
        // model's: nothing here, nor in `Sim`'s Clear, writes it.
        let (mut time, mut world) = (clock(None), WorldBar::default());
        for command in [Command::Select(5), Command::SetSlowDown(false), Command::SetEdge(EdgeMode::Bounce), Command::OpenClear] {
            send(&mut time, &mut world, command);
        }
        let before = (time.rung(), time.slow_down(), world.edge(), time.paused());
        assert_eq!(send(&mut time, &mut world, Command::ConfirmClear), Some(Effect::Clear));
        assert_eq!((time.rung(), time.slow_down(), world.edge(), time.paused()), before);
        assert_eq!(before, (5, false, EdgeMode::Bounce, false));
    }
}
