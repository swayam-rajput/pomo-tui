// app.rs
//
// This module is the "brain" of the app. It knows nothing about terminals or
// rendering -- it only tracks time, phases, and what the user has done.
// Keeping state separate from rendering is the golden rule of TUI architecture.

use std::time::{Duration, Instant};

// How long each phase lasts. Change these to taste.
pub const WORK_DURATION: Duration = Duration::from_secs(25 * 60);
pub const SHORT_BREAK: Duration = Duration::from_secs(5 * 60);
pub const LONG_BREAK: Duration = Duration::from_secs(15 * 60);
pub const LONG_BREAK_AFTER: u32 = 4; // pomodoros before a long break

#[derive(Debug, Clone, PartialEq)]
pub enum Phase {
    Work,
    ShortBreak,
    LongBreak,
}

impl Phase {
    pub fn duration(&self) -> Duration {
        match self {
            Phase::Work => WORK_DURATION,
            Phase::ShortBreak => SHORT_BREAK,
            Phase::LongBreak => LONG_BREAK,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Phase::Work => "FOCUS",
            Phase::ShortBreak => "SHORT BREAK",
            Phase::LongBreak => "LONG BREAK",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimerState {
    Running,
    Paused,
    Done, // phase just completed, waiting for user to advance
}

pub struct App {
    pub phase: Phase,
    pub state: TimerState,
    pub pomodoros_done: u32,

    // elapsed is the "source of truth" for how much time has passed.
    // We store it as a Duration so pausing works correctly: when paused,
    // we freeze elapsed. When resumed, we reset `started_at` to now.
    pub elapsed: Duration,

    // The instant when the current running period started.
    // Only meaningful when state == Running.
    started_at: Instant,

    // Monotonically increasing tick counter. Used by the UI for animations
    // that need to change every frame, independent of timer progress.
    pub tick: u64,
}

impl App {
    pub fn new() -> Self {
        Self {
            phase: Phase::Work,
            state: TimerState::Running,
            pomodoros_done: 0,
            elapsed: Duration::ZERO,
            started_at: Instant::now(),
            tick: 0,
        }
    }

    // Called every ~100ms by the main loop.
    pub fn tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);

        if self.state != TimerState::Running {
            return;
        }

        // Compute how much real time has passed since we last resumed.
        let running_for = self.started_at.elapsed();
        let total = self.elapsed + running_for;

        if total >= self.phase.duration() {
            // Clamp elapsed to exactly the full duration, mark done.
            self.elapsed = self.phase.duration();
            self.state = TimerState::Done;
        }
    }

    // Total time elapsed in the current phase, correctly accounting for pauses.
    pub fn current_elapsed(&self) -> Duration {
        if self.state == TimerState::Running {
            (self.elapsed + self.started_at.elapsed()).min(self.phase.duration())
        } else {
            self.elapsed
        }
    }

    // Progress from 0.0 to 1.0.
    pub fn progress(&self) -> f64 {
        let elapsed = self.current_elapsed().as_secs_f64();
        let total = self.phase.duration().as_secs_f64();
        (elapsed / total).clamp(0.0, 1.0)
    }

    // Remaining time as (minutes, seconds).
    pub fn remaining(&self) -> (u64, u64) {
        let elapsed = self.current_elapsed();
        let total = self.phase.duration();
        let remaining = total.saturating_sub(elapsed);
        let secs = remaining.as_secs();
        (secs / 60, secs % 60)
    }

    pub fn toggle_pause(&mut self) {
        match self.state {
            TimerState::Running => {
                // Freeze elapsed before pausing so we don't lose progress.
                self.elapsed = self.elapsed + self.started_at.elapsed();
                self.state = TimerState::Paused;
            }
            TimerState::Paused => {
                // Resume: reset the start instant, keep accumulated elapsed.
                self.started_at = Instant::now();
                self.state = TimerState::Running;
            }
            TimerState::Done => {} // nothing to toggle
        }
    }

    // Advance to the next phase. Called when state == Done and user presses Enter.
    pub fn advance(&mut self) {
        if self.phase == Phase::Work {
            self.pomodoros_done += 1;
        }

        self.phase = match self.phase {
            Phase::Work => {
                if self.pomodoros_done % LONG_BREAK_AFTER == 0 {
                    Phase::LongBreak
                } else {
                    Phase::ShortBreak
                }
            }
            Phase::ShortBreak | Phase::LongBreak => Phase::Work,
        };

        self.elapsed = Duration::ZERO;
        self.started_at = Instant::now();
        self.state = TimerState::Running;
    }

    // Skip the current phase entirely.
    pub fn skip(&mut self) {
        self.state = TimerState::Done;
        self.elapsed = self.phase.duration();
    }
}
