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

    /// Setup the JAM/PVM toolchain
    Setup(SetupArgs),

    /// Start the local JAM testnet
    Up(UpArgs),

    /// Stop the local JAM testnet
    Down(DownArgs),

    /// Deploy a JAM service to the testnet
    Deploy(DeployArgs),

    /// Monitor the testnet with jamtop
    Monitor(MonitorArgs),

    /// Run end-to-end tests
    Test(TestArgs),
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

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Parser, Debug)]
pub struct SetupArgs {
    /// Install a specific version (default: latest nightly)
    #[arg(long)]
    pub version: Option<String>,

    /// List available toolchain versions
    #[arg(long)]
    pub list: bool,

    /// Update to the latest nightly version
    #[arg(long)]
    pub update: bool,

    /// Show currently installed toolchain info
    #[arg(long)]
    pub info: bool,

    /// Force reinstall even if already installed
    #[arg(long)]
    pub force: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Parser, Debug)]
pub struct UpArgs {
    /// RPC URL for the testnet (default: ws://localhost:19800)
    #[arg(long, default_value = "ws://localhost:19800")]
    pub rpc: String,

    /// Run in foreground (default: background)
    #[arg(long)]
    pub foreground: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Parser, Debug)]
pub struct DownArgs {
    /// Force kill the testnet process
    #[arg(long)]
    pub force: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Parser, Debug)]
pub struct DeployArgs {
    /// Path to the .jam blob to deploy
    pub code: PathBuf,

    /// Initial endowment for the service
    #[arg(long, default_value = "0")]
    pub amount: String,

    /// Memo for the service endowment
    #[arg(long, default_value = "")]
    pub memo: String,

    /// Minimum accumulation gas per work-item
    #[arg(long, short = 'G', default_value = "1000000")]
    pub min_item_gas: String,

    /// Minimum on-transfer gas per memo
    #[arg(long, short = 'g', default_value = "1000000")]
    pub min_memo_gas: String,

    /// Register the service with the Bootstrap service
    #[arg(long, short)]
    pub register: Option<String>,

    /// RPC URL for the testnet
    #[arg(long, default_value = "ws://localhost:19800")]
    pub rpc: String,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Parser, Debug)]
pub struct MonitorArgs {
    /// RPC URL for the testnet
    #[arg(long, default_value = "ws://localhost:19800")]
    pub rpc: String,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Parser, Debug)]
pub struct TestArgs {
    /// Keep testnet running after tests
    #[arg(long)]
    pub keep_running: bool,

    /// Skip testnet startup (assume already running)
    #[arg(long)]
    pub skip_testnet: bool,

    /// Test directory (default: temp directory)
    #[arg(long)]
    pub dir: Option<std::path::PathBuf>,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}
