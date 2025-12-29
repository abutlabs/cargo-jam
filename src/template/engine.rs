use crate::error::{CargoJamError, Result};
use heck::{ToKebabCase, ToLowerCamelCase, ToPascalCase, ToSnakeCase, ToUpperCamelCase};
use liquid::model::Value;
use liquid::{Object, Parser, ParserBuilder};
use liquid_core::{Filter, Runtime, ValueView};
use liquid_derive::{Display_filter, FilterReflection, ParseFilter};
use std::collections::HashMap;

pub struct TemplateEngine {
    parser: Parser,
}

impl TemplateEngine {
    pub fn new() -> Result<Self> {
        let parser = ParserBuilder::with_stdlib()
            .filter(PascalCaseFilter)
            .filter(SnakeCaseFilter)
            .filter(KebabCaseFilter)
            .filter(CamelCaseFilter)
            .filter(UpperCamelCaseFilter)
            .build()
            .map_err(|e| CargoJamError::TemplateRender(format!("Failed to build parser: {}", e)))?;

        Ok(Self { parser })
    }

    pub fn render(&self, template: &str, variables: &HashMap<String, String>) -> Result<String> {
        let template = self.parser.parse(template).map_err(|e| {
            CargoJamError::TemplateRender(format!("Failed to parse template: {}", e))
        })?;

        let mut globals = Object::new();
        for (key, value) in variables {
            globals.insert(key.clone().into(), Value::scalar(value.clone()));
        }

        template.render(&globals).map_err(|e| {
            CargoJamError::TemplateRender(format!("Failed to render template: {}", e))
        })
    }

    pub fn render_filename(
        &self,
        filename: &str,
        variables: &HashMap<String, String>,
    ) -> Result<String> {
        // Handle {{ variable }} in filenames
        if filename.contains("{{") {
            self.render(filename, variables)
        } else {
            Ok(filename.to_string())
        }
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create template engine")
    }
}

// Custom Liquid filters for case conversion

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "pascal_case",
    description = "Convert to PascalCase",
    parsed(PascalCaseFilterImpl)
)]
pub struct PascalCaseFilter;

#[derive(Debug, Default, Display_filter)]
#[name = "pascal_case"]
struct PascalCaseFilterImpl;

impl Filter for PascalCaseFilterImpl {
    fn evaluate(
        &self,
        input: &dyn ValueView,
        _runtime: &dyn Runtime,
    ) -> liquid_core::Result<Value> {
        let s = input.to_kstr();
        Ok(Value::scalar(s.to_pascal_case()))
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "snake_case",
    description = "Convert to snake_case",
    parsed(SnakeCaseFilterImpl)
)]
pub struct SnakeCaseFilter;

#[derive(Debug, Default, Display_filter)]
#[name = "snake_case"]
struct SnakeCaseFilterImpl;

impl Filter for SnakeCaseFilterImpl {
    fn evaluate(
        &self,
        input: &dyn ValueView,
        _runtime: &dyn Runtime,
    ) -> liquid_core::Result<Value> {
        let s = input.to_kstr();
        Ok(Value::scalar(s.to_snake_case()))
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "kebab_case",
    description = "Convert to kebab-case",
    parsed(KebabCaseFilterImpl)
)]
pub struct KebabCaseFilter;

#[derive(Debug, Default, Display_filter)]
#[name = "kebab_case"]
struct KebabCaseFilterImpl;

impl Filter for KebabCaseFilterImpl {
    fn evaluate(
        &self,
        input: &dyn ValueView,
        _runtime: &dyn Runtime,
    ) -> liquid_core::Result<Value> {
        let s = input.to_kstr();
        Ok(Value::scalar(s.to_kebab_case()))
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "camel_case",
    description = "Convert to camelCase",
    parsed(CamelCaseFilterImpl)
)]
pub struct CamelCaseFilter;

#[derive(Debug, Default, Display_filter)]
#[name = "camel_case"]
struct CamelCaseFilterImpl;

impl Filter for CamelCaseFilterImpl {
    fn evaluate(
        &self,
        input: &dyn ValueView,
        _runtime: &dyn Runtime,
    ) -> liquid_core::Result<Value> {
        let s = input.to_kstr();
        Ok(Value::scalar(s.to_lower_camel_case()))
    }
}

#[derive(Clone, ParseFilter, FilterReflection)]
#[filter(
    name = "upper_camel_case",
    description = "Convert to UpperCamelCase",
    parsed(UpperCamelCaseFilterImpl)
)]
pub struct UpperCamelCaseFilter;

#[derive(Debug, Default, Display_filter)]
#[name = "upper_camel_case"]
struct UpperCamelCaseFilterImpl;

impl Filter for UpperCamelCaseFilterImpl {
    fn evaluate(
        &self,
        input: &dyn ValueView,
        _runtime: &dyn Runtime,
    ) -> liquid_core::Result<Value> {
        let s = input.to_kstr();
        Ok(Value::scalar(s.to_upper_camel_case()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple() {
        let engine = TemplateEngine::new().unwrap();
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "my-service".to_string());

        let result = engine.render("Hello {{ name }}", &vars).unwrap();
        assert_eq!(result, "Hello my-service");
    }

    #[test]
    fn test_pascal_case_filter() {
        let engine = TemplateEngine::new().unwrap();
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "my-service".to_string());

        let result = engine.render("{{ name | pascal_case }}", &vars).unwrap();
        assert_eq!(result, "MyService");
    }

    #[test]
    fn test_snake_case_filter() {
        let engine = TemplateEngine::new().unwrap();
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "my-service".to_string());

        let result = engine.render("{{ name | snake_case }}", &vars).unwrap();
        assert_eq!(result, "my_service");
    }
}
