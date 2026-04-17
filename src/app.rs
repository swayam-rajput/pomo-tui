use std::time::{Duration, Instant};
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

pub const WORK_TIME:Duration = Duration::from_secs(1*60);
pub const SHORTBREAK_TIME:Duration = Duration::from_secs(60);
pub const LONGBREAK_TIME:Duration = Duration::from_secs(15*60);


#[derive(Clone, Copy, PartialEq)]
pub enum Screen {
    Timer,
    Settings,
}

impl Phase{
    pub fn duration(&self) -> Duration{
        match self {
            Phase::Work => WORK_TIME,
            Phase::ShortBreak => SHORTBREAK_TIME,
            Phase::LongBreak => LONGBREAK_TIME,
        }
    }
    // pub fn label(&self) -> str{

    // }
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
}


impl App{
    // remove seconds
    pub fn new(seconds:u64)-> Self{
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
            
            settings_idx:0
        }
    }

    pub fn current_duration(&self) -> Duration{
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



    // settings functions
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

        match self.settings_idx {
            0 if self.phase == Phase::Work => self.reset(),
            1 if self.phase == Phase::ShortBreak => self.reset(),
            2 if self.phase == Phase::LongBreak => self.reset(),
            _ => {}
        }
    }

}