# Skill Builder

A generic meta-skill for creating well-structured reusable skills with validation and best practices.

## What It Does

This skill helps users create new skills that are:

- clearly named and discoverable
- structured for progressive disclosure
- easy to validate
- portable across repositories and coding environments

## Recommended Skill Locations

Use one of these patterns:

- project-local skills in `.github/skills/<skill-name>/`
- user-level skills in a custom shared directory such as `<user-skills-dir>/<skill-name>/`

## Included Files

- `SKILL.md` for the creation workflow
- `REFERENCE.md` for detailed rules and examples
- `FORMS.md` for templates and starter layouts
- `scripts/validate-skill.py` for validation
- `templates/` for reusable starting points

## Validation

Run the validator against any skill directory:

```bash
python3 .github/skills/skill-builder/scripts/validate-skill.py <path-to-skill>
```

Examples:

```bash
python3 .github/skills/skill-builder/scripts/validate-skill.py .github/skills/my-new-skill/
python3 .github/skills/skill-builder/scripts/validate-skill.py .github/skills/skill-builder/
```

## Example Requests

- Create a skill for code reviews.
- Help me build a documentation-generation skill.
- Make a skill that automates testing workflows.
- I need a skill for API documentation.

## Requirements

- Python 3.8+
- PyYAML for the validation script

Install PyYAML if needed:

```bash
pip install pyyaml
```

## Tips

1. Make the description explicit about when the skill should trigger.
2. Prefer step-by-step instructions over vague guidance.
3. Keep `SKILL.md` concise and move details into `REFERENCE.md` or `FORMS.md`.
4. Use repository-relative examples when possible.
5. Avoid environment-specific assumptions unless the skill genuinely requires them.

## Related Files

- `SKILL.md`
- `REFERENCE.md`
- `FORMS.md`
- `templates/basic-skill-template.md`
- `templates/advanced-skill-template.md`

## License

MIT
