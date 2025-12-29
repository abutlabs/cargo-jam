use crate::cli::args::SetupArgs;
use crate::error::Result;
use crate::toolchain::config::ToolchainConfig;
use crate::toolchain::download::{
    download_and_install, fetch_releases, get_latest_release, get_release,
};
use crate::toolchain::platform::Platform;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

pub fn execute(args: SetupArgs) -> Result<()> {
    // Handle --info flag
    if args.info {
        return show_info();
    }

    // Handle --list flag
    if args.list {
        return list_releases();
    }

    // Detect platform
    let platform = Platform::detect()?;
    println!(
        "{} Detected platform: {}",
        style("→").cyan(),
        style(platform.to_string()).yellow()
    );

    // Get the release to install
    let release = if let Some(ref version) = args.version {
        println!(
            "{} Fetching release {}...",
            style("→").cyan(),
            style(version).yellow()
        );
        get_release(version)?
    } else {
        println!("{} Fetching latest nightly release...", style("→").cyan());
        get_latest_release()?
    };

    println!(
        "{} Found release: {}",
        style("→").cyan(),
        style(&release.tag_name).green()
    );

    // Check if already installed (unless --force or --update)
    let config = ToolchainConfig::load()?;
    if config.is_installed() && !args.force && !args.update {
        if let Some(ref installed) = config.installed_version {
            if installed == &release.tag_name {
                println!(
                    "\n{} Toolchain {} is already installed at {}",
                    style("✓").green().bold(),
                    style(&release.tag_name).cyan(),
                    style(config.toolchain_path.unwrap().display()).yellow()
                );
                println!(
                    "\nUse {} to reinstall or {} to update to latest.",
                    style("--force").cyan(),
                    style("--update").cyan()
                );
                return Ok(());
            }
        }
    }

    // Create progress spinner
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    // Download and install
    spinner.set_message(format!("Downloading {}...", release.tag_name));
    let install_path = download_and_install(&release, &platform, args.force)?;
    spinner.finish_and_clear();

    println!(
        "\n{} Installed JAM toolchain {} to {}",
        style("✓").green().bold(),
        style(&release.tag_name).cyan(),
        style(install_path.display()).yellow()
    );

    // List installed binaries from the normalized polkajam-nightly directory
    let nightly_dir = install_path.join("polkajam-nightly");
    if nightly_dir.exists() {
        println!("\n{}", style("Installed binaries:").bold());
        if let Ok(bin_entries) = std::fs::read_dir(&nightly_dir) {
            for bin_entry in bin_entries.flatten() {
                let bin_path = bin_entry.path();
                if bin_path.is_file()
                    && !bin_path
                        .extension()
                        .map(|e| e == "md" || e == "txt" || e == "corevm")
                        .unwrap_or(false)
                {
                    let bin_name = bin_path.file_name().unwrap().to_string_lossy();
                    println!("  {} {}", style("✓").green(), bin_name);
                }
            }
        }
    }

    println!(
        "\n{} You can now use {}",
        style("→").cyan(),
        style("cargo polkajam build").green()
    );

    Ok(())
}

fn show_info() -> Result<()> {
    let config = ToolchainConfig::load()?;

    println!("{}", style("JAM Toolchain Info").bold());
    println!();

    if config.is_installed() {
        println!(
            "  {} {}",
            style("Version:").dim(),
            style(config.installed_version.as_deref().unwrap_or("unknown")).green()
        );
        println!(
            "  {} {}",
            style("Location:").dim(),
            style(
                config
                    .toolchain_path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default()
            )
            .yellow()
        );
        if let Some(ref ts) = config.installed_at {
            println!("  {} {}", style("Installed:").dim(), ts);
        }

        // List available binaries
        if let Ok(Some(polkajam_dir)) = ToolchainConfig::polkajam_dir() {
            println!("\n{}", style("Available binaries:").bold());
            if let Ok(entries) = std::fs::read_dir(&polkajam_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file()
                        && !path
                            .extension()
                            .map(|e| e == "md" || e == "txt" || e == "corevm")
                            .unwrap_or(false)
                    {
                        let name = path.file_name().unwrap().to_string_lossy();
                        println!("  • {}", name);
                    }
                }
            }
        }

        // Check for jam-pvm-build
        println!("\n{}", style("Build tools:").bold());
        let jam_build_check = std::process::Command::new("jam-pvm-build")
            .arg("--version")
            .output();

        if let Ok(output) = jam_build_check {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout);
                println!(
                    "  {} jam-pvm-build {}",
                    style("✓").green(),
                    style(version.trim()).dim()
                );
            } else {
                println!("  {} jam-pvm-build (not installed)", style("✗").red());
                println!(
                    "    Install with: {}",
                    style("cargo install jam-pvm-build").cyan()
                );
            }
        } else {
            println!("  {} jam-pvm-build (not installed)", style("✗").red());
            println!(
                "    Install with: {}",
                style("cargo install jam-pvm-build").cyan()
            );
        }
    } else {
        println!("  {} No toolchain installed", style("⚠").yellow());
        println!(
            "\n  Run {} to install the latest nightly.",
            style("cargo polkajam setup").cyan()
        );
    }

    Ok(())
}

fn list_releases() -> Result<()> {
    println!("{} Fetching available releases...\n", style("→").cyan());

    let releases = fetch_releases(10)?;
    let config = ToolchainConfig::load()?;
    let installed = config.installed_version.as_deref();

    println!("{}", style("Available releases:").bold());
    for release in releases {
        let is_installed = installed == Some(release.tag_name.as_str());
        let marker = if is_installed {
            style("(installed)").green()
        } else {
            style("").dim()
        };

        println!(
            "  {} {} {}",
            if is_installed {
                style("✓").green()
            } else {
                style("•").dim()
            },
            style(&release.tag_name).cyan(),
            marker
        );
    }

    println!(
        "\nInstall a specific version with: {}",
        style("cargo polkajam setup --version <tag>").cyan()
    );

    Ok(())
}
