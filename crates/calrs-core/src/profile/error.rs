//! Error type for profile operations.

/// Errors that can occur during profile operations.
#[derive(Debug)]
pub enum ProfileError {
    /// The requested profile does not exist.
    NotFound(String),
    /// A profile with this name already exists.
    AlreadyExists(String),
    /// No profiles exist yet — user must create one first.
    NoProfiles,
    /// An IO or serialization error occurred.
    Io(String),
}

impl std::fmt::Display for ProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ProfileError::NotFound(name) => write!(f, "Profile '{}' not found", name),
            ProfileError::AlreadyExists(name) => write!(f, "Profile '{}' already exists", name),
            ProfileError::NoProfiles => write!(f, "No profiles exist. Create one first."),
            ProfileError::Io(msg) => write!(f, "{}", msg),
        }
    }
}