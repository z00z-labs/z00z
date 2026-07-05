---
name: frontend-ui
description: Create or improve frontend UIs with a two-phase workflow: audit the current interface before changing existing UI, then implement visually distinctive results with strong typography, color, motion, and background direction. Use when the user asks for UI, UX, design, layout, homepage, dashboard, component, landing page, visual refresh, redesign, or to make an interface look better. Prevent redundant additions and generic AI-slop patterns.
---

# Frontend UI

Create visually distinctive frontend interfaces while preventing redundant UI work.

This skill combines two responsibilities that should stay together:

- Audit existing UI before changing it.
- Design and implement interfaces that feel intentional instead of generic.

## When to Use

Use this skill when:
- User asks to improve, redesign, or extend an existing interface
- User requests frontend UI creation (HTML, CSS, JavaScript, or React)
- User asks to build a webpage, landing page, or web interface
- User wants to create visual components or layouts
- User mentions design, aesthetics, or "make it look good"
- User asks for dashboard, portfolio, or application UI
- User asks what should be added to a page or how a UI should be improved

## Core Principles

### 1. Audit Before Implementation

For existing UI, always inspect the current state before suggesting changes.

- Read the target page, component, and nearby supporting files first.
- Inventory what already exists.
- Check for redundancy before proposing additions.
- Prefer the smallest valuable change over bulk additions.

If the user is asking for a brand-new screen, skip the redundancy audit and instead inspect the existing design system or visual language so the new UI fits the product.

### 2. Respect The Existing Design Language

When working inside an established product:

- Preserve the existing component patterns unless they are the problem.
- Match the current spacing, visual density, and tone.
- Avoid adding "showcase" features that do not belong to the page.

When working from scratch, choose a deliberate visual direction rather than defaulting to generic startup UI.

### 3. Avoid Redundant UI

Do not add something just because it sounds useful. Add it only if the audit shows a genuine gap.

Common mistakes to reject:

- Repeating the same metric in multiple places
- Adding dashboards to intentionally minimal pages
- Adding oversized CTA blocks where primary CTAs already exist
- Repackaging existing content as a new "visualization"
- Cluttering document-style pages with sidebars or cards that dilute the core reading flow

### 4. Avoid "AI Slop"

Without explicit pressure, AI tends to converge on bland UI: white backgrounds, purple accents, Inter/Roboto, safe component grids, timid spacing, and forgettable hierarchy. This skill exists to prevent that.

## Phase 1: Audit Existing UI

Run this phase whenever the request changes an existing interface.

### Step 1: Read Current State First

Read the actual page and adjacent files before making recommendations.

Targets to inspect:

- The page or screen file
- The main section or component files used on that page
- Data/content/config files that feed the UI
- Existing styles, tokens, or design system primitives
- Similar components elsewhere in the codebase if the feature may already exist

### Step 2: Inventory What Exists

Document the current state with evidence.

Capture:

- Existing sections and components
- Existing CTAs and their placement
- Existing metrics, lists, charts, or summaries
- Current layout and information density
- Current design character: minimal, editorial, dense, playful, brutalist, product UI, etc.

### Step 3: Run The Redundancy Check

Before proposing any addition, verify:

- Is this information already displayed elsewhere?
- Does similar functionality already exist in another form?
- Would this duplicate an existing CTA or visual hierarchy?
- Would this add bulk without solving a real problem?
- Does it conflict with the page's intended density or reading mode?

### Step 4: Identify Genuine Gaps

Only propose additions that satisfy all of these:

- The gap is real and visible in the code.
- The user benefit is specific.
- The solution is small enough to fit the page.
- It does not duplicate existing content.

### Step 5: Produce An Audit Summary

Use this structure before implementation:

```markdown
## Current State Summary
[2-3 sentences about the page as it exists]

## What Already Exists
- [feature/component]: [purpose and location]
- [feature/component]: [purpose and location]

## Redundancy Risks
- [idea]: duplicates [existing thing]

## Genuine Gaps
- [gap]: [evidence] -> [smallest viable fix]

## Recommendation
- Implement: [best next change]
- Avoid: [things that would add redundancy]
```

If no real gap exists, say so explicitly and do not force a change.

## Phase 2: Choose A Strong Visual Direction

## The Four Design Dimensions

Always address these four areas when creating frontend UIs:

### 1. Typography

**AVOID these overused fonts**:
- Inter, Roboto, Arial, Helvetica, system fonts

**USE distinctive alternatives**:
- **Monospace**: JetBrains Mono, Fira Code, IBM Plex Mono, Space Mono
- **Display/Geometric**: Space Grotesk, Clash Display, Epilogue, Syne
- **Serif**: Playfair Display, Crimson Pro, Libre Baskerville, Merriweather
- **Modern Sans**: Outfit, Manrope, General Sans, Satoshi

**Typography Best Practices**:
- Use **high contrast pairings**: display + monospace, serif + geometric sans
- Employ **extreme weight variations**: 100/200 vs 800/900 (not just 400 vs 600)
- Create **dramatic size jumps**: 3x+ differences, not 1.5x
- Load fonts from Google Fonts and state choices explicitly before coding

**Example pairing**:
```
Heading: Clash Display (800 weight, 4rem)
Body: JetBrains Mono (300 weight, 0.9rem)
Accent: Space Grotesk (700 weight, 1.2rem)
```

### 2. Color & Theme

**AVOID these clichéd schemes**:
- Purple gradients on white backgrounds
- Generic blue-and-white corporate palettes
- Timid, evenly-distributed colors
- Predictable light mode with subtle accents

**USE cohesive aesthetics with CSS variables**:
- **Commit to a theme**: Cyberpunk, Solarpunk, Brutalist, Neo-Tokyo, Retro-Future, Terminal, Vaporwave
- **Dominant colors with sharp accents**: Not all colors equally represented
- **Draw from IDE themes**: Dracula, Nord, Gruvbox, Tokyo Night, Monokai
- **Cultural aesthetics**: Japanese minimalism, Swiss design, Memphis style

**Color Strategy**:
1. Define CSS custom properties for consistency
2. Choose 2-3 dominant colors and 1-2 sharp accents
3. Choose light or dark based on the product, not habit
4. Use gradients sparingly but boldly when used

**Example theme (Cyberpunk)**:
```css
--bg-primary: #0a0e27;
--bg-secondary: #16213e;
--accent-neon: #00ff9f;
--accent-pink: #ff006e;
--text-primary: #e0e0e0;
--text-muted: #8892b0;
```

### 3. Motion

**Implementation approach**:
- **For HTML**: CSS-only animations
- **For React**: Use Framer Motion library when available
- Focus on **high-impact moments** over scattered micro-interactions
- One well-orchestrated page load with staggered reveals > many small animations

**Motion Best Practices**:
- Use `animation-delay` for staggered effects
- Animate on mount/page load for impact
- Keep animations smooth (ease-out, ease-in-out)
- Duration: 0.4s-0.8s for most transitions
- Respect `prefers-reduced-motion`
- Do not rely on animation to communicate essential meaning

**Example staggered reveal**:
```css
.fade-in {
  animation: fadeIn 0.6s ease-out forwards;
  opacity: 0;
}

.fade-in:nth-child(1) { animation-delay: 0.1s; }
.fade-in:nth-child(2) { animation-delay: 0.2s; }
.fade-in:nth-child(3) { animation-delay: 0.3s; }

@keyframes fadeIn {
  to { opacity: 1; transform: translateY(0); }
  from { opacity: 0; transform: translateY(20px); }
}
```

### 4. Backgrounds

**AVOID**: Solid white or solid single-color backgrounds

**USE atmosphere and depth**:
- **Layered CSS gradients**: Multiple gradients with varying opacity
- **Geometric patterns**: Grid lines, dots, diagonal stripes
- **Contextual effects**: Noise texture, grain, subtle animations
- **Thematic elements**: Match overall aesthetic (circuits for tech, organic shapes for nature)

**Background Techniques**:

```css
/* Layered gradients */
background:
  radial-gradient(circle at 20% 50%, rgba(120, 0, 255, 0.3) 0%, transparent 50%),
  radial-gradient(circle at 80% 80%, rgba(0, 255, 200, 0.2) 0%, transparent 50%),
  linear-gradient(135deg, #0a0e27 0%, #16213e 100%);

/* Grid pattern */
background-image:
  linear-gradient(rgba(255, 255, 255, 0.05) 1px, transparent 1px),
  linear-gradient(90deg, rgba(255, 255, 255, 0.05) 1px, transparent 1px);
background-size: 50px 50px;

/* Dots pattern */
background-image: radial-gradient(circle, rgba(255, 255, 255, 0.1) 1px, transparent 1px);
background-size: 20px 20px;
```

## Critical Warnings: Avoid "AI Slop"

**Think outside the box!** It is CRITICAL that you:

- ✅ **Vary between light and dark themes** (don't default to light)
- ✅ **Use different fonts each time** (avoid convergence on Space Grotesk)
- ✅ **Try different aesthetics** (cyberpunk, brutalist, retro, minimalist)
- ❌ **Never use**: Inter, Roboto, Arial as primary fonts
- ❌ **Never default to**: White background with purple accents
- ❌ **Avoid**: Cookie-cutter layouts, predictable component patterns
- ❌ **Don't create**: Generic designs lacking context-specific character

## Design Philosophy Guardrails

When improving an existing interface, verify that the proposal:

- Preserves scanability
- Uses whitespace intentionally
- Keeps information shown once, not repeated
- Respects whether the page is document-like, app-like, or marketing-like
- Adds value, not just visual activity

Questions to ask before implementing:

- Does this solve a visible gap or only add more UI?
- Is this the smallest change that solves the problem?
- Does this fit the page's current density and tone?
- Would this likely survive review, or is it an obvious revert candidate?

## Workflow

When creating frontend UIs, follow this process:

### Step 0: Audit Or Align

- Existing UI: run the audit phase above first.
- New UI inside an existing product: inspect the current design language first.
- New UI from scratch: define a deliberate aesthetic direction first.

### Step 1: Understand Context

Clarify:

- Purpose of the interface
- Target audience
- Desired mood or working style
- Whether the page should feel minimal, dense, editorial, playful, technical, or premium
- Whether the change should preserve an existing design system

### Step 2: Choose Aesthetic Direction
Pick a cohesive theme based on context:
- **Tech/Dev**: Terminal, Cyberpunk, Hacker aesthetic
- **Creative**: Vaporwave, Memphis, Brutalist
- **Professional**: Swiss design, Minimalist, Editorial
- **Organic**: Solarpunk, Nature-inspired, Warm tones

### Step 3: State Design Decisions Explicitly
Before writing code, declare:
```
Theme: [chosen aesthetic]
Fonts: [heading font], [body font], [accent font]
Colors: [dominant colors and accents]
Motion: [key animations planned]
Background: [approach chosen]
Audit result: [what exists / gap being solved]
```

### Step 4: Implement with Best Practices

**Technical Requirements**:
- Use vanilla HTML, CSS, JavaScript OR React as requested
- Include Tailwind CSS if appropriate
- Inline all CSS and JavaScript for single-file deliverables
- Load fonts from Google Fonts
- Use CSS custom properties for theming
- Ensure responsive design (mobile-first approach)
- Keep accessibility intact: contrast, focus states, keyboard reachability, reduced motion

### Step 5: Create High-Impact Moments
- Orchestrate page load animations
- Use staggered reveals for content
- Add hover states that feel responsive
- Consider scroll-triggered animations sparingly

### Step 6: Verify Against The Audit

Before finalizing an existing-page change, confirm:

- No redundant content was added
- New UI solves the stated gap
- Existing hierarchy is clearer, not noisier
- The result still fits the product or page tone

## Examples

### Example 1: Developer Portfolio (Dark Theme)

**Stated Design**:
- Theme: Terminal/Hacker aesthetic with green accents
- Fonts: JetBrains Mono (all text), weight variations for hierarchy
- Colors: Dark bg (#0d1117), green accent (#39ff14), muted gray text
- Motion: Typing animation on hero text, staggered fade-in on projects
- Background: Subtle grid pattern with scanline effect

**Key Features**:
- Monospace typography throughout
- CRT monitor aesthetic with scanlines
- Green phosphor glow effects
- Terminal-style navigation

### Example 2: Creative Agency Landing (Light Theme)

**Stated Design**:
- Theme: Swiss Brutalist with bold typography
- Fonts: Clash Display (headings 900 weight), Space Grotesk (body 300 weight)
- Colors: Off-white (#f5f5f0), black, one sharp red accent (#ff0000)
- Motion: Bold elements sliding in from edges on load
- Background: Off-white with subtle noise texture

**Key Features**:
- Extreme type sizing (hero at 6rem+)
- Asymmetric grid layout
- Sharp geometric shapes as accents
- High contrast black/white with red sparingly

### Example 3: App Dashboard (Dark Theme)

**Stated Design**:
- Theme: Neo-Tokyo night with pink/cyan accents
- Fonts: Outfit (headings 700), IBM Plex Mono (data/numbers)
- Colors: Dark navy (#0a1128), pink (#ff006e), cyan (#00f5ff)
- Motion: Smooth transitions on card hovers, data count-up animations
- Background: Dark with subtle radial gradient and grid

**Key Features**:
- Neon-style accents on interactive elements
- Glassmorphism cards with backdrop blur
- Monospace for all numerical data
- Glowing borders on focus states

## Isolated Dimension Prompting

For targeted control, you can isolate single dimensions:

### Typography-Only Adjustment
"Keeping all other design elements the same, update only the typography using Crimson Pro for headings and Fira Code for body text with extreme weight contrast."

### Theme-Only Adjustment
"Lock the aesthetic to a Solarpunk theme: warm earth tones, organic shapes, nature-inspired patterns, green/gold accents, optimistic futuristic feel."

### Motion-Only Enhancement
"Add a well-orchestrated page load sequence: hero fades in first (0.5s), then navigation staggers in (0.2s delays), then content cards reveal bottom-to-top."

## Font Rotation Strategy

To avoid convergence on the same fonts, rotate through these distinctive combinations:

**Set 1 - Terminal Aesthetic**:
- JetBrains Mono + IBM Plex Mono

**Set 2 - Modern Editorial**:
- Playfair Display + Space Grotesk

**Set 3 - Geometric Bold**:
- Clash Display + Manrope

**Set 4 - Retro Future**:
- Space Mono + Outfit

**Set 5 - Elegant Tech**:
- Crimson Pro + Fira Code

**Set 6 - Neo Brutalist**:
- Syne + General Sans

## Quality Checklist

Before delivering, verify:

- [ ] Existing UI was audited before modification
- [ ] Proposed additions solve a real gap
- [ ] No duplicate metrics, CTAs, or content blocks were added
- [ ] Existing design system or page tone was respected when relevant
- [ ] Fonts are NOT Inter, Roboto, Arial, or system fonts
- [ ] Color scheme is NOT white background with purple accents
- [ ] Typography has extreme weight variations (100-200 vs 800-900)
- [ ] Size jumps are 3x+ for hierarchical contrast
- [ ] Background has depth (gradients, patterns, or effects)
- [ ] Motion is orchestrated (staggered, purposeful)
- [ ] Motion respects reduced-motion users
- [ ] Focus states and contrast remain usable
- [ ] Theme is cohesive and context-appropriate
- [ ] CSS custom properties are used for consistency
- [ ] Design avoids generic, cookie-cutter patterns
- [ ] Aesthetic is distinctive and memorable

## Notes

- **Source**: Based on frontend aesthetics research and evidence-first UI review patterns
- **Merged workflow**: Includes evidence-based UI audit before implementation for existing interfaces
- **Variety is critical**: Consciously vary themes, fonts, and approaches across different projects
- **Context matters**: Match aesthetic to purpose (playful vs professional, technical vs creative)
- **Bold choices**: Better to be distinctive than safe and generic
- **Performance**: Keep animations performant (CSS transforms, not layout changes)
- **Accessibility**: Ensure sufficient contrast ratios and readable font sizes

## References

For deeper exploration, consult:

- `REFERENCE.md` for design direction, accessibility, and audit methodology
- `FORMS.md` for reusable audit templates and checklists
- `resources/examples/` for good/bad audit examples
- `resources/case-studies/` for audit case studies and failure-prevention notes
