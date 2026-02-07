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
