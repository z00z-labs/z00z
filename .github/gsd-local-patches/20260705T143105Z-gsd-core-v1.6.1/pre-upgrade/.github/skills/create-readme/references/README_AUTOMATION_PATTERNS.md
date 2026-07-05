# README Automation Patterns

Use this reference only when the user explicitly wants README generation automation,
editor integration, or Copilot-oriented scaffolding patterns.

## Purpose

Preserve advanced README workflow ideas that are useful but too specific to belong in the main skill body.

## Context-Aware README Generation

When automating README generation in editor tooling, collect:

- detected project type
- stack indicators from dependency files
- existing documentation fragments
- whether project-specific Copilot or editor context files exist

### Example Context Provider

```typescript
import { workspace } from 'vscode';
import { readFileSync, existsSync } from 'fs';
import { join } from 'path';

export class ReadmeContextProvider {
  private static readonly COPILOT_DIR = '.github/copilot';

  static getReadmeContext(): string {
    try {
      const projectRoot = workspace.rootPath || '';
      const context: any = {
        hasCopilotDir: existsSync(join(projectRoot, this.COPILOT_DIR)),
        techStack: this.analyzeTechStack(projectRoot),
        projectType: this.determineProjectType(projectRoot),
        existingDocs: this.findExistingDocumentation(projectRoot),
      };

      return `
## README Generation Context
- Project Type: ${context.projectType}
- Technology Stack: ${context.techStack.join(', ')}
- Existing Documentation: ${context.existingDocs.length} files found
- Copilot Integration: ${context.hasCopilotDir ? 'Enabled' : 'Not configured'}
`;
    } catch {
      return 'No specific project context detected.';
    }
  }
}
```

## Example README Commands

Use command wrappers only if the user is explicitly building editor automation.

```typescript
import { commands, window, workspace } from 'vscode';

export function registerReadmeCommands() {
  commands.registerCommand('readme.generate', async () => {
    const projectType = await window.showQuickPick(
      ['application', 'library', 'cli-tool', 'framework'],
      { placeHolder: 'Select project type' }
    );

    if (!projectType) {
      return;
    }

    const quality = await window.showQuickPick(
      ['basic', 'standard', 'comprehensive'],
      { placeHolder: 'Select README quality level' }
    );

    if (quality) {
      window.showInformationMessage('README.md generation started');
    }
  });
}
```

## When To Keep This Out Of The Main README Skill

- when the task is plain README writing
- when no editor integration is requested
- when these patterns would distract from the core documentation workflow
