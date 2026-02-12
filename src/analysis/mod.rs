pub mod cycles;
pub mod isolation;

use std::fmt;

use crate::graph::SkillGraph;

use cycles::detect_cycles;
use isolation::{SkillRole, classify_skills};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub enum Diagnostic {
    Cycle {
        skill_names: Vec<String>,
    },
    Isolated {
        skill_name: String,
    },
    MissingDependency {
        skill_name: String,
        dependency_name: String,
    },
}

impl Diagnostic {
    pub fn severity(&self) -> Severity {
        match self {
            Diagnostic::Cycle { .. } => Severity::Error,
            Diagnostic::MissingDependency { .. } => Severity::Warning,
            Diagnostic::Isolated { .. } => Severity::Info,
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Diagnostic::Cycle { skill_names } => {
                let chain = skill_names.join(" -> ");
                write!(formatter, "error: circular dependency: {chain}")
            }
            Diagnostic::Isolated { skill_name } => {
                write!(formatter, "info: skill \"{skill_name}\" is isolated (no connections)")
            }
            Diagnostic::MissingDependency {
                skill_name,
                dependency_name,
            } => {
                write!(
                    formatter,
                    "warning: \"{skill_name}\" depends on \"{dependency_name}\" which was not found"
                )
            }
        }
    }
}

pub fn lint(graph: &SkillGraph) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for cycle in detect_cycles(graph) {
        diagnostics.push(Diagnostic::Cycle {
            skill_names: cycle,
        });
    }

    for warning in &graph.warnings {
        if let Some((skill_part, dep_part)) = warning.split_once(": dependency \"") {
            if let Some(dependency_name) = dep_part.strip_suffix("\" not found in scanned skills")
            {
                diagnostics.push(Diagnostic::MissingDependency {
                    skill_name: skill_part.to_string(),
                    dependency_name: dependency_name.to_string(),
                });
            }
        }
    }

    for classification in classify_skills(graph) {
        if classification.role == SkillRole::Isolated {
            diagnostics.push(Diagnostic::Isolated {
                skill_name: classification.name,
            });
        }
    }

    diagnostics
}

pub fn has_errors(diagnostics: &[Diagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity() == Severity::Error)
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
    fn lint_clean_graph_has_no_diagnostics() {
        let skill_set = SkillSet {
            skills: vec![
                make_skill("alpha", vec![]),
                make_skill("beta", vec!["alpha"]),
            ],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let diagnostics = lint(&graph);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn lint_detects_cycle_as_error() {
        let skill_set = SkillSet {
            skills: vec![
                make_skill("alpha", vec!["beta"]),
                make_skill("beta", vec!["alpha"]),
            ],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let diagnostics = lint(&graph);

        assert!(has_errors(&diagnostics));
        assert!(diagnostics.iter().any(|diagnostic| matches!(diagnostic, Diagnostic::Cycle { .. })));
    }

    #[test]
    fn lint_detects_isolated_as_info() {
        let skill_set = SkillSet {
            skills: vec![
                make_skill("connected", vec![]),
                make_skill("consumer", vec!["connected"]),
                make_skill("lonely", vec![]),
            ],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let diagnostics = lint(&graph);

        let isolated: Vec<_> = diagnostics
            .iter()
            .filter(|diagnostic| matches!(diagnostic, Diagnostic::Isolated { .. }))
            .collect();
        assert_eq!(isolated.len(), 1);
        assert!(!has_errors(&diagnostics));
    }

    #[test]
    fn lint_detects_missing_dependency() {
        let skill_set = SkillSet {
            skills: vec![make_skill("alpha", vec!["nonexistent"])],
            warnings: vec![],
        };
        let graph = build_graph(&skill_set);
        let diagnostics = lint(&graph);

        let missing: Vec<_> = diagnostics
            .iter()
            .filter(|diagnostic| matches!(diagnostic, Diagnostic::MissingDependency { .. }))
            .collect();
        assert_eq!(missing.len(), 1);
    }

    #[test]
    fn cycle_display_shows_chain() {
        let diagnostic = Diagnostic::Cycle {
            skill_names: vec!["alpha".to_string(), "beta".to_string()],
        };
        let output = diagnostic.to_string();

        assert!(output.contains("circular dependency"));
        assert!(output.contains("alpha -> beta"));
    }

    #[test]
    fn has_errors_false_for_warnings_only() {
        let diagnostics = vec![Diagnostic::MissingDependency {
            skill_name: "a".to_string(),
            dependency_name: "b".to_string(),
        }];
        assert!(!has_errors(&diagnostics));
    }
}
