use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use indicatif::{ProgressBar, ProgressStyle};
use tokio::sync::Semaphore;
use walkdir::WalkDir;

use crate::config::ScanConfig;
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

    let no_aliases = HashMap::new();
    let (frontmatter, body_content) = extract_frontmatter(&content, file_path, &no_aliases)?;
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

fn collect_files(directory_path: &Path, extensions: &[String]) -> Result<Vec<PathBuf>, SkillError> {
    if !directory_path.is_dir() {
        return Err(SkillError::DirectoryNotFound {
            path: directory_path.to_path_buf(),
        });
    }

    let files: Vec<PathBuf> = WalkDir::new(directory_path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_type().is_file()
                && entry.path().extension().is_some_and(|ext| {
                    extensions
                        .iter()
                        .any(|e| e == ext.to_string_lossy().as_ref())
                })
        })
        .map(|entry| entry.into_path())
        .collect();

    if files.is_empty() {
        return Err(SkillError::NoSkillFiles {
            path: directory_path.to_path_buf(),
        });
    }

    Ok(files)
}

pub fn scan_directory(directory_path: &Path) -> Result<SkillSet, SkillError> {
    let extensions = vec!["md".to_string()];
    let files = collect_files(directory_path, &extensions)?;

    let mut skills = Vec::new();
    let mut warnings = Vec::new();

    for file_path in &files {
        match parse_skill_file(file_path) {
            Ok(skill) => skills.push(skill),
            Err(error) => {
                warnings.push(format!("skipping {}: {error}", file_path.display()));
            }
        }
    }

    Ok(SkillSet { skills, warnings })
}

/// Async scan with configurable parallelism and optional progress bar.
pub async fn scan_directory_async(
    directory_path: &Path,
    scan_config: &ScanConfig,
    aliases: &HashMap<String, String>,
    show_progress: bool,
) -> Result<SkillSet, SkillError> {
    let files = collect_files(directory_path, &scan_config.extensions)?;
    let total = files.len() as u64;

    let progress = if show_progress {
        let bar = ProgressBar::new(total);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40} {pos}/{len} {msg}")
                .expect("valid progress bar template"),
        );
        Some(bar)
    } else {
        None
    };

    let semaphore = Arc::new(Semaphore::new(scan_config.workers));
    let aliases = Arc::new(aliases.clone());
    let progress = Arc::new(progress);

    let mut handles = Vec::with_capacity(files.len());

    for file_path in files {
        let sem = semaphore.clone();
        let aliases = aliases.clone();
        let progress = progress.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.expect("semaphore closed");
            let content = tokio::fs::read_to_string(&file_path).await;

            let result = match content {
                Ok(content) => {
                    let (frontmatter, body_content) =
                        extract_frontmatter(&content, &file_path, &aliases)?;
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
                Err(source) => Err(SkillError::FileRead {
                    path: file_path,
                    source,
                }),
            };

            if let Some(bar) = progress.as_ref() {
                bar.inc(1);
            }

            result
        });

        handles.push(handle);
    }

    let mut skills = Vec::new();
    let mut warnings = Vec::new();

    for handle in handles {
        match handle.await.expect("task panicked") {
            Ok(skill) => skills.push(skill),
            Err(error) => {
                warnings.push(format!("skipping: {error}"));
            }
        }
    }

    if let Some(bar) = Arc::try_unwrap(progress).ok().flatten() {
        bar.finish_with_message("done");
    }

    Ok(SkillSet { skills, warnings })
}
