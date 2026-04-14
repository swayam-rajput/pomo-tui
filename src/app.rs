use std::time::{Duration, Instant};
#[derive(Debug, Clone, PartialEq)]
pub enum TimerState{
    Running, Paused,
}




pub struct App{
    pub duration: Duration,
    pub start: Instant,
    pub state: TimerState,
    pub elapsed: Duration,
}


impl App{
    pub fn new(seconds:u64)-> Self{
        Self { duration: Duration::from_secs(seconds), start: Instant::now(), state: TimerState::Running, elapsed: Duration::ZERO, }
    }

    pub fn progress(&self) -> f64 {
        let elapsed = self.start.elapsed().as_secs_f64();
        let total = self.duration.as_secs_f64();
        (elapsed / total).min(1.0)
    }

    pub fn tgl_pause(&mut self){
        match self.state {
            TimerState::Paused=>{
                self.start = Instant::now();
                self.state = TimerState::Running;
            }
            TimerState::Running=>{
                self.elapsed += self.start.elapsed();
                self.state = TimerState::Paused;
            }
        }
    }

    pub fn reset(&mut self){
        self.elapsed = Duration::ZERO;
        self.start = Instant::now();
        self.state = TimerState::Paused;
    }

    pub fn remaining(&self) -> (u64,u64) {
        let elapsed = if self.state == TimerState::Running{
            self.elapsed + self.start.elapsed()
        }else{
            self.elapsed
        };

        let remaining = self.duration.saturating_sub(elapsed);
        let secs = remaining.as_secs();
        (secs/60,secs%60)
    }

}