use notify_rust::Notification;

use crate::app::Phase;
pub enum NotificationEvent{
    Started,
    Ended
}
pub fn send_notification(phase: &Phase, event: &NotificationEvent) {
    let (title, body) = match (phase, event) {
        (Phase::Work,NotificationEvent::Started) => (
            "focus session started", 
            "time to focus, lock in"
        ),
        (Phase::Work, NotificationEvent::Ended) => (
            "focus session complete",
            "take a break."
        ),
        (Phase::ShortBreak,NotificationEvent::Started) => (
            "short break started", 
            "step away for a bit"
        ),
        (Phase::ShortBreak, NotificationEvent::Ended) => (
            "short break over",
            "back to work"
        ),
        (Phase::LongBreak,NotificationEvent::Started) => (
            "long break started", 
            "get up, stretch out, hydrate"
        ),
        (Phase::LongBreak,NotificationEvent::Ended) => (
            "long break over", 
            "get back workin"
        ),
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