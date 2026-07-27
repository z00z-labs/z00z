# Help maintenance

English Help mirrors the live Demo navigation. `topics.yaml` is generated from
`scripts/port/navigation-model.js`; run
`node scripts/help/write-navigation-manifest.mjs` after a navigation change.
The compiled catalogue is a build artifact—never edit
`scripts/generated/help-catalog.js` directly.

Each route and supported contextual detail has one Markdown page under
`help/en/`, using [TEMPLATE.md](TEMPLATE.md). The path follows the app's root
and workspace nesting. `scripts/check-help.mjs` proves 63 route pages, nine
contextual-detail pages, the required `App View` and `Terms and controls`
sections, and catalogue freshness.

Use `python3 scripts/help/sync_views.py` to capture every English view. It
writes PNGs to `help/assets/en/`, stores comparison baselines in
`help/en/_generated/`, and adds review-only `*-draft-YYYYMMDD.md` pages beside
canonical Markdown when visible terms, headings, controls, component signatures,
or settled presentation signatures change. Canonical Help prose is never overwritten.
Use `--check` for portable baseline integrity and `--verify-current` for a live
Chromium drift gate.

Markdown rendering is an auditable snapshot of the `z00z-website` parser. Run
`node scripts/help/sync-website-markdown.mjs` when that upstream parser changes,
then `node scripts/test-help-markdown-parity.mjs`. Run
`node scripts/help/sync-markdown-runtime.mjs` to prepare the local Mermaid and
KaTeX assets used by the network-free Help presentation, then run
`node scripts/compile-help.mjs`, and `node scripts/check-help.mjs`.

Other locale folders remain preserved during this English-first phase. They are
not published through the new Help catalogue until separately authored and
reviewed.
