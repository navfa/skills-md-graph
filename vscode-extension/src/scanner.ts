import { execFile } from "child_process";
import * as vscode from "vscode";

export interface Skill {
  name: string;
  description: string;
  dependencies: string[];
  inputs: string[];
  outputs: string[];
}

export interface SkillSet {
  skills: Skill[];
  warnings: string[];
}

export function getCliPath(): string {
  const config = vscode.workspace.getConfiguration("skillGraph");
  return config.get<string>("cliPath", "skill-graph");
}

export function scanWorkspace(workspacePath: string): Promise<SkillSet> {
  const cliPath = getCliPath();

  return new Promise((resolve, reject) => {
    execFile(cliPath, ["scan", workspacePath, "--json"], (error, stdout, stderr) => {
      if (error) {
        reject(new Error(`skill-graph CLI failed: ${stderr || error.message}`));
        return;
      }

      try {
        const result: SkillSet = JSON.parse(stdout);
        resolve(result);
      } catch (parseError) {
        reject(new Error(`Failed to parse CLI output: ${parseError}`));
      }
    });
  });
}

export function findSkillByName(skills: Skill[], name: string): Skill | undefined {
  return skills.find((skill) => skill.name === name);
}
