use crate::cli::args::DeployArgs;
use crate::error::{CargoJamError, Result};
use crate::toolchain::config::ToolchainConfig;
use console::style;
use std::process::Command;

pub fn execute(args: DeployArgs) -> Result<()> {
    // Check toolchain is installed
    let config = ToolchainConfig::load()?;
    if !config.is_installed() {
        return Err(CargoJamError::ToolchainMissing {
            tool: "JAM toolchain".to_string(),
            install_hint: "Run 'cargo jam setup' to install the JAM toolchain".to_string(),
        });
    }

    let jamt_bin =
        ToolchainConfig::binary_path("jamt")?.ok_or_else(|| CargoJamError::ToolchainMissing {
            tool: "jamt".to_string(),
            install_hint: "Run 'cargo jam setup --force' to reinstall the toolchain".to_string(),
        })?;

    // Verify the .jam file exists
    if !args.code.exists() {
        return Err(CargoJamError::Build(format!(
            "Service blob not found: {}",
            args.code.display()
        )));
    }

    // Verify it's a .jam file
    if args.code.extension().map(|e| e != "jam").unwrap_or(true) {
        return Err(CargoJamError::Build(format!(
            "Expected a .jam file, got: {}",
            args.code.display()
        )));
    }

    println!(
        "{} Deploying service: {}",
        style("→").cyan(),
        style(args.code.display()).yellow()
    );

    if args.verbose {
        println!("  RPC: {}", style(&args.rpc).dim());
        println!("  Amount: {}", args.amount);
        println!("  Min item gas: {}", args.min_item_gas);
        println!("  Min memo gas: {}", args.min_memo_gas);
    }

    // Build jamt command
    // Note: --rpc is a global option and must come BEFORE the subcommand
    let mut cmd = Command::new(&jamt_bin);
    cmd.arg("--rpc").arg(&args.rpc);
    cmd.arg("create-service");
    cmd.arg(&args.code);
    cmd.arg(&args.amount);

    if !args.memo.is_empty() {
        cmd.arg(&args.memo);
    }

    cmd.arg("--min-item-gas").arg(&args.min_item_gas);
    cmd.arg("--min-memo-gas").arg(&args.min_memo_gas);

    if let Some(ref register) = args.register {
        cmd.arg("--register").arg(register);
    }

    let output = cmd
        .output()
        .map_err(|e| CargoJamError::Build(format!("Failed to execute jamt: {}", e)))?;

    // Print output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !stdout.is_empty() {
        println!("{}", stdout);
    }

    if !output.status.success() {
        if !stderr.is_empty() {
            eprintln!("{}", stderr);
        }
        return Err(CargoJamError::Build(format!(
            "Deployment failed with status: {}",
            output.status
        )));
    }

    println!(
        "\n{} Service deployed successfully!",
        style("✓").green().bold()
    );

    Ok(())
}
