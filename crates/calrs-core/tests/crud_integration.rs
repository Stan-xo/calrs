//! Integration tests for CRUD storage operations.

use calrs_core::models::calendar_item::CalendarItem;
use calrs_core::storage::crud::{delete_item, get_item, insert_item, list_items, update_item};
use calrs_core::storage::sqlite::init_database;
use chrono::Utc;

async fn setup_db() -> sqlx::SqlitePool {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to open in-memory database");
    init_database(&pool).await.expect("Failed to init database");
    pool
}

fn make_test_event() -> CalendarItem {
    CalendarItem::new_event("Test event".to_string(), Utc::now(), Utc::now(), false)
}
fn make_test_task() -> CalendarItem {
    CalendarItem::new_task("Test task".to_string())
}

#[tokio::test]
async fn test_insert_event_assigns_id() {
    let pool = setup_db().await;

    let mut item = make_test_event();

    insert_item(&pool, &mut item).await.expect("Insert failed");
    assert!(item.id > 0);
}

#[tokio::test]
async fn test_get_nonexistent_returns_none() {
    let pool = setup_db().await;
    let fetched = get_item(&pool, 9999).await.expect("Get failed");
    assert!(fetched.is_none());
}

#[tokio::test]
async fn test_list_items() {
    let pool = setup_db().await;
    let mut item1 = make_test_event();
    let mut item2 = make_test_task();

    insert_item(&pool, &mut item1).await.unwrap();
    insert_item(&pool, &mut item2).await.unwrap();

    let items = list_items(&pool).await.expect("List failed");
    assert_eq!(items.len(), 2);
}

#[tokio::test]
async fn test_delete_item() {
    let pool = setup_db().await;
    let mut item = make_test_task();
    insert_item(&pool, &mut item).await.unwrap();

    delete_item(&pool, item.id).await.expect("Delete failed");

    // delete is soft, item still exists in calendar_items but not in list
    let items = list_items(&pool).await.unwrap();
    assert_eq!(items.len(), 0);
}

#[tokio::test]
async fn test_update_item() {
    let pool = setup_db().await;
    let mut item = make_test_event();
    insert_item(&pool, &mut item).await.unwrap();

    item.title = "Updated title".to_string();
    update_item(&pool, &item).await.expect("Update failed");

    let fetched = get_item(&pool, item.id).await.unwrap().unwrap();
    assert_eq!(fetched.title, item.title);
}
