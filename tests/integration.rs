use std::path::Path;

use skills_md_graph::analysis::{Diagnostic, has_errors, lint};
use skills_md_graph::export::{ExportFormat, render_export};
use skills_md_graph::graph::build_graph;
use skills_md_graph::graph::dot::render_dot;
use skills_md_graph::graph::stats::compute_stats;
use skills_md_graph::parser::scan_directory;
use skills_md_graph::query::{query_deps, query_path, query_uses};

#[test]
fn scan_fixtures_finds_valid_skills() {
    let fixtures_path = Path::new("tests/fixtures");
    let skill_set = scan_directory(fixtures_path).unwrap();

    assert_eq!(skill_set.skills.len(), 2);

    let skill_names: Vec<&str> = skill_set.skills.iter().map(|s| s.name.as_str()).collect();
    assert!(skill_names.contains(&"rust-basics"));
    assert!(skill_names.contains(&"error-handling"));
}

#[test]
fn scan_fixtures_reports_warnings_for_invalid_files() {
    let fixtures_path = Path::new("tests/fixtures");
    let skill_set = scan_directory(fixtures_path).unwrap();

    assert!(!skill_set.warnings.is_empty());
    assert!(
        skill_set.warnings.iter().any(|warning| warning.contains("skill-invalid")),
        "expected a warning about skill-invalid.md"
    );
}

#[test]
fn scan_fixtures_parses_dependencies() {
    let fixtures_path = Path::new("tests/fixtures");
    let skill_set = scan_directory(fixtures_path).unwrap();

    let error_handling = skill_set
        .skills
        .iter()
        .find(|skill| skill.name == "error-handling")
        .expect("error-handling skill not found");

    assert_eq!(error_handling.dependencies, vec!["rust-basics"]);
}

#[test]
fn scan_fixtures_produces_valid_json() {
    let fixtures_path = Path::new("tests/fixtures");
    let skill_set = scan_directory(fixtures_path).unwrap();

    let json_output = serde_json::to_string_pretty(&skill_set).unwrap();
    let reparsed: serde_json::Value = serde_json::from_str(&json_output).unwrap();

    assert!(reparsed["skills"].is_array());
    assert_eq!(reparsed["skills"].as_array().unwrap().len(), 2);
}

#[test]
fn scan_nonexistent_directory_returns_error() {
    let result = scan_directory(Path::new("nonexistent/path"));
    assert!(result.is_err());
}

#[test]
fn graph_from_fixtures_has_correct_structure() {
    let skill_set = scan_directory(Path::new("tests/fixtures")).unwrap();
    let graph = build_graph(&skill_set);

    assert_eq!(graph.inner.node_count(), 2);
    assert_eq!(graph.inner.edge_count(), 1);
    assert!(graph.node_indices.contains_key("rust-basics"));
    assert!(graph.node_indices.contains_key("error-handling"));
}

#[test]
fn graph_dot_output_contains_expected_elements() {
    let skill_set = scan_directory(Path::new("tests/fixtures")).unwrap();
    let graph = build_graph(&skill_set);
    let dot = render_dot(&graph);

    assert!(dot.contains("digraph skills"));
    assert!(dot.contains("\"rust-basics\""));
    assert!(dot.contains("\"error-handling\""));
    assert!(dot.contains("\"error-handling\" -> \"rust-basics\""));
}

#[test]
fn graph_stats_from_fixtures_are_accurate() {
    let skill_set = scan_directory(Path::new("tests/fixtures")).unwrap();
    let graph = build_graph(&skill_set);
    let stats = compute_stats(&graph);

    assert_eq!(stats.skill_count, 2);
    assert_eq!(stats.edge_count, 1);
    assert!(!stats.has_cycles);
    assert!(stats.orphan_skills.is_empty());
}

#[test]
fn graph_warns_on_missing_dependency_in_fixtures() {
    let skill_set = scan_directory(Path::new("tests/fixtures")).unwrap();
    let graph = build_graph(&skill_set);

    // rust-basics has no deps, error-handling depends on rust-basics (exists)
    // no missing deps in our current fixtures
    assert!(graph.warnings.is_empty());
}

// --- Epic 3: analysis, query, export ---

#[test]
fn lint_fixtures_has_no_errors() {
    let skill_set = scan_directory(Path::new("tests/fixtures")).unwrap();
    let graph = build_graph(&skill_set);
    let diagnostics = lint(&graph);

    assert!(!has_errors(&diagnostics));
}

#[test]
fn lint_fixtures_reports_isolated_invalid_warning() {
    // skill-invalid.md is skipped at parse time, so only 2 valid skills remain
    // No isolated skills since error-handling depends on rust-basics
    let skill_set = scan_directory(Path::new("tests/fixtures")).unwrap();
    let graph = build_graph(&skill_set);
    let diagnostics = lint(&graph);

    let isolated: Vec<_> = diagnostics
        .iter()
        .filter(|diagnostic| matches!(diagnostic, Diagnostic::Isolated { .. }))
        .collect();
    assert!(isolated.is_empty());
}

#[test]
fn query_uses_on_fixtures() {
    let skill_set = scan_directory(Path::new("tests/fixtures")).unwrap();
    let graph = build_graph(&skill_set);

    let users = query_uses(&graph, "rust-basics").unwrap();
    assert_eq!(users, vec!["error-handling"]);
}

#[test]
fn query_deps_on_fixtures() {
    let skill_set = scan_directory(Path::new("tests/fixtures")).unwrap();
    let graph = build_graph(&skill_set);

    let dependencies = query_deps(&graph, "error-handling").unwrap();
    assert_eq!(dependencies, vec!["rust-basics"]);
}

#[test]
fn query_path_on_fixtures() {
    let skill_set = scan_directory(Path::new("tests/fixtures")).unwrap();
    let graph = build_graph(&skill_set);

    let path_result = query_path(&graph, "error-handling", "rust-basics").unwrap();
    assert_eq!(path_result, vec!["error-handling", "rust-basics"]);
}

#[test]
fn query_unknown_skill_returns_none() {
    let skill_set = scan_directory(Path::new("tests/fixtures")).unwrap();
    let graph = build_graph(&skill_set);

    assert!(query_uses(&graph, "nonexistent").is_none());
}

#[test]
fn export_rdf_from_fixtures() {
    let skill_set = scan_directory(Path::new("tests/fixtures")).unwrap();
    let graph = build_graph(&skill_set);
    let turtle = render_export(&graph, ExportFormat::Rdf);

    assert!(turtle.contains("@prefix skill:"));
    assert!(turtle.contains("skill:dependsOn"));
}

#[test]
fn export_cypher_from_fixtures() {
    let skill_set = scan_directory(Path::new("tests/fixtures")).unwrap();
    let graph = build_graph(&skill_set);
    let cypher = render_export(&graph, ExportFormat::Cypher);

    assert!(cypher.contains("CREATE (:Skill"));
    assert!(cypher.contains("DEPENDS_ON"));
}
