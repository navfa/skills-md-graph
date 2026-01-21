use std::fs;
use std::path::Path;

use crate::error::SkillError;
use crate::model::Skill;

pub mod frontmatter;
pub mod markdown;

pub use frontmatter::{FrontMatter, extract_frontmatter};
pub use markdown::{MarkdownBody, Section, parse_body};

pub fn parse_skill_file(file_path: &Path) -> Result<Skill, SkillError> {
    let content = fs::read_to_string(file_path).map_err(|source| SkillError::FileRead {
        path: file_path.to_path_buf(),
        source,
    })?;

    let (frontmatter, body_content) = extract_frontmatter(&content, file_path)?;
    let body = parse_body(body_content);

    Ok(Skill {
        name: frontmatter.name,
        description: frontmatter.description,
        dependencies: frontmatter.dependencies,
        inputs: frontmatter.inputs,
        outputs: frontmatter.outputs,
        body: body.raw,
    })
}
