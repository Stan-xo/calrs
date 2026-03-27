//! Reminder model.

/// Delay before a calendar item at which a reminder should trigger.
pub enum ReminderDelay {
    Minutes5,
    Minutes10,
    Minutes15,
    Minutes30,
    Hour1,
    Day1,
}

/// A single reminder associated with a calendar item.
pub struct Reminder {
    pub active: bool,
    pub delay: ReminderDelay,
}
