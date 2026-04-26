//! Integration tests for profile management API.

use calrs_core::APP_NAME;
use calrs_core::profile::api::{
    active_profile, active_profile_entry, create_profile, delete_profile, get_profile,
    list_profiles, profile_exists, select_profile, update_profile_timezone,
};
use calrs_core::profile::config::ProfileEntry;
use calrs_core::profile::error::ProfileError;
use std::path::PathBuf;
use tempfile::{TempDir, tempdir};

// helper : create a temporal folder and return is path for the config.toml file
fn temp_config() -> (TempDir, PathBuf) {
    let dir = tempdir().expect("Failed to create temp dir");
    let path = dir.path().join("config.toml");
    (dir, path) // on retourne dir pour qu'il reste en vie le temps du test
}

#[test]
fn test_create_check_and_list_profiles() {
    let (_dir, tmp_path) = temp_config();

    // Create
    create_profile("profile_1", "UTC".to_string(), Some(&tmp_path)).unwrap();
    create_profile("profile_2", "UTC".to_string(), Some(&tmp_path)).unwrap();

    // Check
    assert_eq!(profile_exists("profile_2", Some(&tmp_path)), true);

    // List
    let profiles = list_profiles(Some(&tmp_path)).unwrap();

    assert_eq!(profiles.len(), 2);

    assert_eq!(profiles[0], "profile_1");
}

#[test]
fn test_create_duplicate_fails() {
    let (_dir, tmp_path) = temp_config();

    create_profile("profile_1", "UTC".to_string(), Some(&tmp_path)).unwrap();

    // Second create should fail
    let result = create_profile("profile_1", "UTC".to_string(), Some(&tmp_path));
    assert!(result.is_err());

    // Check error type
    assert!(matches!(
        result.unwrap_err(),
        ProfileError::AlreadyExists(_)
    ));
}

#[test]
fn test_active_and_select_profile() {
    let (_dir, tmp_path) = temp_config();

    // Check actif profile
    let profile_name = active_profile(Some(&tmp_path));
    assert!(profile_name.is_err());
    assert!(matches!(
        profile_name.unwrap_err(),
        ProfileError::NoProfiles
    ));
    let result = active_profile_entry(Some(&tmp_path));
    assert!(result.is_err());
    // eprintln!("Erreur reçue: {:?}", result);
    assert!(matches!(result.unwrap_err(), ProfileError::NoProfiles));

    // Create
    create_profile("profile_1", "UTC".to_string(), Some(&tmp_path)).unwrap();
    create_profile("profile_2", "UTC".to_string(), Some(&tmp_path)).unwrap();
    let profile_name = active_profile(Some(&tmp_path)).unwrap();
    assert_eq!(profile_name, "profile_1");

    // active profil entry
    let result = active_profile_entry(Some(&tmp_path)).unwrap();
    assert!(matches!(result, ProfileEntry { .. }));

    // Select not existing profile
    let result = select_profile("NotFoundProfile", Some(&tmp_path));
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ProfileError::NotFound(_)));

    // Select IO Error
    // Don't know how to test it maybe with permissions on the file

    // Select existinf profile
    select_profile("profile_2", Some(&tmp_path)).unwrap();
    let profile_name = active_profile(Some(&tmp_path)).unwrap();
    assert_eq!(profile_name, "profile_2");
}

#[test]
fn test_delete_profile() {
    let (_dir, tmp_path) = temp_config();

    // Create
    create_profile("profile_1", "UTC".to_string(), Some(&tmp_path)).unwrap();

    // simulate that the db exists
    let db_path_1 = dirs::data_dir()
        .expect("Could not find data directory")
        .join(APP_NAME)
        .join(format!("{}.db", "profile_1"));
    std::fs::create_dir_all(db_path_1.parent().unwrap()).unwrap();
    std::fs::write(&db_path_1, "").unwrap();

    // Delete without purge
    delete_profile("profile_1", false, Some(&tmp_path)).unwrap();

    // Check active profile
    let profile_name = active_profile(Some(&tmp_path));
    assert!(profile_name.is_err());
    assert!(matches!(
        profile_name.unwrap_err(),
        ProfileError::NoProfiles
    ));

    // Check db still exists (no purge)
    assert!(db_path_1.exists());

    // Create two more profiles
    create_profile("profile_1bis", "UTC".to_string(), Some(&tmp_path)).unwrap();
    create_profile("profile_2", "UTC".to_string(), Some(&tmp_path)).unwrap();

    // simulate that profile_1bis db exists
    let db_path_1bis = dirs::data_dir()
        .expect("Could not find data directory")
        .join(APP_NAME)
        .join(format!("{}.db", "profile_1bis"));
    std::fs::create_dir_all(db_path_1bis.parent().unwrap()).unwrap();
    std::fs::write(&db_path_1bis, "").unwrap();

    // Check active profile — first created becomes active
    let profile_name = active_profile(Some(&tmp_path)).unwrap();
    assert_eq!(profile_name, "profile_1bis");

    // Delete with purge
    delete_profile("profile_1bis", true, Some(&tmp_path)).unwrap();

    // Check db deleted (purge)
    assert!(!db_path_1bis.exists());

    // Check active profile falls back to profile_2
    let profile_name = active_profile(Some(&tmp_path)).unwrap();
    assert_eq!(profile_name, "profile_2");
}

#[test]
fn test_get_profile() {
    let (_dir, tmp_path) = temp_config();

    create_profile("profile_1", "UTC".to_string(), Some(&tmp_path)).unwrap();

    // get existing profile
    let profile = get_profile("profile_1", Some(&tmp_path)).unwrap();
    assert_eq!(profile.name, "profile_1");
    assert_eq!(profile.timezone, "UTC");

    // get nonexistent profile
    let result = get_profile("nonexistent", Some(&tmp_path));
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ProfileError::NotFound(_)));
}

#[test]
fn test_update_timezone() {
    let (_dir, tmp_path) = temp_config();

    create_profile("profile_1", "UTC".to_string(), Some(&tmp_path)).unwrap();

    // update timezone
    update_profile_timezone("profile_1", "Europe/Paris", Some(&tmp_path)).unwrap();

    // check timezone updated
    let profile = get_profile("profile_1", Some(&tmp_path)).unwrap();
    assert_eq!(profile.timezone, "Europe/Paris");

    // update nonexistent profile
    let result = update_profile_timezone("nonexistent", "Europe/Paris", Some(&tmp_path));
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ProfileError::NotFound(_)));
}
