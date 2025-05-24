use clap::{Parser, Subcommand};
use anyhow::Result;
use colored::*;

mod commands;

#[derive(Parser)]
#[command(name = "rit")]
#[command(about = "A graphical git-like version control system")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new rit repository
    Init,
    /// Show the working tree status in a graphical format
    Status,
    /// Add file contents to the index
    Add { files: Vec<String> },
    /// Record changes to the repository
    Commit { 
        #[arg(short, long)]
        message: String 
    },
    /// Show commit logs
    Log,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Init => {
            commands::init::run()?;
        }
        Commands::Status => {
            commands::status::run()?;
        }
        Commands::Add { files } => {
            commands::add::run(files)?;
        }
        Commands::Commit { message } => {
            commands::commit::run(message)?;
        }
        Commands::Log => {
            commands::log::run()?;
        }
    }
    
    Ok(())
}
