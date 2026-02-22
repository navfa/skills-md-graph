import * as vscode from "vscode";
import { Skill, findSkillByName } from "../scanner";

export class SkillHoverProvider implements vscode.HoverProvider {
  constructor(private skills: Skill[]) {}

  updateSkills(skills: Skill[]): void {
    this.skills = skills;
  }

  provideHover(
    document: vscode.TextDocument,
    position: vscode.Position,
  ): vscode.Hover | undefined {
    const line = document.lineAt(position.line).text;

    // Match skill names in YAML dependency lists: "  - skill-name"
    const depMatch = line.match(/^\s+-\s+(.+)$/);
    if (!depMatch) {
      return undefined;
    }

    const skillName = depMatch[1].trim();
    const skill = findSkillByName(this.skills, skillName);
    if (!skill) {
      return undefined;
    }

    const markdown = new vscode.MarkdownString();
    markdown.appendMarkdown(`**${skill.name}**\n\n`);
    if (skill.description) {
      markdown.appendMarkdown(`${skill.description}\n\n`);
    }
    if (skill.dependencies.length > 0) {
      markdown.appendMarkdown(`**Dependencies:** ${skill.dependencies.join(", ")}\n`);
    }

    return new vscode.Hover(markdown);
  }
}
