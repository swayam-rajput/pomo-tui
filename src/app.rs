// We need Duration (a length of time, e.g. 25 mins) and Instant (a point in time, like a stopwatch).
use std::time::{Duration, Instant};

// ─── WHAT IS AN ENUM? ────────────────────────────────────────────────────────
// An `enum` lets you define a type that can only be one of a fixed set of values.
// Here, our timer can only ever be in ONE of these three states. This is better
// than using magic strings like "work" or "break" because the compiler will
// warn you if you forget to handle a case.
#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Work,
    ShortBreak,
    LongBreak,
}

// We attach methods to our enum using `impl`.
impl Mode {
    // This returns how long (in seconds) each mode should last.
    pub fn duration_secs(&self) -> u64 {
        // `match` is like a switch statement, but exhaustive —
        // Rust forces you to handle EVERY possible value.
        match self {
            Mode::Work => 25 * 60,
            Mode::ShortBreak => 5 * 60,
            Mode::LongBreak => 15 * 60,
        }
    }

    // Returns a display label for the current mode.
    pub fn label(&self) -> &str {
        match self {
            Mode::Work => "we are working 🎯",
            Mode::ShortBreak => "short break ☕",
            Mode::LongBreak => "long break 🛋️",
        }
    }
}

// ─── WHAT IS A STRUCT? ───────────────────────────────────────────────────────
// A `struct` is a container that groups related data together.
// Think of it like a class in other languages, but without inheritance.
// Our `App` holds ALL the state for the Pomodoro timer.
pub struct App {
    pub mode: Mode,               // Are we working or on a break?
    pub time_remaining: Duration, // How much time is left in this session?
    pub is_running: bool,         // Is the timer counting down right now?
    pub should_quit: bool,        // Did the user press 'q'?
    pub sessions_done: u32,       // How many focus sessions are complete?
    // This tracks "when did the last tick happen?" so we can subtract accurately.
    last_tick: Instant,
}

// ─── impl App ────────────────────────────────────────────────────────────────
// We put all the *behavior* of App here.
impl App {
    // `new()` is a constructor — a convention in Rust (not enforced, just idiomatic).
    // It returns `Self`, which means "an instance of this type (App)".
    pub fn new() -> Self {
        let mode = Mode::Work;
        Self {
            mode,
            time_remaining: Duration::from_secs(mode.duration_secs()),
            is_running: false,
            should_quit: false,
            sessions_done: 0,
            last_tick: Instant::now(),
        }
    }

    // ─── TICK ──────────────────────────────────────────────────────────────
    // This is called ~10 times per second from our main loop.
    // `&mut self` means we can READ and MODIFY the App's data.
    pub fn tick(&mut self) {
        if !self.is_running {
            // Store the tick time even when paused, so we don't
            // accumulate a huge elapsed jump when we resume.
            self.last_tick = Instant::now();
            return;
        }

        // How much time passed since the last tick?
        let elapsed = self.last_tick.elapsed();
        self.last_tick = Instant::now();

        // checked_sub returns None if the result would go below zero.
        // This prevents the timer from wrapping around to a huge number.
        if let Some(new_remaining) = self.time_remaining.checked_sub(elapsed) {
            self.time_remaining = new_remaining;
        } else {
            // Timer finished!
            self.time_remaining = Duration::ZERO;
            self.is_running = false;
            self.advance_mode();
        }
    }

    // ─── ADVANCE MODE ──────────────────────────────────────────────────────
    // Called automatically when a session ends. Moves to the next mode.
    fn advance_mode(&mut self) {
        match self.mode {
            Mode::Work => {
                self.sessions_done += 1;
                // Every 4 work sessions → long break. Otherwise → short break.
                if self.sessions_done % 4 == 0 {
                    self.mode = Mode::LongBreak;
                } else {
                    self.mode = Mode::ShortBreak;
                }
            }
            Mode::ShortBreak | Mode::LongBreak => {
                self.mode = Mode::Work;
            }
        }
        self.time_remaining = Duration::from_secs(self.mode.duration_secs());
    }

    // ─── CONTROLS ──────────────────────────────────────────────────────────
    pub fn toggle(&mut self) {
        self.is_running = !self.is_running;
    }

    pub fn reset(&mut self) {
        self.time_remaining = Duration::from_secs(self.mode.duration_secs());
        self.is_running = false;
        self.last_tick = Instant::now();
    }

    pub fn skip(&mut self) {
        self.advance_mode();
        self.is_running = false;
    }

    // ─── HELPERS ───────────────────────────────────────────────────────────
    // Returns a value from 0.0 (just started) to 1.0 (finished).
    // The UI uses this to draw the progress bar.
    pub fn progress(&self) -> f64 {
        let total = self.mode.duration_secs() as f64;
        let remaining = self.time_remaining.as_secs_f64();
        // Clamp to [0.0, 1.0] to be safe.
        ((total - remaining) / total).clamp(0.0, 1.0)
    }

    // Formats the remaining time as "MM:SS" (e.g. "24:07").
    pub fn time_str(&self) -> String {
        let secs = self.time_remaining.as_secs();
        format!("{:02}:{:02}", secs / 60, secs % 60)
    }
}