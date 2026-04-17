use notify_rust::Notification;

use crate::app::Phase;

pub fn send_notification(phase: &Phase) {
    let (title, body) = match phase {
        Phase::Work => ("focus session complete", "take a break."),
        Phase::ShortBreak => ("short break over", "back to work."),
        Phase::LongBreak => ("long break over", "ready to focus again."),
    };

    print!("\x07"); // ASCII bell
    Notification::new()
        .sound_name("default")
        .timeout(1000)
        .summary(title)
        .body(body)
        .show()    
        .ok();
}