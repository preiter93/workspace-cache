use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "workspace-cache")]
#[command(version)]
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
        /// Output directory for the generated workspace (default: .workspace-cache)
        #[arg(short, long)]
        output: Option<String>,
        /// Fast mode: skip dependency resolution (faster, but less optimal caching)
        #[arg(long)]
        fast: bool,
    },
    /// Build the real workspace
    Build {
        #[arg(long)]
        release: bool,
        /// Only build specific binary/binaries
        #[arg(long)]
        bin: Vec<String>,
    },
    /// Show workspace members a binary depends on
    Members {
        /// Binary/binaries to analyze
        #[arg(long, required = true)]
        bin: Vec<String>,
    },
    /// Generate a Dockerfile for a binary
    Dockerfile {
        /// Binary to build
        #[arg(long, required = true)]
        bin: String,
        /// Build profile (release or debug)
        #[arg(long, default_value = "release")]
        profile: String,
        /// Install workspace-cache from git instead of crates.io
        #[arg(long)]
        from_git: bool,
        /// Version of workspace-cache to install (default: current version)
        #[arg(long)]
        tool_version: Option<String>,
        /// Base image for build stages
        #[arg(long, default_value = "rust:1.94-bookworm")]
        base_image: String,
        /// Runtime image
        #[arg(long, default_value = "debian:bookworm-slim")]
        runtime_image: String,
        /// Output path (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
        /// Fast mode: skip dependency resolution (faster, but less optimal caching)
        #[arg(long)]
        fast: bool,
    },
}

pub fn parse() -> Cli {
    Cli::parse()
}
