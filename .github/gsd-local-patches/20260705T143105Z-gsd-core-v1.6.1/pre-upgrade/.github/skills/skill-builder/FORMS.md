# Skill Builder Forms and Templates

Quick-reference templates for creating skill components.

## Minimal SKILL.md Template

```markdown
---
name: skill-name
description: What this skill does and when to use it
---

# Skill Name

Brief overview.

## When to Use
- Trigger condition 1
- Trigger condition 2

## Process
1. Step one
2. Step two
3. Step three

## Example
User: [request]
Response: [what the skill does]
```

## Standard SKILL.md Template

```markdown
---
name: skill-name
description: Comprehensive description of skill purpose and when it should trigger automatically
---

# Skill Name

One-sentence purpose statement.

## When to Use

This skill triggers when:
- User requests [specific action]
- Task involves [specific domain]
- User asks about [specific topic]

## Prerequisites

Before starting:
- [ ] Check required files exist
- [ ] Verify necessary tools available
- [ ] Confirm input format

## Main Workflow

### Step 1: Preparation
Instructions for preparation phase.

### Step 2: Execution
Main processing steps.

### Step 3: Validation
How to verify success.

## Supporting Files

- **REFERENCE.md**: Detailed specifications
- **FORMS.md**: Output templates
- **scripts/**: Processing utilities

## Examples

### Basic Example
```
User: [simple request]
Assistant: [simple response]
```

### Advanced Example
```
User: [complex request]
Assistant: [complex response with steps]
```

## Notes
- Important considerations
- Known limitations
- Tips for success
```

## REFERENCE.md Template

```markdown
# [Skill Name] Reference

Detailed technical documentation for the skill.

## Specifications

### Section 1: Core Concepts
Detailed explanation of fundamental concepts.

### Section 2: Technical Details
In-depth technical information.

### Section 3: API/Interface Details
Specific interface or API documentation.

## Examples

### Example 1: [Scenario]
Detailed example with explanation.

### Example 2: [Scenario]
Another detailed example.

## Edge Cases

### Case 1: [Description]
How to handle this edge case.

### Case 2: [Description]
How to handle this edge case.

## Troubleshooting

### Issue: [Description]
**Symptoms**: What you'll see
**Cause**: Why it happens
**Solution**: How to fix it
```

## FORMS.md Template

```markdown
# [Skill Name] Forms and Templates

## Template 1: [Name]

```
[Template content here]
```

**Usage**: When to use this template
**Variables**: What to customize

## Template 2: [Name]

```
[Template content here]
```

**Usage**: When to use this template
**Variables**: What to customize

## Output Format Examples

### Format 1: [Type]
```
[Example output]
```

### Format 2: [Type]
```
[Example output]
```
```

## Directory Structure Template

```
skill-name/
├── SKILL.md              # Main instructions (required)
├── REFERENCE.md          # Detailed reference (optional)
├── FORMS.md              # Templates (optional)
├── scripts/              # Executable scripts (optional)
│   ├── script1.py
│   ├── script2.sh
│   └── utils/
│       └── helper.py
└── resources/            # Static resources (optional)
    ├── templates/
    │   ├── basic.txt
    │   └── advanced.txt
    ├── data/
    │   └── reference.json
    └── examples/
        └── sample.md
```

## Frontmatter Examples

### Minimal
```yaml
---
name: simple-skill
description: Does one thing when user requests that thing
---
```

### Standard
```yaml
---
name: document-generator
description: Generate formatted documents from templates when user needs document creation for reports, letters, or structured content
---
```

### Detailed
```yaml
---
name: comprehensive-analyzer
description: Analyze code for quality, security, and performance issues when user requests code review, security audit, or optimization suggestions
---
```

## Script Templates

### Python Script Template
```python
#!/usr/bin/env python3
"""
Script description

Usage:
    python script.py <input> <output>
"""

import sys
import json

def main():
    if len(sys.argv) < 3:
        print("Usage: script.py <input> <output>")
        sys.exit(1)

    input_file = sys.argv[1]
    output_file = sys.argv[2]

    # Process input
    with open(input_file, 'r') as f:
        data = f.read()

    # Transform data
    result = process(data)

    # Write output
    with open(output_file, 'w') as f:
        f.write(result)

def process(data):
    """Main processing logic"""
    # Implementation here
    return data

if __name__ == "__main__":
    main()
```

### Bash Script Template
```bash
#!/bin/bash
# Script description
#
# Usage: ./script.sh <input> <output>

set -e  # Exit on error

if [ "$#" -lt 2 ]; then
    echo "Usage: $0 <input> <output>"
    exit 1
fi

INPUT="$1"
OUTPUT="$2"

# Main processing
process_data() {
    local input="$1"
    local output="$2"

    # Implementation here
    cat "$input" > "$output"
}

# Execute
process_data "$INPUT" "$OUTPUT"
echo "Processing complete: $OUTPUT"
```

## Validation Checklist Template

```markdown
## Skill Validation Checklist

### Required Components
- [ ] SKILL.md exists
- [ ] YAML frontmatter is present and valid
- [ ] name field exists and is valid (lowercase, alphanumeric, hyphens, max 64 chars)
- [ ] description field exists and is valid (non-empty, max 1024 chars)
- [ ] Instructions are clear and actionable

### Content Quality
- [ ] "When to Use" section clearly defines trigger conditions
- [ ] Process/workflow is step-by-step
- [ ] Examples are provided
- [ ] File references are correct
- [ ] No external dependencies (internet, package installation)

### Optional Components (if present)
- [ ] REFERENCE.md is organized and complete
- [ ] FORMS.md contains useful templates
- [ ] Scripts are executable and documented
- [ ] Resources are properly organized

### Testing
- [ ] Skill is discoverable (frontmatter loads)
- [ ] Instructions are clear when triggered
- [ ] File references work correctly
- [ ] Scripts execute without errors
- [ ] Examples demonstrate actual usage
```

## Common Skill Patterns

### Pattern 1: Simple Command Skill
```markdown
---
name: command-helper
description: Execute specific commands with proper options when user requests that command
---

# Command Helper

## When to Use
When user requests [specific command]

## Process
1. Validate inputs
2. Build command with options
3. Execute and capture output
4. Present results
```

### Pattern 2: Generator Skill
```markdown
---
name: content-generator
description: Generate structured content from templates when user needs [specific content type]
---

# Content Generator

## When to Use
When user needs to create [content type]

## Process
1. Gather requirements
2. Select template from FORMS.md
3. Fill template with user data
4. Validate output format
5. Present generated content
```

### Pattern 3: Analyzer Skill
```markdown
---
name: code-analyzer
description: Analyze code for [specific aspects] when user requests code analysis or quality checks
---

# Code Analyzer

## When to Use
When user requests code analysis

## Process
1. Read target code
2. Apply analysis rules from REFERENCE.md
3. Run analysis scripts if needed
4. Compile findings
5. Present report with recommendations
```

### Pattern 4: Workflow Automation Skill
```markdown
---
name: workflow-automator
description: Automate multi-step workflows for [specific domain] when user needs to complete complex multi-step tasks
---

# Workflow Automator

## When to Use
When user needs to complete [specific workflow]

## Process
1. Map workflow steps from REFERENCE.md
2. Execute each step sequentially
3. Validate outputs between steps
4. Handle errors gracefully
5. Provide completion summary
```

## Quick Start Examples

### Example 1: Create Excel Automation Skill
```bash
# Create directory
mkdir -p .github/skills/excel-automation

# Create SKILL.md
cat > .github/skills/excel-automation/SKILL.md << 'EOF'
---
name: excel-automation
description: Create and manipulate Excel spreadsheets with formulas and formatting when user needs spreadsheet generation
---

# Excel Automation

## When to Use
When user requests Excel spreadsheet creation or modification.

## Process
1. Gather requirements (data, formulas, formatting)
2. Generate spreadsheet structure
3. Apply formulas and formatting
4. Save and present to user
EOF
```

### Example 2: Create Documentation Skill
```bash
# Create directory structure
mkdir -p .github/skills/doc-generator/{templates,scripts}

# Create main SKILL.md
cat > .github/skills/doc-generator/SKILL.md << 'EOF'
---
name: doc-generator
description: Generate technical documentation from code and comments when user needs API docs, user guides, or technical specifications
---

# Documentation Generator

## When to Use
When user requests documentation generation.

## Process
1. Scan code for documentation markers
2. Extract information using scripts/extract-docs.py
3. Apply template from templates/doc-template.md
4. Generate final documentation
EOF

# Create template
cat > .github/skills/doc-generator/templates/doc-template.md << 'EOF'
# {TITLE}

## Overview
{OVERVIEW}

## API Reference
{API_REFERENCE}

## Examples
{EXAMPLES}
EOF
```
