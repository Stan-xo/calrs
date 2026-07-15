use clap::Subcommand;

#[derive(Subcommand)]
pub enum ProfileAction {
    /// List all profiles (* on active)
    List,
    /// Create a new profile
    Create {
        name: String,
        #[arg(long, default_value = "UTC")]
        timezone: String,
    },
    /// Delete a profile
    Delete {
        name: String,
        /// Also delete the underlying database file
        #[arg(long)]
        purge: bool,
    },
    /// Select a profile as active
    Select { name: String },
    /// Reset a profile database
    Reset { name: String },
    /// Show or update the timezone of the active profile
    Timezone {
        /// New timezone to set (e.g. "Europe/Paris"). If omitted, shows current.
        zone: Option<String>,
        /// Also update timezone on all existing items
        #[arg(long)]
        all: bool,
    },
}