import * as vscode from "vscode";
import { Skill } from "../scanner";

export class SkillDiagnosticProvider {
  private diagnosticCollection: vscode.DiagnosticCollection;
  private skills: Skill[] = [];

  constructor() {
    this.diagnosticCollection = vscode.languages.createDiagnosticCollection("skill-graph");
  }

  updateSkills(skills: Skill[]): void {
    this.skills = skills;
  }

  async refreshDiagnostics(document: vscode.TextDocument): Promise<void> {
    if (document.languageId !== "markdown") {
      return;
    }

    const diagnostics: vscode.Diagnostic[] = [];
    const knownNames = new Set(this.skills.map((s) => s.name));
    const text = document.getText();
    const lines = text.split("\n");

    let inFrontmatter = false;
    let inDependencies = false;

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];

      if (line.trim() === "---") {
        if (!inFrontmatter) {
          inFrontmatter = true;
        } else {
          break; // end of frontmatter
        }
        continue;
      }

      if (!inFrontmatter) {
        continue;
      }

      if (line.match(/^dependencies:\s*$/)) {
        inDependencies = true;
        continue;
      }

      if (inDependencies) {
        const depMatch = line.match(/^\s+-\s+(.+)$/);
        if (!depMatch) {
          inDependencies = false;
          continue;
        }

        const depName = depMatch[1].trim();
        if (!knownNames.has(depName)) {
          const startChar = line.indexOf(depName);
          const range = new vscode.Range(i, startChar, i, startChar + depName.length);
          const diagnostic = new vscode.Diagnostic(
            range,
            `Unknown dependency: "${depName}"`,
            vscode.DiagnosticSeverity.Warning,
          );
          diagnostic.source = "skill-graph";
          diagnostics.push(diagnostic);
        }
      }
    }

    this.diagnosticCollection.set(document.uri, diagnostics);
  }

  clearDiagnostics(uri: vscode.Uri): void {
    this.diagnosticCollection.delete(uri);
  }

  dispose(): void {
    this.diagnosticCollection.dispose();
  }
}
