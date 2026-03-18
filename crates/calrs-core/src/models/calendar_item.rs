//! CalendarItem: top-level type representing any entry in the calendar.

use chrono::{DateTime, Utc};
use super::event::Event;
use super::task::Task;
use super::reminder::Reminder;

/// Global visibility and lifecycle status of a calendar item.
pub enum GlobalStatus {
    Active,
    /// Cancelled but retained for history.
    Cancelled,
    /// Not yet confirmed or scheduled.
    Draft,
}

/// Discriminates between the two kinds of calendar items.
pub enum ItemKind {
    /// A time-bound entry. See [`Event`].
    Event(Event),
    /// A to-do item. See [`Task`].
    Task(Task),
}

/// Top-level calendar item, representing either an event or a task.
///
/// Common fields are stored here. Kind-specific data is inside [`ItemKind`].
///
/// Items are never hard-deleted: `deleted_at` is set instead,
/// preserving the item for history and sync reconciliation.
pub struct CalendarItem {
    /// Will migrate to UUID for multi-device sync.
    pub id: u64,
    pub title: String,
    /// IANA timezone identifier, e.g. `"Europe/Paris"`.
    pub timezone: String,
    pub status: GlobalStatus,
    pub kind: ItemKind,
    pub description: Option<String>,
    pub place: Option<String>,
    pub links: Option<Vec<String>>,
    pub reminders: Option<Vec<Reminder>>,
    pub icon: Option<String>,
    /// RGBA format: `(red, green, blue, alpha)`.
    pub color: Option<(u8, u8, u8, u8)>,
    /// `None` = active. Set on deletion, preserved for sync reconciliation.
    pub deleted_at: Option<DateTime<Utc>>,
}