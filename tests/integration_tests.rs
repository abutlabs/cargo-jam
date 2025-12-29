//! Integration tests for cargo-polkajam CLI commands
//!
//! Run with: cargo test --test integration_tests
//!
//! Note: These tests require:
//! - jam-pvm-build installed (`cargo install jam-pvm-build`)
//! - Internet connection (for `cargo jam setup`)

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

/// Create a temporary directory for tests
fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "cargo-polkajam-test-{}-{}",
        std::process::id(),
        rand()
    ));
    // Clean up if it exists from a previous run
    if dir.exists() {
        fs::remove_dir_all(&dir).ok();
    }
    fs::create_dir_all(&dir).expect("Failed to create temp dir");
    dir
}

/// Simple random number for unique temp dirs
fn rand() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos()
}

/// Clean up temporary directory
fn cleanup(dir: &PathBuf) {
    if dir.exists() {
        fs::remove_dir_all(dir).ok();
    }
}

#[test]
fn test_help() {
    let output = Command::new(cargo_jam_bin())
        .args(["polkajam", "--help"])
        .output()
        .expect("Failed to run cargo-polkajam jam --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("JAM service") || stdout.contains("Polkadot"));
}

#[test]
fn test_setup_info_no_toolchain() {
    // This test checks --info when no toolchain might be installed
    let output = Command::new(cargo_jam_bin())
        .args(["polkajam", "setup", "--info"])
        .output()
        .expect("Failed to run cargo-polkajam jam setup --info");

    // Should succeed regardless of installation status
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("JAM Toolchain Info"));
}

#[test]
fn test_setup_list() {
    let output = Command::new(cargo_jam_bin())
        .args(["polkajam", "setup", "--list"])
        .output()
        .expect("Failed to run cargo-polkajam jam setup --list");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Available releases"));
    assert!(stdout.contains("nightly"));
}

#[test]
fn test_new_creates_project() {
    let temp = temp_dir();
    let project_name = "test-new-service";
    let project_path = temp.join(project_name);

    let output = Command::new(cargo_jam_bin())
        .args(["polkajam", "new", project_name, "--defaults"])
        .current_dir(&temp)
        .output()
        .expect("Failed to run cargo-polkajam jam new");

    assert!(
        output.status.success(),
        "cargo-polkajam new failed: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify project structure
    assert!(project_path.exists(), "Project directory not created");
    assert!(
        project_path.join("Cargo.toml").exists(),
        "Cargo.toml not created"
    );
    assert!(
        project_path.join("src").join("lib.rs").exists(),
        "src/lib.rs not created"
    );

    // Verify Cargo.toml contents
    let cargo_toml =
        fs::read_to_string(project_path.join("Cargo.toml")).expect("Failed to read Cargo.toml");
    assert!(
        cargo_toml.contains("jam-pvm-common"),
        "Missing jam-pvm-common dependency"
    );
    assert!(
        cargo_toml.contains("polkavm-derive"),
        "Missing polkavm-derive dependency"
    );

    // Verify lib.rs contents
    let lib_rs =
        fs::read_to_string(project_path.join("src").join("lib.rs")).expect("Failed to read lib.rs");
    assert!(
        lib_rs.contains("declare_service!"),
        "Missing declare_service macro"
    );
    assert!(
        lib_rs.contains("impl Service"),
        "Missing Service implementation"
    );

    cleanup(&temp);
}

#[test]
fn test_new_with_custom_name() {
    let temp = temp_dir();
    let project_name = "my-custom-jam-service";
    let project_path = temp.join(project_name);

    let output = Command::new(cargo_jam_bin())
        .args(["polkajam", "new", project_name, "--defaults"])
        .current_dir(&temp)
        .output()
        .expect("Failed to run cargo-polkajam jam new");

    assert!(output.status.success());

    // Verify the service struct uses PascalCase
    let lib_rs =
        fs::read_to_string(project_path.join("src").join("lib.rs")).expect("Failed to read lib.rs");
    assert!(
        lib_rs.contains("MyCustomJamServiceService"),
        "Service name not properly converted to PascalCase"
    );

    cleanup(&temp);
}

#[test]
#[ignore] // Run with: cargo test --test integration_tests -- --ignored
fn test_setup_installs_toolchain() {
    // This test actually downloads the toolchain - may take a while
    let output = Command::new(cargo_jam_bin())
        .args(["polkajam", "setup"])
        .output()
        .expect("Failed to run cargo-polkajam jam setup");

    assert!(
        output.status.success(),
        "cargo-polkajam setup failed: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Installed JAM toolchain") || stdout.contains("already installed"));

    // Verify toolchain was installed
    let home = dirs::home_dir().expect("No home dir");
    let toolchain_path = home
        .join(".cargo-polkajam")
        .join("toolchain")
        .join("polkajam-nightly");
    assert!(toolchain_path.exists(), "Toolchain directory not created");
    assert!(
        toolchain_path.join("jamt").exists(),
        "jamt binary not found"
    );
    assert!(
        toolchain_path.join("polkajam").exists(),
        "polkajam binary not found"
    );
}

#[test]
#[ignore] // Run with: cargo test --test integration_tests -- --ignored
fn test_build_creates_jam_blob() {
    // This test requires jam-pvm-build to be installed
    let temp = temp_dir();
    let project_name = "test-build-service";
    let project_path = temp.join(project_name);

    // Create a new project
    let new_output = Command::new(cargo_jam_bin())
        .args(["polkajam", "new", project_name, "--defaults"])
        .current_dir(&temp)
        .output()
        .expect("Failed to run cargo-polkajam jam new");

    assert!(new_output.status.success(), "cargo-polkajam new failed");

    // Build the project
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

    // Verify .jam blob was created
    let jam_file = project_path.join(format!("{}.jam", project_name));
    assert!(jam_file.exists(), ".jam blob not created at {:?}", jam_file);

    // Verify it's not empty
    let metadata = fs::metadata(&jam_file).expect("Failed to get file metadata");
    assert!(metadata.len() > 0, ".jam blob is empty");

    cleanup(&temp);
}

#[test]
fn test_build_fails_without_jam_project() {
    let temp = temp_dir();

    // Create an empty directory (not a JAM project)
    let output = Command::new(cargo_jam_bin())
        .args(["polkajam", "build"])
        .current_dir(&temp)
        .output()
        .expect("Failed to run cargo-polkajam jam build");

    assert!(
        !output.status.success(),
        "build should fail without Cargo.toml"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Cargo.toml")
            || stderr.contains("not found")
            || stderr.contains("Not a JAM")
    );

    cleanup(&temp);
}
