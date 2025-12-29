use crate::cli::args::UpArgs;
use crate::error::{CargoJamError, Result};
use crate::toolchain::config::ToolchainConfig;
use console::style;
use std::fs;
use std::process::{Command, Stdio};

const PID_FILE: &str = "testnet.pid";

pub fn execute(args: UpArgs) -> Result<()> {
    // Check toolchain is installed
    let config = ToolchainConfig::load()?;
    if !config.is_installed() {
        return Err(CargoJamError::ToolchainMissing {
            tool: "JAM toolchain".to_string(),
            install_hint: "Run 'cargo polkajam setup' to install the JAM toolchain".to_string(),
        });
    }

    let testnet_bin = ToolchainConfig::binary_path("polkajam-testnet")?.ok_or_else(|| {
        CargoJamError::ToolchainMissing {
            tool: "polkajam-testnet".to_string(),
            install_hint: "Run 'cargo polkajam setup --force' to reinstall the toolchain"
                .to_string(),
        }
    })?;

    // Check if already running
    let home_dir = ToolchainConfig::home_dir()?;
    let pid_file = home_dir.join(PID_FILE);

    if pid_file.exists() {
        let pid_str = fs::read_to_string(&pid_file)?;
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            // Check if process is still running
            if is_process_running(pid) {
                println!(
                    "{} Testnet is already running (PID: {})",
                    style("→").cyan(),
                    style(pid).yellow()
                );
                println!("  RPC endpoint: {}", style("ws://localhost:19800").green());
                println!("\n  Stop with: {}", style("cargo polkajam down").cyan());
                return Ok(());
            }
        }
        // Stale PID file, remove it
        fs::remove_file(&pid_file)?;
    }

    if args.foreground {
        // Run in foreground
        println!(
            "{} Starting JAM testnet in foreground...",
            style("→").cyan()
        );
        println!("  RPC endpoint: {}", style(&args.rpc).green());
        println!("  Press Ctrl+C to stop\n");

        let status = Command::new(&testnet_bin)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| CargoJamError::Build(format!("Failed to start testnet: {}", e)))?;

        if !status.success() {
            return Err(CargoJamError::Build(
                "Testnet exited with error".to_string(),
            ));
        }
    } else {
        // Run in background
        println!(
            "{} Starting JAM testnet in background...",
            style("→").cyan()
        );

        let child = Command::new(&testnet_bin)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| CargoJamError::Build(format!("Failed to start testnet: {}", e)))?;

        let pid = child.id();

        // Save PID to file
        fs::write(&pid_file, pid.to_string())?;

        println!(
            "{} Testnet started (PID: {})",
            style("✓").green().bold(),
            style(pid).yellow()
        );
        println!("  RPC endpoint: {}", style("ws://localhost:19800").green());
        println!("\n  Stop with: {}", style("cargo polkajam down").cyan());
        println!(
            "  View logs: {}",
            style("cargo polkajam up --foreground").dim()
        );
    }

    Ok(())
}

#[cfg(unix)]
fn is_process_running(pid: i32) -> bool {
    use std::process::Command;
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(windows)]
fn is_process_running(pid: i32) -> bool {
    use std::process::Command;
    Command::new("tasklist")
        .args(["/FI", &format!("PID eq {}", pid)])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
        .unwrap_or(false)
}
