use crate::error::{CargoJamError, Result};
use std::path::PathBuf;
use tempfile::TempDir;

pub struct GitTemplateSource {
    url: String,
    branch: Option<String>,
    subpath: Option<PathBuf>,
    temp_dir: Option<TempDir>,
}

impl GitTemplateSource {
    pub fn new(url: String) -> Self {
        Self {
            url,
            branch: None,
            subpath: None,
            temp_dir: None,
        }
    }

    pub fn branch(mut self, branch: Option<String>) -> Self {
        self.branch = branch;
        self
    }

    pub fn subpath(mut self, subpath: Option<PathBuf>) -> Self {
        self.subpath = subpath;
        self
    }

    pub fn fetch(&mut self) -> Result<PathBuf> {
        let temp_dir = TempDir::new().map_err(|e| {
            CargoJamError::Io(std::io::Error::other(format!(
                "Failed to create temp directory: {}",
                e
            )))
        })?;

        let clone_path = temp_dir.path();

        // Expand shorthand URLs
        let url = self.expand_url(&self.url);

        // Clone the repository
        let mut builder = git2::build::RepoBuilder::new();

        if let Some(ref branch) = self.branch {
            builder.branch(branch);
        }

        builder.clone(&url, clone_path).map_err(|e| {
            CargoJamError::Git(format!("Failed to clone repository '{}': {}", url, e))
        })?;

        // Determine the template path
        let template_path = if let Some(ref subpath) = self.subpath {
            clone_path.join(subpath)
        } else {
            clone_path.to_path_buf()
        };

        if !template_path.exists() {
            return Err(CargoJamError::Git(format!(
                "Template path '{}' not found in repository",
                template_path.display()
            )));
        }

        // Store temp dir to keep it alive
        self.temp_dir = Some(temp_dir);

        Ok(template_path)
    }

    fn expand_url(&self, url: &str) -> String {
        // Support shorthand URLs like gh:owner/repo, gl:owner/repo, etc.
        if let Some(rest) = url.strip_prefix("gh:") {
            format!("https://github.com/{}.git", rest)
        } else if let Some(rest) = url.strip_prefix("github:") {
            format!("https://github.com/{}.git", rest)
        } else if let Some(rest) = url.strip_prefix("gl:") {
            format!("https://gitlab.com/{}.git", rest)
        } else if let Some(rest) = url.strip_prefix("gitlab:") {
            format!("https://gitlab.com/{}.git", rest)
        } else if let Some(rest) = url.strip_prefix("bb:") {
            format!("https://bitbucket.org/{}.git", rest)
        } else if let Some(rest) = url.strip_prefix("bitbucket:") {
            format!("https://bitbucket.org/{}.git", rest)
        } else {
            url.to_string()
        }
    }
}
