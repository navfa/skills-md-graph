use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum SkillError {
    #[error("invalid YAML frontmatter in {path}: {source}")]
    InvalidYaml {
        path: PathBuf,
        source: serde_yaml::Error,
    },

    #[error("failed to read file {path}: {source}")]
    FileRead {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("directory not found: {path}")]
    DirectoryNotFound { path: PathBuf },

    #[error("no skill files (.md) found in {path}")]
    NoSkillFiles { path: PathBuf },
}
