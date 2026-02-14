pub mod cypher;
pub mod rdf;

use crate::graph::SkillGraph;

#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Rdf,
    Cypher,
}

pub fn render_export(graph: &SkillGraph, format: ExportFormat) -> String {
    match format {
        ExportFormat::Rdf => rdf::render_turtle(graph),
        ExportFormat::Cypher => cypher::render_cypher(graph),
    }
}
