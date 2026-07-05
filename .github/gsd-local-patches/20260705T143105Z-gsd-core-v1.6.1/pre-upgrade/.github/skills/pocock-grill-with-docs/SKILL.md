---
name: grill-with-docs
description: Grilling session that challenges a plan against the existing domain model, sharpens terminology, and updates only the documentation locations the user or project already provides. Use when user wants to stress-test a plan against project language and documented decisions.
argument-hint: "[plan or decision] [docs target path|chat-only]"
---

# Grill With Docs

## Examples of usage

- `grill-with-docs "replace billing sync with event publication"`
- `grill-with-docs plan.md docs/domain/CONTEXT.md`
- `grill-with-docs "stress-test the session model" chat-only`
- `grill-with-docs "should we split the reporting module?" docs/decisions/`

<what-to-do>

Interview me relentlessly about every aspect of this plan until we reach a shared understanding. Walk down each branch of the design tree, resolving dependencies between decisions one-by-one. For each question, provide your recommended answer.

Ask the questions one at a time, waiting for feedback on each question before continuing.

If a question can be answered by exploring the codebase, explore the codebase instead.

</what-to-do>

<supporting-info>

## Domain awareness

During codebase exploration, also look for existing documentation:

### Output location discipline

Use documentation locations in this order:

1. The exact file or directory the user names.
2. Existing project documentation files that already carry the relevant purpose.
3. Chat-only notes when no durable destination is specified.

Do not create a new documentation tree just because this skill mentions one. If a glossary, context map, decision log, ADR directory, or issue tracker location is missing, ask the user where to write or keep the result in chat.

### Common project patterns

Some repos have a single context file:

```
/
├── CONTEXT.md
├── docs/
│   └── adr/
│       ├── 0001-event-sourced-orders.md
│       └── 0002-postgres-for-write-model.md
└── src/
```

Some repos have multiple context files and a map that points to where each one lives:

```
/
├── CONTEXT-MAP.md
├── docs/
│   └── adr/                          ← system-wide decisions
├── src/
│   ├── ordering/
│   │   ├── CONTEXT.md
│   │   └── docs/adr/                 ← context-specific decisions
│   └── billing/
│       ├── CONTEXT.md
│       └── docs/adr/
```

Treat these as examples, not required folder trees. Create or update files only at user-approved destinations.

## During the session

### Challenge against the glossary

When the user uses a term that conflicts with the existing glossary or context language, call it out immediately. "Your glossary defines 'cancellation' as X, but you seem to mean Y — which is it?"

### Sharpen fuzzy language

When the user uses vague or overloaded terms, propose a precise canonical term. "You're saying 'account' — do you mean the Customer or the User? Those are different things."

### Discuss concrete scenarios

When domain relationships are being discussed, stress-test them with specific scenarios. Invent scenarios that probe edge cases and force the user to be precise about the boundaries between concepts.

### Cross-reference with code

When the user states how something works, check whether the code agrees. If you find a contradiction, surface it: "Your code cancels entire Orders, but you just said partial cancellation is possible — which is right?"

### Update glossary inline

When a term is resolved and the user or project has identified a glossary/context destination, update it right there. Don't batch these up — capture them as they happen. Use the format in [CONTEXT-FORMAT.md](./CONTEXT-FORMAT.md).

If no destination is identified, keep a chat-only "Resolved terminology" list and ask where to write it before editing files.

The glossary/context file should be totally devoid of implementation details. Do not treat it as a spec, a scratch pad, or a repository for implementation decisions. It is a glossary and nothing else.

### Offer ADRs sparingly

Only offer to create an ADR when all three are true:

1. **Hard to reverse** — the cost of changing your mind later is meaningful
2. **Surprising without context** — a future reader will wonder "why did they do it this way?"
3. **The result of a real trade-off** — there were genuine alternatives and you picked one for specific reasons

If any of the three is missing, skip the ADR. Use the format in [ADR-FORMAT.md](./ADR-FORMAT.md).

</supporting-info>
