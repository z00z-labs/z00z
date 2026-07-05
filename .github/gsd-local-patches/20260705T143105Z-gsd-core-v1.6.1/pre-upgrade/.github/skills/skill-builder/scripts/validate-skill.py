#!/usr/bin/env python3
"""
Skill Validation Script

Validates a skill directory structure and content.

Usage:
    python validate-skill.py <path-to-skill-directory>
"""

import sys
import os
import re
import yaml

def validate_skill(skill_path):
    """Validate a skill directory"""
    errors = []
    warnings = []
    info = []

    # Check if directory exists
    if not os.path.isdir(skill_path):
        errors.append(f"Directory does not exist: {skill_path}")
        return errors, warnings, info

    info.append(f"Validating skill at: {skill_path}")

    # Check for SKILL.md
    skill_md_path = os.path.join(skill_path, "SKILL.md")
    if not os.path.isfile(skill_md_path):
        errors.append("Missing required file: SKILL.md")
        return errors, warnings, info

    info.append("✓ SKILL.md exists")

    # Read SKILL.md
    with open(skill_md_path, 'r') as f:
        content = f.read()

    # Check for frontmatter
    if not content.startswith('---\n'):
        errors.append("SKILL.md must start with YAML frontmatter (---)")
        return errors, warnings, info

    # Extract frontmatter
    parts = content.split('---\n', 2)
    if len(parts) < 3:
        errors.append("Invalid YAML frontmatter format")
        return errors, warnings, info

    frontmatter_text = parts[1]
    body = parts[2]

    # Parse YAML
    try:
        frontmatter = yaml.safe_load(frontmatter_text)
    except yaml.YAMLError as e:
        errors.append(f"Invalid YAML frontmatter: {e}")
        return errors, warnings, info

    info.append("✓ YAML frontmatter is valid")

    # Validate name field
    if 'name' not in frontmatter:
        errors.append("Missing required field: name")
    else:
        name = frontmatter['name']
        info.append(f"✓ name field exists: {name}")

        # Validate name format
        if len(name) > 64:
            errors.append(f"name exceeds 64 characters: {len(name)}")

        if not re.match(r'^[a-z0-9-]+$', name):
            errors.append(f"name contains invalid characters (must be lowercase, alphanumeric, hyphens only): {name}")

        if re.match(r'^[a-z0-9-]+$', name) and len(name) <= 64:
            info.append("✓ name format is valid")

    # Validate description field
    if 'description' not in frontmatter:
        errors.append("Missing required field: description")
    else:
        description = frontmatter['description']
        info.append(f"✓ description field exists ({len(description)} chars)")

        if len(description) == 0:
            errors.append("description cannot be empty")

        if len(description) > 1024:
            errors.append(f"description exceeds 1024 characters: {len(description)}")

        if len(description) > 0 and len(description) <= 1024:
            info.append("✓ description length is valid")

        # Check if description is helpful
        if len(description) < 20:
            warnings.append("description is very short - consider adding more detail about when to use this skill")

    # Validate body content
    if len(body.strip()) == 0:
        warnings.append("SKILL.md body is empty - add instructions for the host assistant")
    else:
        info.append(f"✓ SKILL.md has instructions ({len(body)} chars)")

        # Check for recommended sections
        if "when to use" not in body.lower():
            warnings.append("Consider adding a 'When to Use' section to clarify trigger conditions")

        if "example" not in body.lower():
            warnings.append("Consider adding examples to demonstrate usage")

    # Check for optional supporting files
    optional_files = {
        'REFERENCE.md': 'detailed reference information',
        'FORMS.md': 'templates and forms',
    }

    for filename, purpose in optional_files.items():
        filepath = os.path.join(skill_path, filename)
        if os.path.isfile(filepath):
            info.append(f"✓ Optional file present: {filename} ({purpose})")

    # Check for scripts directory
    scripts_dir = os.path.join(skill_path, 'scripts')
    if os.path.isdir(scripts_dir):
        scripts = [f for f in os.listdir(scripts_dir) if os.path.isfile(os.path.join(scripts_dir, f))]
        if scripts:
            info.append(f"✓ Scripts directory contains {len(scripts)} file(s)")

            # Check if scripts are executable
            for script in scripts:
                script_path = os.path.join(scripts_dir, script)
                if not os.access(script_path, os.X_OK):
                    warnings.append(f"Script may not be executable: scripts/{script}")

    # Check for resources directory
    resources_dir = os.path.join(skill_path, 'resources')
    if os.path.isdir(resources_dir):
        resource_count = sum(len(files) for _, _, files in os.walk(resources_dir))
        info.append(f"✓ Resources directory contains {resource_count} file(s)")

    # Check for file references in body
    file_refs = re.findall(r'(?:REFERENCE\.md|FORMS\.md|scripts/[\w.-]+|resources/[\w/.-]+)', body)
    if file_refs:
        info.append(f"Found {len(file_refs)} file reference(s) in instructions")

        # Validate file references exist
        for ref in set(file_refs):
            ref_path = os.path.join(skill_path, ref)
            if not os.path.exists(ref_path):
                warnings.append(f"Referenced file does not exist: {ref}")

    return errors, warnings, info

def print_results(errors, warnings, info):
    """Print validation results"""
    print("\n" + "="*60)
    print("SKILL VALIDATION RESULTS")
    print("="*60 + "\n")

    # Print info
    if info:
        print("INFO:")
        for msg in info:
            print(f"  {msg}")
        print()

    # Print warnings
    if warnings:
        print("WARNINGS:")
        for msg in warnings:
            print(f"  ⚠️  {msg}")
        print()

    # Print errors
    if errors:
        print("ERRORS:")
        for msg in errors:
            print(f"  ❌ {msg}")
        print()

    # Print summary
    print("="*60)
    if errors:
        print("❌ VALIDATION FAILED")
        print(f"   {len(errors)} error(s), {len(warnings)} warning(s)")
        return False
    elif warnings:
        print("⚠️  VALIDATION PASSED WITH WARNINGS")
        print(f"   {len(warnings)} warning(s)")
        return True
    else:
        print("✅ VALIDATION PASSED")
        print("   No errors or warnings")
        return True

def main():
    if len(sys.argv) < 2:
        print("Usage: python validate-skill.py <path-to-skill-directory>")
        sys.exit(1)

    skill_path = sys.argv[1]

    errors, warnings, info = validate_skill(skill_path)
    success = print_results(errors, warnings, info)

    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()
