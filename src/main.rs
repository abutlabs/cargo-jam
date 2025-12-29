use anyhow::Result;
use cargo_polkajam::cli::args::{Cargo, PolkajamCommand};
use cargo_polkajam::cli::commands;
use clap::Parser;
use console::style;

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", style("error:").red().bold(), e);

        // Print cause chain if available
        let mut source = e.source();
        while let Some(cause) = source {
            eprintln!("  {} {}", style("caused by:").yellow(), cause);
            source = cause.source();
        }

        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let Cargo::Polkajam(args) = Cargo::parse();

    match args.command {
        PolkajamCommand::New(new_args) => {
            commands::new::execute(new_args)?;
        }
        PolkajamCommand::Build(build_args) => {
            commands::build::execute(build_args)?;
        }
        PolkajamCommand::Setup(setup_args) => {
            commands::setup::execute(setup_args)?;
        }
        PolkajamCommand::Up(up_args) => {
            commands::up::execute(up_args)?;
        }
        PolkajamCommand::Down(down_args) => {
            commands::down::execute(down_args)?;
        }
        PolkajamCommand::Deploy(deploy_args) => {
            commands::deploy::execute(deploy_args)?;
        }
        PolkajamCommand::Monitor(monitor_args) => {
            commands::monitor::execute(monitor_args)?;
        }
        PolkajamCommand::Test(test_args) => {
            commands::test::execute(test_args)?;
        }
    }

    Ok(())
}
