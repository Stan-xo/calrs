//! SQLite storage backend for calrs.
//!
//! Handles database connection and schema initialization.
//! All calendar data is stored in a single SQLite file located
//! in the OS data directory under `calrs/`.
//!
//! # Database schema
//!
//! ```text
//! calendar_items
//!   id, kind, title, timezone, status, description,
//!   place, icon, color_r/g/b/a, deleted_at
//!
//!   +-- events (item_id -> calendar_items.id)
//!   |     date_start, date_end, full_day, state
//!   |
//!   +-- tasks (item_id -> calendar_items.id)
//!   |     deadline, state, criticality
//!   |
//!   +-- reminders (item_id -> calendar_items.id)
//!   |     active, delay
//!   |
//!   +-- links (item_id -> calendar_items.id)
//!   |     url
//!   |
//!   +-- recurrences (item_id -> calendar_items.id)
//!   |     frequency, interval, end_kind, end_value
//!   |
//!   +-- recurrence_exceptions (item_id -> calendar_items.id)
//!         exception_date
//! ```
//!
//! # Profile support
//!
//! Multiple databases can coexist under `calrs/` by passing different
//! `db_name` values to [`open_database`]. This enables user profiles
//! or isolated test databases.
//!
//! # Examples
//!
//! ```rust,no_run
//! use calrs_core::storage::sqlite::{open_database, init_database};
//!
//! #[tokio::main]
//! async fn main() {
//!     let pool = open_database("calrs").await.unwrap();
//!     init_database(&pool).await.unwrap();
//! }
//! ```

use crate::APP_NAME;
use sqlx::SqlitePool;
use std::path::PathBuf;

/// Opens or creates the SQLite database for the given profile name.
///
/// The database file is stored in the OS data directory under `calrs/{db_name}.db`.
/// The directory is created if it does not exist.
pub async fn open_database(db_name: &str) -> Result<SqlitePool, sqlx::Error> {
    let db_path: PathBuf = dirs::data_dir()
        .expect("Could not find data directory")
        .join(APP_NAME)
        .join(format!("{}.db", db_name));
    let sql_url = format!("sqlite://{}?mode=rwc", db_path.to_str().unwrap()); // mode "read" "write" "create"

    // Creation of the database file in the OS data folder
    std::fs::create_dir_all(db_path.parent().unwrap()).map_err(sqlx::Error::Io)?; // `map_err`convert error std to sqlx, `? propagate error (like raise in python)

    // Connect database
    SqlitePool::connect(&sql_url).await
}

/// Initializes the database schema.
///
/// Creates all required tables if they do not already exist.
/// Safe to call on every startup — existing data is never affected.
pub async fn init_database(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // calendar_items
    // - kind is `event` or `task`
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS calendar_items (
        id          INTEGER PRIMARY KEY,
        kind        TEXT NOT NULL, 
        title       TEXT NOT NULL,
        timezone    TEXT NOT NULL,
        status      TEXT NOT NULL,
        description TEXT,
        place       TEXT,
        icon        TEXT,
        color_r     INTEGER,
        color_g     INTEGER,
        color_b     INTEGER,
        color_a     INTEGER,
        deleted_at  TEXT
        )",
    )
    .execute(pool)
    .await?;
    // events
    // - linked to calendar_items via item_id
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS events (
        item_id     INTEGER PRIMARY KEY,
        date_start  DATETIME NOT NULL,
        date_end    DATETIME NOT NULL,
        full_day    BOOLEAN NOT NULL,
        state       TEXT NOT NULL,
        parent_id   INTEGER,
        FOREIGN KEY (item_id) REFERENCES calendar_items(id)
        )",
    )
    .execute(pool)
    .await?;
    // tasks
    // - linked to calendar_items via item_id
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS tasks (
        item_id     INTEGER PRIMARY KEY,
        deadline    DATETIME,
        state       TEXT NOT NULL,
        criticality TEXT,
        FOREIGN KEY (item_id) REFERENCES calendar_items(id)
        )",
    )
    .execute(pool)
    .await?;
    // reminders
    // - linked to calendar_items via item_id
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS reminders (
        id          INTEGER PRIMARY KEY,
        item_id     INTEGER NOT NULL,
        active      BOOLEAN NOT NULL,
        delay       TEXT NOT NULL,
        FOREIGN KEY (item_id) REFERENCES calendar_items(id)
        )",
    )
    .execute(pool)
    .await?;
    // links
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS links (
        id          INTEGER PRIMARY KEY,
        item_id     INTEGER NOT NULL,
        url         TEXT NOT NULL,
        FOREIGN KEY (item_id) REFERENCES calendar_items(id)
        )",
    )
    .execute(pool)
    .await?;
    // recurrences
    // - end_kind is  "never", "after_occurrences" or "until_date"
    // - end value is "never" (for NULL), a number or a date
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS recurrences (
        item_id     INTEGER PRIMARY KEY,
        frequency   TEXT NOT NULL,
        interval    INTEGER NOT NULL,
        end_kind    TEXT NOT NULL,
        end_value   TEXT,
        FOREIGN KEY (item_id) REFERENCES calendar_items(id)
        )",
    )
    .execute(pool)
    .await?;
    // recurrence_exceptions
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS recurrence_exceptions (
        id          INTEGER PRIMARY KEY,
        item_id     INTEGER NOT NULL,
        exception_date DATETIME NOT NULL,
        FOREIGN KEY (item_id) REFERENCES calendar_items(id)
        )",
    )
    .execute(pool)
    .await?;

    Ok(())
}
