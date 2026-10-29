//! The banner (m0_contrat.md §1.8.5, §4.2, §4.3, §4.11): over the world's top centre after an automatic slow-down, until the
//! player changes the rung. `view` says what it shows, read from the clock (`TimeControl` sets and clears it); `Feed` is the
//! door the slow-down's two sources go through — the latch's report and the Tracker's events — so that an event is booked
//! once however late its second source arrives (§1.9.4: each event fires once per object).
//!
//! The egui layer (desktop.rs) reads only the `Banner` and draws it in the banner tokens.

use sr_physics::time::{Event, TimeControl};

use crate::strings;

/// What the banner says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Banner {
    pub text: String,
}

/// The banner as the clock stands: the text of the event that slowed it, none after the player changes the rung (or when
/// nothing slowed it).
pub fn view(time: &TimeControl) -> Option<Banner> {
    time.banner().map(|event| Banner { text: strings::get(event.banner_key()).to_string() })
}

/// The events the slow-down answers, by the id the observers write (`sr_engine::watch::RunEvent::kind`). `sink_formed` is
/// logged only and `ending` is not an event of the slow-down (§1.9.4).
fn slow_down_event(kind: &str) -> Option<Event> {
    match kind {
        "ignition" => Some(Event::Ignition),
        "core_collapse" => Some(Event::CoreCollapse),
        "supernova" => Some(Event::Supernova),
        _ => None,
    }
}

/// Feeds the clock's `on_event` from both sources and books each event once.
///
/// The latch stops a frame at a core collapse and its report reaches the app first; the Tracker names the same collapse from
/// that frame's block summary, the same frame or later. The latch's slow-down is the one that counts (§1.8.5), so the
/// Tracker's `core_collapse` that follows it is the same event and is not booked again — otherwise a player who had already
/// sped up would see a second banner for it. Ignition and supernova come from the Tracker alone.
#[derive(Debug, Default)]
pub struct Feed {
    /// Events already booked, by object (§1.9.4).
    fired: Vec<(u32, Event)>,
    /// The latch slowed the clock for a collapse the Tracker has not yet named.
    latched: bool,
}

impl Feed {
    pub fn new() -> Feed {
        Feed::default()
    }

    /// A frame the latch cut (`Report::latch_fired`): core collapse, ×0.1. The clock ignores it with the setting off.
    pub fn latch(&mut self, time: &mut TimeControl) {
        self.latched = true;
        time.on_event(Event::CoreCollapse);
    }

    /// An event of the Tracker, as the observers' `kind` writes it, for `object`.
    pub fn event(&mut self, time: &mut TimeControl, object: u32, kind: &str) {
        let Some(event) = slow_down_event(kind) else { return };
        if self.fired.contains(&(object, event)) {
            return;
        }
        self.fired.push((object, event));
        if event == Event::CoreCollapse && std::mem::take(&mut self.latched) {
            return;
        }
        time.on_event(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::topbar::{self, Command};

    /// The default ladder is ×0.1 ×0.3 ×1 ×3 ×10 …, the game starting at ×1; `rung` is the one selected.
    fn clock(rung: usize) -> TimeControl {
        let mut time = TimeControl::new(None, None).unwrap();
        time.select(rung);
        time
    }

    fn value(time: &TimeControl) -> f64 {
        time.rungs()[time.rung()].value
    }

    fn text(time: &TimeControl) -> Option<String> {
        view(time).map(|b| b.text)
    }

    #[test]
    fn nothing_slowed_nothing_shown() {
        assert_eq!(view(&clock(2)), None);
        assert_eq!(view(&clock(8)), None);
    }

    #[test]
    fn ignition_slows_to_one_and_says_so() {
        let (mut time, mut feed) = (clock(5), Feed::new());
        feed.event(&mut time, 1, "ignition");
        assert_eq!(value(&time), 1.0);
        assert_eq!(text(&time).as_deref(), Some("Slowed down: hydrogen ignition — press + to speed up again"));
    }

    #[test]
    fn the_latch_slows_to_a_tenth_with_the_core_collapse_text() {
        let (mut time, mut feed) = (clock(5), Feed::new());
        feed.latch(&mut time);
        assert_eq!(value(&time), 0.1);
        assert_eq!(text(&time).as_deref(), Some("Slowed down: core collapse — press + to speed up again"));
    }

    #[test]
    fn supernova_slows_to_a_tenth_with_its_text() {
        let (mut time, mut feed) = (clock(5), Feed::new());
        feed.event(&mut time, 1, "supernova");
        assert_eq!(value(&time), 0.1);
        assert_eq!(text(&time).as_deref(), Some("Slowed down: supernova — press + to speed up again"));
    }

    #[test]
    fn the_banner_stays_until_the_player_changes_the_rung() {
        let (mut time, mut feed) = (clock(5), Feed::new());
        feed.event(&mut time, 1, "supernova");
        // Frames pass, a pause and a step change nothing about the rung: the banner stays (no auto-return, D8).
        topbar::apply(&mut time, Command::TogglePause);
        topbar::apply(&mut time, Command::Step);
        topbar::apply(&mut time, Command::TogglePause);
        assert!(view(&time).is_some());
        topbar::apply(&mut time, Command::Faster);
        assert_eq!((view(&time), value(&time)), (None, 0.3));
        feed.event(&mut time, 2, "supernova");
        topbar::apply(&mut time, Command::Select(2));
        assert_eq!(view(&time), None, "a click on a rung ends it too");
    }

    #[test]
    fn the_setting_off_means_no_slow_down_and_no_banner() {
        let (mut time, mut feed) = (clock(5), Feed::new());
        topbar::apply(&mut time, Command::SetSlowDown(false));
        feed.event(&mut time, 1, "ignition");
        feed.latch(&mut time);
        feed.event(&mut time, 1, "core_collapse");
        feed.event(&mut time, 1, "supernova");
        assert_eq!((value(&time), view(&time)), (30.0, None));
    }

    #[test]
    fn only_ever_slower() {
        let (mut time, mut feed) = (clock(0), Feed::new());
        feed.event(&mut time, 1, "ignition");
        assert_eq!((value(&time), view(&time)), (0.1, None), "×0.1 is already slower than ×1");
        let mut time = clock(1);
        feed.event(&mut time, 2, "ignition");
        assert_eq!((value(&time), view(&time)), (0.3, None));
    }

    #[test]
    fn events_that_are_not_the_slow_downs_do_nothing() {
        let (mut time, mut feed) = (clock(5), Feed::new());
        feed.event(&mut time, 1, "sink_formed");
        feed.event(&mut time, 1, "ending");
        feed.event(&mut time, 1, "something_else");
        assert_eq!((value(&time), view(&time)), (30.0, None));
    }

    #[test]
    fn each_event_fires_once_per_object() {
        let (mut time, mut feed) = (clock(5), Feed::new());
        feed.event(&mut time, 1, "ignition");
        topbar::apply(&mut time, Command::Select(5));
        feed.event(&mut time, 1, "ignition");
        assert_eq!((value(&time), view(&time)), (30.0, None), "the same object's ignition does not slow it again");
        feed.event(&mut time, 2, "ignition");
        assert_eq!((value(&time), text(&time).is_some()), (1.0, true), "another object's does");
    }

    #[test]
    fn a_core_collapse_named_two_frames_after_the_latch_is_not_a_second_banner() {
        let (mut time, mut feed) = (clock(5), Feed::new());
        feed.latch(&mut time);
        assert_eq!((value(&time), text(&time).is_some()), (0.1, true));
        // The player speeds up at once; the Tracker's event for the same collapse reaches the app two frames late.
        topbar::apply(&mut time, Command::Faster);
        assert_eq!(view(&time), None);
        feed.event(&mut time, 1, "core_collapse");
        assert_eq!((value(&time), view(&time)), (0.3, None), "no second slow-down, no second banner");
        // The supernova that follows is its own event.
        feed.event(&mut time, 1, "supernova");
        assert_eq!(value(&time), 0.1);
        assert_eq!(text(&time).as_deref(), Some("Slowed down: supernova — press + to speed up again"));
    }

    #[test]
    fn a_collapse_the_latch_did_not_cut_still_slows_the_clock() {
        // The latch's frame was never read back as cut (it is a Tracker event alone): the event is the slow-down.
        let (mut time, mut feed) = (clock(5), Feed::new());
        feed.event(&mut time, 1, "core_collapse");
        assert_eq!(value(&time), 0.1);
        assert_eq!(text(&time).as_deref(), Some("Slowed down: core collapse — press + to speed up again"));
    }

    #[test]
    fn a_second_latch_slows_the_clock_again() {
        // The latch re-arms by its own rule and a later cut is a later slow-down (§1.8.5).
        let (mut time, mut feed) = (clock(5), Feed::new());
        feed.latch(&mut time);
        topbar::apply(&mut time, Command::Select(5));
        feed.latch(&mut time);
        assert_eq!((value(&time), text(&time).is_some()), (0.1, true));
    }
}
