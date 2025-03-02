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
    Deps {
        /// Only include dependencies for specific package(s)
        #[arg(short, long)]
        package: Vec<String>,
    },
    /// Build the real workspace
    Build {
        #[arg(long)]
        release: bool,
        /// Only build specific package(s)
        #[arg(short, long)]
        package: Vec<String>,
    },
    /// Show which workspace members a package depends on
    DepsOf {
        /// Package(s) to analyze
        #[arg(short, long, required = true)]
        package: Vec<String>,
    },
}

pub fn parse() -> Cli {
    Cli::parse()
}
