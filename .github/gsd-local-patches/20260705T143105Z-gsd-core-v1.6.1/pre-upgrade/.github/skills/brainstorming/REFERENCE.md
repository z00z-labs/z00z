# Brainstorming Reference

## Objective

Generate diverse idea candidates autonomously, reject weak options, and return a practical shortlist with experiment-ready next steps.

## SSoT Variant Axes

Each variant rotates independently across:

- perspective: user, buyer, operator, builder, maintainer, edge-case adversary
- horizon: immediate, quarterly, annual, long-term
- mechanism: workflow, product, pricing, tooling, community, automation
- risk mode: low-risk optimization, medium novelty, high-upside wildcard
- method category from `brain-methods.csv`

Seeds are internal only and must never be exposed directly.

## SSoT Seed And Model Contract (Canonical)

Treat this section as normative for all brainstorming runs.

- Generate one internal seed per variant before idea generation.
- Seed format: lowercase alphanumeric, fixed length `16`, alphabet `a-z0-9`.
- Default mode: non-deterministic seeds per run.
- Reproducible mode (only when explicitly requested): derive deterministic seeds from stable inputs (`topic`, `goal`, normalized constraints, `variant_index`, optional user salt).
- Never expose raw seeds in user output or stored artifacts.
- Never reuse a seed inside one run.
- Run every variant through strict 3-role SSoT model flow:
  - `Generator`: creates idea variants.
  - `Critic`: challenges novelty, feasibility, and hidden assumptions.
  - `Selector`: keeps only ideas that pass quality gates.

## Required Gates

An idea is accepted only if all checks pass:

1. Actionability: includes a concrete implementation or experiment step.
2. Distinctness: not a wording clone of another accepted idea.
3. Feasibility: does not violate hard constraints.
4. Learning value: defines what will be learned from execution.
5. Scope clarity: states target user or system boundary.

## Rejection Rules

Reject an idea when any of the following hold:

- purely abstract with no first step
- duplicate or near-duplicate of another idea
- depends on unavailable or forbidden resources
- contradicts hard constraints
- no measurable experiment can be defined

## Scoring

Use 1-5 for each dimension:

- novelty
- feasibility
- leverage
- learning value

Total score = novelty + feasibility + leverage + learning.

## Selection Rule

For shortlist mode, prefer ideas with:

- highest total score
- highest leverage under constraints
- lowest dependency burden
- clearest first experiment

## No-Candidate Policy

If no idea passes gates:

- report transparent no-candidate result
- include top rejection reasons
- rerun guidance: increase variants, relax non-critical constraints, or pivot method categories
