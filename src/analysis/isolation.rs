use petgraph::Direction;

use crate::graph::SkillGraph;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillRole {
    /// No incoming or outgoing edges — completely disconnected
    Isolated,
    /// No outgoing edges (no dependencies) but used by others
    Leaf,
    /// No incoming edges (nobody depends on it) but has dependencies
    Root,
}

#[derive(Debug, Clone)]
pub struct SkillClassification {
    pub name: String,
    pub role: SkillRole,
}

pub fn classify_skills(graph: &SkillGraph) -> Vec<SkillClassification> {
    let mut classifications = Vec::new();

    for node_index in graph.inner.node_indices() {
        let incoming_count = graph
            .inner
            .edges_directed(node_index, Direction::Incoming)
            .count();
        let outgoing_count = graph
            .inner
            .edges_directed(node_index, Direction::Outgoing)
            .count();

        let role = match (incoming_count, outgoing_count) {
            (0, 0) => Some(SkillRole::Isolated),
            (_, 0) => Some(SkillRole::Leaf),
            (0, _) => Some(SkillRole::Root),
            _ => None,
        };

        if let Some(role) = role {
            classifications.push(SkillClassification {
                name: graph.inner[node_index].name.clone(),
                role,
            });
        }
    }

    classifications
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
    fn detects_isolated_skill() {
        let skill_set = SkillSet {
            skills: vec![
                make_skill("connected-a", vec![]),
                make_skill("connected-b", vec!["connected-a"]),
                make_skill("lonely", vec![]),
            ],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let classified = classify_skills(&graph);

        let isolated: Vec<_> = classified
            .iter()
            .filter(|classification| classification.role == SkillRole::Isolated)
            .collect();
        assert_eq!(isolated.len(), 1);
        assert_eq!(isolated[0].name, "lonely");
    }

    #[test]
    fn detects_leaf_skill() {
        let skill_set = SkillSet {
            skills: vec![
                make_skill("base", vec![]),
                make_skill("consumer", vec!["base"]),
            ],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let classified = classify_skills(&graph);

        let leaves: Vec<_> = classified
            .iter()
            .filter(|classification| classification.role == SkillRole::Leaf)
            .collect();
        assert_eq!(leaves.len(), 1);
        assert_eq!(leaves[0].name, "base");
    }

    #[test]
    fn detects_root_skill() {
        let skill_set = SkillSet {
            skills: vec![
                make_skill("base", vec![]),
                make_skill("consumer", vec!["base"]),
            ],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let classified = classify_skills(&graph);

        let roots: Vec<_> = classified
            .iter()
            .filter(|classification| classification.role == SkillRole::Root)
            .collect();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].name, "consumer");
    }

    #[test]
    fn middle_node_has_no_special_role() {
        let skill_set = SkillSet {
            skills: vec![
                make_skill("base", vec![]),
                make_skill("middle", vec!["base"]),
                make_skill("top", vec!["middle"]),
            ],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let classified = classify_skills(&graph);

        assert!(classified
            .iter()
            .all(|classification| classification.name != "middle"));
    }

    #[test]
    fn empty_graph_has_no_classifications() {
        let skill_set = SkillSet {
            skills: vec![],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let classified = classify_skills(&graph);

        assert!(classified.is_empty());
    }
}
