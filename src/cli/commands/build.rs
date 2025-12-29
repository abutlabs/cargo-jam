use crate::build::pipeline::BuildPipeline;
use crate::cli::args::BuildArgs;
use crate::error::{CargoJamError, Result};
use crate::toolchain::config::ToolchainConfig;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;

pub fn execute(args: BuildArgs) -> Result<()> {
    let project_path = args
        .path
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    // Validate this is a JAM service project
    validate_jam_project(&project_path)?;

    let spinner = create_spinner("Building JAM service with jam-pvm-build...");

    let mut pipeline = BuildPipeline::new(project_path.clone());

    pipeline = pipeline.release(args.release);

    if let Some(output) = args.output {
        pipeline = pipeline.output(output);
    }

    if args.verbose {
        pipeline = pipeline.verbose(true);
    }

    match pipeline.run() {
        Ok(output_path) => {
            spinner.finish_and_clear();
            println!(
                "\n{} Built JAM service: {}",
                style("✓").green().bold(),
                style(output_path.display()).cyan()
            );

            println!(
                "\n{} Deploy with: {} create-service {}",
                style("→").cyan(),
                style("jamt").green(),
                style(output_path.display()).yellow()
            );

            // Show jamt path hint
            if let Ok(Some(jamt_path)) = ToolchainConfig::binary_path("jamt") {
                println!("  Full path: {}", style(jamt_path.display()).dim());
            }

            Ok(())
        }
        Err(e) => {
            spinner.finish_and_clear();
            Err(e)
        }
    }
}

fn create_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    spinner
}

fn validate_jam_project(path: &Path) -> Result<()> {
    let cargo_toml = path.join("Cargo.toml");

    if !cargo_toml.exists() {
        return Err(CargoJamError::NotJamProject(
            "Cargo.toml not found".to_string(),
        ));
    }

    let content = std::fs::read_to_string(&cargo_toml)?;

    // Check for JAM dependencies
    if !content.contains("jam-pvm-common") && !content.contains("jam_pvm_common") {
        return Err(CargoJamError::NotJamProject(
            "jam-pvm-common dependency not found in Cargo.toml".to_string(),
        ));
    }

    Ok(())
}
