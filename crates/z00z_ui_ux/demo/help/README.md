# Help maintenance

`topics.yaml` is the canonical Help-topic registry. It contains one global topic,
one contextual topic for every canonical route, and the supported detail/dialog
topics. The compiled catalogue is a build artifact; never edit
`scripts/generated/help-catalog.js` directly.

Author Markdown in:

```text
help/<locale>/
  app/
  wallets/
  telemetry/
  dapps/
  messenger/
  contacts/
  settings/
```

Every one of the ten locales must have exact canonical path/topic parity.
`topics.yaml` references only the current topic directories. Original tracked
Markdown remains at the paths listed in `preserved-sources.json`, including the
legacy `network/` sources and English locale-root documents, and must never be
deleted by Help tooling. Shared `paths` are required in every locale;
`optionalLocalePaths` protects user-maintained originals that may exist only in
one locale without making them runtime topics. Those preserved sources are
provenance inputs, not runtime catalogue entries. `_drafts/` is also excluded
from the runtime catalogue.

English Help also carries the same content-navigation contract used by
`z00z-website`: each published section has a front-matter-only `index.md`, and
its `_meta.yaml` owns the section title, neutral structural icon, and explicit
page order. The root `_meta.yaml` orders the Help sections. `check-help.mjs`
requires this English content map to match `topics.yaml`; preserved legacy
sources remain on disk but are intentionally excluded from that map.

To add or change Help:

1. Add or update the stable entry in `topics.yaml`.
2. Edit the English Markdown source in its canonical group.
3. Run `node scripts/scaffold-help.mjs <topic-id>` for a new topic.
4. Review all translated documents and record synchronized English hashes with
   `node scripts/sync-help.mjs --record-reviewed`.
5. Run `node scripts/compile-help.mjs` and `node scripts/check-help.mjs`.

The global Help action and contextual question button always open the same
standalone Help application. Its global tree has root-only multi-open
accordions. Deeper sibling topics render as a desktop internal rail or mobile
sticky horizontal tabs. The app and Help remain independently closable.

In a packaged Tauri build the renderer invokes only `open_or_focus_help` with
`topicId`, `locale`, `palette`, and optional `section`. Never add wallet IDs,
labels, addresses, form values, raw URLs, paths, secrets, or generic
window-creation fields to that payload.
