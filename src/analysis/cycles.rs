use petgraph::algo::tarjan_scc;

use crate::graph::SkillGraph;

/// A cycle is a list of skill names forming a circular dependency chain.
pub type Cycle = Vec<String>;

/// Detect all cycles in the graph using Tarjan's SCC algorithm.
/// Returns only strongly connected components of size > 1.
pub fn detect_cycles(graph: &SkillGraph) -> Vec<Cycle> {
    let components = tarjan_scc(&graph.inner);

    components
        .into_iter()
        .filter(|component| component.len() > 1)
        .map(|component| {
            component
                .into_iter()
                .map(|node_index| graph.inner[node_index].name.clone())
                .collect()
        })
        .collect()
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
    fn no_cycles_in_dag() {
        let skill_set = SkillSet {
            skills: vec![
                make_skill("alpha", vec![]),
                make_skill("beta", vec!["alpha"]),
                make_skill("gamma", vec!["beta"]),
            ],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);

        assert!(detect_cycles(&graph).is_empty());
    }

    #[test]
    fn detects_simple_cycle() {
        let skill_set = SkillSet {
            skills: vec![
                make_skill("alpha", vec!["beta"]),
                make_skill("beta", vec!["alpha"]),
            ],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let cycles = detect_cycles(&graph);

        assert_eq!(cycles.len(), 1);
        assert!(cycles[0].contains(&"alpha".to_string()));
        assert!(cycles[0].contains(&"beta".to_string()));
    }

    #[test]
    fn detects_three_node_cycle() {
        let skill_set = SkillSet {
            skills: vec![
                make_skill("alpha", vec!["gamma"]),
                make_skill("beta", vec!["alpha"]),
                make_skill("gamma", vec!["beta"]),
            ],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let cycles = detect_cycles(&graph);

        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].len(), 3);
    }

    #[test]
    fn ignores_non_cyclic_nodes() {
        let skill_set = SkillSet {
            skills: vec![
                make_skill("alpha", vec!["beta"]),
                make_skill("beta", vec!["alpha"]),
                make_skill("gamma", vec!["alpha"]),
            ],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let cycles = detect_cycles(&graph);

        assert_eq!(cycles.len(), 1);
        // gamma is not part of the cycle
        assert!(!cycles[0].contains(&"gamma".to_string()));
    }
}
