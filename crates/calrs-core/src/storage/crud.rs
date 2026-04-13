//! CRUD operations for calendar items.
//!
//! All functions operate within a SQLite transaction to ensure
//! consistency across the multiple tables that make up a single item.
//!
//! # Operations
//! - [`insert_item`] : inserts a new item and assigns its generated id
//! - [`get_item`] :
//! - [`list_items`] :
//! - [`update_items`] :
//! - [`delete_item`] :

use crate::models::calendar_item::{CalendarItem, GlobalStatus, ItemKind};
use crate::models::calendar_item_row::{CalendarItemRow, EventRow, LinkRow, ReminderRow, TaskRow};
use crate::models::event::EventState;
use crate::models::recurrence::{RecurrenceEnd, RecurrenceFrequency};
use crate::models::reminder::{Reminder, ReminderDelay};
use crate::models::task::{Criticality, TaskState};
use sqlx::SqlitePool;

/// Inserts a new calendar item into the database.
///
/// Writes to all relevant tables in a single transaction.
/// The generated id is assigned back to `item.id` on success.
/// If any step fails, the transaction is rolled back.
pub async fn insert_item(pool: &SqlitePool, item: &mut CalendarItem) -> Result<(), sqlx::Error> {
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
                 VALUES (?, ?, ?, ?, ?, ?)",
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
                    RecurrenceEnd::AfterOccurrences(n) => {
                        ("after_occurrences", Some(n.to_string()))
                    }
                    RecurrenceEnd::UntilDate(date) => ("until_date", Some(date.to_rfc3339())),
                };

                sqlx::query(
                    "INSERT INTO recurrences (item_id, frequency, interval, end_kind, end_value)
                    VALUES (?, ?, ?, ?, ?)",
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
                            VALUES (?, ?)",
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
                 VALUES (?, ?, ?, ?)",
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
                 VALUES (?, ?, ?)",
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
                 VALUES (?, ?)",
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

/// Fetches a single calendar item by its id.
///
/// Returns `None` if no item with the given id exists.
/// Reminders and links are loaded and attached to the returned item.
pub async fn get_item(pool: &SqlitePool, id: u64) -> Result<Option<CalendarItem>, sqlx::Error> {
    // `Query as` : automatic row mapping via FromRow derive and TryFrom impl
    let row = sqlx::query_as::<_, CalendarItemRow>("SELECT * FROM calendar_items WHERE id = ?")
        .bind(id as i64)
        .fetch_optional(pool)
        .await?;

    // If no row found, return None
    let row = match row {
        None => return Ok(None),
        Some(r) => r,
    };

    // fetch event or task row selon kind
    let (event_row, task_row) = match row.kind.as_str() {
        "event" => {
            let event_row = sqlx::query_as::<_, EventRow>("SELECT * FROM events WHERE item_id = ?")
                .bind(id as i64)
                .fetch_optional(pool)
                .await?;
            (event_row, None)
        }
        "task" => {
            let task_row = sqlx::query_as::<_, TaskRow>("SELECT * FROM tasks WHERE item_id = ?")
                .bind(id as i64)
                .fetch_optional(pool)
                .await?;
            (None, task_row)
        }
        _ => return Ok(None),
    };

    // reconstruction
    let mut item = CalendarItem::try_from((row, event_row, task_row))
        .map_err(|e| sqlx::Error::Decode(e.into()))?;
    // Alternative :
    // let item: CalendarItem = (row, event_row, task_row)
    //     .try_into()
    //     .map_err(|e: String| sqlx::Error::Decode(e.into()))?;

    // fetch reminders
    let reminders = sqlx::query_as::<_, ReminderRow>("SELECT * FROM reminders WHERE item_id = ?")
        .bind(id as i64)
        .fetch_all(pool) // All item not only one
        .await?;

    // attach to item if any
    // reminders.into_iter() : consumes the Vec and creates an iterator over ReminderRow
    // .map(|r| ...) : transforms each ReminderRow into a Reminder
    //   |r| is a closure (like a lambda in Python) — r is the current ReminderRow
    // .collect() : consumes the iterator and assembles the results into a Vec<Reminder>
    //   Rust infers the target type Vec<Reminder> from the annotation above
    //
    // Equivalent in Python :
    //   reminders = [Reminder(r.active, parse_delay(r.delay)) for r in reminder_rows]
    // iter()      — emprunte les éléments (&ReminderRow), le Vec original reste utilisable
    // into_iter() — consomme les éléments (ReminderRow), le Vec original est détruit
    //               on utilise into_iter() quand on n'a plus besoin du Vec source
    if !reminders.is_empty() {
        let reminders: Result<Vec<Reminder>, sqlx::Error> = reminders
            .into_iter()
            .map(|r| -> Result<Reminder, sqlx::Error> {
                let delay = match r.delay.as_str() {
                    "Minutes5" => ReminderDelay::Minutes5,
                    "Minutes10" => ReminderDelay::Minutes10,
                    "Minutes15" => ReminderDelay::Minutes15,
                    "Minutes30" => ReminderDelay::Minutes30,
                    "Hour1" => ReminderDelay::Hour1,
                    "Hour2" => ReminderDelay::Hour2,
                    "Hour6" => ReminderDelay::Hour6,
                    "Hour12" => ReminderDelay::Hour12,
                    "Day1" => ReminderDelay::Day1,
                    "Day2" => ReminderDelay::Day2,
                    "Day3" => ReminderDelay::Day3,
                    "Week1" => ReminderDelay::Week1,
                    "Week2" => ReminderDelay::Week2,
                    "Week3" => ReminderDelay::Week3,
                    "Month1" => ReminderDelay::Month1,
                    "Year1" => ReminderDelay::Year1,
                    other => {
                        return Err(sqlx::Error::Decode(
                            format!("Unknown reminder delay: {}", other).into(),
                        ));
                    }
                };
                Ok(Reminder {
                    active: r.active,
                    delay,
                })
            })
            .collect();

        let reminders = reminders?;
        item.reminders = Some(reminders);
    }

    // fetch links
    let links = sqlx::query_as::<_, LinkRow>("SELECT * FROM links WHERE item_id = ?")
        .bind(id as i64)
        .fetch_all(pool)
        .await?;

    // attach to item if any
    if !links.is_empty() {
        item.links = Some(links.into_iter().map(|l| l.url).collect());
    }

    Ok(Some(item))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::calendar_item::CalendarItem;
    use crate::storage::sqlite::{init_database, open_database};
    use chrono::Utc;

    #[tokio::test]
    async fn test_open_database_creates_file() {
        let pool = open_database("test.db").await;
        assert!(pool.is_ok());
    }

    // helper: opens an in-memory database ready to use
    async fn setup_db() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to open in-memory database");
        init_database(&pool).await.expect("Failed to init database");
        pool
    }

    #[tokio::test]
    async fn test_insert_event_assigns_id() {
        let pool = setup_db().await;

        let mut item =
            CalendarItem::new_event("Test event".to_string(), Utc::now(), Utc::now(), false);

        assert_eq!(item.id, 0); // avant insert, id = 0

        insert_item(&pool, &mut item).await.expect("Insert failed");

        assert!(item.id > 0); // après insert, id assigné par SQLite
    }
}
