use crate::error::{CargoJamError, Result};
use crate::toolchain::config::ToolchainConfig;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Wrapper around jamt (JAM Tool) for building JAM service blobs
pub struct JamtBuilder {
    verbose: bool,
}

impl JamtBuilder {
    pub fn new() -> Self {
        Self { verbose: false }
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Check if jamt is available via the installed toolchain
    pub fn is_available() -> bool {
        ToolchainConfig::binary_path("jamt")
            .ok()
            .flatten()
            .map(|p| p.exists())
            .unwrap_or(false)
    }

    /// Get the jamt binary path
    pub fn binary_path() -> Result<PathBuf> {
        ToolchainConfig::binary_path("jamt")?.ok_or_else(|| CargoJamError::ToolchainMissing {
            tool: "jamt".to_string(),
            install_hint: "Run 'cargo jam setup' to install the JAM toolchain".to_string(),
        })
    }

    /// Build a JAM service blob from source
    pub fn build(&self, elf_path: &Path, output_path: &Path) -> Result<PathBuf> {
        let jamt_path = Self::binary_path()?;

        let mut cmd = Command::new(&jamt_path);
        cmd.arg("build").arg(elf_path).arg("-o").arg(output_path);

        if self.verbose {
            cmd.arg("--verbose");
        }

        let output = cmd
            .output()
            .map_err(|e| CargoJamError::Build(format!("Failed to execute jamt: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CargoJamError::Build(format!(
                "jamt build failed:\n{}",
                stderr
            )));
        }

        Ok(output_path.to_path_buf())
    }
}

impl Default for JamtBuilder {
    fn default() -> Self {
        Self::new()
    }
}
