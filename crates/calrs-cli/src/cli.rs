use clap::{Parser, Subcommand};
use crate::commands::add::AddKind;
use crate::commands::profile::ProfileAction;
 

#[derive(Parser)]
#[command(name = "calrs")]
#[command(about = "A personal calendar and task manager")]
pub struct Cli {
    /// Profile to use for this command (overrides active profile)
    #[arg(long, global = true)]
    pub profile: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List calendar items
    List {
        #[arg(long)]
        kind: Option<String>,
        #[arg(long)]
        status: Option<String>,
    },
    /// Add a new item
    Add {
        #[command(subcommand)]
        kind: AddKind,
    },
    /// Show a single item by id
    Show { id: u64 },
    /// Delete an item by id
    Delete { id: u64 },
    /// Manage profiles
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
    /// Update an item
    Update {
        id: u64,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        status: Option<String>,
    },  // id + options modifiables
    /// Search items by title
    Search { key:String },  // query: String
    /// Show items for today
    Today,           // pas d'arguments
    /// Show items for the current week
    Week,            // pas d'arguments
    /// Show items for a specific date
    Date {  },    // date: String
    /// Show items between two dates
    Range {  },   // start: String, end: String
}