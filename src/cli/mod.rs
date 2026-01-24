use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "skill-graph", about = "Analyze and graph skill dependencies")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Scan a directory for skill markdown files
    Scan {
        /// Path to the directory containing skill files
        path: PathBuf,

        /// Output results as JSON
        #[arg(long)]
        json: bool,
    },
}
