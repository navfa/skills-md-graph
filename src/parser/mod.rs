use std::fs;
use std::path::Path;

use walkdir::WalkDir;

use crate::error::SkillError;
use crate::model::{Skill, SkillSet};

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

pub fn scan_directory(directory_path: &Path) -> Result<SkillSet, SkillError> {
    if !directory_path.is_dir() {
        return Err(SkillError::DirectoryNotFound {
            path: directory_path.to_path_buf(),
        });
    }

    let mut skills = Vec::new();
    let mut warnings = Vec::new();

    let markdown_files: Vec<_> = WalkDir::new(directory_path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_type().is_file()
                && entry.path().extension().is_some_and(|ext| ext == "md")
        })
        .collect();

    if markdown_files.is_empty() {
        return Err(SkillError::NoSkillFiles {
            path: directory_path.to_path_buf(),
        });
    }

    for entry in markdown_files {
        let file_path = entry.path();
        match parse_skill_file(file_path) {
            Ok(skill) => skills.push(skill),
            Err(error) => {
                warnings.push(format!("skipping {}: {error}", file_path.display()));
            }
        }
    }

    Ok(SkillSet { skills, warnings })
}
