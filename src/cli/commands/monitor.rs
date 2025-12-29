use crate::cli::args::MonitorArgs;
use crate::error::{CargoJamError, Result};
use crate::toolchain::config::ToolchainConfig;
use console::style;
use std::process::{Command, Stdio};

pub fn execute(args: MonitorArgs) -> Result<()> {
    // Check toolchain is installed
    let config = ToolchainConfig::load()?;
    if !config.is_installed() {
        return Err(CargoJamError::ToolchainMissing {
            tool: "JAM toolchain".to_string(),
            install_hint: "Run 'cargo polkajam setup' to install the JAM toolchain".to_string(),
        });
    }

    let jamtop_bin =
        ToolchainConfig::binary_path("jamtop")?.ok_or_else(|| CargoJamError::ToolchainMissing {
            tool: "jamtop".to_string(),
            install_hint: "Run 'cargo polkajam setup --force' to reinstall the toolchain"
                .to_string(),
        })?;

    println!("{} Starting JAM testnet monitor...", style("→").cyan());

    if args.verbose {
        println!("  RPC: {}", style(&args.rpc).dim());
    }

    println!("  Press 'q' to quit\n");

    // Run jamtop in foreground with inherited stdio for interactive TUI
    let mut cmd = Command::new(&jamtop_bin);
    cmd.arg("--rpc").arg(&args.rpc);

    let status = cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| CargoJamError::Build(format!("Failed to start jamtop: {}", e)))?;

    if !status.success() {
        return Err(CargoJamError::Build("jamtop exited with error".to_string()));
    }

    Ok(())
}
