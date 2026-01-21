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
