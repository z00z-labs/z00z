# Frontend UI - Reference Guide

Extended reference for a merged workflow that combines UI audit discipline with distinctive frontend design.

## Font Library

### Monospace

- JetBrains Mono
- Fira Code
- IBM Plex Mono
- Space Mono
- Source Code Pro
- Inconsolata

### Display Or Geometric Sans

- Clash Display
- Epilogue
- Syne
- Outfit
- Manrope
- General Sans
- Satoshi

### Serif

- Playfair Display
- Crimson Pro
- Libre Baskerville
- Merriweather
- Lora
- Spectral
- Cormorant

## Font Pairing Matrix

| Heading | Body | Style | Use Case |
| ------- | ---- | ----- | -------- |
| Clash Display | JetBrains Mono | Geometric + Mono | Tech products, dev tools |
| Playfair Display | Space Grotesk | Serif + Geometric | Editorial, portfolios |
| Syne | Fira Code | Unusual + Mono | Experimental, creative tech |
| Crimson Pro | Outfit | Serif + Sans | Professional, elegant |
| Space Mono | Space Mono | Mono + Mono | Terminal, retro-future |
| Clash Display | Manrope | Display + Rounded | Modern apps, SaaS |

## Theme Directions

### Cyberpunk Or Neo-Tokyo

- dark backgrounds
- neon accents
- glow effects
- futuristic contrast

### Terminal Or Hacker

- monospace typography
- phosphor green accents
- scanline or CRT cues
- dense technical atmosphere

### Brutalist Or Swiss

- high contrast
- strong geometric structure
- bold type
- restrained color palette

### Solarpunk

- warm earth tones
- organic shapes
- optimistic tone
- green and gold accents

### Vaporwave

- pink and cyan accents
- dreamy gradients
- retro grids
- stylized nostalgia

## Motion Patterns

Prefer a few strong motion moments over many weak ones.

- staggered reveal on page load
- edge slide for major content blocks
- subtle hover transformations on interactive elements
- no essential meaning hidden behind motion

Always respect reduced motion.

## Background Patterns

Useful background moves:

- layered radial and linear gradients
- grid overlays
- dot patterns
- diagonal stripe textures
- subtle noise for editorial or print-like surfaces

## Audit Methodology

Use this section when the request is about improving an existing interface.

### Current-State Inventory

Before recommending changes, identify:

- which sections and components already exist
- which metrics, summaries, or charts are already shown
- which CTAs already exist and where they sit in the hierarchy
- whether the page is product-like, editorial, dashboard-like, or intentionally minimal
- whether the visual density is sparse, balanced, or dense

### Redundancy Signals

Treat these as warnings:

- a new visualization restates data already visible elsewhere
- a new CTA competes with the page's primary CTA
- a dashboard pattern is added to a reading-first page
- the page becomes noisier without becoming more useful
- the same fact appears in multiple formats with no added value

### Genuine Gap Criteria

Only recommend a UI addition when:

- the missing element is visible in the code or page structure
- the user benefit is concrete
- the fix can be explained briefly
- the addition fits the page's current design philosophy

### Clean Design Heuristics

When in doubt, prefer:

- one strong section over multiple weaker sections
- one clear CTA over multiple equal-priority actions
- one place where a metric is shown over repeated metric surfaces
- more whitespace over more chrome
- stronger hierarchy over more decoration

## UX Notes

### Left-Attention Bias

Users often notice the left side of a layout first. Put primary value propositions, key copy, and high-priority actions in stronger attention zones unless the product pattern clearly dictates otherwise.

### F-Pattern Scanning

For copy-heavy pages:

- put key information early
- start headings with informative words
- keep paragraphs short
- use lists where scanability matters

### Cognitive Load

Reduce competing choices.

- keep primary actions limited
- group related controls
- avoid turning one page into multiple competing page types

## Accessibility Checks

Run these checks even for visual refreshes:

- text contrast is sufficient for body copy and controls
- focus states remain visible
- icon-only buttons have labels
- motion is optional for reduced-motion users
- interactive elements remain keyboard reachable

## Audit Output Model

```markdown
## Current State Summary
[what the page currently does]

## What Already Exists
- [component]: [purpose]

## Redundancy Risks
- [proposed addition]: duplicates [existing UI]

## Genuine Gaps
- [gap]: [evidence] -> [small fix]

## Recommendation
- Implement: [best change]
- Avoid: [redundant change]
```

## Performance Notes

Prefer animating:

- `transform`
- `opacity`

Avoid animating layout-heavy properties such as width, height, top, left, margin, and padding when possible.

## Supporting Files

- Use `FORMS.md` for audit templates and repeatable checklists.
- Use `resources/examples/` to compare good and bad audit behavior.
- Use `resources/case-studies/` to understand failure modes and prevention patterns.
