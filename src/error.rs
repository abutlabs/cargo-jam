use thiserror::Error;

#[derive(Error, Debug)]
pub enum CargoJamError {
    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Invalid project name '{name}': {reason}")]
    InvalidProjectName { name: String, reason: String },

    #[error("Template configuration error: {0}")]
    TemplateConfig(String),

    #[error("Template rendering error: {0}")]
    TemplateRender(String),

    #[error("Git operation failed: {0}")]
    Git(String),

    #[error("Build failed: {0}")]
    Build(String),

    #[error("Toolchain not found: {tool}. Install with: {install_hint}")]
    ToolchainMissing { tool: String, install_hint: String },

    #[error("Project already exists at: {0}")]
    ProjectExists(String),

    #[error("Not a JAM service project: {0}")]
    NotJamProject(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
}

pub type Result<T> = std::result::Result<T, CargoJamError>;
