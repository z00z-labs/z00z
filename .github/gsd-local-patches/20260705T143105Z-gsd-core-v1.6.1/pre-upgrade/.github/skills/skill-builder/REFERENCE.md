# Skill Builder Reference

Comprehensive reference for creating reusable skills for coding assistants
and project tooling.

## Frontmatter Specification

### name (required)

- **Type**: string
- **Max Length**: 64 characters
- **Format**: lowercase letters, numbers, hyphens only
- **Pattern**: `^[a-z0-9-]+$`

Valid examples:

- `excel-automation`
- `code-reviewer`
- `api-doc-generator`
- `test-suite-builder`

Invalid examples:

- `Excel_Automation` (uppercase and underscore)
- `code reviewer` (space)
- `api.doc.generator` (dots)
- `Test-Suite!` (uppercase and special char)

### description (required)

- **Type**: string
- **Min Length**: 1 character
- **Max Length**: 1024 characters
- **Purpose**: help the host assistant decide when to invoke the skill

Good descriptions with natural language keywords:

```yaml
description: Auto-invoked when user wants to fix broken code, not working, crashed, or error message. Also triggers on technical terms like debug, stack trace, error analysis, production incidents.
```

```yaml
description: Auto-invoked when storing data, saving information, organizing data, or database is slow. Also triggers on database design, schema, query performance, migrations.
```

```yaml
description: Auto-invoked when user wants to put app online, make it live, publish site, or launch app. Also triggers on deployment, CI/CD, Docker, Kubernetes, production.
```

Why this works:

- Natural language: `fix broken code` matches how non-technical users speak.
- Technical terms: `stack trace` matches how developers speak.
- Both audiences can trigger the skill.

Poor descriptions:

```yaml
description: Excel stuff
# Too vague, unclear when to use
```

```yaml
description: Handles database schema migrations and query optimization
# Only technical jargon; non-technical users won't trigger this
```

```yaml
description: A comprehensive, enterprise-grade, production-ready solution for automated generation of Excel workbooks with advanced formula support, complex chart creation, multi-sheet management, conditional formatting, data validation, pivot tables, and extensive customization capabilities for business intelligence and data analysis workflows
# Too long, exceeds clarity needs
```

## Conversation-First Authoring

When building a new skill, do not start with a fixed questionnaire by
default. First inspect the active conversation and extract the workflow already
present in the discussion.

### What To Extract

Capture these elements before asking follow-up questions:

- repeated user intent that should become trigger language
- the actual ordered steps being performed
- branching logic and decision points
- validation or completion checks
- phrases that describe success and failure

### When To Ask Questions

Ask questions only when one of these remains unclear:

- the outcome the skill should produce
- whether the skill is workspace-scoped or personal
- whether the skill should be a checklist or a full workflow
- whether supporting files or scripts are needed

Good follow-up questions are narrow and unblock drafting quickly.

### Draft-Then-Refine Loop

Prefer this sequence:

1. extract the workflow from the conversation
2. write a first draft of `SKILL.md`
3. find the most ambiguous parts
4. ask only the minimum follow-up questions needed
5. revise and validate

This approach preserves the real workflow better than collecting every detail
before drafting.

### Source Precedence

If both a repository-local skill builder and an extension-shipped
`create-skill` prompt exist, use the repository-local skill builder as the
canonical source for:

- file layout
- frontmatter rules
- validation rules
- progressive disclosure strategy

An external prompt can still help with conversation flow, but it should not
override repository-local rules.

## Natural Language Keywords For Non-Technical Users

### Why This Matters

Not all users are developers. Many are:

- product managers who say `put this online`, not `deploy to production`
- designers who say `change the colors`, not `update design tokens`
- business users who say `save this data`, not `implement database schema`
- content creators who say `my site crashed`, not `analyze stack trace`

Your skill must be accessible to both audiences:

- natural language, which matches how people naturally speak
- technical jargon, which matches how developers speak

### The Dual-Keyword Strategy

Template pattern:

```yaml
description: Auto-invoked when user wants to [NATURAL LANGUAGE PHRASES]. Also triggers on technical terms like [TECHNICAL JARGON].
```

Example:

```yaml
description: Auto-invoked when user wants to fix broken code, not working, crashed, or error message. Also triggers on technical terms like debug, stack trace, error analysis, production incidents.
```

### Natural Language Keyword Categories

#### 1. Problem or issue keywords

Natural: broken, not working, crashed, error, failed, stuck, won't load,
stopped

Technical: exception, stack trace, runtime error, null pointer, undefined
reference

Example:

```yaml
# Debugger skill
description: Auto-invoked when something's broken, not working, error message, crashed, fix this, why isn't working, or app stopped working. Also triggers on debugging, stack trace, production incidents.
```

#### 2. Action or intent keywords

Natural: save, store, organize, put online, make live, publish, change,
update, fix

Technical: persist, deploy, migrate, refactor, optimize, synchronize

Example:

```yaml
# DevOps skill
description: Auto-invoked when user wants to put app online, make it live, publish site, launch app, or go live. Also triggers on deployment, CI/CD, container orchestration, infrastructure setup.
```

#### 3. Data or content keywords

Natural: save data, store information, slow database, where to save, organize
data

Technical: schema design, query optimization, normalization, indexing,
migrations

Example:

```yaml
# Database skill
description: Auto-invoked when storing data, saving information, organizing data, database is slow, or where to save data. Also triggers on database design, schema, query performance, migrations.
```

#### 4. Quality or review keywords

Natural: check my code, is this secure, look over code, is this good, find
problems

Technical: code review, security audit, OWASP, static analysis, vulnerability
scan

Example:

```yaml
# Code reviewer skill
description: Auto-invoked when user wants to check my code, is this secure, review code, look over code, or is this good. Also triggers on code review, security audit, PR analysis, OWASP.
```

#### 5. Creation or generation keywords

Natural: write docs, explain code, create guide, document this, make
instructions

Technical: API documentation, OpenAPI, technical writing, changelog, ADR

Example:

```yaml
# Documentation skill
description: Auto-invoked when user wants to write docs, create documentation, explain code, write README, or make user guide. Also triggers on technical documentation, API docs, OpenAPI.
```

#### 6. Improvement keywords

Natural: too long, simplify, clean up code, make easier to read, organize
better

Technical: refactor, extract method, reduce complexity, DRY, SOLID principles

Example:

```yaml
# Refactoring skill
description: Auto-invoked when file is too long, code needs simplifying, clean up code, organize better, or make code easier to read. Also triggers on refactor, extract, split component.
```

#### 7. Intelligence or AI keywords

Natural: chatbot, smart search, AI helper, answer questions, intelligent
assistant

Technical: RAG, LLM integration, vector database, embeddings, semantic search

Example:

```yaml
# AI/ML skill
description: Auto-invoked when user wants to add chatbot, smart search, AI helper, answer questions automatically, or intelligent assistant. Also triggers on RAG, vector database, LLM integration.
```

#### 8. Visual or design keywords

Natural: change design, new look, redesign, change colors, update fonts,
rebrand

Technical: design system, design tokens, theme migration, CSS-in-JS, styling

Example:

```yaml
# Design system skill
description: Auto-invoked when user wants to change entire design, new look everywhere, redesign everything, change all colors/fonts, or rebrand. Also triggers on design system migration, theme changes.
```

### Common Anti-Patterns To Avoid

#### Wrong: technical jargon only

```yaml
description: Handles database schema migrations and query optimization using normalization techniques and index strategies
```

Problem: non-technical users will not say `schema migrations` or
`normalization`.

#### Correct: natural plus technical

```yaml
description: Auto-invoked when storing data, saving information, organizing data, or database is slow. Also triggers on database design, schema, query performance, migrations.
```

#### Wrong: too vague

```yaml
description: Database stuff and data things
```

Problem: not specific enough for the host assistant to match user requests.

#### Correct: specific natural language

```yaml
description: Auto-invoked when storing data, saving information, organizing data, database is slow, or where to save data.
```

#### Wrong: natural language only

```yaml
description: Auto-invoked when user wants to put stuff online or make it work
```

Problem: developers using technical terms will not trigger the skill.

#### Correct: both audiences covered

```yaml
description: Auto-invoked when user wants to put app online, make it live, publish site. Also triggers on deployment, CI/CD, Docker, Kubernetes.
```

### Testing Your Keywords

Ask yourself:

1. Non-technical user test:
   - Would a product manager, designer, or business user use these words?
   - Can someone explain their problem without technical jargon?
2. Technical user test:
   - Would a developer recognize these technical terms?
   - Are industry-standard terms included?
3. Coverage test:
   - Do keywords cover different ways to express the same need?
   - Examples: `save data`, `store information`, `organize data`
4. Specificity test:
   - Are keywords specific enough to match the skill's purpose?
   - Avoid generic terms such as `stuff`, `things`, or `work`

### Validation Scoring

Natural language keywords check, worth 3 points:

- **3 points**: 2 or more natural language keywords
- **2 points**: 1 natural language keyword
- **1 point**: only technical jargon
- **0 points**: no clear keywords

Passing threshold: 80 percent, which is 24 out of 29 total points.

### Real-World Examples From Custom Skills

#### Example 1: ai-ml-implementation

```yaml
description: Research-backed AI/ML feature implementation. Auto-invoked when user wants to add chatbot, smart search, AI helper, answer questions automatically, chat with data, intelligent assistant, AI-powered search, conversational AI, question answering, or any AI features. Also triggers on technical terms like RAG, vector database, embeddings, LLM integration, semantic search, knowledge base.
```

Natural: chatbot, smart search, AI helper, answer questions automatically

Technical: RAG, vector database, embeddings, LLM integration

Why it works: both a PM asking to add a chatbot and a developer asking to
implement RAG can trigger it.

#### Example 2: debugger

```yaml
description: Production debugging, log analysis, stack trace interpretation, and root cause analysis. Auto-invoked when something's broken, not working, error message, crashed, fix this, why isn't working, bug, something wrong, or app stopped working. Also triggers on technical terms like debugging, stack trace, production incidents, CI/CD failures, error analysis.
```

Natural: broken, not working, crashed, fix this, app stopped working

Technical: stack trace, production incidents, CI/CD failures

Why it works: user frustration language like `this is broken` and developer
terms like `stack trace` both work.

#### Example 3: git-manager

```yaml
description: Git workflow standardization, commit conventions, branch strategies, and PR management. Auto-invoked when user wants to save changes, save work, track changes, commit code, create pull request, or merge code. Also triggers on technical terms like git commit, PR, pull request, branching, merge, version control, Conventional Commits, semantic versioning.
```

Natural: save changes, save work, track changes

Technical: git commit, PR, Conventional Commits, semantic versioning

Why it works: both a new developer asking to save work and an experienced
developer asking to create a PR can trigger it.

### Quick Reference Checklist

When writing descriptions:

- [ ] includes 2 or more natural language keywords
- [ ] includes 2 or more technical keywords
- [ ] uses the `Auto-invoked when user wants to...` pattern
- [ ] separates natural and technical language with
  `Also triggers on technical terms like...`
- [ ] covers different ways to express the same intent
- [ ] is specific enough to match the skill's purpose
- [ ] would pass validation for natural language keyword quality

## File Organization Patterns

### Minimal skill

```text
skill-name/
└── SKILL.md
```

Use when the skill is simple and self-contained.

### Basic skill

```text
skill-name/
├── SKILL.md
└── REFERENCE.md
```

Use when instructions need detailed reference information that should not load
initially.

### Standard skill

```text
skill-name/
├── SKILL.md
├── REFERENCE.md
└── FORMS.md
```

Use when the skill generates structured output using templates.

### Advanced skill

```text
skill-name/
├── SKILL.md
├── REFERENCE.md
├── FORMS.md
├── scripts/
│   ├── process.py
│   └── validate.sh
└── resources/
    ├── templates/
    │   ├── base.txt
    │   └── advanced.txt
    └── data/
        └── reference.json
```

Use when the workflow needs executable code and multiple resources.

## Progressive Disclosure Details

### Level 1: Metadata

Token cost: about 100 tokens per skill

Content: YAML frontmatter only

Purpose: skill discovery and matching

```yaml
---
name: example-skill
description: What it does and when to use it
---
```

### Level 2: Instructions

Token cost: under 5,000 tokens

Content: `SKILL.md` body

Purpose: core procedures and workflow

Keep instructions focused:

- include the core workflow only
- reference other files for details
- use examples sparingly
- link to `REFERENCE.md` for specifications

### Level 3: Resources

Token cost: 0 from context loading, because resources are accessed on demand

Content: supporting files

Purpose: detailed information, templates, and scripts

Access pattern:

```bash
# Read reference when needed
cat .github/skills/skill-name/REFERENCE.md

# Load template when formatting output
cat .github/skills/skill-name/resources/template.txt

# Execute script when processing data
python3 .github/skills/skill-name/scripts/process.py
```

## Validation Rules

### Name Validation

```python
import re


def is_valid_skill_name(name):
    if len(name) > 64:
        return False
    if not re.match(r"^[a-z0-9-]+$", name):
        return False
    return True
```

### Description Validation

```python
def is_valid_description(desc):
    if len(desc) == 0 or len(desc) > 1024:
        return False
    return True
```

### Structure Validation

```bash
# Check required files
test -f SKILL.md || echo "Missing SKILL.md"

# Check frontmatter exists
head -1 SKILL.md | grep -q "^---$" || echo "Missing frontmatter"

# Verify name and description in frontmatter
grep -q "^name: " SKILL.md || echo "Missing name field"
grep -q "^description: " SKILL.md || echo "Missing description field"
```

## Runtime Constraints

### No Internet Access

Skills cannot:

- make HTTP requests
- access external APIs
- download resources
- fetch remote data

Workaround: pre-include all necessary data in `resources/`.

### No Package Installation

Skills cannot:

- run `pip install`
- run `npm install`
- install system packages
- download dependencies at runtime

Workaround: only use pre-installed packages available in the current coding
environment.

### Available Pre-Installed Packages

Common packages available:

- **Python**: requests, pandas, numpy, json, csv, re, os, sys
- **Node.js**: basic built-ins
- **Shell**: standard Unix utilities

Verify availability before relying on a specific package.

## Best Practice Examples

### Good: progressive disclosure

```markdown
# Data Analyzer Skill

## Process

1. Read input data
2. For processing rules, see REFERENCE.md section 3.2
3. Apply template from FORMS.md
4. Run scripts/analyze.py if complex processing is needed
```

### Bad: everything embedded

```markdown
# Data Analyzer Skill

## Process

1. Read input data
2. Apply these 47 detailed rules:
   - Rule 1: [500 words of explanation]
   - Rule 2: [500 words of explanation]
   - ...
3. Use this template:
   [Entire template embedded here]
4. Processing algorithm:
   [Entire script code pasted here]
```

### Good: clear trigger conditions

```markdown
## When to Use

Invoke this skill when users:

- Request code review
- Ask to check code for issues
- Want security analysis
- Need best practices validation
```

### Bad: vague triggers

```markdown
## When to Use

Use this skill for code stuff.
```

## Example Skills

### Simple: code formatter

```markdown
---
name: code-formatter
description: Format code files according to language-specific style guides when user requests code formatting
---

# Code Formatter

## When to Use

- User asks to format code
- User mentions style guide compliance
- Code formatting is requested

## Process

1. Identify language from file extension
2. Apply appropriate formatter such as prettier, black, or gofmt
3. Show diff and apply changes
4. Confirm formatting completed
```

### Medium: test generator

```markdown
---
name: test-generator
description: Generate unit tests for code functions when user requests test creation or test coverage improvements
---

# Test Generator

## When to Use

- User requests tests for a function or class
- User wants to improve test coverage
- User mentions unit testing

## Process

1. Analyze target code to understand functionality
2. Identify test cases using patterns from REFERENCE.md
3. Generate test code using templates from FORMS.md
4. Explain test coverage

## Supporting Files

- REFERENCE.md: test patterns and edge cases
- FORMS.md: test templates for each language
```

### Complex: API documentation generator

```markdown
---
name: api-doc-generator
description: Generate comprehensive API documentation from code comments and endpoint definitions for REST APIs
---

# API Documentation Generator

## When to Use

- User requests API documentation
- Need to document REST endpoints
- OpenAPI or Swagger generation is requested

## Workflow

1. Scan codebase for API endpoints using scripts/scan-endpoints.sh
2. Extract comments and parameters
3. For documentation format, see REFERENCE.md sections 2 through 4
4. Generate output using templates from FORMS.md
5. Run scripts/validate-docs.py to verify completeness

## File Structure

- scripts/scan-endpoints.sh: find API endpoints
- scripts/validate-docs.py: verify documentation
- REFERENCE.md: OpenAPI specification details
- FORMS.md: documentation templates
- resources/examples/: sample API docs
```

## Troubleshooting

### Skill Not Triggering

Possible causes:

- description does not match the user request
- name contains invalid characters
- `SKILL.md` has malformed frontmatter
- file is in the wrong directory

Solutions:

- make the description more specific to real use cases
- validate the name against the rules
- check YAML frontmatter syntax
- verify file location such as `.github/skills/` or the configured user skills
  directory

### Instructions Too Long

Symptoms:

- slow skill loading
- token budget issues

Solutions:

- move detailed content to `REFERENCE.md`
- extract templates to `FORMS.md`
- use file references instead of embedding
- keep `SKILL.md` under 5k tokens

### Resources Not Loading

Possible causes:

- incorrect file paths
- files not in the skill directory
- permission issues

Solutions:

- use absolute paths that point at the real skill root, for example
  `/path/to/skills/skill-name/file.txt`
- verify files exist with the `ls` command
- check file permissions
