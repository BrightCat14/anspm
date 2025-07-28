mod cli;
mod config;
mod pkg;
mod repo;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let args = cli::Args::parse();
    match args.command {
        cli::Commands::Install { name } => pkg::install(&name, true),
        cli::Commands::Remove { name } => pkg::remove(&name),
        cli::Commands::Reinstall { name } => pkg::reinstall(&name),
        cli::Commands::List => pkg::list(),
        cli::Commands::Search { query } => repo::search(&query),
        cli::Commands::Update { only } => pkg::update(only.as_deref()),
        cli::Commands::CleanCache => pkg::clean_cache(),
        cli::Commands::Repo(subcmd) => match subcmd {
            cli::RepoCommands::Add { url, name, key } => repo::repo_add(&url, name.as_deref(), key.as_deref()),
            cli::RepoCommands::Remove { name } => repo::repo_remove(&name),
            cli::RepoCommands::List => repo::repo_list(),
            cli::RepoCommands::UpdateKeys => repo::repo_update_keys(),
            cli::RepoCommands::Verify { name } => repo::repo_verify(&name),
        },
    }
}
