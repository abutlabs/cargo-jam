use crate::error::{CargoJamError, Result};
use crate::template::config::{Placeholder, TemplateConfig};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use std::collections::HashMap;

pub struct PromptRunner {
    theme: ColorfulTheme,
}

impl PromptRunner {
    pub fn new() -> Self {
        Self {
            theme: ColorfulTheme::default(),
        }
    }

    pub fn collect_variables(
        &self,
        config: &TemplateConfig,
        existing: &HashMap<String, String>,
    ) -> Result<HashMap<String, String>> {
        let mut variables = HashMap::new();

        for (key, placeholder) in &config.placeholders {
            // Skip if already defined
            if existing.contains_key(key) {
                continue;
            }

            // Skip project_name as it's handled separately
            if key == "project_name" {
                continue;
            }

            let value = self.prompt_placeholder(key, placeholder)?;
            variables.insert(key.clone(), value);
        }

        Ok(variables)
    }

    fn prompt_placeholder(&self, _key: &str, placeholder: &Placeholder) -> Result<String> {
        match placeholder {
            Placeholder::String {
                prompt,
                default,
                choices,
                regex,
            } => {
                if let Some(choices) = choices {
                    self.prompt_select(prompt, choices, default.as_deref())
                } else {
                    self.prompt_string(prompt, default.as_deref(), regex.as_deref())
                }
            }
            Placeholder::Bool { prompt, default } => {
                let result = self.prompt_bool(prompt, default.unwrap_or(false))?;
                Ok(result.to_string())
            }
        }
    }

    pub fn prompt_string(
        &self,
        prompt: &str,
        default: Option<&str>,
        regex: Option<&str>,
    ) -> Result<String> {
        let mut input = Input::<String>::with_theme(&self.theme).with_prompt(prompt);

        if let Some(default) = default {
            input = input.default(default.to_string());
        }

        if let Some(pattern) = regex {
            let re = regex::Regex::new(pattern).map_err(|e| {
                CargoJamError::TemplateConfig(format!("Invalid regex '{}': {}", pattern, e))
            })?;

            input = input.validate_with(move |input: &String| -> std::result::Result<(), String> {
                if re.is_match(input) {
                    Ok(())
                } else {
                    Err(format!("Input must match pattern: {}", pattern))
                }
            });
        }

        input
            .interact_text()
            .map_err(|e| CargoJamError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
    }

    pub fn prompt_select(
        &self,
        prompt: &str,
        choices: &[String],
        default: Option<&str>,
    ) -> Result<String> {
        let default_index = default
            .and_then(|d| choices.iter().position(|c| c == d))
            .unwrap_or(0);

        let selection = Select::with_theme(&self.theme)
            .with_prompt(prompt)
            .items(choices)
            .default(default_index)
            .interact()
            .map_err(|e| CargoJamError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        Ok(choices[selection].clone())
    }

    pub fn prompt_bool(&self, prompt: &str, default: bool) -> Result<bool> {
        Confirm::with_theme(&self.theme)
            .with_prompt(prompt)
            .default(default)
            .interact()
            .map_err(|e| CargoJamError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
    }
}

impl Default for PromptRunner {
    fn default() -> Self {
        Self::new()
    }
}
