use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::config::apply_aliases;
use crate::error::SkillError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontMatter {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default)]
    pub outputs: Vec<String>,
}

pub fn extract_frontmatter<'a>(
    content: &'a str,
    file_path: &Path,
    aliases: &HashMap<String, String>,
) -> Result<(FrontMatter, &'a str), SkillError> {
    let trimmed = content.trim_start();

    if !trimmed.starts_with("---") {
        return Err(SkillError::InvalidYaml {
            path: file_path.to_path_buf(),
            source: serde_yaml::from_str::<FrontMatter>("").unwrap_err(),
        });
    }

    let after_opening = &trimmed[3..];
    let closing_position = after_opening
        .find("\n---")
        .ok_or_else(|| SkillError::InvalidYaml {
            path: file_path.to_path_buf(),
            source: serde_yaml::from_str::<FrontMatter>("").unwrap_err(),
        })?;

    let yaml_raw = &after_opening[..closing_position];
    let body_start = closing_position + 4; // skip "\n---"
    let body = after_opening[body_start..].trim_start_matches('\n');

    let yaml_content = apply_aliases(yaml_raw, aliases);
    let frontmatter: FrontMatter =
        serde_yaml::from_str(&yaml_content).map_err(|source| SkillError::InvalidYaml {
            path: file_path.to_path_buf(),
            source,
        })?;

    Ok((frontmatter, body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_path() -> PathBuf {
        PathBuf::from("test.md")
    }

    fn no_aliases() -> HashMap<String, String> {
        HashMap::new()
    }

    #[test]
    fn valid_frontmatter_with_all_fields() {
        let content = "---\nname: my-skill\ndescription: A test skill\ndependencies:\n  - dep-a\ninputs:\n  - input-1\noutputs:\n  - output-1\n---\n\nBody content here.";
        let (frontmatter, body) =
            extract_frontmatter(content, &test_path(), &no_aliases()).unwrap();

        assert_eq!(frontmatter.name, "my-skill");
        assert_eq!(frontmatter.description, "A test skill");
        assert_eq!(frontmatter.dependencies, vec!["dep-a"]);
        assert_eq!(frontmatter.inputs, vec!["input-1"]);
        assert_eq!(frontmatter.outputs, vec!["output-1"]);
        assert_eq!(body, "Body content here.");
    }

    #[test]
    fn valid_frontmatter_with_minimal_fields() {
        let content = "---\nname: minimal\n---\n\nSome body.";
        let (frontmatter, body) =
            extract_frontmatter(content, &test_path(), &no_aliases()).unwrap();

        assert_eq!(frontmatter.name, "minimal");
        assert_eq!(frontmatter.description, "");
        assert!(frontmatter.dependencies.is_empty());
        assert_eq!(body, "Some body.");
    }

    #[test]
    fn missing_opening_delimiter() {
        let content = "name: no-delimiter\n---\nBody.";
        let result = extract_frontmatter(content, &test_path(), &no_aliases());
        assert!(result.is_err());
    }

    #[test]
    fn missing_closing_delimiter() {
        let content = "---\nname: unclosed\nBody without closing.";
        let result = extract_frontmatter(content, &test_path(), &no_aliases());
        assert!(result.is_err());
    }

    #[test]
    fn invalid_yaml_content() {
        let content = "---\n: : : broken yaml [[\n---\nBody.";
        let result = extract_frontmatter(content, &test_path(), &no_aliases());
        assert!(result.is_err());
    }

    #[test]
    fn empty_body_after_frontmatter() {
        let content = "---\nname: no-body\n---\n";
        let (frontmatter, body) =
            extract_frontmatter(content, &test_path(), &no_aliases()).unwrap();

        assert_eq!(frontmatter.name, "no-body");
        assert_eq!(body, "");
    }

    #[test]
    fn aliases_resolve_before_parsing() {
        let mut aliases = HashMap::new();
        aliases.insert("deps".to_string(), "dependencies".to_string());

        let content = "---\nname: aliased\ndeps:\n  - alpha\n  - beta\n---\n";
        let (frontmatter, _) = extract_frontmatter(content, &test_path(), &aliases).unwrap();

        assert_eq!(frontmatter.dependencies, vec!["alpha", "beta"]);
    }
}
