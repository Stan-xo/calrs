//! Intermediate flat structs for mapping SQLite rows to CalendarItem.
//!
//! These structs mirror the database tables exactly and are used
//! as an intermediate step before converting to domain types.

use chrono::DateTime;
use chrono::Utc;
use sqlx::FromRow;

/// Flat representation of a `calendar_items` row.
#[derive(Debug, FromRow)]
pub struct CalendarItemRow {
    pub id: i64,
    pub kind: String,
    pub title: String,
    pub timezone: String,
    pub status: String,
    pub description: Option<String>,
    pub place: Option<String>,
    pub icon: Option<String>,
    pub color_r: Option<i64>,
    pub color_g: Option<i64>,
    pub color_b: Option<i64>,
    pub color_a: Option<i64>,
    pub deleted_at: Option<String>,
}

/// Flat representation of an `events` row.
#[derive(Debug, FromRow)]
pub struct EventRow {
    pub item_id: i64,
    pub date_start: DateTime<Utc>,
    pub date_end: DateTime<Utc>,
    pub full_day: bool,
    pub state: String,
    pub parent_id: Option<i64>,
}

/// Flat representation of a `tasks` row.
#[derive(Debug, FromRow)]
pub struct TaskRow {
    pub item_id: i64,
    pub deadline: Option<DateTime<Utc>>,
    pub state: String,
    pub criticality: Option<String>,
}

/// Flat representation of a `reminder` row.
#[derive(Debug, FromRow)]
pub struct ReminderRow {
    pub item_id: i64,
    pub active: bool,
    pub delay: String,
}

/// Flat representation of a `reminder` row.
#[derive(Debug, FromRow)]
pub struct LinkRow {
    pub item_id: i64,
    pub url: String,
}
