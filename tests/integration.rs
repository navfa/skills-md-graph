use std::path::Path;

use skills_md_graph::parser::scan_directory;

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
