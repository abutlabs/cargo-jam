use crate::error::{CargoJamError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the installed toolchain
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ToolchainConfig {
    /// Currently installed version (e.g., "nightly-2025-12-29")
    pub installed_version: Option<String>,
    /// Path to the toolchain directory
    pub toolchain_path: Option<PathBuf>,
    /// Installation timestamp
    pub installed_at: Option<String>,
}

impl ToolchainConfig {
    /// Get the cargo-polkajam home directory (~/.cargo-polkajam)
    pub fn home_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            CargoJamError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine home directory",
            ))
        })?;
        Ok(home.join(".cargo-polkajam"))
    }

    /// Get the config file path (~/.cargo-polkajam/config.toml)
    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::home_dir()?.join("config.toml"))
    }

    /// Get the toolchain installation directory (~/.cargo-polkajam/toolchain)
    pub fn toolchain_dir() -> Result<PathBuf> {
        Ok(Self::home_dir()?.join("toolchain"))
    }

    /// Get the path to a specific toolchain binary
    pub fn binary_path(binary_name: &str) -> Result<Option<PathBuf>> {
        let config = Self::load()?;
        if let Some(toolchain_path) = config.toolchain_path {
            let binary_path = toolchain_path.join("polkajam-nightly").join(binary_name);
            if binary_path.exists() {
                return Ok(Some(binary_path));
            }
        }
        Ok(None)
    }

    /// Get the path to the polkajam toolchain directory
    pub fn polkajam_dir() -> Result<Option<PathBuf>> {
        let config = Self::load()?;
        if let Some(toolchain_path) = config.toolchain_path {
            let nightly_dir = toolchain_path.join("polkajam-nightly");
            if nightly_dir.exists() {
                return Ok(Some(nightly_dir));
            }
        }
        Ok(None)
    }

    /// Load the config from disk
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&config_path)?;
        let config: ToolchainConfig = toml::from_str(&content)
            .map_err(|e| CargoJamError::TemplateConfig(format!("Failed to parse config: {}", e)))?;
        Ok(config)
    }

    /// Save the config to disk
    pub fn save(&self) -> Result<()> {
        let home_dir = Self::home_dir()?;
        std::fs::create_dir_all(&home_dir)?;

        let config_path = Self::config_path()?;
        let content = toml::to_string_pretty(self).map_err(|e| {
            CargoJamError::TemplateConfig(format!("Failed to serialize config: {}", e))
        })?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    /// Check if a toolchain is installed
    pub fn is_installed(&self) -> bool {
        if let Some(ref path) = self.toolchain_path {
            path.exists() && self.installed_version.is_some()
        } else {
            false
        }
    }

    /// Update config after installation
    pub fn set_installed(&mut self, version: &str, path: PathBuf) {
        self.installed_version = Some(version.to_string());
        self.toolchain_path = Some(path);
        self.installed_at = Some(chrono_lite_now());
    }
}

/// Simple timestamp without pulling in chrono
fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}
