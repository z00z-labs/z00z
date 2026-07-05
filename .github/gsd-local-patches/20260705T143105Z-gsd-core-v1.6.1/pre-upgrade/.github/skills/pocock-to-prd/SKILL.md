---
name: to-prd
description: Turn the current conversation context into a PRD and write it to the user-requested destination, such as a file, issue draft, or chat-only response. Use when user wants to create a PRD from the current context.
argument-hint: "[feature/context] [output path|issue target|chat-only]"
---

# To PRD

## Examples of usage

- `to-prd "offline checkout mode" docs/prd/offline-checkout.md`
- `to-prd "wallet backup recovery" chat-only`
- `to-prd "agent observability" issue draft`
- `to-prd "billing export workflow" product/PRD.md`

This skill takes the current conversation context and codebase understanding and produces a PRD. Do NOT interview the user — just synthesize what you already know.

## Destination Rules

- Write the PRD only to the destination the user provides, or return it in chat when the user asks for chat-only output.
- If the user asks for an issue, create or draft an issue only when the current environment provides an issue-tracker tool and the target project, labels, and repository are clear.
- If no destination is provided, ask where the PRD should go before creating files or publishing anything.
- Do not assume a setup command, issue tracker, triage label, folder tree, or repository-specific PRD path.

## Process

1. Explore the repo to understand the current state of the codebase, if you haven't already. Use the project's domain glossary vocabulary throughout the PRD, and respect any ADRs in the area you're touching.

2. Sketch out the major modules you will need to build or modify to complete the implementation. Actively look for opportunities to extract deep modules that can be tested in isolation.

A deep module (as opposed to a shallow module) is one which encapsulates a lot of functionality in a simple, testable interface which rarely changes.

Record module and testing assumptions directly in the PRD. If the source context is ambiguous, mark the uncertainty in the PRD instead of interviewing the user.

3. Write the PRD using the template below, then deliver it to the requested destination. Do not publish to an issue tracker or apply labels unless the user explicitly requested that target and the label vocabulary is known.

<prd-template>

## Problem Statement

The problem that the user is facing, from the user's perspective.

## Solution

The solution to the problem, from the user's perspective.

## User Stories

A LONG, numbered list of user stories. Each user story should be in the format of:

1. As an <actor>, I want a <feature>, so that <benefit>

<user-story-example>
1. As a mobile bank customer, I want to see balance on my accounts, so that I can make better informed decisions about my spending
</user-story-example>

This list of user stories should be extremely extensive and cover all aspects of the feature.

## Implementation Decisions

A list of implementation decisions that were made. This can include:

- The modules that will be built/modified
- The interfaces of those modules that will be modified
- Technical clarifications from the developer
- Architectural decisions
- Schema changes
- API contracts
- Specific interactions

Do NOT include specific file paths or code snippets. They may end up being outdated very quickly.

Exception: if a prototype produced a snippet that encodes a decision more precisely than prose can (state machine, reducer, schema, type shape), inline it within the relevant decision and note briefly that it came from a prototype. Trim to the decision-rich parts — not a working demo, just the important bits.

## Testing Decisions

A list of testing decisions that were made. Include:

- A description of what makes a good test (only test external behavior, not implementation details)
- Which modules will be tested
- Prior art for the tests (i.e. similar types of tests in the codebase)

## Out of Scope

A description of the things that are out of scope for this PRD.

## Further Notes

Any further notes about the feature.

</prd-template>
