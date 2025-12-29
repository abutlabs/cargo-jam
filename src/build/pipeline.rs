use crate::error::{CargoJamError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

const RISC_V_TARGET: &str = "riscv32ema-unknown-none-elf";

pub struct BuildPipeline {
    project_path: PathBuf,
    release: bool,
    output_path: Option<PathBuf>,
    skip_link: bool,
    verbose: bool,
}

impl BuildPipeline {
    pub fn new(project_path: PathBuf) -> Self {
        Self {
            project_path,
            release: true,
            output_path: None,
            skip_link: false,
            verbose: false,
        }
    }

    pub fn release(mut self, release: bool) -> Self {
        self.release = release;
        self
    }

    pub fn output(mut self, path: PathBuf) -> Self {
        self.output_path = Some(path);
        self
    }

    pub fn skip_link(mut self, skip: bool) -> Self {
        self.skip_link = skip;
        self
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Execute the full PVM build pipeline
    pub fn run(&self) -> Result<PathBuf> {
        // Check for required tools
        self.check_toolchain()?;

        // Step 1: Run cargo build targeting RISC-V
        let elf_path = self.cargo_build()?;

        if self.skip_link {
            return Ok(elf_path);
        }

        // Step 2: Link with polkatool to produce .jam blob
        let jam_path = self.polkatool_link(&elf_path)?;

        Ok(jam_path)
    }

    fn check_toolchain(&self) -> Result<()> {
        // Check for rustup target
        let output = Command::new("rustup")
            .args(["target", "list", "--installed"])
            .output()
            .map_err(|_| CargoJamError::ToolchainMissing {
                tool: "rustup".to_string(),
                install_hint: "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
                    .to_string(),
            })?;

        let installed_targets = String::from_utf8_lossy(&output.stdout);
        if !installed_targets.contains(RISC_V_TARGET) {
            return Err(CargoJamError::ToolchainMissing {
                tool: format!("RISC-V target ({})", RISC_V_TARGET),
                install_hint: format!("rustup target add {}", RISC_V_TARGET),
            });
        }

        // Check for polkatool (only if we're linking)
        if !self.skip_link {
            let polkatool_check = Command::new("polkatool").arg("--version").output();

            if polkatool_check.is_err() {
                return Err(CargoJamError::ToolchainMissing {
                    tool: "polkatool".to_string(),
                    install_hint: "cargo install polkatool".to_string(),
                });
            }
        }

        Ok(())
    }

    fn cargo_build(&self) -> Result<PathBuf> {
        let mut cmd = Command::new("cargo");
        cmd.arg("build")
            .arg("--target")
            .arg(RISC_V_TARGET)
            .current_dir(&self.project_path);

        if self.release {
            cmd.arg("--release");
        }

        // Enable build-std for no_std support
        cmd.arg("-Z").arg("build-std=core,alloc");
        cmd.arg("-Z").arg("build-std-features=panic_immediate_abort");

        if self.verbose {
            cmd.arg("--verbose");
        }

        let status = cmd.status().map_err(|e| {
            CargoJamError::Build(format!("Failed to execute cargo build: {}", e))
        })?;

        if !status.success() {
            return Err(CargoJamError::Build(format!(
                "Cargo build failed with status: {}",
                status
            )));
        }

        // Determine output ELF path
        let profile = if self.release { "release" } else { "debug" };
        let project_name = self.get_project_name()?;

        Ok(self
            .project_path
            .join("target")
            .join(RISC_V_TARGET)
            .join(profile)
            .join(&project_name))
    }

    fn polkatool_link(&self, elf_path: &Path) -> Result<PathBuf> {
        let output_path = self.output_path.clone().unwrap_or_else(|| {
            let name = elf_path.file_stem().unwrap().to_str().unwrap();
            self.project_path.join(format!("{}.jam", name))
        });

        let mut cmd = Command::new("polkatool");
        cmd.arg("jam-service")
            .arg(elf_path)
            .arg("-o")
            .arg(&output_path);

        if self.verbose {
            cmd.arg("--verbose");
        }

        let status = cmd.status().map_err(|e| {
            CargoJamError::Build(format!("Failed to execute polkatool: {}", e))
        })?;

        if !status.success() {
            return Err(CargoJamError::Build(format!(
                "polkatool linking failed with status: {}",
                status
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
            .map(|s| s.replace('-', "_"))
            .ok_or_else(|| CargoJamError::Build("Missing package name in Cargo.toml".to_string()))
    }
}
