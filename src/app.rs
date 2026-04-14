use std::time::{Duration, Instant};

// ─── WHAT IS AN ENUM? ────────────────────────────────────────────────────────
// An enum is a type that can only be ONE of a fixed set of values.
#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Work,
    ShortBreak,
    LongBreak,
}

impl Mode {
    pub fn label(&self) -> &str {
        match self {
            Mode::Work => "we are working 🎯",
            Mode::ShortBreak => "short break ☕",
            Mode::LongBreak => "long break 🛋️",
        }
    }
}

// The app can show either the timer or the settings screen.
#[derive(Clone, Copy, PartialEq)]
pub enum Screen {
    Timer,
    Settings,
}

// ─── APP STATE ───────────────────────────────────────────────────────────────
pub struct App {
    pub mode: Mode,
    pub screen: Screen,
    pub is_running: bool,
    pub should_quit: bool,
    pub sessions_done: u32,

    // ── Custom durations (user-editable) ────────────────────────────────
    pub work_secs: u64,
    pub short_break_secs: u64,
    pub long_break_secs: u64,

    // Settings screen: which row is highlighted (0=Work, 1=Short, 2=Long)
    pub settings_idx: usize,

    // ── Accurate timing ─────────────────────────────────────────────────
    // Instead of subtracting every 100ms (which drifts), we store:
    //   • When we last pressed Play          → session_start
    //   • How much time had elapsed before   → elapsed_before_pause
    //
    // time_remaining = total - (elapsed_before_pause + how_long_running_since_start)
    //
    // This means the progress bar is ALWAYS computed fresh from real clock
    // values, so it moves smoothly every render frame.
    session_start: Option<Instant>,
    elapsed_before_pause: Duration,
}

impl App {
    pub fn new() -> Self {
        Self {
            mode: Mode::Work,
            screen: Screen::Timer,
            is_running: false,
            should_quit: false,
            sessions_done: 0,

            work_secs: 25 * 60,
            short_break_secs: 5 * 60,
            long_break_secs: 15 * 60,

            settings_idx: 0,

            session_start: None,
            elapsed_before_pause: Duration::ZERO,
        }
    }

    // ─── TIMING HELPERS ──────────────────────────────────────────────────

    // How many seconds is the current mode supposed to last?
    pub fn total_secs(&self) -> u64 {
        match self.mode {
            Mode::Work => self.work_secs,
            Mode::ShortBreak => self.short_break_secs,
            Mode::LongBreak => self.long_break_secs,
        }
    }

    // Total elapsed time = what we saved before pausing + how long we've been running since.
    fn total_elapsed(&self) -> Duration {
        let running = self
            .session_start
            .map(|s| s.elapsed())
            .unwrap_or(Duration::ZERO);
        self.elapsed_before_pause + running
    }

    // Time remaining, always computed fresh — never drifts.
    pub fn time_remaining(&self) -> Duration {
        let total = Duration::from_secs(self.total_secs());
        total.checked_sub(self.total_elapsed()).unwrap_or(Duration::ZERO)
    }

    // Progress as a value from 0.0 (just started) to 1.0 (done).
    // This is what makes the bar move smoothly between render frames.
    pub fn progress(&self) -> f64 {
        let total = self.total_secs() as f64;
        if total == 0.0 {
            return 0.0;
        }
        (self.total_elapsed().as_secs_f64() / total).clamp(0.0, 1.0)
    }

    // Format "MM:SS" for display.
    pub fn time_str(&self) -> String {
        let secs = self.time_remaining().as_secs();
        format!("{:02}:{:02}", secs / 60, secs % 60)
    }

    // ─── TICK ────────────────────────────────────────────────────────────
    // Called every 50ms from the main loop.
    // We only need to check if the session just finished — the countdown
    // is computed live by `time_remaining()`, so no arithmetic needed here.
    pub fn tick(&mut self) {
        if self.is_running && self.time_remaining() == Duration::ZERO {
            self.is_running = false;
            self.session_start = None;
            self.advance_mode();
        }
    }

    // ─── CONTROLS ────────────────────────────────────────────────────────
    pub fn toggle(&mut self) {
        if self.is_running {
            // Pause: save how much time has elapsed so far.
            if let Some(start) = self.session_start.take() {
                self.elapsed_before_pause += start.elapsed();
            }
            self.is_running = false;
        } else {
            // Play: record when we started this running session.
            self.session_start = Some(Instant::now());
            self.is_running = true;
        }
    }

    pub fn reset(&mut self) {
        self.is_running = false;
        self.session_start = None;
        self.elapsed_before_pause = Duration::ZERO;
    }

    pub fn skip(&mut self) {
        self.reset();
        self.advance_mode();
    }

    // ─── SETTINGS SCREEN CONTROLS ─────────────────────────────────────────

    // Move the cursor up/down in the settings list.
    pub fn settings_up(&mut self) {
        if self.settings_idx > 0 {
            self.settings_idx -= 1;
        }
    }

    pub fn settings_down(&mut self) {
        if self.settings_idx < 2 {
            self.settings_idx += 1;
        }
    }

    // Adjust the currently selected setting by `delta` minutes.
    // We allow 1–99 minutes for any session.
    pub fn adjust_selected(&mut self, delta: i64) {
        let target = match self.settings_idx {
            0 => &mut self.work_secs,
            1 => &mut self.short_break_secs,
            _ => &mut self.long_break_secs,
        };
        let minutes = (*target as i64 / 60 + delta).clamp(1, 99);
        *target = minutes as u64 * 60;

        // If we changed the active mode's duration, reset the timer so it
        // doesn't confusingly show a time beyond the new total.
        self.reset();
    }

    // ─── INTERNAL ────────────────────────────────────────────────────────
    fn advance_mode(&mut self) {
        match self.mode {
            Mode::Work => {
                self.sessions_done += 1;
                self.mode = if self.sessions_done % 4 == 0 {
                    Mode::LongBreak
                } else {
                    Mode::ShortBreak
                };
            }
            Mode::ShortBreak | Mode::LongBreak => {
                self.mode = Mode::Work;
            }
        }
        self.elapsed_before_pause = Duration::ZERO;
    }
}