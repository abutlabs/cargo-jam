use std::collections::HashMap;

/// Holds the collected template variable values
#[derive(Debug, Default)]
pub struct TemplateValues {
    values: HashMap<String, String>,
}

impl TemplateValues {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.values.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    pub fn into_map(self) -> HashMap<String, String> {
        self.values
    }

    pub fn extend(&mut self, other: HashMap<String, String>) {
        self.values.extend(other);
    }
}

impl From<HashMap<String, String>> for TemplateValues {
    fn from(values: HashMap<String, String>) -> Self {
        Self { values }
    }
}

impl From<TemplateValues> for HashMap<String, String> {
    fn from(tv: TemplateValues) -> Self {
        tv.values
    }
}
