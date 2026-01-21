use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub heading: String,
    pub level: u8,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownBody {
    pub raw: String,
    pub sections: Vec<Section>,
}

pub fn parse_body(content: &str) -> MarkdownBody {
    let mut sections = Vec::new();
    let mut current_heading: Option<String> = None;
    let mut current_level: u8 = 0;
    let mut current_lines: Vec<&str> = Vec::new();

    for line in content.lines() {
        if let Some((level, heading)) = parse_heading(line) {
            if let Some(heading_text) = current_heading.take() {
                sections.push(Section {
                    heading: heading_text,
                    level: current_level,
                    content: current_lines.join("\n").trim().to_string(),
                });
                current_lines.clear();
            }
            current_heading = Some(heading.to_string());
            current_level = level;
        } else if current_heading.is_some() {
            current_lines.push(line);
        }
    }

    if let Some(heading_text) = current_heading {
        sections.push(Section {
            heading: heading_text,
            level: current_level,
            content: current_lines.join("\n").trim().to_string(),
        });
    }

    MarkdownBody {
        raw: content.to_string(),
        sections,
    }
}

fn parse_heading(line: &str) -> Option<(u8, &str)> {
    let trimmed = line.trim_start();
    let hash_count = trimmed.bytes().take_while(|&byte| byte == b'#').count();

    if hash_count >= 2 && hash_count <= 6 {
        let rest = trimmed[hash_count..].trim();
        if !rest.is_empty() {
            return Some((hash_count as u8, rest));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_body() {
        let result = parse_body("");
        assert!(result.sections.is_empty());
        assert_eq!(result.raw, "");
    }

    #[test]
    fn multiple_sections() {
        let content = "## Description\n\nThis is the description.\n\n## Usage\n\nHow to use it.\n\n### Example\n\nSome example code.";
        let result = parse_body(content);

        assert_eq!(result.sections.len(), 3);

        assert_eq!(result.sections[0].heading, "Description");
        assert_eq!(result.sections[0].level, 2);
        assert_eq!(result.sections[0].content, "This is the description.");

        assert_eq!(result.sections[1].heading, "Usage");
        assert_eq!(result.sections[1].level, 2);
        assert_eq!(result.sections[1].content, "How to use it.");

        assert_eq!(result.sections[2].heading, "Example");
        assert_eq!(result.sections[2].level, 3);
        assert_eq!(result.sections[2].content, "Some example code.");
    }

    #[test]
    fn body_without_headings() {
        let content = "Just some text\nwithout any headings.";
        let result = parse_body(content);

        assert!(result.sections.is_empty());
        assert_eq!(result.raw, content);
    }

    #[test]
    fn section_with_empty_content() {
        let content = "## Empty Section\n## Next Section\n\nHas content.";
        let result = parse_body(content);

        assert_eq!(result.sections.len(), 2);
        assert_eq!(result.sections[0].heading, "Empty Section");
        assert_eq!(result.sections[0].content, "");
        assert_eq!(result.sections[1].content, "Has content.");
    }
}
