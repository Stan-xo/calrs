
mod commands;
mod cli;

use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // resolve profile name — flag overrides active profile
    let profile = match &cli.profile {
        Some(p) => p.clone(),
        None => match calrs_core::profile::active_profile() {
            Ok(p) => p,
            Err(calrs_core::profile::ProfileError::NoProfiles) => {
                eprintln!("No profile found. Create one with: calrs profile create <name>");
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
    };

    match cli.command {
        Commands::Profile { action } => commands::profile::handle(action).await,
        _ => {
            // open database for all other commands
            let pool = calrs_core::storage::sqlite::open_database(&profile)
                .await
                .expect("Failed to open database");
            calrs_core::storage::sqlite::init_database(&pool)
                .await
                .expect("Failed to init database");

            match cli.command {
                Commands::List { kind, status } => {
                    commands::list::handle(&pool, kind, status).await
                }
                Commands::Add { kind } => commands::add::handle(&pool, kind).await,
                Commands::Show { id } => commands::show::handle(&pool, id).await,
                Commands::Delete { id } => commands::delete::handle(&pool, id).await,
                _ => unreachable!(),
            }
        }
    }
}