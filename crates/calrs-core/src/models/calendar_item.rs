//! CalendarItem: top-level type representing any entry in the calendar.

use super::event::{Event, EventState,};
use super::reminder::{Reminder, ReminderDelay};
use super::recurrence::{Recurrence, RecurrenceFrequency, RecurrenceEnd};
use super::task::{Task, TaskState, Criticality};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

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

/// Inserts a new calendar item into the database.
///
/// Writes to all relevant tables in a single transaction.
/// The generated id is assigned back to `item.id` on success.
/// If any step fails, the transaction is rolled back.
pub async fn insert_item(
    pool: &SqlitePool,
    item: &mut CalendarItem,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // convert enums to strings for SQLite
    let kind = match &item.kind {
        ItemKind::Event(_) => "event",
        ItemKind::Task(_) => "task",
    };
    let status = match &item.status {
        GlobalStatus::Active => "active",
        GlobalStatus::Cancelled => "cancelled",
        GlobalStatus::Draft => "draft",
    };
    let (color_r, color_g, color_b, color_a) = match item.color {
    Some((r, g, b, a)) => (Some(r), Some(g), Some(b), Some(a)),
    None => (None, None, None, None),
};

    // INSERT calendar_items
    // (? : pour des parametres bindés (.bind()), plus sur et evite les injection de SQL)
    let result = sqlx::query(
        "INSERT INTO calendar_items
            (kind, title, timezone, status, description, place, icon, color_r, color_g, color_b, color_a, deleted_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(kind)
    .bind(&item.title)
    .bind(&item.timezone)
    .bind(status)
    .bind(&item.description)
    .bind(&item.place)
    .bind(&item.icon)
    .bind(color_r.map(|v| v as i64))
    .bind(color_g.map(|v| v as i64))
    .bind(color_b.map(|v| v as i64))
    .bind(color_a.map(|v| v as i64))
    .bind(&item.deleted_at)
    .execute(&mut *tx)
    .await?;

    // assign the SQLite generated id back to the item
    item.id = result.last_insert_rowid() as u64;

    // INSERT event or task
    match &item.kind {
        ItemKind::Event(event) => {
            // convert state to string
            let state = match event.state {
                EventState::Planned => "planned",
                EventState::Confirmed => "confirmed",
                EventState::Finished => "finished",
            };

            sqlx::query(
                "INSERT INTO events (item_id, date_start, date_end, full_day, state, parent_id)
                 VALUES (?, ?, ?, ?, ?, ?)"
            )
            .bind(item.id as i64)
            .bind(&event.date_start)
            .bind(&event.date_end)
            .bind(event.full_day)
            .bind(state)
            .bind(event.parent_id.map(|id| id as i64))
            .execute(&mut *tx)
            .await?;

            // INSERT recurrence if present
            if let Some(recurrence) = &event.recurrence {
                let frequency = match recurrence.frequency {
                    RecurrenceFrequency::Daily => "daily",
                    RecurrenceFrequency::Weekly => "weekly",
                    RecurrenceFrequency::Monthly => "monthly",
                    RecurrenceFrequency::Yearly => "yearly",
                };
                let (end_kind, end_value) = match &recurrence.end {
                    RecurrenceEnd::Never => ("never", None),
                    RecurrenceEnd::AfterOccurrences(n) => ("after_occurrences", Some(n.to_string())),
                    RecurrenceEnd::UntilDate(date) => ("until_date", Some(date.to_rfc3339())),
                };

                sqlx::query(
                    "INSERT INTO recurrences (item_id, frequency, interval, end_kind, end_value)
                    VALUES (?, ?, ?, ?, ?)"
                )
                .bind(item.id as i64)
                .bind(frequency)
                .bind(recurrence.interval as i64)
                .bind(end_kind)
                .bind(end_value)
                .execute(&mut *tx)
                .await?;

                // INSERT exceptions if present
                if let Some(exceptions) = &event.exceptions {
                    for exception in exceptions {
                        sqlx::query(
                            "INSERT INTO recurrence_exceptions (item_id, exception_date)
                            VALUES (?, ?)"
                        )
                        .bind(item.id as i64)
                        .bind(exception)
                        .execute(&mut *tx)
                        .await?;
                    }
                }
            }
        }
        ItemKind::Task(task) => {
            // convert state to string
            let state = match task.state {
                TaskState::Todo => "Todo",
                TaskState::Doing => "Doing",
                TaskState::Done => "Done",
            };            
            // convert state to string
            let criticality = match task.criticality {
                Some(Criticality::Low) => Some("low"),
                Some(Criticality::Medium) => Some("medium"),
                Some(Criticality::High) => Some("high"),
                Some(Criticality::Urgent) => Some("urgent"),
                Some(Criticality::Blocked) => Some("blocked"),
                None => None,
            };

            sqlx::query(
                "INSERT INTO tasks (item_id, deadline, state, criticality)
                 VALUES (?, ?, ?, ?)"
            )
            .bind(item.id as i64)
            .bind(&task.deadline)
            .bind(state)
            .bind(criticality)
            .execute(&mut *tx)
            .await?;

        }
    }

    // INSERT reminders if present
    if let Some(reminders) = &item.reminders {
        for reminder in reminders {
            let delay = match reminder.delay {
                ReminderDelay::Minutes5 => "Minutes5",
                ReminderDelay::Minutes10 => "Minutes10",
                ReminderDelay::Minutes15 => "Minutes15",
                ReminderDelay::Minutes30 => "Minutes30",
                ReminderDelay::Hour1 => "Hour1",
                ReminderDelay::Hour2 => "Hour2",
                ReminderDelay::Hour6 => "Hour6",
                ReminderDelay::Hour12 => "Hour12",
                ReminderDelay::Day1 => "Day1",
                ReminderDelay::Day2 => "Day2",
                ReminderDelay::Day3 => "Day3",
                ReminderDelay::Week1 => "Week1",
                ReminderDelay::Week2 => "Week2",
                ReminderDelay::Week3 => "Week3",
                ReminderDelay::Month1 => "Month1",
                ReminderDelay::Year1 => "Year1",
            };
            sqlx::query(
                "INSERT INTO reminders (item_id, active, delay)
                 VALUES (?, ?, ?)"
            )
            .bind(item.id as i64)
            .bind(reminder.active)
            .bind(delay)
            .execute(&mut *tx)
            .await?;
        }
    }

    // INSERT links if present
    if let Some(links) = &item.links {
        for link in links {
             sqlx::query(
                "INSERT INTO links (item_id, url)
                 VALUES (?, ?)"
            )
            .bind(item.id as i64)
            .bind(link)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}