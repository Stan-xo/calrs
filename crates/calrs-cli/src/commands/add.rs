use clap::Subcommand;

#[derive(Subcommand)]
pub enum AddKind {
    /// Add an event
    Event {
        title: String,
        #[arg(long)]
        start: String,
        #[arg(long)]
        end: String,
        #[arg(long)]
        full_day: bool,
    },
    /// Add a task
    Task {
        title: String,
        #[arg(long)]
        deadline: Option<String>,
    },
}