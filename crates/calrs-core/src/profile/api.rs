//! Public API for profile management.

use super::config::{self, ProfileEntry};
use super::error::ProfileError;

/// Lists all known profiles.
pub fn list_profiles() -> Result<Vec<String>, ProfileError> {
    let cfg = config::load().map_err(ProfileError::Io)?;
    Ok(cfg.profiles.into_iter().map(|p| p.name).collect())
}

/// Returns the name of the currently active profile.
pub fn active_profile() -> Result<String, ProfileError> {
    let cfg = config::load().map_err(ProfileError::Io)?;
    if cfg.profiles.is_empty() {
        return Err(ProfileError::NoProfiles);
    }
    Ok(cfg.active_profile)
}

/// Returns true if a profile with the given name exists.
pub fn profile_exists(name: &str) -> bool {
    match config::load() {
        Ok(cfg) => cfg.profiles.iter().any(|p| p.name == name),
        Err(_) => false,
    }
}

/// Creates a new profile.
///
/// If this is the first profile, it becomes the active one automatically.
pub fn create_profile(name: &str) -> Result<(), ProfileError> {
    let mut cfg = config::load().map_err(ProfileError::Io)?;

    if cfg.profiles.iter().any(|p| p.name == name) {
        return Err(ProfileError::AlreadyExists(name.to_string()));
    }

    cfg.profiles.push(ProfileEntry {
        name: name.to_string(),
    });

    if cfg.active_profile.is_empty() {
        cfg.active_profile = name.to_string();
    }

    config::save(&cfg).map_err(ProfileError::Io)?;
    Ok(())
}

/// Deletes a profile from the configuration.
///
/// If the deleted profile was active, the active profile becomes the first
/// remaining one (or empty if no profiles remain).
/// Note: this does NOT delete the underlying `.db` file.
pub fn delete_profile(name: &str) -> Result<(), ProfileError> {
    let mut cfg = config::load().map_err(ProfileError::Io)?;

    let index = cfg.profiles.iter().position(|p| p.name == name)
        .ok_or_else(|| ProfileError::NotFound(name.to_string()))?;

    cfg.profiles.remove(index);

    if cfg.active_profile == name {
        cfg.active_profile = cfg.profiles.first()
            .map(|p| p.name.clone())
            .unwrap_or_default();
    }

    config::save(&cfg).map_err(ProfileError::Io)?;
    Ok(())
}

/// Selects an existing profile as the active one.
pub fn select_profile(name: &str) -> Result<(), ProfileError> {
    let mut cfg = config::load().map_err(ProfileError::Io)?;

    if !cfg.profiles.iter().any(|p| p.name == name) {
        return Err(ProfileError::NotFound(name.to_string()));
    }

    cfg.active_profile = name.to_string();
    config::save(&cfg).map_err(ProfileError::Io)?;
    Ok(())
}