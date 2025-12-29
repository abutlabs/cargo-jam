use crate::cli::args::TestArgs;
use crate::error::{CargoJamError, Result};
use crate::toolchain::config::ToolchainConfig;
use console::style;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

const TEST_SERVICE_NAME: &str = "jam-test-service";

pub fn execute(args: TestArgs) -> Result<()> {
    println!(
        "\n{} Running cargo-jam end-to-end tests\n",
        style("🧪").bold()
    );

    // Check toolchain is installed
    let config = ToolchainConfig::load()?;
    if !config.is_installed() {
        return Err(CargoJamError::ToolchainMissing {
            tool: "JAM toolchain".to_string(),
            install_hint: "Run 'cargo jam setup' to install the JAM toolchain".to_string(),
        });
    }

    // Create test directory
    let test_dir = args
        .dir
        .clone()
        .unwrap_or_else(|| std::env::temp_dir().join("cargo-jam-test"));

    // Clean up previous test if exists
    if test_dir.exists() {
        fs::remove_dir_all(&test_dir)?;
    }
    fs::create_dir_all(&test_dir)?;

    let service_dir = test_dir.join(TEST_SERVICE_NAME);

    // Track test results
    let mut passed = 0;
    let mut failed = 0;
    let start_time = Instant::now();

    // Test 1: Create new service
    print_test_header("1", "Create new JAM service");
    match run_cargo_jam(
        &["new", TEST_SERVICE_NAME, "--defaults"],
        Some(&test_dir),
        args.verbose,
    ) {
        Ok(output) => {
            if service_dir.exists() && service_dir.join("Cargo.toml").exists() {
                print_test_pass("Service created successfully");
                if args.verbose {
                    println!("{}", output);
                }
                passed += 1;
            } else {
                print_test_fail("Service directory not created");
                failed += 1;
            }
        }
        Err(e) => {
            print_test_fail(&format!("Failed to create service: {}", e));
            failed += 1;
        }
    }

    // Test 2: Build service
    print_test_header("2", "Build JAM service");
    let jam_file = service_dir.join(format!("{}.jam", TEST_SERVICE_NAME));
    match run_cargo_jam(&["build"], Some(&service_dir), args.verbose) {
        Ok(output) => {
            if jam_file.exists() {
                let size = fs::metadata(&jam_file).map(|m| m.len()).unwrap_or(0);
                print_test_pass(&format!("Built {} ({} bytes)", jam_file.display(), size));
                if args.verbose {
                    println!("{}", output);
                }
                passed += 1;
            } else {
                print_test_fail("JAM blob not created");
                println!("Expected: {}", jam_file.display());
                failed += 1;
            }
        }
        Err(e) => {
            print_test_fail(&format!("Failed to build: {}", e));
            failed += 1;
        }
    }

    // Test 3: Deploy to local testnet (start, deploy, stop)
    if !args.skip_testnet {
        print_test_header("3", "Deploy to local testnet");

        let mut test3_passed = true;
        let mut testnet_started = false;

        // Step 1: Start testnet
        println!("  {} Starting testnet...", style("→").cyan());
        match run_cargo_jam(&["up"], None, args.verbose) {
            Ok(output) => {
                println!("  {} Testnet started", style("✓").green());
                if args.verbose {
                    println!("{}", output);
                }
                testnet_started = true;
            }
            Err(e) => {
                if e.to_string().contains("already running") {
                    println!("  {} Testnet already running", style("✓").green());
                } else {
                    print_test_fail(&format!("Failed to start testnet: {}", e));
                    test3_passed = false;
                }
            }
        }

        // Step 2: Wait for testnet to initialize
        if test3_passed {
            println!(
                "  {} Waiting for testnet to initialize...",
                style("→").cyan()
            );
            std::thread::sleep(Duration::from_secs(3));
        }

        // Step 3: Deploy service
        if test3_passed {
            println!("  {} Deploying service...", style("→").cyan());
            match run_cargo_jam(&["deploy", jam_file.to_str().unwrap()], None, args.verbose) {
                Ok(output) => {
                    if output.contains("deployed successfully")
                        || output.contains("created at slot")
                    {
                        println!("  {} Service deployed", style("✓").green());
                        if let Some(line) = output
                            .lines()
                            .find(|l| l.contains("Service") && l.contains("created"))
                        {
                            println!("    {}", style(line.trim()).dim());
                        }
                    } else {
                        print_test_fail("Deploy succeeded but output unexpected");
                        println!("{}", output);
                        test3_passed = false;
                    }
                }
                Err(e) => {
                    print_test_fail(&format!("Failed to deploy: {}", e));
                    test3_passed = false;
                }
            }
        }

        // Step 4: Stop testnet (cleanup)
        if testnet_started && !args.keep_running {
            println!("  {} Stopping testnet...", style("→").cyan());
            match run_cargo_jam(&["down"], None, args.verbose) {
                Ok(_) => {
                    println!("  {} Testnet stopped", style("✓").green());
                }
                Err(e) => {
                    println!("  {} Failed to stop testnet: {}", style("!").yellow(), e);
                }
            }
        } else if args.keep_running {
            println!(
                "  {} Testnet left running (--keep-running)",
                style("→").cyan()
            );
        }

        if test3_passed {
            print_test_pass("Deployment complete");
            passed += 1;
        } else {
            failed += 1;
        }
    } else {
        print_test_header("3", "Deploy to local testnet (skipped)");
        println!("  {} Skipped (--skip-testnet)", style("→").cyan());
    }

    // Clean up test directory
    if !args.verbose {
        let _ = fs::remove_dir_all(&test_dir);
    } else {
        println!(
            "\n  {} Test artifacts at: {}",
            style("→").cyan(),
            test_dir.display()
        );
    }

    // Print summary
    let elapsed = start_time.elapsed();
    println!("\n{}", style("─".repeat(50)).dim());
    println!(
        "\n{} Test Results: {} passed, {} failed (in {:.1}s)\n",
        if failed == 0 {
            style("✓").green().bold()
        } else {
            style("✗").red().bold()
        },
        style(passed).green(),
        if failed > 0 {
            style(failed).red()
        } else {
            style(failed).dim()
        },
        elapsed.as_secs_f32()
    );

    if failed > 0 {
        return Err(CargoJamError::Build(format!("{} test(s) failed", failed)));
    }

    Ok(())
}

fn run_cargo_jam(args: &[&str], cwd: Option<&PathBuf>, verbose: bool) -> Result<String> {
    let cargo_jam = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("cargo-jam")))
        .unwrap_or_else(|| PathBuf::from("cargo-jam"));

    let mut cmd = Command::new(&cargo_jam);
    cmd.arg("jam");
    cmd.args(args);

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    if verbose {
        println!(
            "  {} {:?} jam {}",
            style("$").dim(),
            cargo_jam,
            args.join(" ")
        );
    }

    let output = cmd
        .output()
        .map_err(|e| CargoJamError::Build(format!("Failed to execute cargo-jam: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(CargoJamError::Build(format!(
            "Command failed: {}\n{}",
            stderr, stdout
        )));
    }

    Ok(format!("{}{}", stdout, stderr))
}

fn print_test_header(num: &str, name: &str) {
    println!(
        "\n{} Test {}: {}",
        style("▶").cyan(),
        style(num).bold(),
        name
    );
}

fn print_test_pass(msg: &str) {
    println!("  {} {}", style("✓").green().bold(), msg);
}

fn print_test_fail(msg: &str) {
    println!("  {} {}", style("✗").red().bold(), msg);
}
