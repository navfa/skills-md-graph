use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum SkillError {
    #[error("{path}: invalid YAML frontmatter — {source}\n  hint: ensure the file starts with `---` and contains valid YAML")]
    InvalidYaml {
        path: PathBuf,
        source: serde_yaml::Error,
    },

    #[error("{path}: failed to read file — {source}\n  hint: check that the file exists and is readable")]
    FileRead {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("directory not found: {path}\n  hint: provide a valid directory path")]
    DirectoryNotFound { path: PathBuf },

    #[error("no .md skill files found in {path}\n  hint: add markdown files with YAML frontmatter to this directory")]
    NoSkillFiles { path: PathBuf },
}
