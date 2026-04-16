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

pub const WORK_TIME:Duration = Duration::from_secs(30);
// pub const WORK_TIME:Duration = Duration::from_secs(1*60);
pub const SHORTBREAK_TIME:Duration = Duration::from_secs(5*60);
pub const LONGBREAK_TIME:Duration = Duration::from_secs(15*60);
pub const LONG_BREAK_AFTER: u32 = 4; // pomodoros before a long break


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
    pub pomodoros_done: u32
}


impl App{
    // remove seconds
    pub fn new(seconds:u64)-> Self{
        Self { start: Instant::now(), state: TimerState::Running, elapsed: Duration::ZERO, phase:Phase::Work, pomodoros_done:0, }
    }

    pub fn progress(&self) -> f64 {
        let elapsed = if self.state == TimerState::Running{
            self.elapsed + self.start.elapsed()
        }
        else{
            self.elapsed
        };
        let total = self.phase.duration().as_secs_f64();
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

        let remaining = self.phase.duration().saturating_sub(elapsed);
        let secs = remaining.as_secs();
        (secs/60,secs%60)
    }

    pub fn tick(&mut self){
        if self.state != TimerState::Running{
            return;
        }

        let elapsed_now = self.elapsed + self.start.elapsed();
        if elapsed_now >= self.phase.duration(){
            self.elapsed = self.phase.duration();
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

}