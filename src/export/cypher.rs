use std::fmt::Write;

use crate::graph::SkillGraph;

pub fn render_cypher(graph: &SkillGraph) -> String {
    let mut output = String::new();

    for node_index in graph.inner.node_indices() {
        let node = &graph.inner[node_index];
        let escaped_description = node.description.replace('\'', "\\'");

        writeln!(
            output,
            "CREATE (:Skill {{name: '{}', description: '{}'}});",
            node.name, escaped_description
        )
        .unwrap();
    }

    if graph.inner.node_count() > 0 && graph.inner.edge_count() > 0 {
        writeln!(output).unwrap();
    }

    for edge_index in graph.inner.edge_indices() {
        let (source, target) = graph.inner.edge_endpoints(edge_index).unwrap();
        let source_name = &graph.inner[source].name;
        let target_name = &graph.inner[target].name;

        writeln!(
            output,
            "MATCH (a:Skill {{name: '{source_name}'}}), (b:Skill {{name: '{target_name}'}}) CREATE (a)-[:DEPENDS_ON]->(b);"
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
    fn cypher_creates_nodes() {
        let skill_set = SkillSet {
            skills: vec![make_skill("alpha", vec![])],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let cypher = render_cypher(&graph);

        assert!(cypher.contains("CREATE (:Skill {name: 'alpha'"));
    }

    #[test]
    fn cypher_creates_relationships() {
        let skill_set = SkillSet {
            skills: vec![
                make_skill("alpha", vec![]),
                make_skill("beta", vec!["alpha"]),
            ],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let cypher = render_cypher(&graph);

        assert!(cypher.contains("MATCH (a:Skill {name: 'beta'}), (b:Skill {name: 'alpha'}) CREATE (a)-[:DEPENDS_ON]->(b)"));
    }

    #[test]
    fn cypher_empty_graph() {
        let skill_set = SkillSet {
            skills: vec![],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let cypher = render_cypher(&graph);

        assert!(cypher.is_empty());
    }
}
