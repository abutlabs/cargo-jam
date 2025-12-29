use crate::error::{CargoJamError, Result};
use regex::Regex;

/// Validate a project name for use as a Rust crate name
pub fn validate_project_name(name: &str) -> Result<()> {
    // Check for empty name
    if name.is_empty() {
        return Err(CargoJamError::InvalidProjectName {
            name: name.to_string(),
            reason: "Project name cannot be empty".to_string(),
        });
    }

    // Check for valid Rust crate name pattern
    let re = Regex::new(r"^[a-z][a-z0-9_-]*$").unwrap();
    if !re.is_match(name) {
        return Err(CargoJamError::InvalidProjectName {
            name: name.to_string(),
            reason: "Must start with a lowercase letter and contain only lowercase letters, numbers, underscores, and hyphens".to_string(),
        });
    }

    // Check for reserved names
    let reserved = [
        "self", "super", "crate", "Self", "test", "std", "core", "alloc",
    ];
    if reserved.contains(&name) {
        return Err(CargoJamError::InvalidProjectName {
            name: name.to_string(),
            reason: format!("'{}' is a reserved Rust keyword", name),
        });
    }

    // Check maximum length
    if name.len() > 64 {
        return Err(CargoJamError::InvalidProjectName {
            name: name.to_string(),
            reason: "Project name must be 64 characters or less".to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_names() {
        assert!(validate_project_name("my-service").is_ok());
        assert!(validate_project_name("my_service").is_ok());
        assert!(validate_project_name("myservice").is_ok());
        assert!(validate_project_name("my-service-123").is_ok());
    }

    #[test]
    fn test_invalid_names() {
        assert!(validate_project_name("").is_err());
        assert!(validate_project_name("My-Service").is_err());
        assert!(validate_project_name("123service").is_err());
        assert!(validate_project_name("-service").is_err());
        assert!(validate_project_name("self").is_err());
    }
}
