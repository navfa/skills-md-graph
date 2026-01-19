use std::path::Path;

use serde::{Deserialize, Serialize};

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

    let yaml_content = &after_opening[..closing_position];
    let body_start = closing_position + 4; // skip "\n---"
    let body = after_opening[body_start..].trim_start_matches('\n');

    let frontmatter: FrontMatter =
        serde_yaml::from_str(yaml_content).map_err(|source| SkillError::InvalidYaml {
            path: file_path.to_path_buf(),
            source,
        })?;

    Ok((frontmatter, body))
}
