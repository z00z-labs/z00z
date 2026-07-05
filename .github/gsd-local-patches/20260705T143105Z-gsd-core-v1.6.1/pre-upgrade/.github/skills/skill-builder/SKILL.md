---
name: skill-builder
description: Guide users through creating new skills by extracting reusable workflows from conversation, choosing the right structure, and validating the final skill.
---

# Skill Builder

This skill helps you create well-structured skills. Use it when the user wants to build a new reusable skill for a coding assistant, local automation flow, or project-specific tooling setup.

## When to Use This Skill

Invoke this skill when users:
- Want to create a new skill
- Ask how to build or structure a skill
- Need help with skill frontmatter or organization
- Want to validate an existing skill structure

## Source Of Truth

When multiple "create skill" prompts or helpers exist, this repository skill is the canonical source of truth for structure, validation, and file layout.

External prompts can still be useful, but only as thin conversation flow helpers. Keep these rules authoritative:
- Use this skill for directory layout, frontmatter rules, progressive disclosure, and validation.
- Use external prompts only for lightweight orchestration such as extracting a workflow from the active conversation.
- Do not replace repository-local rules with extension-shipped prompt defaults.

## Skill Creation Workflow

### Step 1: Extract the Workflow from Conversation First

Before asking questions, review the active conversation and check whether a reusable workflow already exists.

Extract these elements when possible:
1. The step-by-step process being followed
2. Decision points and branching logic
3. Quality gates, validation checks, or completion criteria
4. Repeated wording that should become trigger language

If the workflow is already clear, draft the skill immediately and avoid a full discovery questionnaire.

### Step 2: Clarify Only Missing Information

Ask only the unanswered questions. Prefer targeted clarification over a generic interview.

Use these prompts when needed:
1. **What outcome should this skill produce?**
2. **Should this skill be workspace-scoped or personal?**
3. **Should it be a quick checklist or a full multi-step workflow?**
4. **Does it need supporting files such as templates, scripts, or reference data?**

### Step 3: Determine Skill Location

Based on scope, choose the directory:
- **Project-specific**: `.github/skills/[skill-name]/` (within project root)
- **Personal/general**: `<user-skills-dir>/[skill-name]/` (available everywhere)

### Step 4: Choose the Smallest Useful Structure

Start with the smallest structure that fits the workflow. Do not create extra files unless they carry real value.

Use this progression:
- `SKILL.md` only for simple, self-contained behavior
- add `REFERENCE.md` for detailed technical context or long examples
- add `FORMS.md` for reusable templates and structured outputs
- add `scripts/` only when executable automation is genuinely needed
- add `resources/` for static templates, sample data, or reference material

### Step 5: Create Directory Structure

Use this pattern:
```
skill-name/
├── SKILL.md           (required: main instructions)
├── REFERENCE.md       (optional: detailed reference info)
├── FORMS.md           (optional: templates and forms)
├── scripts/           (optional: executable code)
│   ├── script1.py
│   └── script2.sh
└── resources/         (optional: templates, data files)
    └── template.txt
```

### Step 6: Write the Initial SKILL.md Draft

The SKILL.md must follow this structure:

```markdown
---
name: skill-name
description: Clear description of what this skill does and when to use it
---

# Skill Name

Main instructions for LLM go here.

## When to Use

Describe scenarios where this skill should be invoked.

## How It Works

Step-by-step instructions for LLM to follow.

## Examples

Provide examples of usage.
```

**Frontmatter Requirements:**
- `name`: Max 64 chars, lowercase letters/numbers/hyphens only (e.g., "excel-automation")
- `description`: Non-empty, max 1024 chars, should clearly indicate when to use the skill
  - **IMPORTANT:** Include BOTH natural language AND technical keywords
  - Natural language: How non-technical users would describe it ("fix broken code", "save data", "put online")
  - Technical terms: Jargon that developers use ("debug", "database schema", "deploy")
  - Example: "Auto-invoked when user wants to fix broken code, not working, or crashed. Also triggers on debug, stack trace, error analysis."

**Best Practices for Instructions:**
- Write like an "onboarding guide for a new team member"
- Use clear, step-by-step procedures
- Include contextual examples
- Reference supporting files instead of embedding everything
- Keep instructions under 5k tokens for efficient loading
- Use progressive disclosure: reference detailed info in REFERENCE.md
- Preserve the extracted workflow from the conversation instead of flattening it into generic advice
- Capture decision points explicitly when the workflow branches

### Step 7: Iterate on the Draft

After drafting the skill:
1. Identify the most ambiguous or weak parts
2. Ask only the smallest set of follow-up questions needed to remove ambiguity
3. Update the draft based on those answers
4. Re-check that the skill still reflects the original workflow accurately

This draft-first loop is usually better than collecting every detail up front.

### Step 8: Add Supporting Files (Optional)

**REFERENCE.md**: Detailed technical information that LLM can read when needed
- API documentation
- Detailed specifications
- Comprehensive examples
- Technical constraints

**FORMS.md**: Templates and structured formats
- Form templates
- Output format examples
- Structured data patterns

**scripts/**: Executable code for complex operations
- Python scripts for data processing
- Shell scripts for automation
- Utilities that LLM can invoke via bash

**resources/**: Static files and templates
- Document templates
- Sample data
- Reference materials

### Step 9: Validate the Skill

Check these requirements:
- [ ] `SKILL.md` exists with valid YAML frontmatter
- [ ] `name` is lowercase, alphanumeric with hyphens, max 64 chars
- [ ] `description` is clear, non-empty, max 1024 chars
- [ ] **Description includes natural language keywords** (not just technical jargon)
- [ ] The reusable workflow was extracted from the conversation when available
- [ ] Clarifying questions were limited to missing information
- [ ] Instructions are clear and actionable
- [ ] File references are correct
- [ ] No internet/API dependencies
- [ ] Only uses pre-installed packages

**Run automated validation:**
```bash
python .github/skills/skill-builder/scripts/validate-skill.py <path-to-skill-directory>
```

**Validation Scoring (29 points total):**
- SKILL.md exists: 1 point
- Valid frontmatter: 3 points
- Name validation: 2 points
- Description validation: 3 points
- Content structure: 5 points
- Token count <5000: 2 points
- Supporting files: 3 points
- Scripts directory: 2 points
- Industry standards: 3 points
- Examples: 2 points
- **Natural language keywords: 3 points** (NEW)

**Passing threshold:** 80% (24/29 points)

### Step 10: Test the Skill

After creation:
1. Verify the skill is discoverable (LLM should see it in metadata)
2. Test with a sample task that should trigger it
3. Ensure resources load correctly when referenced
4. Check that instructions are clear and complete
5. Confirm the produced behavior matches the original workflow you extracted

## Progressive Disclosure Strategy

Design skills to minimize token usage:

1. **Always Loaded (Metadata)**: ~100 tokens
   - Just the YAML frontmatter

2. **Loaded When Triggered (Instructions)**: <5k tokens
   - Core procedures from SKILL.md body

3. **Loaded As Needed (Resources)**: No token cost
   - Files LLM reads via bash commands
   - Scripts LLM executes when required
   - Templates LLM accesses on demand

## Common Skill Patterns

### Document Generation Skill
```
doc-generator/
├── SKILL.md          (generation instructions)
├── templates/        (document templates)
└── scripts/          (formatting scripts)
```

### Code Analysis Skill
```
code-analyzer/
├── SKILL.md          (analysis procedures)
├── REFERENCE.md      (pattern definitions)
└── scripts/          (analysis tools)
```

### Automation Skill
```
automation-helper/
├── SKILL.md          (automation workflows)
├── FORMS.md          (configuration templates)
└── scripts/          (automation scripts)
```

## Implementation Steps

When helping users build a skill:

1. **Extract Workflow**: Reuse the process already visible in the conversation when possible
2. **Clarify Gaps**: Ask only for missing outcome, scope, and depth details
3. **Plan Structure**: Choose the smallest file set that fits the skill
4. **Create Directory**: Make the skill directory in the correct location
5. **Write SKILL.md**: Create the first draft with proper frontmatter and instructions
6. **Iterate**: Tighten ambiguous sections with minimal follow-up questions
7. **Add Supporting Files**: Create REFERENCE.md, FORMS.md, scripts as needed
8. **Validate**: Run the validator and check manual quality gates
9. **Document Usage**: Explain how to trigger and test the skill
10. **Show Location**: Provide the full path to the created skill

## Validation Checklist

Before completing, verify:
- [ ] Directory created in correct location
- [ ] SKILL.md has valid frontmatter
- [ ] Name follows naming rules
- [ ] Description is clear and helpful
- [ ] Description includes natural language and technical trigger language
- [ ] The final instructions preserve the conversation-derived workflow
- [ ] Instructions are actionable
- [ ] Examples are provided
- [ ] Supporting files are organized logically
- [ ] No external dependencies
- [ ] File paths in instructions are correct

## Tips for Great Skills

1. **Clear Descriptions**: Make it obvious when LLM should use the skill
2. **Conversation First**: Draft from the real workflow in the chat before asking broad questions
3. **Step-by-Step**: Write procedures as numbered steps
4. **Examples First**: Show examples before explaining
5. **Minimize Context**: Use progressive disclosure extensively
6. **Self-Contained**: Don't assume external resources are available
7. **Tested Procedures**: Ensure instructions actually work
8. **Helpful Metadata**: Write descriptions that help with discovery

## Common Mistakes to Avoid

- Using uppercase or special characters in skill name
- Making descriptions too vague or too long
- Asking a full questionnaire when the workflow is already visible in the conversation
- Replacing repository-local rules with extension-shipped prompt defaults
- Embedding all content in SKILL.md instead of using supporting files
- Referencing non-existent files
- Assuming internet access or external APIs
- Creating overly complex skills that should be split
- Forgetting to include "when to use" guidance

## Reference Files

See the templates directory for:
- Basic skill template
- Advanced skill template with supporting files
- Example skills for common use cases

Use these local references when authoring or validating:
- `REFERENCE.md`
- `templates/basic-skill-template.md`
- `scripts/validate-skill.py`
