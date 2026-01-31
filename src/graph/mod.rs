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
