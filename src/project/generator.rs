use crate::error::{CargoJamError, Result};
use crate::template::config::TemplateConfig;
use crate::template::engine::TemplateEngine;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct ProjectGenerator {
    template_dir: PathBuf,
    output_dir: PathBuf,
    config: TemplateConfig,
    engine: TemplateEngine,
}

impl ProjectGenerator {
    pub fn new(template_dir: PathBuf, output_dir: PathBuf, config: TemplateConfig) -> Self {
        Self {
            template_dir,
            output_dir,
            config,
            engine: TemplateEngine::new().expect("Failed to create template engine"),
        }
    }

    pub fn generate(&self, variables: &HashMap<String, String>) -> Result<()> {
        // Create output directory
        std::fs::create_dir_all(&self.output_dir)?;

        // Walk through template directory
        for entry in WalkDir::new(&self.template_dir) {
            let entry = entry.map_err(|e| {
                CargoJamError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to walk directory: {}", e),
                ))
            })?;

            let path = entry.path();
            let relative_path = path
                .strip_prefix(&self.template_dir)
                .unwrap_or(path);

            // Skip the template directory itself
            if relative_path.as_os_str().is_empty() {
                continue;
            }

            let relative_str = relative_path.to_string_lossy().to_string();

            // Check if this path should be ignored
            if self.config.should_ignore_file(&relative_str) {
                continue;
            }

            // Process the filename (may contain template variables)
            let processed_filename = self.process_filename(&relative_str, variables)?;

            // Determine the output path
            let output_path = self.output_dir.join(&processed_filename);

            if entry.file_type().is_dir() {
                // Create directory
                std::fs::create_dir_all(&output_path)?;
            } else if entry.file_type().is_file() {
                // Ensure parent directory exists
                if let Some(parent) = output_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                // Process file
                self.process_file(path, &output_path, &relative_str, variables)?;
            }
        }

        Ok(())
    }

    fn process_filename(
        &self,
        filename: &str,
        variables: &HashMap<String, String>,
    ) -> Result<String> {
        let mut result = filename.to_string();

        // Strip .liquid extension if present
        if result.ends_with(".liquid") {
            result = result[..result.len() - 7].to_string();
        }

        // Process any {{ variable }} placeholders in the filename
        if result.contains("{{") {
            result = self.engine.render_filename(&result, variables)?;
        }

        Ok(result)
    }

    fn process_file(
        &self,
        source_path: &Path,
        output_path: &Path,
        relative_path: &str,
        variables: &HashMap<String, String>,
    ) -> Result<()> {
        let is_liquid = source_path
            .extension()
            .map(|e| e == "liquid")
            .unwrap_or(false);

        let should_process = is_liquid || self.config.should_process_file(relative_path);

        if should_process {
            // Read the file content
            let content = std::fs::read_to_string(source_path)?;

            // Render the template
            let rendered = self.engine.render(&content, variables)?;

            // Write the output
            std::fs::write(output_path, rendered)?;
        } else {
            // Copy the file as-is
            std::fs::copy(source_path, output_path)?;
        }

        Ok(())
    }
}
