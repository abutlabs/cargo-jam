use anyhow::Result;
use cargo_jam::cli::args::{Cargo, JamCommand};
use cargo_jam::cli::commands;
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
    let Cargo::Jam(args) = Cargo::parse();

    match args.command {
        JamCommand::New(new_args) => {
            commands::new::execute(new_args)?;
        }
        JamCommand::Build(build_args) => {
            commands::build::execute(build_args)?;
        }
        JamCommand::Setup(setup_args) => {
            commands::setup::execute(setup_args)?;
        }
        JamCommand::Up(up_args) => {
            commands::up::execute(up_args)?;
        }
        JamCommand::Down(down_args) => {
            commands::down::execute(down_args)?;
        }
        JamCommand::Deploy(deploy_args) => {
            commands::deploy::execute(deploy_args)?;
        }
        JamCommand::Monitor(monitor_args) => {
            commands::monitor::execute(monitor_args)?;
        }
        JamCommand::Test(test_args) => {
            commands::test::execute(test_args)?;
        }
    }

    Ok(())
}
