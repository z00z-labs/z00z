# deep-wiki in Codex, Copilot, and Other Agents

Verified in this workspace on 2026-06-27.

This guide is for a new user who wants to use `deep-wiki` confidently, not just
know that it exists.

If `deep-wiki` is not installed yet in this repository, use:

```bash
./scripts/install_deep_wiki.sh
```

This workspace currently has:

- Codex plugin: `deep-wiki@personal`
- Repository remote: `https://github.com/z00z-labs/z00z.git`
- Current branch: `z00z-dev`

Examples below use this repository when showing concrete prompts.

## 🎯 What Deep Wiki Is For

`deep-wiki` is a documentation and repository-analysis plugin.

Use it when you want one of these outcomes:

- a complete wiki for the whole codebase
- a fast first-pass wiki for a large repo
- a precise answer about how some subsystem works
- a deeper research pass with traced code paths and evidence
- onboarding guides for new contributors or non-engineering readers
- `llms.txt` files so other agents can understand the repo faster
- a VitePress site or GitHub Pages deployment for the generated docs

The most important mental model is this:

- `ask` answers a question in chat
- `research` investigates a topic deeply in chat
- `page` focuses on one documentable topic
- `catalogue` maps the documentation structure first
- `generate` creates the full documentation set
- `crisp` creates a smaller, faster documentation set

## 🧭 How To Invoke It

`deep-wiki` is not a standalone shell binary. You normally use it from agent
chat surfaces.

### Codex

In Codex, the plugin is installed as `deep-wiki@personal`.

The safest ways to trigger it are:

1. Natural-language requests that clearly match the workflow
2. Command-shaped prompts such as `/deep-wiki:generate` or `/deep-wiki:ask ...`

Treat those command-shaped prompts as chat input, not as shell commands.

Good Codex examples:

```text
Generate a source-cited wiki for this repository.
Research how the wallet scan pipeline works with file citations.
/deep-wiki:ask What does this repo do?
/deep-wiki:research How does wallet session construction work?
```

### GitHub Copilot

This repository has projected prompt files under `.github/prompts/`, so Copilot
can use the explicit command-style prompts:

```text
/deep-wiki:generate
/deep-wiki:crisp
/deep-wiki:ask How does genesis validation work?
/deep-wiki:page Wallet Session Lifecycle
```

### Other Agents

Generic agents can use:

- `AGENTS.md`
- `commands/*.md`
- `skills/*/SKILL.md`

That means the plugin is portable even if the host does not understand Codex or
Copilot-specific wiring.

## 🚀 Quick Start

If you are new and just want the fastest path, use this order:

1. If you want a quick repo map, run `/deep-wiki:crisp`
2. If you want durable full docs, run `/deep-wiki:generate`
3. If you want one answer, run `/deep-wiki:ask ...`
4. If you want a serious investigation, run `/deep-wiki:research ...`

Type these into the agent chat input. Do not run them in `bash`.

For this repository, good first prompts are:

```text
/deep-wiki:crisp

/deep-wiki:generate
Use the GitHub remote for linked citations.

/deep-wiki:ask What are the main crates and what does each one own?

/deep-wiki:research How does the wallet broadcast and retry pipeline work?
Trace actual code paths, separate facts from inference, and end with risks.
```

## 🔑 One Important Thing About Citations

Most major `deep-wiki` workflows start by resolving the source repository
context.

You should be ready to answer one of these:

- provide a remote repo URL when you want clickable linked citations
- answer `local` when the repository is local-only or private

For this repository, the easy answer is:

```text
Use the GitHub remote with linked citations.
```

If you want local file references only, answer:

```text
local
```

Use remote citations when:

- you want shareable markdown with clickable source links
- you plan to publish the generated wiki
- you want teammates or other agents to inspect the sources quickly

Use local citations when:

- the repo is private or not pushed anywhere
- you only care about local work inside this machine

## 🧩 The Main Commands And What They Do

| Command | Best use | Main result | Writes files? |
|---|---|---|---|
| `/deep-wiki:generate` | Full codebase documentation | Full wiki + onboarding + VitePress packaging | Yes |
| `/deep-wiki:crisp` | Fast first pass | Small but usable wiki | Yes |
| `/deep-wiki:catalogue` | Scope the repo before writing | Hierarchical JSON documentation map | Usually chat output unless you ask to save |
| `/deep-wiki:page <topic>` | One topic only | One focused page | Usually yes if you ask for a target page |
| `/deep-wiki:ask <question>` | One question | Direct answer with citations | No, chat answer |
| `/deep-wiki:research <topic>` | Deep investigation | Multi-iteration research report | No, chat answer |
| `/deep-wiki:onboard` | New-reader docs | Four audience-specific onboarding guides | Yes |
| `/deep-wiki:llms` | Agent-readable summaries | `llms.txt` and `llms-full.txt` | Yes |
| `/deep-wiki:agents` | Agent instructions | `AGENTS.md` files where missing | Yes |
| `/deep-wiki:build` | Browsable site | VitePress wiki scaffolding | Yes |
| `/deep-wiki:deploy` | Publish docs | GitHub Pages workflow | Yes |
| `/deep-wiki:ado` | Azure DevOps wiki export | Node build script + converted output path | Yes |
| `/deep-wiki:changelog` | Change summary | Git-based changelog | Usually chat output unless you ask to save |

## 🌍 Run It On The Whole Codebase

If your goal is “document the entire repository,” you usually choose between
`crisp` and `generate`.

### Option A: Fast first pass

Use this when:

- the repo is large
- you want a quick map before going deeper
- you want to avoid a heavy documentation pass immediately

Prompt:

```text
/deep-wiki:crisp
Use the GitHub remote for linked citations.
```

What to expect:

- 5 to 8 core wiki pages
- one contributor onboarding guide
- VitePress scaffolding
- `llms.txt`
- optional deploy workflow if missing

This is the best first run for a repo you barely know.

### Option B: Full documentation pass

Use this when:

- you want the most complete output
- you want onboarding for multiple audiences
- you want a publishable wiki structure
- you are willing to trade more time for more depth

Prompt:

```text
/deep-wiki:generate
Use the GitHub remote for linked citations.
Generate the full wiki for the whole repository.
```

What to expect:

- a complete documentation catalogue
- full wiki pages with citations and Mermaid diagrams
- four onboarding guides
- `llms.txt` and `llms-full.txt`
- VitePress site scaffolding
- optional `AGENTS.md` generation where allowed

### Which One Should You Start With

Use this rule:

- start with `crisp` if you are exploring
- start with `generate` if the docs are an actual deliverable

For a very large codebase, a practical workflow is:

1. run `crisp`
2. read the generated architecture and module pages
3. run `page` or `research` for the hard areas
4. run full `generate` later if needed

## ❓ Ask Questions About The Repo

Use `/deep-wiki:ask` when you want an answer now, not a documentation project.

Basic example:

```text
/deep-wiki:ask What does this repo do?
```

Better examples:

```text
/deep-wiki:ask What are the main crates and what boundary does each one own?

/deep-wiki:ask How does wallet scan state survive restart?
Answer in Russian, but keep code citations.

/deep-wiki:ask Which files define genesis configuration loading and validation?
Give me a table first, then a short explanation.
```

What `/deep-wiki:ask` is good at:

- architecture questions
- “where is X implemented?”
- “which files matter?”
- “how does this request flow work?”
- “what are the config files and what do they control?”

What the answer should usually include:

- a direct answer
- code-grounded explanation
- key files table
- citations
- sometimes a small diagram when flow or architecture matters

Important detail:

- `ask` is designed to answer in the same language as the question

So if you ask in Russian, a Russian answer is a reasonable expectation.

## 🔬 Do Deep Investigation

Use `/deep-wiki:research` when the topic is too big for a one-shot Q&A answer.

This mode is intentionally heavier than `ask`.

It is meant for:

- tracing real call chains
- understanding failure modes
- identifying risks and technical debt
- building a structured mental model of one subsystem

Example prompts:

```text
/deep-wiki:research How does wallet session construction work?

/deep-wiki:research How does the rollup node runtime initialize its dependencies?
Trace actual code paths, include confidence ratings, and highlight unknowns.

/deep-wiki:research How does HJMT publication flow through storage, runtime, and wallet surfaces?
Focus on state transitions, persistence seams, and retry behavior.
```

What makes `research` different from `ask`:

- it runs as a 5-iteration research cycle
- it keeps a running knowledge map
- it is expected to separate facts from inference
- it includes diagrams and structured tables
- it is better for “understand this subsystem end to end”

If you only want a quick answer, use `ask`.
If you want an analysis artifact, use `research`.

## 📝 Generate One Page Instead Of A Whole Wiki

Use `/deep-wiki:page` when you know the topic and do not need the whole repo
documented yet.

Examples:

```text
/deep-wiki:page Wallet Session Lifecycle

/deep-wiki:page Genesis Configuration Loading
Write it for a new engineer and include sequence and state diagrams.

/deep-wiki:page HJMT Publication Contract
Focus on invariants, storage, and tests.
```

Best practice with `page`:

- name the topic clearly
- say who the audience is
- say how deep you want it
- say what diagram types or sections you want
- if you care about the output path, explicitly ask where to save it

For example:

```text
/deep-wiki:page Wallet Broadcast Retry
Create a page for a contributor audience.
Keep it under one page plus one diagram and save it under the retry section of the wiki.
```

## 🗂️ Generate The Structure First

Use `/deep-wiki:catalogue` when you want the map before the writing.

This is good for:

- large repos
- scoping discussions
- agreeing on documentation sections before a full generate pass
- deciding what is worth documenting deeply

Example:

```text
/deep-wiki:catalogue
Use linked citations and derive sections from the actual repository structure.
```

What you get:

- a hierarchical JSON structure
- sections for getting started and deep dive
- prompts for later page generation

This is a strong choice if you want to review the plan before spending tokens on
full page generation.

## 👥 Generate Onboarding Guides

Use `/deep-wiki:onboard` when your main goal is reader onboarding, not general
reference docs.

It generates four guides:

- contributor guide
- staff engineer guide
- executive guide
- product manager guide

Example:

```text
/deep-wiki:onboard
Use linked citations and make the contributor guide practical for first-week setup.
```

This is useful when:

- a new engineer is joining
- you want leadership-facing summaries
- you want PM-safe documentation without code-level depth

If you only need contributor onboarding quickly, `crisp` may be enough because
it creates one contributor guide. If you need all audiences, use `onboard` or
the full `generate`.

## 🤖 Generate llms.txt For Other Agents

Use `/deep-wiki:llms` when you want other coding agents to understand the repo
through a standard summary file.

Example:

```text
/deep-wiki:llms
Use the generated wiki pages and keep the summaries dense and specific.
```

Expected outputs:

- `./llms.txt`
- `wiki/llms.txt`
- `wiki/llms-full.txt`

Use this when:

- you want machine-readable documentation entry points
- you want future agent sessions to load the repo faster
- you want a documentation summary without reading the whole wiki

## 🧱 Build A Browsable Wiki Site

Use `/deep-wiki:build` when you already have wiki markdown and want a proper
VitePress site around it.

Example:

```text
/deep-wiki:build
```

This workflow creates:

- `wiki/package.json`
- `wiki/.vitepress/config.mts`
- theme files
- public assets
- a developer-style landing page

Important:

- this is for packaging existing wiki content
- it is not the best first step if you do not have pages yet

A common sequence is:

1. `generate` or `crisp`
2. `build`
3. preview locally
4. `deploy`

## 🌐 Deploy To GitHub Pages

Use `/deep-wiki:deploy` when the `wiki/` site already exists and you want CI to
publish it.

Example:

```text
/deep-wiki:deploy
```

It is expected to:

- check that `wiki/` exists
- inspect existing Pages workflows
- create `.github/workflows/deploy-wiki.yml` if appropriate
- update VitePress base path if needed
- require GitHub Pages to be enabled in repository settings

Use this after `build`, not before.

## 🏢 Export For Azure DevOps Wiki

Use `/deep-wiki:ado` when you need Azure DevOps wiki-compatible markdown.

Example:

```text
/deep-wiki:ado
```

Expected result:

- a Node.js build script such as `scripts/build-ado-wiki.js`
- transformed output under `dist/ado-wiki/`

This is relevant when:

- your publishing target is ADO Wiki instead of GitHub Pages
- you need Mermaid and markdown adjusted for ADO compatibility

## 📦 What Files Deep Wiki Usually Produces

When you run file-writing workflows, these are the most common outputs:

| Output | Why it exists |
|---|---|
| `wiki/` | Main generated documentation folder |
| `wiki/index.md` | Technical landing page for the wiki |
| `wiki/onboarding/` | Audience-specific onboarding guides |
| `wiki/.vitepress/` | VitePress configuration and theme |
| `llms.txt` | Root discovery file for other agents |
| `wiki/llms.txt` | Wiki-local link summary |
| `wiki/llms-full.txt` | Full inlined documentation bundle |
| `.github/workflows/deploy-wiki.yml` | GitHub Pages deployment workflow |
| `dist/ado-wiki/` | Azure DevOps-compatible markdown output |

Not every command writes all of these. `ask` and `research` are primarily
chat-output workflows.

## 🧠 What Kinds Of Answers You Can Request

You are not limited to “explain this.”

You can ask `deep-wiki` to shape the answer in a specific way.

Examples:

```text
/deep-wiki:ask How does the wallet RPC layer work?
Start with a 10-line summary, then give a key-files table.
```

```text
/deep-wiki:research How does genesis asset loading work?
Focus on invariants, validation layers, and likely failure modes.
```

```text
/deep-wiki:ask How does this repo start up?
Answer as a checklist for a new contributor.
```

```text
/deep-wiki:ask Explain the rollup node runtime like I am a PM.
No Rust jargon.
```

Useful answer shapes to request:

- short summary first
- full deep dive second
- key files table
- glossary
- reading order
- sequence diagram
- state diagram
- risk list
- design trade-off table
- FAQ
- executive summary
- contributor-oriented explanation
- PM-safe explanation
- “facts first, then inference”
- “only show sources and conclusions”

## 🛠️ Better Prompting Patterns

Weak prompt:

```text
/deep-wiki:ask How does wallet work?
```

Better prompt:

```text
/deep-wiki:ask How does wallet session construction work?
Focus on initialization, password state, and inactive-session variants.
Give me a table of the main files first.
```

Weak prompt:

```text
/deep-wiki:research explain HJMT
```

Better prompt:

```text
/deep-wiki:research How does HJMT publication move through runtime, storage, and wallet-facing surfaces?
Trace actual code paths, mark unresolved areas, and end with top 5 risks.
```

Weak prompt:

```text
/deep-wiki:page genesis
```

Better prompt:

```text
/deep-wiki:page Genesis Configuration and Validation
Write it for a new Rust contributor.
Include a sequence diagram and a troubleshooting section.
```

The general rule is:

- say the topic
- say the audience
- say the output shape
- say the depth
- say whether you want linked or local citations

## ⚠️ Common Mistakes

### Asking For Too Much Too Early

Do not start with full `generate` if what you really need is one answer.

Use:

- `ask` for one question
- `research` for one subsystem
- `page` for one document
- `catalogue` for planning

### Forgetting To Set Citation Mode

If the host asks whether the repo is local-only or has a source URL, answer it.
That choice affects every citation in the output.

### Using `build` Before You Have Content

`build` packages markdown into a site. It is not the right first step if pages
do not exist yet.

### Expecting `ask` To Replace `research`

`ask` is for targeted answers.
`research` is for layered investigation.

### Forgetting To Ask For Audience And Format

If you care about the answer shape, say so. The plugin is much more useful when
you specify:

- audience
- depth
- tables vs prose
- diagrams
- summary length

## ✅ Recommended Workflows

### Workflow 1: “I just opened an unfamiliar repo”

```text
/deep-wiki:crisp
/deep-wiki:ask What are the main subsystems and which one should I read first?
```

### Workflow 2: “I need durable docs for the whole project”

```text
/deep-wiki:generate
/deep-wiki:llms
/deep-wiki:build
/deep-wiki:deploy
```

### Workflow 3: “I need to understand one risky subsystem”

```text
/deep-wiki:research How does the wallet broadcast retry pipeline work?
/deep-wiki:page Wallet Broadcast Retry
```

### Workflow 4: “I need docs other agents can consume”

```text
/deep-wiki:generate
/deep-wiki:llms
/deep-wiki:agents
```

## 📍 Practical Prompts For This Repository

These are good prompts to reuse in `z00z` specifically:

```text
/deep-wiki:ask What are the main crates in z00z and what does each one own?
```

```text
/deep-wiki:research How does wallet session construction work in z00z?
Trace actual code paths and end with a contributor reading order.
```

```text
/deep-wiki:page Genesis Configuration and Validation in z00z
```

```text
/deep-wiki:ask Which files define rollup-node runtime startup and dependency wiring?
Give me a compact table first.
```

```text
/deep-wiki:research How does HJMT publication flow through z00z runtime and storage?
Focus on invariants, persistence, retry behavior, and failure surfaces.
```

## 🧪 How To Tell If You Used The Right Command

You probably chose the right tool if:

- `ask` gave you a direct answer with files you can open immediately
- `research` produced a staged analysis, not a shallow summary
- `page` gave you one focused artifact instead of a repo dump
- `crisp` gave you a small wiki quickly
- `generate` gave you a real documentation project, not just a chat answer

If the result feels too broad:

- move from `generate` to `page` or `ask`

If the result feels too shallow:

- move from `ask` to `research`

If the scope still feels fuzzy:

- run `catalogue` first

## 🔚 Final Advice For A New User

If you remember only one strategy, use this one:

1. start with `crisp` on a repo you do not know
2. use `ask` for concrete questions
3. use `research` for anything architecture-heavy or risky
4. use `generate` only when you want durable full docs
5. add `llms` and `build` when the output should become shared infrastructure

That workflow keeps the plugin useful instead of turning every request into a
full documentation project.
