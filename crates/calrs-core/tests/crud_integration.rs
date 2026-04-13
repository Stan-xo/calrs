//! Integration tests for CRUD storage operations.

use calrs_core::models::calendar_item::CalendarItem;
use calrs_core::storage::crud::insert_item;
use calrs_core::storage::sqlite::{init_database, open_database};
use chrono::Utc;

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
        "Integration test event".to_string(),
        Utc::now(),
        Utc::now(),
        false,
    );

    insert_item(&pool, &mut item).await.expect("Insert failed");
    assert!(item.id > 0);
}
