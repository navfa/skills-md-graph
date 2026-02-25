use std::fmt::Write;

use crate::graph::SkillGraph;

pub fn render_turtle(graph: &SkillGraph) -> String {
    let mut output = String::new();

    writeln!(output, "@prefix skill: <http://example.org/skill/> .").unwrap();
    writeln!(
        output,
        "@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> ."
    )
    .unwrap();
    writeln!(
        output,
        "@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> ."
    )
    .unwrap();
    writeln!(output).unwrap();

    for node_index in graph.inner.node_indices() {
        let node = &graph.inner[node_index];
        let escaped_description = node.description.replace('"', "\\\"");

        writeln!(output, "skill:{} rdf:type skill:Skill ;", node.name).unwrap();
        writeln!(output, "    rdfs:label \"{}\" ;", node.name).unwrap();
        writeln!(output, "    rdfs:comment \"{}\" .", escaped_description).unwrap();
        writeln!(output).unwrap();
    }

    for edge_index in graph.inner.edge_indices() {
        let (source, target) = graph.inner.edge_endpoints(edge_index).unwrap();
        let source_name = &graph.inner[source].name;
        let target_name = &graph.inner[target].name;

        writeln!(
            output,
            "skill:{source_name} skill:dependsOn skill:{target_name} ."
        )
        .unwrap();
    }

    output
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
    fn turtle_contains_prefixes() {
        let skill_set = SkillSet {
            skills: vec![make_skill("alpha", vec![])],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let turtle = render_turtle(&graph);

        assert!(turtle.contains("@prefix skill:"));
        assert!(turtle.contains("@prefix rdf:"));
    }

    #[test]
    fn turtle_contains_skill_declaration() {
        let skill_set = SkillSet {
            skills: vec![make_skill("alpha", vec![])],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let turtle = render_turtle(&graph);

        assert!(turtle.contains("skill:alpha rdf:type skill:Skill"));
        assert!(turtle.contains("rdfs:label \"alpha\""));
    }

    #[test]
    fn turtle_contains_dependency_relation() {
        let skill_set = SkillSet {
            skills: vec![
                make_skill("alpha", vec![]),
                make_skill("beta", vec!["alpha"]),
            ],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let turtle = render_turtle(&graph);

        assert!(turtle.contains("skill:beta skill:dependsOn skill:alpha"));
    }
}
