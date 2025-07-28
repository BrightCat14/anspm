use clap::{Parser, Subcommand};
use colored::Colorize;

#[derive(Parser)]
#[command(name = "anspm")]
#[command(version, about = "Akaruineko's Package Manager", long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install a package
    Install { name: String },
    /// Remove a package
    Remove { name: String },
    /// Update a package
    Update {
        #[arg(short, long)]
        only: Option<String>,  // --only package_name
    },
    /// Reinstall a package
    Reinstall { name: String },
    /// List installed packages
    List,
    /// Search for packages
    Search { query: String },
    /// Clean cache
    CleanCache,
    /// Repository operations
    #[command(subcommand)]
    Repo(RepoCommands),

    #[clap(hide = true)]  // <-- easter egg, because why not?
    Why,
}

#[derive(Subcommand)]
pub enum RepoCommands {
    /// Add a repository
    Add {
        url: String,
        #[arg(short, long, help = "Custom repository name")]
        name: Option<String>
    },
    /// Remove a repository
    Remove {
        name: String
    },
    /// List all repositories
    List,
    /// Update GPG keys
    UpdateKeys,
    /// Verify repository signature
    Verify {
        name: String
    },
}


pub fn print_error(message: &str) {
    eprintln!("{}: {}", "ERROR".red().bold(), message);
}

pub fn print_success(message: &str) {
    println!("{}: {}", "SUCCESS".green().bold(), message);
}

pub fn print_info(message: &str) {
    println!("{}: {}", "INFO".blue().bold(), message);
}
