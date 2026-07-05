# Frontend UI

A unified skill for frontend UI creation and UI/UX improvement work.

## What This Skill Does

This skill combines two workflows that belong together:

- create visually distinctive interfaces that avoid generic AI-slop output
- audit existing UI before changing it so redundant features are not added

It is meant for both:

- brand-new pages, components, and screens
- improvements to existing pages, layouts, and interaction flows

## Why This Merge Exists

Design-generation guidance alone is not enough for existing products. Without an audit step, AI often adds sections, cards, dashboards, or visualizations that duplicate what already exists.

Audit guidance alone is also not enough. Once a real gap is found, the implementation still needs strong hierarchy, typography, color, motion, and background direction.

This merged skill keeps those two responsibilities in one package.

## What's Included

- `SKILL.md` for the main workflow
- `REFERENCE.md` for design patterns, audit guidance, and accessibility rules
- `FORMS.md` for repeatable templates and checklists
- `resources/examples/` for good and bad audit examples
- `resources/case-studies/` for case studies and failure-prevention notes

## Core Workflow

### Existing Interface

When the user wants to improve an existing page, the skill should:

1. Read the current page and related components first.
2. Inventory what already exists.
3. Check for redundancy.
4. Identify genuine gaps.
5. Propose the smallest valuable fix.
6. Implement it with strong design choices.

### New Interface

When the user wants a new UI, the skill should:

1. Inspect the surrounding product or design system if one exists.
2. Choose a clear aesthetic direction.
3. Implement with strong typography, theme, motion, and background design.

## Design Dimensions

The skill still uses four primary design dimensions.

### 1. Typography

- Distinctive font alternatives to overused choices
- High-contrast pairing strategies
- Extreme weight variation for hierarchy
- Large size jumps where the layout needs stronger contrast

### 2. Color And Theme

- Cohesive aesthetic themes
- Dominant colors with sharp accents
- CSS variable strategies for consistency
- Palettes chosen deliberately instead of default startup colors

### 3. Motion

- CSS-first animations for static pages
- Framer Motion patterns for React when available
- High-impact, orchestrated load sequences
- Staggered reveals instead of random micro-motion everywhere

### 4. Backgrounds

- Layered gradients
- Geometric patterns
- Contextual textures or noise
- Depth that supports the chosen visual direction

## Improvement Requests This Skill Should Catch

The merged workflow should help reject:

- repeated metrics or charts
- duplicate CTAs
- oversized feature cards that compete with primary actions
- dashboard additions on editorial or minimal pages
- visual bulk that adds activity but not value

## Example Requests

- "Create a landing page for a developer portfolio"
- "Build a dashboard with a cyberpunk aesthetic"
- "Make a brutalist-style blog homepage"
- "Improve the homepage"
- "What should I add to this page?"
- "Redesign this component without adding clutter"
- "Make this interface feel cleaner and more premium"

## Quality Standards

Every UI created with this skill should be checked against:

- whether the current page was audited first when relevant
- whether the proposed change solves a real gap
- whether the result adds clarity instead of redundancy
- font distinctiveness
- color direction originality
- typography contrast
- background depth and interest
- motion discipline
- theme cohesiveness
- accessibility basics such as contrast, focus states, and reduced motion

## Learn More

- [SKILL.md](SKILL.md) - complete workflow instructions
- [REFERENCE.md](REFERENCE.md) - design and audit reference
- [FORMS.md](FORMS.md) - audit templates and checklists
- [resources/README.md](resources/README.md) - examples and case studies

## Acknowledgments

This unified skill keeps a strong frontend aesthetics foundation and adds an evidence-first UI audit workflow for existing interfaces.
