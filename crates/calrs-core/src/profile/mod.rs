//! Profile management for calrs.
//!
//! Each profile is an isolated SQLite database, allowing multiple
//! independent calendars (e.g., personal vs work) on the same machine.

pub mod api;
pub mod config;
pub mod error;

// re-export for convenience: users can write `profile::list_profiles()`
// instead of `profile::api::list_profiles()`
pub use api::*;
pub use error::ProfileError;