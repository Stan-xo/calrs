use clap::{Parser, Subcommand};
use crate::commands::add::AddKind;
use crate::commands::profile::ProfileAction;

#[derive(Parser)]
#[command(name = "calrs")]
#[command(about = "A personal calendar and task manager")]
pub struct Cli {
    /// Profile to use for this command
    #[arg(long, global = true, short)]
    pub profile: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new item
    Add {
        #[command(subcommand)]
        kind: AddKind,
    },
    /// Show items for a specific date
    Date { date: String },
    /// Delete an item by id
    Delete { id: u64 },
    /// List calendar items
    List {
        #[arg(long, short)]
        kind: Option<String>,
        #[arg(long, short = 'S')]
        status: Option<String>,
        #[arg(long, short)]
        state: Option<String>,
    },
    /// Manage profiles
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
    /// Show items between two dates
    Range { start: String, end: String },
    /// Search items by title
    Search { query: String },
    /// Show a single item by id
    Show { id: u64 },
    /// Show items for today
    Today,
    /// Update an item by id
    Update {
        id: u64,
        #[arg(long, short)]
        title: Option<String>,
        #[arg(long, short = 'z')]
        timezone: Option<String>,
        #[arg(long, short = 'S')]
        status: Option<String>,
        #[arg(long, short)]
        description: Option<String>,
        #[arg(long, short)]
        place: Option<String>,
        #[arg(long, short)]
        icon: Option<String>,
        #[arg(long, short = 'r')]
        color_r: Option<u8>,
        #[arg(long, short = 'g')]
        color_g: Option<u8>,
        #[arg(long, short = 'b')]
        color_b: Option<u8>,
        #[arg(long, short = 'a')]
        color_a: Option<u8>,
    },
    /// Show items for the current week
    Week,
}