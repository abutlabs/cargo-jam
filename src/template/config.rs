use crate::error::{CargoJamError, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct TemplateConfig {
    pub template: TemplateMetadata,
    #[serde(default)]
    pub placeholders: HashMap<String, Placeholder>,
    #[serde(default)]
    pub conditional: HashMap<String, ConditionalConfig>,
}

#[derive(Debug, Deserialize)]
pub struct TemplateMetadata {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub ignore: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Placeholder {
    String {
        prompt: String,
        #[serde(default)]
        default: Option<String>,
        #[serde(default)]
        regex: Option<String>,
        #[serde(default)]
        choices: Option<Vec<String>>,
    },
    Bool {
        prompt: String,
        #[serde(default)]
        default: Option<bool>,
    },
}

impl Placeholder {
    pub fn prompt(&self) -> &str {
        match self {
            Placeholder::String { prompt, .. } => prompt,
            Placeholder::Bool { prompt, .. } => prompt,
        }
    }

    pub fn default_value(&self) -> Option<String> {
        match self {
            Placeholder::String { default, .. } => default.clone(),
            Placeholder::Bool { default, .. } => default.map(|b| b.to_string()),
        }
    }

    pub fn choices(&self) -> Option<&Vec<String>> {
        match self {
            Placeholder::String { choices, .. } => choices.as_ref(),
            Placeholder::Bool { .. } => None,
        }
    }

    pub fn regex(&self) -> Option<&str> {
        match self {
            Placeholder::String { regex, .. } => regex.as_deref(),
            Placeholder::Bool { .. } => None,
        }
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Placeholder::Bool { .. })
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct ConditionalConfig {
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub ignore: Vec<String>,
}

impl TemplateConfig {
    pub fn load_from_dir(dir: &Path) -> Result<Self> {
        let config_path = dir.join("cargo-polkajam.toml");

        if !config_path.exists() {
            return Err(CargoJamError::TemplateConfig(
                "cargo-polkajam.toml not found in template directory".to_string(),
            ));
        }

        let content = std::fs::read_to_string(&config_path)?;
        let config: TemplateConfig = toml::from_str(&content).map_err(|e| {
            CargoJamError::TemplateConfig(format!("Failed to parse cargo-polkajam.toml: {}", e))
        })?;

        Ok(config)
    }

    pub fn should_process_file(&self, path: &str) -> bool {
        // Check if file should be processed with Liquid
        if self.template.include.is_empty() {
            // If no include patterns, process all non-ignored files
            return !self.should_ignore_file(path);
        }

        for pattern in &self.template.include {
            if glob_match(pattern, path) {
                return true;
            }
        }

        false
    }

    pub fn should_ignore_file(&self, path: &str) -> bool {
        for pattern in &self.template.ignore {
            if glob_match(pattern, path) {
                return true;
            }
        }

        // Always ignore cargo-polkajam.toml itself
        if path == "cargo-polkajam.toml" {
            return true;
        }

        false
    }
}

fn glob_match(pattern: &str, path: &str) -> bool {
    // Simple glob matching
    if pattern.contains('*') {
        let pattern = pattern.replace("**", ".*").replace('*', "[^/]*");
        if let Ok(re) = regex::Regex::new(&format!("^{}$", pattern)) {
            return re.is_match(path);
        }
    }
    path == pattern || path.starts_with(&format!("{}/", pattern))
}
