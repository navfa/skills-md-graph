use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

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

    /// Run diagnostics on skill files (cycles, isolation, missing deps)
    Lint {
        /// Path to the directory containing skill files
        path: PathBuf,
    },

    /// Query the skill dependency graph
    Query {
        /// Path to the directory containing skill files
        path: PathBuf,

        /// Find skills that depend on this skill
        #[arg(long)]
        uses: Option<String>,

        /// Find all transitive dependencies of this skill
        #[arg(long)]
        deps: Option<String>,

        /// Find shortest path between two skills (comma-separated: "from,to")
        #[arg(long, value_delimiter = ',', num_args = 2)]
        path_between: Option<Vec<String>>,

        /// Output results as JSON
        #[arg(long)]
        json: bool,
    },

    /// Export the skill graph to external formats
    Export {
        /// Path to the directory containing skill files
        path: PathBuf,

        /// Export format
        #[arg(long, value_enum)]
        format: ExportFormatArg,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ExportFormatArg {
    Rdf,
    Cypher,
}
