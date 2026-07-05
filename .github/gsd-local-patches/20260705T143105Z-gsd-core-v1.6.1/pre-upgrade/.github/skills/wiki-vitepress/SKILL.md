---
name: wiki-vitepress
description: Packages generated wiki Markdown into a VitePress static site with dark theme, Mermaid diagrams that preserve the `mermaid-spectrum` semantic palette, click-to-zoom, and production build output. Use when the user wants to create a browsable website from generated wiki pages.
license: MIT
metadata:
  author: Microsoft
  version: "1.0.0"
---

# Wiki VitePress Packager

Transform generated wiki Markdown files into a polished VitePress static site with dark theme and interactive Mermaid diagrams that preserve semantic inline colors.

## When to Activate

- User asks to "build a site" or "package as VitePress"
- User runs the `/deep-wiki:build` command
- User wants a browsable HTML output from generated wiki pages

## VitePress Scaffolding

Generate the following structure in a `wiki-site/` directory:

```
wiki-site/
├── .vitepress/
│   ├── config.mts
│   └── theme/
│       ├── index.ts
│       └── custom.css
├── public/
├── [generated .md pages]
├── package.json
└── index.md
```

## Config Requirements (`config.mts`)

- Use `withMermaid` wrapper from `vitepress-plugin-mermaid`
- Set `appearance: 'dark'` for dark-only theme
- Configure `themeConfig.nav` and `themeConfig.sidebar` from the catalogue structure
- Mermaid config must use `theme: 'base'` and theme variables that mirror the `mermaid-spectrum` palette so unstyled elements remain compatible without flattening styled diagrams:

```typescript
mermaid: {
  theme: 'base',
  themeVariables: {
    background: '#FFFFFF',
    primaryColor: '#F3E5F5',
    primaryTextColor: '#4A148C',
    primaryBorderColor: '#8E24AA',
    lineColor: '#546E7A',
    secondaryColor: '#E3F2FD',
    secondaryTextColor: '#0D47A1',
    secondaryBorderColor: '#1E88E5',
    tertiaryColor: '#FFF3E0',
    tertiaryTextColor: '#E65100',
    tertiaryBorderColor: '#FB8C00',
    mainBkg: '#F3E5F5',
    nodeBorder: '#8E24AA',
    clusterBkg: '#ECEFF1',
    clusterBorder: '#546E7A',
    titleColor: '#263238',
    edgeLabelBackground: '#FFFFFF',
    noteBkgColor: '#E8F5E9',
    noteTextColor: '#1B5E20',
    critBkgColor: '#FFE0E0',
    critBorderColor: '#D32F2F',
    doneTaskBkgColor: '#E8F5E9',
    doneTaskBorderColor: '#43A047'
  }
}
```

## Mermaid Spectrum Preservation

### Layer 1: Theme Variables (in `config.mts`)
Set via `mermaid.themeVariables` as shown above.

### Layer 2: Container Styling Only (`custom.css`)
Style the Mermaid container, not the nodes themselves. Do NOT override SVG node fill/stroke/text colors with `!important`, or you will erase semantic inline colors:

```css
.vp-doc .mermaid {
  margin: 1.5rem 0;
  padding: 1rem;
  border: 1px solid rgba(84, 110, 122, 0.24);
  border-radius: 16px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.98), rgba(248, 250, 252, 0.98));
  box-shadow: 0 14px 28px rgba(12, 17, 23, 0.12);
  overflow-x: auto;
}
```

### Rule: Never Recolor Inline Styles in `theme/index.ts`
`theme/index.ts` may add zoom behavior, but it must never mutate Mermaid node colors. The generator already emits semantic `style`, `classDef`, and `box rgb(...)` directives; keep them intact.

## Click-to-Zoom for Mermaid Diagrams

Wrap each `.mermaid` container in a clickable wrapper that opens a fullscreen modal:

```typescript
document.querySelectorAll('.mermaid').forEach(el => {
  el.style.cursor = 'zoom-in'
  el.addEventListener('click', () => {
    const modal = document.createElement('div')
    modal.className = 'mermaid-zoom-modal'
    modal.innerHTML = el.outerHTML
    modal.addEventListener('click', () => modal.remove())
    document.body.appendChild(modal)
  })
})
```

Modal CSS:
```css
.mermaid-zoom-modal {
  position: fixed; inset: 0;
  background: rgba(0,0,0,0.9);
  display: flex; align-items: center; justify-content: center;
  z-index: 9999; cursor: zoom-out;
}
.mermaid-zoom-modal .mermaid { transform: scale(1.5); }
```

## Post-Processing Rules

Before VitePress build, scan all `.md` files and fix:
- Replace `<br/>` with `<br>` (Vue template compiler compatibility)
- Wrap bare `<T>` generic parameters in backticks outside code fences
- Ensure every page has YAML frontmatter with `title` and `description`

## Build

```bash
cd wiki-site && npm install && npm run docs:build
```

Output goes to `wiki-site/.vitepress/dist/`.

## Known Gotchas

- Mermaid renders async — SVGs may not exist when `onMounted` fires. Retry zoom binding if needed.
- `isCustomElement` compiler option for bare `<T>` causes worse crashes — do NOT use it
- Do NOT add `.mermaid .node` or `.mermaid text` force-overrides with `!important`; that destroys semantic colors
- `enhanceApp()` runs during SSR where `document` doesn't exist — use `setup()` only
