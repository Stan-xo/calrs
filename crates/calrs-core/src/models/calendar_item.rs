//! CalendarItem: top-level type representing any entry in the calendar.

use super::event::{Event, EventState,};
use super::reminder::Reminder;
use super::recurrence::Recurrence;
use super::task::{Task, TaskState, Criticality};
use chrono::{DateTime, Utc};

use crate::DEFAULT_TIMEZONE;

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

impl CalendarItem {
    /// Constructor for event
    pub fn new_event(
        title: String,
        date_start: DateTime<Utc>,
        date_end: DateTime<Utc>,
        full_day: bool,
    ) -> Self {
        CalendarItem {
            id: 0,          // assigned by SQLite on insert
            title,          // == title: title
            timezone: DEFAULT_TIMEZONE.to_string(),
            status: GlobalStatus::Active,
            kind: ItemKind::Event(Event {
                date_start,
                date_end,
                full_day,
                state: EventState::Planned,
                recurrence: None,
                exceptions: None,
                parent_id: None,
            }),
            description: None,
            place: None,
            links: None,
            reminders: None,
            icon: None,
            color: None,
            deleted_at: None,
        }
    }

    /// Constructor for Task
    pub fn new_task(title: String) -> Self{
        CalendarItem {
            id: 0,          // assigned by SQLite on insert
            title,          // == title: title
            timezone: DEFAULT_TIMEZONE.to_string(),
            status: GlobalStatus::Active,
            kind: ItemKind::Task(Task {
                deadline: None,
                state: TaskState::Todo,
                criticality: None,
            }),
            description: None,
            place: None,
            links: None,
            reminders: None,
            icon: None,
            color: None,
            deleted_at: None,
        }
    }

    /// Common builder
    pub fn with_timezone(mut self, timezone:String) -> Self{
        self.timezone = timezone;
        self
    }
    pub fn with_status(mut self, status: GlobalStatus) -> Self{
        self.status = status;
        self
    }
    pub fn with_description(mut self, description: String) -> Self{
        self.description = Some(description);
        self
    }
    pub fn with_place(mut self, place: String) -> Self{
        self.place = Some(place);
        self
    }
    pub fn with_links(mut self, links: Vec<String>) -> Self{
        self.links = Some(links);
        self
    }
    pub fn with_reminders(mut self, reminders: Vec<Reminder>) -> Self{
        self.reminders = Some(reminders);
        self
    }
    pub fn with_icon(mut self, icon: String) -> Self{
        self.icon = Some(icon);
        self
    }
    pub fn with_color(mut self, color: (u8, u8, u8, u8)) -> Self{
        self.color = Some(color);
        self
    }

    // Event builder
    pub fn with_event_state(mut self, state: EventState) -> Self {
        if let ItemKind::Event(ref mut event) = self.kind {
            event.state = state;
        }
        self
    }
    pub fn with_recurrence(mut self, recurrence: Recurrence) -> Self {
        if let ItemKind::Event(ref mut event) = self.kind {
            event.recurrence = Some(recurrence);
        }
        self
    }
    pub fn with_exceptions(mut self, exceptions: Vec<DateTime<Utc>>) -> Self {
        if let ItemKind::Event(ref mut event) = self.kind {
            event.exceptions = Some(exceptions);
        }
        self
    }

    // Task builder
    pub fn with_task_state(mut self, state: TaskState) -> Self {
        if let ItemKind::Task(ref mut task) = self.kind {
            task.state = state;
        }
        self
    }
    pub fn with_deadline(mut self, deadline: DateTime<Utc>) -> Self {
        if let ItemKind::Task(ref mut task) = self.kind {
            task.deadline = Some(deadline);
        }
        self
    }
    pub fn with_criticality(mut self, criticality: Criticality) -> Self {
        if let ItemKind::Task(ref mut task) = self.kind {
            task.criticality = Some(criticality);
        }
        self
    }

    // Common setteur
    pub fn set_id(&mut self, id: u64) {
        self.id = id;
    }
}
