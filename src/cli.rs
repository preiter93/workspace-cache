use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "workspace-cache")]
#[command(about = "Optimizes dependency caching for Rust workspaces")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Generate minimal workspace for dependency caching
    Deps,
    /// Build the real workspace
    Build {
        #[arg(long)]
        release: bool,
    },
}

pub fn parse() -> Cli {
    Cli::parse()
}
