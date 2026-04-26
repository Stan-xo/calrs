//! Public API for profile management.

use super::config::{self, ProfileEntry};
use super::error::ProfileError;
use std::path::Path;

/// Lists all known profiles.
pub fn list_profiles(path: Option<&Path>) -> Result<Vec<String>, ProfileError> {
    let cfg = config::load(path).map_err(ProfileError::Io)?;
    Ok(cfg.profiles.into_iter().map(|p| p.name).collect())
}

/// Returns the name of the currently active profile.
pub fn active_profile(path: Option<&Path>) -> Result<String, ProfileError> {
    let cfg = config::load(path).map_err(ProfileError::Io)?;
    if cfg.profiles.is_empty() {
        return Err(ProfileError::NoProfiles);
    }
    Ok(cfg.active_profile)
}

/// Returns true if a profile with the given name exists.
pub fn profile_exists(name: &str, path: Option<&Path>) -> bool {
    match config::load(path) {
        Ok(cfg) => cfg.profiles.iter().any(|p| p.name == name),
        Err(_) => false,
    }
}

/// Creates a new profile.
///
/// If this is the first profile, it becomes the active one automatically.
pub fn create_profile(
    name: &str,
    timezone: String,
    path: Option<&Path>,
) -> Result<(), ProfileError> {
    let mut cfg = config::load(path).map_err(ProfileError::Io)?;

    if cfg.profiles.iter().any(|p| p.name == name) {
        return Err(ProfileError::AlreadyExists(name.to_string()));
    }

    cfg.profiles.push(ProfileEntry {
        name: name.to_string(),
        timezone,
    });

    if cfg.active_profile.is_empty() {
        cfg.active_profile = name.to_string();
    }

    config::save(&cfg, path).map_err(ProfileError::Io)?;
    Ok(())
}

/// Deletes a profile from the configuration.
///
/// If `purge` is true, also deletes the underlying `.db` file.
/// Note: if `purge` is false and the profile is recreated with the same name,
/// existing data will be recovered from the `.db` file.
pub fn delete_profile(name: &str, purge: bool, path: Option<&Path>) -> Result<(), ProfileError> {
    let mut cfg = config::load(path).map_err(ProfileError::Io)?;

    let index = cfg
        .profiles
        .iter()
        .position(|p| p.name == name)
        .ok_or_else(|| ProfileError::NotFound(name.to_string()))?;

    cfg.profiles.remove(index);

    if cfg.active_profile == name {
        cfg.active_profile = cfg
            .profiles
            .first()
            .map(|p| p.name.clone())
            .unwrap_or_default();
    }

    config::save(&cfg, path).map_err(ProfileError::Io)?;

    // purge : delete the .db file if requested
    if purge {
        let db_path = dirs::data_dir()
            .expect("Could not find data directory")
            .join(crate::APP_NAME)
            .join(format!("{}.db", name));

        if db_path.exists() {
            std::fs::remove_file(&db_path)
                .map_err(|e| ProfileError::Io(format!("Failed to delete database: {}", e)))?;
        }
    }

    Ok(())
}

/// Resets a profile by deleting its `.db` file.
///
/// The profile remains in the configuration. On next use, a fresh
/// database will be created automatically by `open_database`.
/// This is equivalent to deleting and recreating the profile.
pub fn reset_profile(name: &str, path: Option<&Path>) -> Result<(), ProfileError> {
    if !profile_exists(name, path) {
        return Err(ProfileError::NotFound(name.to_string()));
    }

    let db_path = dirs::data_dir()
        .expect("Could not find data directory")
        .join(crate::APP_NAME)
        .join(format!("{}.db", name));

    if db_path.exists() {
        std::fs::remove_file(&db_path)
            .map_err(|e| ProfileError::Io(format!("Failed to reset database: {}", e)))?;
    }

    Ok(())
}

/// Selects an existing profile as the active one.
pub fn select_profile(name: &str, path: Option<&Path>) -> Result<(), ProfileError> {
    let mut cfg = config::load(path).map_err(ProfileError::Io)?;

    if !cfg.profiles.iter().any(|p| p.name == name) {
        return Err(ProfileError::NotFound(name.to_string()));
    }

    cfg.active_profile = name.to_string();
    config::save(&cfg, path).map_err(ProfileError::Io)?;
    Ok(())
}
/// Returns the profile entry for the given name.
pub fn get_profile(name: &str, path: Option<&Path>) -> Result<ProfileEntry, ProfileError> {
    let cfg = config::load(path).map_err(ProfileError::Io)?;
    cfg.profiles
        .into_iter()
        .find(|p| p.name == name)
        .ok_or_else(|| ProfileError::NotFound(name.to_string()))
}

/// Returns the active profile entry.
pub fn active_profile_entry(path: Option<&Path>) -> Result<ProfileEntry, ProfileError> {
    let name = active_profile(path)?;
    get_profile(&name, path)
}

/// Updates the timezone of an existing profile.
///
/// Use `update_all_timezone` from crud to also update existing items.
pub fn update_profile_timezone(
    name: &str,
    timezone: &str,
    path: Option<&Path>,
) -> Result<(), ProfileError> {
    let mut cfg = config::load(path).map_err(ProfileError::Io)?;

    let profile = cfg
        .profiles
        .iter_mut()
        .find(|p| p.name == name)
        .ok_or_else(|| ProfileError::NotFound(name.to_string()))?;

    profile.timezone = timezone.to_string();

    config::save(&cfg, path).map_err(ProfileError::Io)?;
    Ok(())
}
