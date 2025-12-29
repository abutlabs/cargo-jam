use crate::error::{CargoJamError, Result};
use include_dir::{include_dir, Dir};
use std::path::PathBuf;
use tempfile::TempDir;

// Embed the templates directory at compile time
static TEMPLATES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates");

pub struct BundledTemplates {
    temp_dir: Option<TempDir>,
}

impl BundledTemplates {
    pub fn new() -> Self {
        Self { temp_dir: None }
    }

    pub fn list(&self) -> Vec<String> {
        TEMPLATES_DIR
            .dirs()
            .map(|d| d.path().file_name().unwrap().to_string_lossy().to_string())
            .collect()
    }

    pub fn extract(&mut self, template_name: &str) -> Result<PathBuf> {
        let template_dir = TEMPLATES_DIR
            .get_dir(template_name)
            .ok_or_else(|| CargoJamError::TemplateNotFound(template_name.to_string()))?;

        // Create a temporary directory to extract the template
        let temp_dir = TempDir::new().map_err(|e| {
            CargoJamError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create temp directory: {}", e),
            ))
        })?;

        let extract_path = temp_dir.path().to_path_buf();

        // Extract all files from the embedded directory
        self.extract_dir(template_dir, &extract_path)?;

        // Store the temp dir to keep it alive
        self.temp_dir = Some(temp_dir);

        Ok(extract_path)
    }

    fn extract_dir(&self, dir: &Dir<'_>, dest: &PathBuf) -> Result<()> {
        std::fs::create_dir_all(dest)?;

        // Extract files
        for file in dir.files() {
            let file_path = dest.join(file.path().file_name().unwrap());
            std::fs::write(&file_path, file.contents())?;
        }

        // Extract subdirectories
        for subdir in dir.dirs() {
            let subdir_name = subdir.path().file_name().unwrap();
            let subdir_dest = dest.join(subdir_name);
            self.extract_dir(subdir, &subdir_dest)?;
        }

        Ok(())
    }
}

impl Default for BundledTemplates {
    fn default() -> Self {
        Self::new()
    }
}
