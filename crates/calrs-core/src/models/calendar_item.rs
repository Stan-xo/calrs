//! CalendarItem: top-level type representing any entry in the calendar.

use super::calendar_item_row::{CalendarItemRow, EventRow, TaskRow};
use super::event::{Event, EventState};
use super::recurrence::Recurrence;
use super::reminder::Reminder;
use super::task::{Criticality, Task, TaskState};
use chrono::{DateTime, Utc};
use std::convert::TryFrom;

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
            id: 0, // assigned by SQLite on insert
            title, // == title: title
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
    pub fn new_task(title: String) -> Self {
        CalendarItem {
            id: 0, // assigned by SQLite on insert
            title, // == title: title
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
    pub fn with_timezone(mut self, timezone: String) -> Self {
        self.timezone = timezone;
        self
    }
    pub fn with_status(mut self, status: GlobalStatus) -> Self {
        self.status = status;
        self
    }
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
    pub fn with_place(mut self, place: String) -> Self {
        self.place = Some(place);
        self
    }
    pub fn with_links(mut self, links: Vec<String>) -> Self {
        self.links = Some(links);
        self
    }
    pub fn with_reminders(mut self, reminders: Vec<Reminder>) -> Self {
        self.reminders = Some(reminders);
        self
    }
    pub fn with_icon(mut self, icon: String) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn with_color(mut self, color: (u8, u8, u8, u8)) -> Self {
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

impl TryFrom<(CalendarItemRow, Option<EventRow>, Option<TaskRow>)> for CalendarItem {
    type Error = String;

    fn try_from(
        (row, event_row, task_row): (CalendarItemRow, Option<EventRow>, Option<TaskRow>),
    ) -> Result<Self, Self::Error> {
        // convert status
        let status = match row.status.as_str() {
            "active" => GlobalStatus::Active,
            "cancelled" => GlobalStatus::Cancelled,
            "draft" => GlobalStatus::Draft,
            other => return Err(format!("Unknown status: {}", other)),
        };

        // convert color
        let color = match (row.color_r, row.color_g, row.color_b, row.color_a) {
            (Some(r), Some(g), Some(b), Some(a)) => Some((r as u8, g as u8, b as u8, a as u8)),
            _ => None,
        };

        // convert deleted_at
        let deleted_at = match row.deleted_at {
            Some(s) => Some(
                DateTime::parse_from_rfc3339(&s)
                    .map_err(|e| format!("Invalid deleted_at date: {}", e))?
                    .with_timezone(&chrono::Utc),
            ),
            None => None,
        };

        // convert kind
        let kind = match row.kind.as_str() {
            "event" => {
                let e = event_row.ok_or("Missing event row for kind=event")?;

                let state = match e.state.as_str() {
                    "planned" => EventState::Planned,
                    "confirmed" => EventState::Confirmed,
                    "finished" => EventState::Finished,
                    other => return Err(format!("Unknown event state: {}", other)),
                };

                ItemKind::Event(Event {
                    date_start: e.date_start,
                    date_end: e.date_end,
                    full_day: e.full_day,
                    state,
                    recurrence: None, // chargé séparément si besoin
                    exceptions: None, // chargé séparément si besoin
                    parent_id: e.parent_id.map(|id| id as u64),
                })
            }
            "task" => {
                let t = task_row.ok_or("Missing task row for kind=task")?;

                let state = match t.state.as_str() {
                    "todo" => TaskState::Todo,
                    "doing" => TaskState::Doing,
                    "done" => TaskState::Done,
                    other => return Err(format!("Unknown task state: {}", other)),
                };

                let criticality = match t.criticality.as_deref() {
                    Some("low") => Some(Criticality::Low),
                    Some("medium") => Some(Criticality::Medium),
                    Some("high") => Some(Criticality::High),
                    Some("urgent") => Some(Criticality::Urgent),
                    Some("blocked") => Some(Criticality::Blocked),
                    None => None,
                    Some(other) => return Err(format!("Unknown criticality: {}", other)),
                };

                ItemKind::Task(Task {
                    deadline: t.deadline,
                    state,
                    criticality,
                })
            }
            other => return Err(format!("Unknown kind: {}", other)),
        };

        Ok(CalendarItem {
            id: row.id as u64,
            title: row.title,
            timezone: row.timezone,
            status,
            kind,
            description: row.description,
            place: row.place,
            links: None,     // chargé séparément
            reminders: None, // chargé séparément
            icon: row.icon,
            color,
            deleted_at,
        })
    }
}
