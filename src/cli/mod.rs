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

    /// Build and render a dependency graph
    Graph {
        /// Path to the directory containing skill files
        path: PathBuf,

        /// Write DOT output to a file instead of stdout
        #[arg(long)]
        output: Option<PathBuf>,

        /// Generate a PNG image (requires Graphviz installed)
        #[arg(long)]
        png: Option<PathBuf>,

        /// Display graph statistics
        #[arg(long)]
        stats: bool,
    },
}
