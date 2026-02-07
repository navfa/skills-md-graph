use petgraph::algo::is_cyclic_directed;
use petgraph::Direction;

use super::SkillGraph;

#[derive(Debug, Clone)]
pub struct GraphStats {
    pub skill_count: usize,
    pub edge_count: usize,
    pub orphan_skills: Vec<String>,
    pub has_cycles: bool,
}

pub fn compute_stats(graph: &SkillGraph) -> GraphStats {
    let skill_count = graph.inner.node_count();
    let edge_count = graph.inner.edge_count();
    let has_cycles = is_cyclic_directed(&graph.inner);

    let orphan_skills = graph
        .inner
        .node_indices()
        .filter(|&node_index| {
            let incoming = graph.inner.edges_directed(node_index, Direction::Incoming).count();
            let outgoing = graph.inner.edges_directed(node_index, Direction::Outgoing).count();
            incoming == 0 && outgoing == 0
        })
        .map(|node_index| graph.inner[node_index].name.clone())
        .collect();

    GraphStats {
        skill_count,
        edge_count,
        orphan_skills,
        has_cycles,
    }
}

impl std::fmt::Display for GraphStats {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(formatter, "Skills: {}", self.skill_count)?;
        writeln!(formatter, "Dependencies: {}", self.edge_count)?;
        writeln!(formatter, "Cycles detected: {}", if self.has_cycles { "yes" } else { "no" })?;

        if self.orphan_skills.is_empty() {
            writeln!(formatter, "Orphan skills: none")?;
        } else {
            writeln!(formatter, "Orphan skills ({}):", self.orphan_skills.len())?;
            for name in &self.orphan_skills {
                writeln!(formatter, "  - {name}")?;
            }
        }

        Ok(())
    }
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
    fn counts_skills_and_edges() {
        let skill_set = SkillSet {
            skills: vec![
                make_skill("alpha", vec![]),
                make_skill("beta", vec!["alpha"]),
                make_skill("gamma", vec!["alpha", "beta"]),
            ],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let stats = compute_stats(&graph);

        assert_eq!(stats.skill_count, 3);
        assert_eq!(stats.edge_count, 3);
    }

    #[test]
    fn detects_orphan_skills() {
        let skill_set = SkillSet {
            skills: vec![
                make_skill("connected", vec![]),
                make_skill("uses-connected", vec!["connected"]),
                make_skill("lonely", vec![]),
            ],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let stats = compute_stats(&graph);

        assert_eq!(stats.orphan_skills, vec!["lonely"]);
    }

    #[test]
    fn no_orphans_when_all_connected() {
        let skill_set = SkillSet {
            skills: vec![
                make_skill("alpha", vec![]),
                make_skill("beta", vec!["alpha"]),
            ],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let stats = compute_stats(&graph);

        assert!(stats.orphan_skills.is_empty());
    }

    #[test]
    fn detects_cycle() {
        let skill_set = SkillSet {
            skills: vec![
                make_skill("alpha", vec!["beta"]),
                make_skill("beta", vec!["alpha"]),
            ],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let stats = compute_stats(&graph);

        assert!(stats.has_cycles);
    }

    #[test]
    fn no_cycle_in_dag() {
        let skill_set = SkillSet {
            skills: vec![
                make_skill("alpha", vec![]),
                make_skill("beta", vec!["alpha"]),
                make_skill("gamma", vec!["beta"]),
            ],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let stats = compute_stats(&graph);

        assert!(!stats.has_cycles);
    }

    #[test]
    fn display_formats_correctly() {
        let skill_set = SkillSet {
            skills: vec![
                make_skill("alpha", vec![]),
                make_skill("beta", vec!["alpha"]),
                make_skill("lonely", vec![]),
            ],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let stats = compute_stats(&graph);
        let output = stats.to_string();

        assert!(output.contains("Skills: 3"));
        assert!(output.contains("Dependencies: 1"));
        assert!(output.contains("Cycles detected: no"));
        assert!(output.contains("lonely"));
    }
}
