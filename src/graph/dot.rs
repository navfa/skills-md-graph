use std::fmt::Write;
use std::path::Path;
use std::process::Command;

use crate::error::SkillError;

use super::SkillGraph;

pub fn render_dot(graph: &SkillGraph) -> String {
    let mut output = String::new();
    writeln!(output, "digraph skills {{").unwrap();
    writeln!(output, "    rankdir=LR;").unwrap();
    writeln!(output, "    node [shape=box, style=rounded];").unwrap();
    writeln!(output).unwrap();

    for node_index in graph.inner.node_indices() {
        let node = &graph.inner[node_index];
        let escaped_label = node.name.replace('"', "\\\"");
        writeln!(
            output,
            "    \"{}\" [label=\"{}\"];",
            escaped_label, escaped_label
        )
        .unwrap();
    }

    writeln!(output).unwrap();

    for edge in graph.inner.edge_indices() {
        let (source, target) = graph.inner.edge_endpoints(edge).unwrap();
        let source_name = &graph.inner[source].name;
        let target_name = &graph.inner[target].name;
        writeln!(output, "    \"{}\" -> \"{}\";", source_name, target_name).unwrap();
    }

    writeln!(output, "}}").unwrap();
    output
}

pub fn render_png(graph: &SkillGraph, output_path: &Path) -> Result<(), SkillError> {
    let dot_content = render_dot(graph);

    let child = Command::new("dot")
        .args(["-Tpng", "-o"])
        .arg(output_path)
        .stdin(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();

    let mut child = match child {
        Ok(child) => child,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(SkillError::GraphvizNotFound);
        }
        Err(error) => {
            return Err(SkillError::GraphvizFailed {
                message: error.to_string(),
            });
        }
    };

    use std::io::Write as IoWrite;
    if let Some(ref mut stdin) = child.stdin {
        stdin
            .write_all(dot_content.as_bytes())
            .map_err(|error| SkillError::GraphvizFailed {
                message: error.to_string(),
            })?;
    }
    drop(child.stdin.take());

    let output = child
        .wait_with_output()
        .map_err(|error| SkillError::GraphvizFailed {
            message: error.to_string(),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SkillError::GraphvizFailed {
            message: stderr.into_owned(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::build_graph;
    use crate::model::{Skill, SkillSet};

    fn make_skill(name: &str, dependencies: Vec<&str>) -> Skill {
        Skill {
            name: name.to_string(),
            description: format!("{name} description"),
            dependencies: dependencies.into_iter().map(String::from).collect(),
            inputs: vec![],
            outputs: vec![],
            body: String::new(),
        }
    }

    #[test]
    fn dot_contains_digraph_header() {
        let skill_set = SkillSet {
            skills: vec![make_skill("alpha", vec![])],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let dot = render_dot(&graph);

        assert!(dot.starts_with("digraph skills {"));
        assert!(dot.contains("rankdir=LR"));
        assert!(dot.contains("shape=box, style=rounded"));
    }

    #[test]
    fn dot_contains_node_declarations() {
        let skill_set = SkillSet {
            skills: vec![make_skill("alpha", vec![]), make_skill("beta", vec![])],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let dot = render_dot(&graph);

        assert!(dot.contains("\"alpha\" [label=\"alpha\"]"));
        assert!(dot.contains("\"beta\" [label=\"beta\"]"));
    }

    #[test]
    fn dot_contains_edges() {
        let skill_set = SkillSet {
            skills: vec![
                make_skill("alpha", vec![]),
                make_skill("beta", vec!["alpha"]),
            ],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let dot = render_dot(&graph);

        assert!(dot.contains("\"beta\" -> \"alpha\""));
    }

    #[test]
    fn dot_empty_graph() {
        let skill_set = SkillSet {
            skills: vec![],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let dot = render_dot(&graph);

        assert!(dot.contains("digraph skills {"));
        assert!(dot.ends_with("}\n"));
        assert!(!dot.contains("->"));
    }
}
