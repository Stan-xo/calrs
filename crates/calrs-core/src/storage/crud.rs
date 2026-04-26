//! CRUD operations for calendar items.
//!
//! All functions operate within a SQLite transaction to ensure
//! consistency across the multiple tables that make up a single item.
//!
//! # Operations
//! - [`insert_item`] : inserts a new item and assigns its generated id
//! - [`get_item`] : get an item
//! - [`list_items`] : list items
//! - [`delete_item`] : delete an item
//! - [`update_item`] : update an item

use crate::models::calendar_item::{CalendarItem, GlobalStatus, ItemKind};
use crate::models::calendar_item_row::{CalendarItemRow, EventRow, LinkRow, ReminderRow, TaskRow};
use crate::models::event::EventState;
use crate::models::recurrence::{Recurrence, RecurrenceEnd, RecurrenceFrequency};
use crate::models::reminder::{Reminder, ReminderDelay};
use crate::models::task::{Criticality, TaskState};
use chrono::Utc;
use sqlx::SqlitePool;

/// Inserts a new calendar item into the database.
///
/// Writes to all relevant tables in a single transaction.
/// The generated id is assigned back to `item.id` on success.
/// If any step fails, the transaction is rolled back.
pub async fn insert_item(pool: &SqlitePool, item: &mut CalendarItem) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // convert enums to strings for SQLite
    let kind = kind_to_string(&item.kind);
    let status = status_to_string(&item.status);

    let (color_r, color_g, color_b, color_a) = color_to_u8_colors(item.color);

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
    .bind(item.deleted_at)
    .execute(&mut *tx)
    .await?;

    // assign the SQLite generated id back to the item
    item.id = result.last_insert_rowid() as u64;

    // INSERT event or task
    match &item.kind {
        ItemKind::Event(event) => {
            // convert state to string
            let state = state_event_to_string(&event.state);

            sqlx::query(
                "INSERT INTO events (item_id, date_start, date_end, full_day, state, parent_id)
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(item.id as i64)
            .bind(event.date_start)
            .bind(event.date_end)
            .bind(event.full_day)
            .bind(state)
            .bind(event.parent_id.map(|id| id as i64))
            .execute(&mut *tx)
            .await?;

            // INSERT recurrence if present
            if let Some(recurrence) = &event.recurrence {
                insert_recurrence(&mut tx, item.id, recurrence).await?;

                // INSERT exceptions if present
                if let Some(exceptions) = &event.exceptions {
                    insert_exceptions(&mut tx, item.id, exceptions).await?;
                }
            }
        }
        ItemKind::Task(task) => {
            // convert state to string
            let state = state_task_to_string(&task.state);
            // convert state to string
            let criticality = criticality_to_sting(&task.criticality);

            sqlx::query(
                "INSERT INTO tasks (item_id, deadline, state, criticality)
                 VALUES (?, ?, ?, ?)",
            )
            .bind(item.id as i64)
            .bind(task.deadline)
            .bind(state)
            .bind(criticality)
            .execute(&mut *tx)
            .await?;
        }
    }

    // INSERT reminders if present
    if let Some(reminders) = &item.reminders {
        insert_reminders(&mut tx, item.id, reminders).await?;
    }

    if let Some(links) = &item.links {
        insert_links(&mut tx, item.id, links).await?;
    }

    tx.commit().await?;
    Ok(())
}

/// Updates an existing calendar item in the database.
///
/// All tables are updated within a single transaction.
/// If any step fails, the transaction is rolled back.
pub async fn update_item(pool: &SqlitePool, item: &CalendarItem) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    let kind = kind_to_string(&item.kind);
    let status = status_to_string(&item.status);
    let (color_r, color_g, color_b, color_a) = color_to_u8_colors(item.color);

    // UPDATE calendar_items
    sqlx::query(
        "UPDATE calendar_items SET
            kind = ?, title = ?, timezone = ?, status = ?,
            description = ?, place = ?, icon = ?,
            color_r = ?, color_g = ?, color_b = ?, color_a = ?,
            deleted_at = ?
         WHERE id = ?",
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
    .bind(item.deleted_at)
    .bind(item.id as i64)
    .execute(&mut *tx)
    .await?;

    // UPDATE events or tasks
    match &item.kind {
        ItemKind::Event(event) => {
            // UPDATE events
            let state = state_event_to_string(&event.state);
            sqlx::query(
                "UPDATE events SET
                    date_start = ?, date_end = ?, full_day = ?, state = ?, parent_id = ?
                    WHERE item_id = ?",
            )
            .bind(event.date_start)
            .bind(event.date_end)
            .bind(event.full_day)
            .bind(state)
            .bind(event.parent_id.map(|id| id as i64))
            .bind(item.id as i64)
            .execute(&mut *tx)
            .await?;

            // DELETE recurrence and exceptions
            sqlx::query("DELETE FROM recurrences WHERE item_id = ?")
                .bind(item.id as i64)
                .execute(&mut *tx)
                .await?;
            sqlx::query("DELETE FROM recurrence_exceptions WHERE item_id = ?")
                .bind(item.id as i64)
                .execute(&mut *tx)
                .await?;

            // INSERT recurrence and exceptions
            if let Some(recurrence) = &event.recurrence {
                insert_recurrence(&mut tx, item.id, recurrence).await?;
            }
            if let Some(exceptions) = &event.exceptions {
                insert_exceptions(&mut tx, item.id, exceptions).await?;
            }
        }
        ItemKind::Task(task) => {
            // UPDATE tasks
            //     "INSERT INTO tasks (item_id, deadline, state, criticality)
            //  VALUES (?, ?, ?, ?)",
            let state = state_task_to_string(&task.state);
            let criticality = criticality_to_sting(&task.criticality);
            sqlx::query(
                "UPDATE tasks SET
                    deadline = ?, state = ?, criticality = ?
                    WHERE item_id = ?",
            )
            .bind(task.deadline)
            .bind(state)
            .bind(criticality)
            .bind(item.id as i64)
            .execute(&mut *tx)
            .await?;
        }
    }

    // DELETE + INSERT for reminders and links
    sqlx::query("DELETE FROM reminders WHERE item_id = ?")
        .bind(item.id as i64)
        .execute(&mut *tx)
        .await?;
    if let Some(reminders) = &item.reminders {
        insert_reminders(&mut tx, item.id, reminders).await?;
    }

    sqlx::query("DELETE FROM links WHERE item_id = ?")
        .bind(item.id as i64)
        .execute(&mut *tx)
        .await?;
    if let Some(links) = &item.links {
        insert_links(&mut tx, item.id, links).await?;
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

    Ok(Some(build_item(pool, row).await?))
}

/// Soft-deletes a calendar item by setting its `deleted_at` timestamp.
///
/// The item is never physically removed from the database,
/// preserving history and enabling sync reconciliation.
pub async fn delete_item(pool: &SqlitePool, id: u64) -> Result<(), sqlx::Error> {
    let utc_now = Utc::now();

    sqlx::query("UPDATE calendar_items SET deleted_at = ? WHERE id = ?")
        .bind(utc_now)
        .bind(id as i64)
        .execute(pool)
        .await?;

    Ok(())
}

/// Deletes all items from the database, effectively resetting the profile.
pub async fn reset_database(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM recurrence_exceptions")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM recurrences")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM reminders")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM links").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM events").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM tasks").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM calendar_items")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

/// Fetches all active calendar items.
///
/// Deleted items (where `deleted_at` is set) are excluded.
pub async fn list_items(pool: &SqlitePool) -> Result<Vec<CalendarItem>, sqlx::Error> {
    let mut items: Vec<CalendarItem> = Vec::new();

    let rows = sqlx::query_as::<_, CalendarItemRow>(
        "SELECT * FROM calendar_items WHERE deleted_at IS NULL",
    )
    .fetch_all(pool)
    .await?;

    // .map interdit sur les fonctions async
    for row in rows {
        let item = build_item(pool, row).await?;
        items.push(item)
    }

    Ok(items)
}

// Builds a complete CalendarItem from a row, fetching kind, reminders and links.
async fn build_item(pool: &SqlitePool, row: CalendarItemRow) -> Result<CalendarItem, sqlx::Error> {
    let id = row.id as u64;

    // fetch event or task row selon kind
    let (event_row, task_row) = fetch_kind(pool, id, row.kind.as_str()).await?;

    // reconstruction
    let mut item = CalendarItem::try_from((row, event_row, task_row))
        .map_err(|e| sqlx::Error::Decode(e.into()))?;
    // Alternative :
    // let item: CalendarItem = (row, event_row, task_row)
    //     .try_into()
    //     .map_err(|e: String| sqlx::Error::Decode(e.into()))?;

    // fetch reminders
    item.reminders = fetch_reminders(pool, id).await?;

    // fetch links
    item.links = fetch_links(pool, id).await?;

    Ok(item)
}

/// Updates the timezone of all calendar items for a given profile.
///
/// Useful when the user changes their profile timezone and wants
/// to apply it retroactively to all existing items.
pub async fn update_all_timezone(pool: &SqlitePool, timezone: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE calendar_items SET timezone = ? WHERE deleted_at IS NULL")
        .bind(timezone)
        .execute(pool)
        .await?;
    Ok(())
}

// fetchers
async fn fetch_kind(
    pool: &SqlitePool,
    id: u64,
    kind: &str,
) -> Result<(Option<EventRow>, Option<TaskRow>), sqlx::Error> {
    // fetch event or task row selon kind
    match kind {
        "event" => {
            let event_row = sqlx::query_as::<_, EventRow>("SELECT * FROM events WHERE item_id = ?")
                .bind(id as i64)
                .fetch_optional(pool)
                .await?;
            Ok((event_row, None))
        }
        "task" => {
            let task_row = sqlx::query_as::<_, TaskRow>("SELECT * FROM tasks WHERE item_id = ?")
                .bind(id as i64)
                .fetch_optional(pool)
                .await?;
            Ok((None, task_row))
        }
        _ => Ok((None, None)),
    }
}

async fn fetch_reminders(pool: &SqlitePool, id: u64) -> Result<Option<Vec<Reminder>>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ReminderRow>("SELECT * FROM reminders WHERE item_id = ?")
        .bind(id as i64)
        .fetch_all(pool)
        .await?;

    if rows.is_empty() {
        return Ok(None);
    }

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

    let reminders: Result<Vec<Reminder>, sqlx::Error> = rows
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
                "Day1" => ReminderDelay::Hour12,
                "Day2" => ReminderDelay::Hour12,
                "Day3" => ReminderDelay::Hour12,
                "Week1" => ReminderDelay::Hour12,
                "Week2" => ReminderDelay::Hour12,
                "Week3" => ReminderDelay::Hour12,
                "Month1" => ReminderDelay::Hour12,
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

    Ok(Some(reminders?))
}

async fn fetch_links(pool: &SqlitePool, id: u64) -> Result<Option<Vec<String>>, sqlx::Error> {
    let rows = sqlx::query_as::<_, LinkRow>("SELECT * FROM links WHERE item_id = ?")
        .bind(id as i64)
        .fetch_all(pool)
        .await?;

    if rows.is_empty() {
        return Ok(None);
    }

    Ok(Some(rows.into_iter().map(|l| l.url).collect()))
}

// Conv enum and other
fn kind_to_string(kind: &ItemKind) -> &'static str {
    match kind {
        ItemKind::Event(_) => "event",
        ItemKind::Task(_) => "task",
    }
}

fn status_to_string(status: &GlobalStatus) -> &'static str {
    match status {
        GlobalStatus::Active => "active",
        GlobalStatus::Cancelled => "cancelled",
        GlobalStatus::Draft => "draft",
    }
}

fn color_to_u8_colors(
    color: Option<(u8, u8, u8, u8)>,
) -> (Option<u8>, Option<u8>, Option<u8>, Option<u8>) {
    match color {
        Some((r, g, b, a)) => (Some(r), Some(g), Some(b), Some(a)),
        None => (None, None, None, None),
    }
}

fn state_event_to_string(state: &EventState) -> &'static str {
    match state {
        EventState::Planned => "planned",
        EventState::Confirmed => "confirmed",
        EventState::Finished => "finished",
    }
}

fn state_task_to_string(state: &TaskState) -> &'static str {
    match state {
        TaskState::Todo => "todo",
        TaskState::Doing => "doing",
        TaskState::Done => "done",
    }
}

fn rec_frequency_to_sting(frequency: &RecurrenceFrequency) -> &'static str {
    match frequency {
        RecurrenceFrequency::Daily => "daily",
        RecurrenceFrequency::Weekly => "weekly",
        RecurrenceFrequency::Monthly => "monthly",
        RecurrenceFrequency::Yearly => "yearly",
    }
}

fn rec_end_to_sting(end: &RecurrenceEnd) -> (&'static str, Option<String>) {
    match end {
        RecurrenceEnd::Never => ("never", None),
        RecurrenceEnd::AfterOccurrences(n) => ("after_occurrences", Some(n.to_string())),
        RecurrenceEnd::UntilDate(date) => ("until_date", Some(date.to_rfc3339())),
    }
}

fn criticality_to_sting(criticality: &Option<Criticality>) -> Option<&'static str> {
    match criticality {
        Some(Criticality::Low) => Some("low"),
        Some(Criticality::Medium) => Some("medium"),
        Some(Criticality::High) => Some("high"),
        Some(Criticality::Urgent) => Some("urgent"),
        Some(Criticality::Blocked) => Some("blocked"),
        None => None,
    }
}

fn rem_delay_to_string(delay: &ReminderDelay) -> &'static str {
    match delay {
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
    }
}

// insert
async fn insert_reminders(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    item_id: u64,
    reminders: &[Reminder],
) -> Result<(), sqlx::Error> {
    for reminder in reminders {
        let delay = rem_delay_to_string(&reminder.delay);
        sqlx::query("INSERT INTO reminders (item_id, active, delay) VALUES (?, ?, ?)")
            .bind(item_id as i64)
            .bind(reminder.active)
            .bind(delay)
            .execute(&mut **tx) // double déréf nécessaire ici
            .await?;
    }
    Ok(())
}

async fn insert_links(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    item_id: u64,
    links: &[String],
) -> Result<(), sqlx::Error> {
    for link in links {
        sqlx::query("INSERT INTO links (item_id, url) VALUES (?, ?)")
            .bind(item_id as i64)
            .bind(link)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

async fn insert_recurrence(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    item_id: u64,
    recurrence: &Recurrence,
) -> Result<(), sqlx::Error> {
    let frequency = rec_frequency_to_sting(&recurrence.frequency);
    let (end_kind, end_value) = rec_end_to_sting(&recurrence.end);

    sqlx::query(
        "INSERT INTO recurrences (item_id, frequency, interval, end_kind, end_value)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(item_id as i64)
    .bind(frequency)
    .bind(recurrence.interval as i64)
    .bind(end_kind)
    .bind(end_value)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn insert_exceptions(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    item_id: u64,
    exceptions: &[chrono::DateTime<chrono::Utc>],
) -> Result<(), sqlx::Error> {
    for exception in exceptions {
        sqlx::query("INSERT INTO recurrence_exceptions (item_id, exception_date) VALUES (?, ?)")
            .bind(item_id as i64)
            .bind(exception)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::calendar_item::CalendarItem;
    use crate::storage::sqlite::{init_database, open_database};
    use chrono::Utc;

    #[tokio::test]
    async fn test_open_database_creates_file() {
        let pool = open_database("test").await;
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

        let mut item = CalendarItem::new_event(
            "Test event".to_string(),
            "UTC".to_string(),
            Utc::now(),
            Utc::now(),
            false,
        );

        assert_eq!(item.id, 0); // avant insert, id = 0

        insert_item(&pool, &mut item).await.expect("Insert failed");

        assert!(item.id > 0); // après insert, id assigné par SQLite
    }
}
