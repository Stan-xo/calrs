use clap::Subcommand;

#[derive(Subcommand)]
pub enum ProfileAction {
    /// List all profiles
    List,
    /// Create a new profile
    Create { name: String },
    /// Delete a profile
    Delete { name: String },
    /// Select a profile as active
    Select { name: String },
}
