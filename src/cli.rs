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
        /// Only include dependencies for specific binary/binaries
        #[arg(long)]
        bin: Vec<String>,
        /// Skip fetching dependencies (faster, but less optimal caching)
        #[arg(long)]
        no_deps: bool,
    },
    /// Build the real workspace
    Build {
        #[arg(long)]
        release: bool,
        /// Only build specific binary/binaries
        #[arg(long)]
        bin: Vec<String>,
    },
    /// Resolve which workspace members a binary depends on
    Resolve {
        /// Binary/binaries to analyze
        #[arg(long, required = true)]
        bin: Vec<String>,
    },
    /// Generate a Dockerfile for a binary
    Dockerfile {
        /// Binary to build
        #[arg(long, required = true)]
        bin: String,
        /// Base image for build stages
        #[arg(long, default_value = "rust:1.94-bookworm")]
        base_image: String,
        /// Runtime image
        #[arg(long, default_value = "debian:bookworm-slim")]
        runtime_image: String,
        /// Output path (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
        /// Skip fetching dependencies (faster, but less optimal caching)
        #[arg(long)]
        no_deps: bool,
    },
}

pub fn parse() -> Cli {
    Cli::parse()
}
