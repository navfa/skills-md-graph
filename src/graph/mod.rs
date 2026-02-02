use std::collections::HashMap;

use petgraph::graph::{DiGraph, NodeIndex};

use crate::model::SkillSet;

#[derive(Debug, Clone)]
pub struct SkillNode {
    pub name: String,
    pub description: String,
}

pub struct SkillGraph {
    pub inner: DiGraph<SkillNode, ()>,
    pub node_indices: HashMap<String, NodeIndex>,
    pub warnings: Vec<String>,
}

pub fn build_graph(skill_set: &SkillSet) -> SkillGraph {
    let mut graph = DiGraph::new();
    let mut node_indices = HashMap::new();
    let mut warnings = Vec::new();

    for skill in &skill_set.skills {
        let node = SkillNode {
            name: skill.name.clone(),
            description: skill.description.clone(),
        };
        let index = graph.add_node(node);
        node_indices.insert(skill.name.clone(), index);
    }

    for skill in &skill_set.skills {
        let source_index = node_indices[&skill.name];

        for dependency_name in &skill.dependencies {
            match node_indices.get(dependency_name) {
                Some(&target_index) => {
                    graph.add_edge(source_index, target_index, ());
                }
                None => {
                    warnings.push(format!(
                        "{}: dependency \"{}\" not found in scanned skills",
                        skill.name, dependency_name
                    ));
                }
            }
        }
    }

    SkillGraph {
        inner: graph,
        node_indices,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Skill;

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
    fn builds_nodes_for_each_skill() {
        let skill_set = SkillSet {
            skills: vec![make_skill("alpha", vec![]), make_skill("beta", vec![])],
            warnings: vec![],
        };

        let graph = build_graph(&skill_set);

        assert_eq!(graph.inner.node_count(), 2);
        assert!(graph.node_indices.contains_key("alpha"));
        assert!(graph.node_indices.contains_key("beta"));
    }

    #[test]
    fn creates_edges_for_valid_dependencies() {
        let skill_set = SkillSet {
            skills: vec![
                make_skill("alpha", vec![]),
                make_skill("beta", vec!["alpha"]),
            ],
            warnings: vec![],
        };

        let graph = build_graph(&skill_set);

        assert_eq!(graph.inner.edge_count(), 1);
        let beta_index = graph.node_indices["beta"];
        let alpha_index = graph.node_indices["alpha"];
        assert!(graph.inner.contains_edge(beta_index, alpha_index));
    }

    #[test]
    fn warns_on_missing_dependency() {
        let skill_set = SkillSet {
            skills: vec![make_skill("alpha", vec!["nonexistent"])],
            warnings: vec![],
        };

        let graph = build_graph(&skill_set);

        assert_eq!(graph.inner.edge_count(), 0);
        assert_eq!(graph.warnings.len(), 1);
        assert!(graph.warnings[0].contains("nonexistent"));
    }

    #[test]
    fn handles_empty_skill_set() {
        let skill_set = SkillSet {
            skills: vec![],
            warnings: vec![],
        };

        let graph = build_graph(&skill_set);

        assert_eq!(graph.inner.node_count(), 0);
        assert_eq!(graph.inner.edge_count(), 0);
        assert!(graph.warnings.is_empty());
    }

    #[test]
    fn handles_diamond_dependency() {
        let skill_set = SkillSet {
            skills: vec![
                make_skill("base", vec![]),
                make_skill("left", vec!["base"]),
                make_skill("right", vec!["base"]),
                make_skill("top", vec!["left", "right"]),
            ],
            warnings: vec![],
        };

        let graph = build_graph(&skill_set);

        assert_eq!(graph.inner.node_count(), 4);
        assert_eq!(graph.inner.edge_count(), 4);
        assert!(graph.warnings.is_empty());
    }
}
