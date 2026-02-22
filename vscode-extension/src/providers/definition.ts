import * as vscode from "vscode";
import { Skill, findSkillByName } from "../scanner";

export class SkillDefinitionProvider implements vscode.DefinitionProvider {
  private skillLocations: Map<string, vscode.Uri> = new Map();

  constructor(private skills: Skill[]) {}

  updateSkills(skills: Skill[]): void {
    this.skills = skills;
  }

  updateLocations(locations: Map<string, vscode.Uri>): void {
    this.skillLocations = locations;
  }

  provideDefinition(
    document: vscode.TextDocument,
    position: vscode.Position,
  ): vscode.Location | undefined {
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

    const uri = this.skillLocations.get(skillName);
    if (!uri) {
      return undefined;
    }

    return new vscode.Location(uri, new vscode.Position(0, 0));
  }
}

export async function buildSkillLocations(
  skills: Skill[],
): Promise<Map<string, vscode.Uri>> {
  const locations = new Map<string, vscode.Uri>();

  const mdFiles = await vscode.workspace.findFiles("**/*.md");

  for (const uri of mdFiles) {
    const content = (await vscode.workspace.openTextDocument(uri)).getText();
    const nameMatch = content.match(/^---[\s\S]*?name:\s*(.+)[\s\S]*?---/m);
    if (nameMatch) {
      const name = nameMatch[1].trim();
      const skill = findSkillByName(skills, name);
      if (skill) {
        locations.set(name, uri);
      }
    }
  }

  return locations;
}
