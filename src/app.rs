use std::time::{Duration, Instant};

pub struct Timer {
    pub time: Duration,
    pub is_running: bool,
    pub start: Instant,
}

impl Timer {
    pub fn new(seconds: u64) -> Self {
        Self { 
            time: Duration::from_secs(seconds),
            start: Instant::now(),
            is_running: true,
        }
    }

    pub fn progress(&self) -> f64 {
        let elapsed = self.start.elapsed().as_secs_f64();
        let total = self.time.as_secs_f64();
        (elapsed / total).min(1.0)
    }

    pub fn is_done(&self) -> bool {
        self.start.elapsed() >= self.time
    }
}