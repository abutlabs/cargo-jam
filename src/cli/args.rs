use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// cargo-jam: Generate JAM service projects for Polkadot
#[derive(Parser, Debug)]
#[command(name = "cargo", bin_name = "cargo")]
pub enum Cargo {
    /// JAM service generation and build tools
    Jam(JamArgs),
}

#[derive(Parser, Debug)]
#[command(version, about = "Generate JAM service projects for Polkadot")]
pub struct JamArgs {
    #[command(subcommand)]
    pub command: JamCommand,
}

#[derive(Subcommand, Debug)]
pub enum JamCommand {
    /// Create a new JAM service project
    New(NewArgs),

    /// Build a JAM service for PVM deployment
    Build(BuildArgs),
}

#[derive(Parser, Debug)]
pub struct NewArgs {
    /// Name of the new JAM service project
    pub name: Option<String>,

    /// Template to use (default: basic-service)
    #[arg(short, long, default_value = "basic-service")]
    pub template: String,

    /// Use a git repository as template source
    #[arg(long, conflicts_with = "template")]
    pub git: Option<String>,

    /// Git branch to use (requires --git)
    #[arg(long, requires = "git")]
    pub branch: Option<String>,

    /// Subdirectory within git repo containing template
    #[arg(long, requires = "git")]
    pub path: Option<PathBuf>,

    /// Output directory (default: ./<name>)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Skip interactive prompts, use defaults
    #[arg(long)]
    pub defaults: bool,

    /// Define template variables (key=value)
    #[arg(short, long, value_name = "KEY=VALUE")]
    pub define: Vec<String>,

    /// Template values file (TOML format)
    #[arg(long)]
    pub values_file: Option<PathBuf>,

    /// Don't initialize git repository
    #[arg(long)]
    pub no_git: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Parser, Debug)]
pub struct BuildArgs {
    /// Path to the JAM service project (default: current directory)
    #[arg(short, long)]
    pub path: Option<PathBuf>,

    /// Build in release mode (default: true)
    #[arg(long, default_value = "true")]
    pub release: bool,

    /// Output path for the .jam blob
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Skip polkatool linking (cargo build only)
    #[arg(long)]
    pub no_link: bool,

    /// Additional cargo build arguments
    #[arg(last = true)]
    pub cargo_args: Vec<String>,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}
