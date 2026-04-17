use std::time::{Duration, Instant};

use crate::notify::send_notification;
#[derive(Debug, Clone, PartialEq)]
pub enum TimerState{
    Running, Paused, Done
}

#[derive(Debug, Clone, PartialEq)]
pub enum Phase{
    Work,
    ShortBreak,
    LongBreak,
}

pub const WORK_TIME:Duration = Duration::from_secs(10);
pub const SHORTBREAK_TIME:Duration = Duration::from_secs(10);
pub const LONGBREAK_TIME:Duration = Duration::from_secs(10);


#[derive(Clone, Copy, PartialEq)]
pub enum Screen {
    Timer,
    Settings,
}

#[derive(Clone, Copy, PartialEq)]
pub enum NotificationMode {
    Off,
    WorkOnly,
    BreakOnly,
    All,
}




pub struct App{
    pub phase: Phase,
    pub start: Instant,
    pub state: TimerState,
    pub elapsed: Duration,
    pub pomodoros_done: u32,
    pub tick: u64,

    pub screen:Screen,
    
    pub settings_idx: usize,
    
    pub work_secs: u64,
    pub short_break_secs: u64,
    pub long_break_secs: u64,

    pub notif_mode: NotificationMode,
}


impl App{
    // remove seconds
    pub fn new()-> Self{
        Self { 
            start: Instant::now(),
            state: TimerState::Running,
            elapsed: Duration::ZERO,
            phase:Phase::Work,
            pomodoros_done:0,
            tick:0,

            screen:Screen::Timer,

            work_secs:WORK_TIME.as_secs(),
            short_break_secs:SHORTBREAK_TIME.as_secs(),
            long_break_secs:LONGBREAK_TIME.as_secs(),
            
            settings_idx:0,

            notif_mode: NotificationMode::WorkOnly,
        }
    }

    fn current_duration(&self) -> Duration{
        match self.phase {
            Phase::Work => Duration::from_secs(self.work_secs),
            Phase::ShortBreak => Duration::from_secs(self.short_break_secs),
            Phase::LongBreak => Duration::from_secs(self.long_break_secs),
        }
    }

    pub fn progress(&self) -> f64 {
        let elapsed = if self.state == TimerState::Running{
            self.elapsed + self.start.elapsed()
        }
        else{
            self.elapsed
        };
        let total = self.current_duration().as_secs_f64();
        let current = elapsed.as_secs_f64();
        (current / total).min(1.0)
    }

    pub fn toggle_pause(&mut self){
        match self.state {
            TimerState::Paused=>{
                self.start = Instant::now();
                self.state = TimerState::Running;
            }
            TimerState::Running=>{
                self.elapsed += self.start.elapsed();
                self.state = TimerState::Paused;
            }
            TimerState::Done=>{
                //
            }
        }
    }

    pub fn reset(&mut self){
        self.elapsed = Duration::ZERO;
        self.start = Instant::now();
        self.state = TimerState::Running;
    }

    pub fn remaining(&self) -> (u64,u64) {
        let elapsed = if self.state == TimerState::Running{
            self.elapsed + self.start.elapsed()
        }else{
            self.elapsed
        };

        let remaining = self.current_duration().saturating_sub(elapsed);
        let secs = remaining.as_secs();
        (secs/60,secs%60)
    }

    pub fn tick(&mut self){
        if self.state != TimerState::Running{
            return;
        }

        let elapsed_now = self.elapsed + self.start.elapsed();
        if elapsed_now >= self.current_duration(){
            self.elapsed = self.current_duration();
            self.state = TimerState::Done;
            if self.should_notify() {
                send_notification(&self.phase);
            }
        }
    }


    pub fn advance(&mut self){
        if self.phase == Phase::Work{
            self.pomodoros_done +=1;
        }
        self.phase = match self.phase {
            Phase::Work => {
                if self.pomodoros_done % 4 == 0 {
                    Phase::LongBreak
                }else {
                    Phase::ShortBreak
                }
            }
            Phase::LongBreak | Phase::ShortBreak => Phase::Work,
        };

        self.elapsed = Duration::ZERO;
        self.start = Instant::now();
        self.state = TimerState::Running;
    }

    pub fn skip(&mut self) {
        self.state = TimerState::Done;
        self.elapsed = self.current_duration();
    }

    // notif settings
    pub fn should_notify(&self) -> bool{
        match self.notif_mode {
            NotificationMode::Off => false,

            NotificationMode::WorkOnly => {
                self.phase == Phase::Work
            },

            NotificationMode::BreakOnly => {
                matches!(self.phase, Phase::ShortBreak | Phase::LongBreak)
            },

            NotificationMode::All => true,
        }
    }

    pub fn cycle_notification_mode(&mut self, delta: i32) {
        let modes = [
            NotificationMode::Off,
            NotificationMode::WorkOnly,
            NotificationMode::BreakOnly,
            NotificationMode::All,
        ];

        let mut idx = modes
            .iter()
            .position(|m| *m == self.notif_mode)
            .unwrap();

        idx = (idx as i32 + delta).rem_euclid(modes.len() as i32) as usize;

        self.notif_mode = modes[idx];
    }
    

    // settings functions
    pub fn settings_up(&mut self) {
        if self.settings_idx > 0 {
            self.settings_idx -= 1;
        }
    }

    pub fn settings_down(&mut self) {
        if self.settings_idx < 3 {
            self.settings_idx += 1;
        }
    }

    // Adjust the currently selected setting by `delta` minutes.
    // allow 1–99 minutes for any session.
    pub fn adjust_selected(&mut self, delta: i64) {
        match self.settings_idx {
            0 | 1 | 2 => {
                let target = match self.settings_idx {
                    0 => &mut self.work_secs,
                    1 => &mut self.short_break_secs,
                    _ => &mut self.long_break_secs,
                };

                let minutes = (*target as i64 / 60 + delta).clamp(1, 99);
                *target = minutes as u64 * 60;

                // reset only if current phase matches
                match self.settings_idx {
                    0 if self.phase == Phase::Work => self.reset(),
                    1 if self.phase == Phase::ShortBreak => self.reset(),
                    2 if self.phase == Phase::LongBreak => self.reset(),
                    _ => {}
                }
            }

            3 => {
                self.cycle_notification_mode(delta as i32);
            }

            _ => {}
        }
}
}