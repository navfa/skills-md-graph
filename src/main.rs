use std::fs;
use std::process;

use anyhow::Result;
use clap::Parser;

use skills_md_graph::analysis::{has_errors, lint};
use skills_md_graph::cli::{Cli, Command, ExportFormatArg};
use skills_md_graph::config::load_config;
use skills_md_graph::export::{ExportFormat, render_export};
use skills_md_graph::graph::build_graph;
use skills_md_graph::graph::dot::{render_dot, render_png};
use skills_md_graph::graph::stats::compute_stats;
use skills_md_graph::parser::{scan_directory, scan_directory_async};
use skills_md_graph::query::{query_deps, query_path, query_uses};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Scan {
            path,
            json,
            workers,
            progress,
            config,
        } => {
            let mut cfg = load_config(config.as_deref(), &path);
            if let Some(w) = workers {
                cfg.scan.workers = w;
            }

            let skill_set =
                scan_directory_async(&path, &cfg.scan, &cfg.schema.aliases, progress).await?;

            if json {
                println!("{}", serde_json::to_string_pretty(&skill_set)?);
            } else {
                println!("Found {} skill(s):", skill_set.skills.len());
                for skill in &skill_set.skills {
                    println!("  - {} : {}", skill.name, skill.description);
                }
                for warning in &skill_set.warnings {
                    eprintln!("  warning: {warning}");
                }
            }
        }

        Command::Graph {
            path,
            output,
            png,
            stats,
        } => {
            let skill_set = scan_directory(&path)?;
            let graph = build_graph(&skill_set);

            for warning in &graph.warnings {
                eprintln!("warning: {warning}");
            }

            if let Some(png_path) = png {
                render_png(&graph, &png_path)?;
                eprintln!("PNG written to {}", png_path.display());
            }

            let dot_output = render_dot(&graph);
            match output {
                Some(file_path) => {
                    fs::write(&file_path, &dot_output)?;
                    eprintln!("DOT written to {}", file_path.display());
                }
                None => print!("{dot_output}"),
            }

            if stats {
                let graph_stats = compute_stats(&graph);
                eprintln!("\n{graph_stats}");
            }
        }

        Command::Lint { path } => {
            let skill_set = scan_directory(&path)?;
            let graph = build_graph(&skill_set);
            let diagnostics = lint(&graph);

            if diagnostics.is_empty() {
                println!("No issues found.");
            } else {
                for diagnostic in &diagnostics {
                    println!("{diagnostic}");
                }

                if has_errors(&diagnostics) {
                    process::exit(1);
                }
            }
        }

        Command::Query {
            path,
            uses,
            deps,
            path_between,
            json,
        } => {
            let skill_set = scan_directory(&path)?;
            let graph = build_graph(&skill_set);

            let results = if let Some(skill_name) = uses {
                match query_uses(&graph, &skill_name) {
                    Some(names) => names,
                    None => {
                        eprintln!("skill \"{skill_name}\" not found");
                        process::exit(1);
                    }
                }
            } else if let Some(skill_name) = deps {
                match query_deps(&graph, &skill_name) {
                    Some(names) => names,
                    None => {
                        eprintln!("skill \"{skill_name}\" not found");
                        process::exit(1);
                    }
                }
            } else if let Some(nodes) = path_between {
                let from = &nodes[0];
                let to = &nodes[1];
                match query_path(&graph, from, to) {
                    Some(path_result) => path_result,
                    None => {
                        eprintln!("no path found between \"{from}\" and \"{to}\"");
                        process::exit(1);
                    }
                }
            } else {
                eprintln!("specify one of --uses, --deps, or --path-between");
                process::exit(1);
            };

            if json {
                println!("{}", serde_json::to_string_pretty(&results)?);
            } else if results.is_empty() {
                println!("No results.");
            } else {
                for name in &results {
                    println!("  - {name}");
                }
            }
        }

        Command::Export { path, format } => {
            let skill_set = scan_directory(&path)?;
            let graph = build_graph(&skill_set);

            let export_format = match format {
                ExportFormatArg::Rdf => ExportFormat::Rdf,
                ExportFormatArg::Cypher => ExportFormat::Cypher,
            };

            print!("{}", render_export(&graph, export_format));
        }
    }

    Ok(())
}
