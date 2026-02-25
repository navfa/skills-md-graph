use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub schema: SchemaConfig,
    #[serde(default)]
    pub scan: ScanConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaConfig {
    #[serde(default = "default_required_fields")]
    pub required_fields: Vec<String>,
    #[serde(default = "default_optional_fields")]
    pub optional_fields: Vec<String>,
    #[serde(default)]
    pub aliases: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    #[serde(default = "default_workers")]
    pub workers: usize,
    #[serde(default = "default_extensions")]
    pub extensions: Vec<String>,
}

fn default_required_fields() -> Vec<String> {
    vec!["name".to_string()]
}

fn default_optional_fields() -> Vec<String> {
    vec![
        "description".to_string(),
        "dependencies".to_string(),
        "inputs".to_string(),
        "outputs".to_string(),
    ]
}

fn default_workers() -> usize {
    num_cpus()
}

fn default_extensions() -> Vec<String> {
    vec!["md".to_string()]
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(4)
}

impl Default for SchemaConfig {
    fn default() -> Self {
        Self {
            required_fields: default_required_fields(),
            optional_fields: default_optional_fields(),
            aliases: HashMap::new(),
        }
    }
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            workers: default_workers(),
            extensions: default_extensions(),
        }
    }
}

const CONFIG_FILENAME: &str = ".skill-graph.toml";

/// Search for `.skill-graph.toml` starting from `start_dir` and walking up parents.
pub fn discover_config_path(start_dir: &Path) -> Option<PathBuf> {
    let mut current = start_dir.to_path_buf();
    loop {
        let candidate = current.join(CONFIG_FILENAME);
        if candidate.is_file() {
            return Some(candidate);
        }
        if !current.pop() {
            return None;
        }
    }
}

/// Load config from an explicit path, or discover it from `start_dir`.
/// Returns default config if no file is found.
pub fn load_config(explicit_path: Option<&Path>, start_dir: &Path) -> Config {
    let config_path = explicit_path
        .map(PathBuf::from)
        .or_else(|| discover_config_path(start_dir));

    match config_path {
        Some(path) => match fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content).unwrap_or_default(),
            Err(_) => Config::default(),
        },
        None => Config::default(),
    }
}

/// Apply aliases to raw YAML content by replacing aliased keys.
pub fn apply_aliases(yaml_content: &str, aliases: &HashMap<String, String>) -> String {
    if aliases.is_empty() {
        return yaml_content.to_string();
    }

    let mut result = yaml_content.to_string();
    for (alias, canonical) in aliases {
        // Replace "alias:" at the start of a line with "canonical:"
        let pattern = format!("\n{alias}:");
        let replacement = format!("\n{canonical}:");
        result = result.replace(&pattern, &replacement);

        // Handle first line too
        if result.starts_with(&format!("{alias}:")) {
            result = format!("{canonical}:{}", &result[alias.len() + 1..]);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn default_config_has_sensible_values() {
        let config = Config::default();

        assert_eq!(config.schema.required_fields, vec!["name"]);
        assert!(
            config
                .schema
                .optional_fields
                .contains(&"description".to_string())
        );
        assert!(config.schema.aliases.is_empty());
        assert!(config.scan.workers > 0);
        assert_eq!(config.scan.extensions, vec!["md"]);
    }

    #[test]
    fn load_config_returns_default_when_no_file() {
        let config = load_config(None, Path::new("/nonexistent"));

        assert_eq!(config.schema.required_fields, vec!["name"]);
    }

    #[test]
    fn load_config_from_toml_file() {
        let temp_dir = std::env::temp_dir().join("skill-graph-test-config");
        fs::create_dir_all(&temp_dir).unwrap();
        let config_path = temp_dir.join(CONFIG_FILENAME);

        fs::write(
            &config_path,
            r#"
[schema]
required_fields = ["name", "description"]

[schema.aliases]
deps = "dependencies"
desc = "description"

[scan]
workers = 4
extensions = ["md", "skill.md"]
"#,
        )
        .unwrap();

        let config = load_config(None, &temp_dir);

        assert_eq!(config.schema.required_fields, vec!["name", "description"]);
        assert_eq!(config.schema.aliases.get("deps").unwrap(), "dependencies");
        assert_eq!(config.scan.workers, 4);
        assert_eq!(config.scan.extensions, vec!["md", "skill.md"]);

        fs::remove_file(&config_path).unwrap();
        let _ = fs::remove_dir(&temp_dir);
    }

    #[test]
    fn apply_aliases_replaces_keys() {
        let mut aliases = HashMap::new();
        aliases.insert("deps".to_string(), "dependencies".to_string());
        aliases.insert("desc".to_string(), "description".to_string());

        let yaml = "name: test\ndeps:\n  - alpha\ndesc: hello";
        let result = apply_aliases(yaml, &aliases);

        assert!(result.contains("dependencies:"));
        assert!(result.contains("description:"));
        assert!(!result.contains("deps:"));
        assert!(!result.contains("desc:"));
    }

    #[test]
    fn apply_aliases_noop_when_empty() {
        let aliases = HashMap::new();
        let yaml = "name: test\ndeps:\n  - alpha";
        let result = apply_aliases(yaml, &aliases);

        assert_eq!(result, yaml);
    }

    #[test]
    fn discover_config_walks_up_parents() {
        let temp_dir = std::env::temp_dir().join("skill-graph-test-discover");
        let child_dir = temp_dir.join("sub").join("deep");
        fs::create_dir_all(&child_dir).unwrap();

        let config_path = temp_dir.join(CONFIG_FILENAME);
        fs::write(&config_path, "[schema]\n").unwrap();

        let found = discover_config_path(&child_dir);
        assert_eq!(found, Some(config_path.clone()));

        fs::remove_file(&config_path).unwrap();
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
