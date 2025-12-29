//! End-to-end testnet deployment tests for cargo-polkajam
//!
//! These tests require a running local testnet.
//!
//! ## Running the tests:
//!
//! 1. Start the local testnet in one terminal:
//!    ```bash
//!    ~/.cargo-polkajam/toolchain/polkajam-nightly/polkajam-testnet
//!    ```
//!
//! 2. Run the testnet tests in another terminal:
//!    ```bash
//!    cargo test --test testnet_tests -- --ignored --nocapture
//!    ```
//!
//! Note: All testnet tests are marked as `#[ignore]` to prevent them from
//! running during regular `cargo test`. Use `--ignored` to run them.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Get the path to the cargo-polkajam binary
fn cargo_jam_bin() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push("cargo-polkajam");
    path
}

/// Get the path to jamt binary
fn jamt_bin() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let jamt = home
        .join(".cargo-polkajam")
        .join("toolchain")
        .join("polkajam-nightly")
        .join("jamt");
    if jamt.exists() {
        Some(jamt)
    } else {
        None
    }
}

/// Get the path to polkajam-testnet binary
fn testnet_bin() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let testnet = home
        .join(".cargo-polkajam")
        .join("toolchain")
        .join("polkajam-nightly")
        .join("polkajam-testnet");
    if testnet.exists() {
        Some(testnet)
    } else {
        None
    }
}

/// Create a temporary directory for tests
fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cargo-polkajam-testnet-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("Failed to create temp dir");
    dir
}

/// Clean up temporary directory
fn cleanup(dir: &PathBuf) {
    if dir.exists() {
        fs::remove_dir_all(dir).ok();
    }
}

/// Check if testnet is running by trying to connect
fn is_testnet_running() -> bool {
    // Try to run jamt with a simple command
    if let Some(jamt) = jamt_bin() {
        let output = Command::new(&jamt).args(["queue"]).output();

        match output {
            Ok(o) => o.status.success(),
            Err(_) => false,
        }
    } else {
        false
    }
}

#[test]
#[ignore]
fn test_full_deployment_workflow() {
    // Check prerequisites
    let jamt = jamt_bin().expect("jamt not found. Run 'cargo jam setup' first.");

    if !is_testnet_running() {
        panic!(
            "Testnet is not running!\n\
             Start it with: ~/.cargo-polkajam/toolchain/polkajam-nightly/polkajam-testnet\n\
             Then re-run this test."
        );
    }

    let temp = temp_dir();
    let project_name = "testnet-deploy-service";
    let project_path = temp.join(project_name);

    println!("=== Step 1: Creating new JAM service ===");
    let new_output = Command::new(cargo_jam_bin())
        .args(["polkajam", "new", project_name, "--defaults"])
        .current_dir(&temp)
        .output()
        .expect("Failed to run cargo-polkajam jam new");

    assert!(
        new_output.status.success(),
        "cargo-polkajam new failed: {:?}",
        String::from_utf8_lossy(&new_output.stderr)
    );
    println!("Created project at {:?}", project_path);

    println!("=== Step 2: Building JAM service ===");
    let build_output = Command::new(cargo_jam_bin())
        .args(["polkajam", "build"])
        .current_dir(&project_path)
        .output()
        .expect("Failed to run cargo-polkajam jam build");

    assert!(
        build_output.status.success(),
        "cargo-polkajam build failed: {:?}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let jam_file = project_path.join(format!("{}.jam", project_name));
    assert!(jam_file.exists(), ".jam blob not created");
    println!("Built service: {:?}", jam_file);

    println!("=== Step 3: Deploying to testnet ===");
    let deploy_output = Command::new(&jamt)
        .args(["create-service", jam_file.to_str().unwrap()])
        .output()
        .expect("Failed to run jamt create-service");

    println!(
        "Deploy stdout: {}",
        String::from_utf8_lossy(&deploy_output.stdout)
    );
    println!(
        "Deploy stderr: {}",
        String::from_utf8_lossy(&deploy_output.stderr)
    );

    assert!(
        deploy_output.status.success(),
        "jamt create-service failed: {:?}",
        String::from_utf8_lossy(&deploy_output.stderr)
    );

    println!("=== Deployment successful! ===");

    cleanup(&temp);
}

#[test]
#[ignore]
fn test_jamt_available() {
    let jamt = jamt_bin().expect("jamt not found. Run 'cargo jam setup' first.");

    let output = Command::new(&jamt)
        .arg("--help")
        .output()
        .expect("Failed to run jamt --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("JAM CLI tool") || stdout.contains("jamt"));
    println!("jamt is available at: {:?}", jamt);
}

#[test]
#[ignore]
fn test_testnet_binary_available() {
    let testnet = testnet_bin().expect("polkajam-testnet not found. Run 'cargo jam setup' first.");

    let output = Command::new(&testnet)
        .arg("--help")
        .output()
        .expect("Failed to run polkajam-testnet --help");

    assert!(output.status.success());
    println!("polkajam-testnet is available at: {:?}", testnet);
}

#[test]
#[ignore]
fn test_testnet_connection() {
    if !is_testnet_running() {
        panic!(
            "Testnet is not running!\n\
             Start it with: ~/.cargo-polkajam/toolchain/polkajam-nightly/polkajam-testnet"
        );
    }

    let jamt = jamt_bin().expect("jamt not found");

    // Try to inspect the queue
    let output = Command::new(&jamt)
        .args(["queue"])
        .output()
        .expect("Failed to run jamt queue");

    assert!(
        output.status.success(),
        "Failed to connect to testnet: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    println!("Successfully connected to testnet");
    println!("Queue status: {}", String::from_utf8_lossy(&output.stdout));
}
