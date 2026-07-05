---
name: advanced-skill-name
description: Detailed description of this skill's purpose, capabilities, and when it should be automatically invoked
---

# Advanced Skill Name

Comprehensive description of the skill's purpose and value.

## When to Use

This skill should be invoked when:
- Specific trigger condition 1
- Specific trigger condition 2
- User requests match pattern X

Do NOT use this skill when:
- Out of scope condition 1
- Alternative skill is better suited

## Prerequisites

Before using this skill:
- Check for required files in `resources/`
- Verify scripts in `scripts/` are accessible
- Ensure necessary tools are available

## Core Workflow

### Phase 1: Preparation
1. Gather required information from user
2. Load necessary resources from `resources/`
3. Validate inputs and prerequisites

### Phase 2: Execution
1. Follow main procedure
2. Reference REFERENCE.md for detailed specifications
3. Use templates from FORMS.md as needed
4. Execute scripts from `scripts/` if required

### Phase 3: Completion
1. Validate output
2. Present results to user
3. Provide next steps or recommendations

## Supporting Files

This skill uses these supporting files:

- **REFERENCE.md**: Detailed technical specifications and API docs
- **FORMS.md**: Templates and structured output formats
- **scripts/utility.py**: Data processing utilities
- **resources/template.txt**: Base template for outputs

Access these files as needed using bash commands.

## Examples

### Example 1: Simple Case
```
User: Create a basic widget
Assistant:
1. Loading template from resources/widget-template.txt
2. Applying configuration
3. Generating output
```

### Example 2: Complex Case with Scripts
```
User: Create an advanced widget with custom processing
Assistant:
1. Gathering requirements
2. Reading specifications from REFERENCE.md
3. Running scripts/process-data.py
4. Applying template from FORMS.md
5. Validating result
```

## Error Handling

Common issues and solutions:

- **Missing resource**: Check if file exists in resources/ before reading
- **Script failure**: Validate inputs before running scripts
- **Invalid format**: Reference FORMS.md for correct structure

## Progressive Disclosure Strategy

To minimize token usage:
1. Load only core instructions from this file
2. Read REFERENCE.md only when detailed specs needed
3. Access templates from FORMS.md only when formatting output
4. Execute scripts only when processing required

## Best Practices

- Always validate inputs before processing
- Use templates consistently
- Follow the reference specifications exactly
- Provide clear feedback to user at each phase

## Limitations

- Only works with pre-installed packages
- No internet access
- Cannot install runtime dependencies
- Maximum file size constraints apply
