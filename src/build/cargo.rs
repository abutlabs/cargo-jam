use crate::error::{CargoJamError, Result};
use std::path::Path;
use std::process::Command;

/// Wrapper around cargo build for JAM services
pub struct CargoBuilder {
    target: String,
    release: bool,
    verbose: bool,
}

impl CargoBuilder {
    pub fn new() -> Self {
        Self {
            target: "riscv32ema-unknown-none-elf".to_string(),
            release: true,
            verbose: false,
        }
    }

    pub fn release(mut self, release: bool) -> Self {
        self.release = release;
        self
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn build(&self, project_path: &Path) -> Result<()> {
        let mut cmd = Command::new("cargo");
        cmd.arg("build")
            .arg("--target")
            .arg(&self.target)
            .current_dir(project_path);

        if self.release {
            cmd.arg("--release");
        }

        if self.verbose {
            cmd.arg("--verbose");
        }

        // Build-std flags for no_std
        cmd.arg("-Z").arg("build-std=core,alloc");
        cmd.arg("-Z").arg("build-std-features=panic_immediate_abort");

        let output = cmd.output().map_err(|e| {
            CargoJamError::Build(format!("Failed to execute cargo: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CargoJamError::Build(format!(
                "Cargo build failed:\n{}",
                stderr
            )));
        }

        Ok(())
    }
}

impl Default for CargoBuilder {
    fn default() -> Self {
        Self::new()
    }
}
