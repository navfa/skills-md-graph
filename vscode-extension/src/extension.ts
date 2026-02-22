import * as vscode from "vscode";
import { scanWorkspace } from "./scanner";
import { SkillHoverProvider } from "./providers/hover";
import { SkillDefinitionProvider, buildSkillLocations } from "./providers/definition";
import { SkillDiagnosticProvider } from "./providers/diagnostic";

const MARKDOWN_SELECTOR: vscode.DocumentSelector = { language: "markdown", scheme: "file" };

export function activate(context: vscode.ExtensionContext): void {
  const hoverProvider = new SkillHoverProvider([]);
  const definitionProvider = new SkillDefinitionProvider([]);
  const diagnosticProvider = new SkillDiagnosticProvider();

  context.subscriptions.push(
    vscode.languages.registerHoverProvider(MARKDOWN_SELECTOR, hoverProvider),
    vscode.languages.registerDefinitionProvider(MARKDOWN_SELECTOR, definitionProvider),
    diagnosticProvider,
  );

  async function refresh(): Promise<void> {
    const workspaceFolders = vscode.workspace.workspaceFolders;
    if (!workspaceFolders || workspaceFolders.length === 0) {
      return;
    }

    const workspacePath = workspaceFolders[0].uri.fsPath;

    try {
      const result = await scanWorkspace(workspacePath);

      hoverProvider.updateSkills(result.skills);
      definitionProvider.updateSkills(result.skills);
      diagnosticProvider.updateSkills(result.skills);

      const locations = await buildSkillLocations(result.skills);
      definitionProvider.updateLocations(locations);

      // Refresh diagnostics for all open markdown documents
      for (const editor of vscode.window.visibleTextEditors) {
        if (editor.document.languageId === "markdown") {
          await diagnosticProvider.refreshDiagnostics(editor.document);
        }
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      vscode.window.showWarningMessage(`Skill Graph: ${message}`);
    }
  }

  // Initial scan
  refresh();

  // Re-scan on markdown file save
  context.subscriptions.push(
    vscode.workspace.onDidSaveTextDocument((document) => {
      if (document.languageId === "markdown") {
        refresh();
      }
    }),
  );

  // Update diagnostics on document open/change
  context.subscriptions.push(
    vscode.workspace.onDidOpenTextDocument((document) => {
      if (document.languageId === "markdown") {
        diagnosticProvider.refreshDiagnostics(document);
      }
    }),
    vscode.workspace.onDidChangeTextDocument((event) => {
      if (event.document.languageId === "markdown") {
        diagnosticProvider.refreshDiagnostics(event.document);
      }
    }),
    vscode.workspace.onDidCloseTextDocument((document) => {
      diagnosticProvider.clearDiagnostics(document.uri);
    }),
  );
}

export function deactivate(): void {
  // cleanup handled by disposables
}
