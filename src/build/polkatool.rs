use crate::error::{CargoJamError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Wrapper around polkatool for linking JAM service blobs
pub struct PolkatoolLinker {
    verbose: bool,
}

impl PolkatoolLinker {
    pub fn new() -> Self {
        Self { verbose: false }
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Check if polkatool is available
    pub fn is_available() -> bool {
        Command::new("polkatool")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Link an ELF binary into a JAM service blob
    pub fn link(&self, elf_path: &Path, output_path: &Path) -> Result<PathBuf> {
        if !Self::is_available() {
            return Err(CargoJamError::ToolchainMissing {
                tool: "polkatool".to_string(),
                install_hint: "cargo install polkatool".to_string(),
            });
        }

        let mut cmd = Command::new("polkatool");
        cmd.arg("jam-service").arg(elf_path).arg("-o").arg(output_path);

        if self.verbose {
            cmd.arg("--verbose");
        }

        let output = cmd.output().map_err(|e| {
            CargoJamError::Build(format!("Failed to execute polkatool: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CargoJamError::Build(format!(
                "polkatool linking failed:\n{}",
                stderr
            )));
        }

        Ok(output_path.to_path_buf())
    }
}

impl Default for PolkatoolLinker {
    fn default() -> Self {
        Self::new()
    }
}
