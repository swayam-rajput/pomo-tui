mod ui;
mod app;
use std::time::{ Duration, Instant};
use std::thread::sleep;

fn main(){
    let duration = Duration::from_secs(10);
    let start = Instant::now();

    loop{
        let elapsed = start.elapsed();
        if elapsed >= duration {
            println!("Time is up");
            break;
        }
        let remaining = duration - elapsed;
        println!("Time remaining: {} seconds", remaining.as_secs());
        sleep(Duration::from_secs(1));
    }    
}