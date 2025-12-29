use crate::error::{CargoJamError, Result};
use crate::toolchain::config::ToolchainConfig;
use std::path::PathBuf;
use std::process::Command;

pub struct BuildPipeline {
    project_path: PathBuf,
    output_path: Option<PathBuf>,
    profile: BuildProfile,
    auto_install: bool,
    verbose: bool,
}

#[derive(Clone, Copy, Default)]
pub enum BuildProfile {
    Debug,
    #[default]
    Release,
    Production,
}

impl BuildProfile {
    fn as_str(&self) -> &'static str {
        match self {
            BuildProfile::Debug => "debug",
            BuildProfile::Release => "release",
            BuildProfile::Production => "production",
        }
    }
}

impl BuildPipeline {
    pub fn new(project_path: PathBuf) -> Self {
        Self {
            project_path,
            output_path: None,
            profile: BuildProfile::Release,
            auto_install: true,
            verbose: false,
        }
    }

    pub fn profile(mut self, profile: BuildProfile) -> Self {
        self.profile = profile;
        self
    }

    pub fn release(mut self, release: bool) -> Self {
        self.profile = if release {
            BuildProfile::Release
        } else {
            BuildProfile::Debug
        };
        self
    }

    pub fn output(mut self, path: PathBuf) -> Self {
        self.output_path = Some(path);
        self
    }

    pub fn auto_install(mut self, auto: bool) -> Self {
        self.auto_install = auto;
        self
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Execute the PVM build pipeline using jam-pvm-build
    pub fn run(&self) -> Result<PathBuf> {
        // Check for required tools
        self.check_toolchain()?;

        // Build using jam-pvm-build
        let jam_path = self.jam_pvm_build()?;

        Ok(jam_path)
    }

    fn check_toolchain(&self) -> Result<()> {
        // Check for jam-pvm-build
        let jam_build_check = Command::new("jam-pvm-build").arg("--version").output();

        if jam_build_check.is_err() || !jam_build_check.unwrap().status.success() {
            return Err(CargoJamError::ToolchainMissing {
                tool: "jam-pvm-build".to_string(),
                install_hint: "Install with: cargo install jam-pvm-build".to_string(),
            });
        }

        // Check for JAM toolchain (for jamt and other tools)
        let config = ToolchainConfig::load()?;
        if !config.is_installed() {
            return Err(CargoJamError::ToolchainMissing {
                tool: "JAM toolchain".to_string(),
                install_hint: "Run 'cargo polkajam setup' to install the JAM toolchain".to_string(),
            });
        }

        Ok(())
    }

    fn jam_pvm_build(&self) -> Result<PathBuf> {
        let mut cmd = Command::new("jam-pvm-build");

        // Set the project path
        cmd.arg(&self.project_path);

        // Set output path if specified
        if let Some(ref output) = self.output_path {
            cmd.arg("-o").arg(output);
        }

        // Set build profile
        cmd.arg("-p").arg(self.profile.as_str());

        // Set module type to service
        cmd.arg("-m").arg("service");

        // Auto-install rustc dependencies if enabled
        if self.auto_install {
            cmd.arg("--auto-install");
        }

        if self.verbose {
            println!(
                "Running: jam-pvm-build {:?}",
                cmd.get_args().collect::<Vec<_>>()
            );
        }

        let output = cmd
            .output()
            .map_err(|e| CargoJamError::Build(format!("Failed to execute jam-pvm-build: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(CargoJamError::Build(format!(
                "jam-pvm-build failed:\n{}\n{}",
                stdout, stderr
            )));
        }

        // Determine output path
        let output_path = if let Some(ref path) = self.output_path {
            path.clone()
        } else {
            // jam-pvm-build outputs to current directory with crate name
            let project_name = self.get_project_name()?;
            std::env::current_dir()?.join(format!("{}.jam", project_name))
        };

        if !output_path.exists() {
            // Try looking in project directory
            let project_name = self.get_project_name()?;
            let alt_path = self.project_path.join(format!("{}.jam", project_name));
            if alt_path.exists() {
                return Ok(alt_path);
            }

            return Err(CargoJamError::Build(format!(
                "Build completed but output file not found at expected path: {}",
                output_path.display()
            )));
        }

        Ok(output_path)
    }

    fn get_project_name(&self) -> Result<String> {
        // Parse Cargo.toml to get package name
        let cargo_toml = self.project_path.join("Cargo.toml");
        let content = std::fs::read_to_string(&cargo_toml)?;

        let manifest: toml::Value = toml::from_str(&content)
            .map_err(|e| CargoJamError::Build(format!("Failed to parse Cargo.toml: {}", e)))?;

        manifest
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| CargoJamError::Build("Missing package name in Cargo.toml".to_string()))
    }
}
