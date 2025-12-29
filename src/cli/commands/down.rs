use crate::cli::args::DownArgs;
use crate::error::{CargoJamError, Result};
use crate::toolchain::config::ToolchainConfig;
use console::style;
use std::fs;

const PID_FILE: &str = "testnet.pid";

pub fn execute(args: DownArgs) -> Result<()> {
    let home_dir = ToolchainConfig::home_dir()?;
    let pid_file = home_dir.join(PID_FILE);

    if !pid_file.exists() {
        println!("{} No testnet is currently running", style("→").cyan());
        return Ok(());
    }

    let pid_str = fs::read_to_string(&pid_file)?;
    let pid: i32 = pid_str
        .trim()
        .parse()
        .map_err(|_| CargoJamError::Build("Invalid PID in testnet.pid file".to_string()))?;

    if !is_process_running(pid) {
        // Process not running, clean up stale PID file
        fs::remove_file(&pid_file)?;
        println!(
            "{} Testnet was not running (cleaned up stale PID file)",
            style("→").cyan()
        );
        return Ok(());
    }

    println!(
        "{} Stopping JAM testnet (PID: {})...",
        style("→").cyan(),
        style(pid).yellow()
    );

    // Kill the process
    let signal = if args.force { "KILL" } else { "TERM" };

    if kill_process(pid, signal) {
        // Wait a moment for process to terminate
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Clean up PID file
        fs::remove_file(&pid_file)?;

        println!("{} Testnet stopped", style("✓").green().bold());
    } else {
        return Err(CargoJamError::Build(format!(
            "Failed to stop testnet (PID: {}). Try 'cargo polkajam down --force'",
            pid
        )));
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

#[cfg(unix)]
fn kill_process(pid: i32, signal: &str) -> bool {
    use std::process::Command;
    let sig = if signal == "KILL" { "-9" } else { "-15" };
    Command::new("kill")
        .args([sig, &pid.to_string()])
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

#[cfg(windows)]
fn kill_process(pid: i32, signal: &str) -> bool {
    use std::process::Command;
    let args = if signal == "KILL" {
        vec!["/F", "/PID", &pid.to_string()]
    } else {
        vec!["/PID", &pid.to_string()]
    };
    Command::new("taskkill")
        .args(&args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
