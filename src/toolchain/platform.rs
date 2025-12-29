use crate::error::{CargoJamError, Result};

/// Supported platform targets for the polkajam toolchain
#[derive(Debug, Clone, PartialEq)]
pub enum Platform {
    MacosAarch64,
    MacosX86_64,
    LinuxX86_64,
    LinuxAarch64,
    WindowsX86_64,
}

impl Platform {
    /// Detect the current platform
    pub fn detect() -> Result<Self> {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        match (os, arch) {
            ("macos", "aarch64") => Ok(Platform::MacosAarch64),
            ("macos", "x86_64") => Ok(Platform::MacosX86_64),
            ("linux", "x86_64") => Ok(Platform::LinuxX86_64),
            ("linux", "aarch64") => Ok(Platform::LinuxAarch64),
            ("windows", "x86_64") => Ok(Platform::WindowsX86_64),
            _ => Err(CargoJamError::ToolchainMissing {
                tool: "polkajam".to_string(),
                install_hint: format!(
                    "Unsupported platform: {}-{}. Supported: macos-aarch64, macos-x86_64, linux-x86_64, linux-aarch64, windows-x86_64",
                    os, arch
                ),
            }),
        }
    }

    /// Get the asset name suffix for this platform
    pub fn asset_suffix(&self) -> &'static str {
        match self {
            Platform::MacosAarch64 => "macos-aarch64",
            Platform::MacosX86_64 => "macos-x86_64",
            Platform::LinuxX86_64 => "linux-x86_64",
            Platform::LinuxAarch64 => "linux-aarch64",
            Platform::WindowsX86_64 => "windows-x86_64",
        }
    }

    /// Get the archive extension for this platform
    pub fn archive_extension(&self) -> &'static str {
        match self {
            Platform::WindowsX86_64 => "zip",
            _ => "tar.gz",
        }
    }

    /// Build the expected asset filename for a given version
    pub fn asset_filename(&self, _version: &str) -> String {
        format!(
            "polkajam-{}.{}",
            self.asset_suffix(),
            self.archive_extension()
        )
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.asset_suffix())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        // This will pass on whatever platform runs the test
        let platform = Platform::detect();
        assert!(platform.is_ok());
    }

    #[test]
    fn test_asset_suffix() {
        assert_eq!(Platform::MacosAarch64.asset_suffix(), "macos-aarch64");
        assert_eq!(Platform::LinuxX86_64.asset_suffix(), "linux-x86_64");
    }
}
