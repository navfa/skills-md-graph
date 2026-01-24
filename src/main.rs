use anyhow::Result;
use clap::Parser;

use skills_md_graph::cli::{Cli, Command};
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
    }

    Ok(())
}
