use std::collections::{HashSet, VecDeque};

use petgraph::Direction;

use crate::graph::SkillGraph;

/// Find all skills that directly depend on the given skill (reverse dependencies).
pub fn query_uses(graph: &SkillGraph, skill_name: &str) -> Option<Vec<String>> {
    let &node_index = graph.node_indices.get(skill_name)?;

    let dependents = graph
        .inner
        .neighbors_directed(node_index, Direction::Incoming)
        .map(|neighbor_index| graph.inner[neighbor_index].name.clone())
        .collect();

    Some(dependents)
}

/// Find all transitive dependencies of the given skill (DFS iterative).
pub fn query_deps(graph: &SkillGraph, skill_name: &str) -> Option<Vec<String>> {
    let &start_index = graph.node_indices.get(skill_name)?;

    let mut visited = HashSet::new();
    let mut stack = vec![start_index];
    let mut transitive_dependencies = Vec::new();

    while let Some(current_index) = stack.pop() {
        for neighbor_index in graph
            .inner
            .neighbors_directed(current_index, Direction::Outgoing)
        {
            if visited.insert(neighbor_index) {
                transitive_dependencies.push(graph.inner[neighbor_index].name.clone());
                stack.push(neighbor_index);
            }
        }
    }

    Some(transitive_dependencies)
}

/// Find the shortest path between two skills (BFS).
pub fn query_path(graph: &SkillGraph, from: &str, to: &str) -> Option<Vec<String>> {
    let &from_index = graph.node_indices.get(from)?;
    let &to_index = graph.node_indices.get(to)?;

    if from_index == to_index {
        return Some(vec![from.to_string()]);
    }

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    // Each entry: (current node, path so far)
    queue.push_back((from_index, vec![from.to_string()]));
    visited.insert(from_index);

    while let Some((current_index, current_path)) = queue.pop_front() {
        // Search both directions for reachability
        let neighbors: Vec<_> = graph
            .inner
            .neighbors_directed(current_index, Direction::Outgoing)
            .chain(
                graph
                    .inner
                    .neighbors_directed(current_index, Direction::Incoming),
            )
            .collect();

        for neighbor_index in neighbors {
            if !visited.insert(neighbor_index) {
                continue;
            }

            let mut next_path = current_path.clone();
            next_path.push(graph.inner[neighbor_index].name.clone());

            if neighbor_index == to_index {
                return Some(next_path);
            }

            queue.push_back((neighbor_index, next_path));
        }
    }

    None
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

    fn build_test_graph() -> SkillGraph {
        let skill_set = SkillSet {
            skills: vec![
                make_skill("base", vec![]),
                make_skill("middle", vec!["base"]),
                make_skill("top", vec!["middle"]),
                make_skill("lonely", vec![]),
            ],
            warnings: vec![],
        };
        build_graph(&skill_set)
    }

    #[test]
    fn query_uses_finds_direct_dependents() {
        let graph = build_test_graph();
        let users = query_uses(&graph, "base").unwrap();

        assert_eq!(users, vec!["middle"]);
    }

    #[test]
    fn query_uses_returns_empty_for_unused_skill() {
        let graph = build_test_graph();
        let users = query_uses(&graph, "top").unwrap();

        assert!(users.is_empty());
    }

    #[test]
    fn query_uses_returns_none_for_unknown_skill() {
        let graph = build_test_graph();

        assert!(query_uses(&graph, "nonexistent").is_none());
    }

    #[test]
    fn query_deps_finds_transitive_dependencies() {
        let graph = build_test_graph();
        let dependencies = query_deps(&graph, "top").unwrap();

        assert!(dependencies.contains(&"middle".to_string()));
        assert!(dependencies.contains(&"base".to_string()));
        assert_eq!(dependencies.len(), 2);
    }

    #[test]
    fn query_deps_returns_empty_for_leaf() {
        let graph = build_test_graph();
        let dependencies = query_deps(&graph, "base").unwrap();

        assert!(dependencies.is_empty());
    }

    #[test]
    fn query_deps_returns_none_for_unknown_skill() {
        let graph = build_test_graph();

        assert!(query_deps(&graph, "nonexistent").is_none());
    }

    #[test]
    fn query_path_finds_direct_connection() {
        let graph = build_test_graph();
        let path = query_path(&graph, "middle", "base").unwrap();

        assert_eq!(path, vec!["middle", "base"]);
    }

    #[test]
    fn query_path_finds_indirect_connection() {
        let graph = build_test_graph();
        let path = query_path(&graph, "top", "base").unwrap();

        assert_eq!(path.len(), 3);
        assert_eq!(path.first().unwrap(), "top");
        assert_eq!(path.last().unwrap(), "base");
    }

    #[test]
    fn query_path_same_node() {
        let graph = build_test_graph();
        let path = query_path(&graph, "base", "base").unwrap();

        assert_eq!(path, vec!["base"]);
    }

    #[test]
    fn query_path_returns_none_for_disconnected_nodes() {
        let graph = build_test_graph();
        let path = query_path(&graph, "lonely", "base");

        assert!(path.is_none());
    }

    #[test]
    fn query_path_returns_none_for_unknown_skill() {
        let graph = build_test_graph();

        assert!(query_path(&graph, "nonexistent", "base").is_none());
    }
}
