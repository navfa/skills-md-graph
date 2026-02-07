use std::fs;

use anyhow::Result;
use clap::Parser;

use skills_md_graph::cli::{Cli, Command};
use skills_md_graph::graph::build_graph;
use skills_md_graph::graph::dot::{render_dot, render_png};
use skills_md_graph::graph::stats::compute_stats;
use skills_md_graph::parser::scan_directory;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Scan { path, json } => {
            let skill_set = scan_directory(&path)?;

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
    }

    Ok(())
}
