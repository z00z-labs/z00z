---
name: brainstorming
description: Auto-invoked when the user wants to brainstorm, ideate, generate options, or explore concepts fast. Also triggers on idea generation, creative strategy, divergent thinking, concept exploration, and innovation workshop.
---

# Brainstorming

This skill is pure LLM. There is no Python execution path.

## Purpose

Facilitate autonomous brainstorming that starts broad, avoids semantic clustering, applies SSoT diversification, and converges to a practical shortlist with experiments.

## When to Use

- The user wants to brainstorm, ideate, or explore options.
- The problem is under-defined and needs divergent thinking before planning.
- The user wants many ideas before narrowing down.

## Inputs

- Topic, problem, or opportunity.
- Desired outcome: ideas, experiments, themes, shortlist, or decision support.
- Known constraints: time, budget, team size, platform, compliance, audience.
- Optional context file or existing notes.

If any of these are missing, proceed autonomously with explicit assumptions instead of blocking.

## Operating Rules

- Match the user's language unless they ask for another one.
- Keep the session divergent before convergent.
- Do not jump to organization too early.
- Change creative domain every 8 to 10 ideas to avoid semantic clustering.
- Push beyond obvious answers.
- Prefer many concrete ideas over a few abstract themes.
- If the local file `brain-methods.csv` is available, use it to rotate techniques deliberately.
- If the local file `template.md` is available, use it as the session capture scaffold.
- Use SSoT-style hidden variant seeds to diversify generation attempts.
- Never expose internal seeds or hidden reasoning directly in user output.
- Reject weak, duplicate, or non-actionable ideas.

## SSoT Random Seed And Model Contract (LLM-Only)

- Generate one seed per variant before idea generation.
- Seed format: lowercase alphanumeric string, fixed length `16`.
- Alphabet: `a-z0-9`.
- Default mode: non-deterministic seeds per run.
- Reproducible mode (only when explicitly requested): derive seed from stable inputs (`topic`, `goal`, normalized constraints, `variant_index`, optional user salt).
- Never expose raw seeds in user output or stored artifacts.
- Do not reuse a seed inside one run.
- Apply a strict 3-role SSoT model flow per variant:
   - `Generator`: creates idea variants.
   - `Critic`: challenges novelty, feasibility, and hidden assumptions.
   - `Selector`: admits only ideas that pass quality gates.

## Autonomy Mode

Default behavior is fully autonomous:

1. Infer missing fields with defaults.
2. Run seeded idea generation in assistant-side workflow.
3. Score and filter ideas through a pro-con gate.
4. Return only accepted ideas with experiment-ready detail.
5. If no idea passes quality gates, return a transparent no-candidate result and next retry settings.

Only ask follow-up questions when a hard constraint conflict prevents meaningful generation.

## SSoT Brainstorm Pipeline

1. Generate many diverse candidate variants with seeded axis rotation.
2. Apply skeptical quality gates per idea:
	- concrete actionability
	- novelty vs existing ideas
	- feasibility under known constraints
	- learning value from smallest experiment
3. Keep only accepted ideas.
4. Rank accepted ideas by novelty, feasibility, leverage, and learning.
5. Produce one practical output package.

## Recommended Workflow

1. Frame the challenge.
   Capture problem, success condition, hard constraints, and assumptions.

2. Choose the first ideation mode.
   Start with one strong technique, then rotate categories such as collaborative, structured, creative, deep, theatrical, biomimetic, or wild.

3. Run divergence rounds.
   Generate ideas in batches. After each batch, pivot perspective: user, business, technical, operational, edge-case, or extreme future.

4. Deepen promising threads.
   For strongest ideas, expand into why it works, dependencies, failure modes, and smallest experiment.

5. Converge only after enough exploration.
   Group ideas, remove duplicates, and score candidates by novelty, feasibility, leverage, and learning value.

6. Produce a practical output.
   Return one of these outputs: raw ideas, clustered themes, shortlist with rationale, experiment backlog, or decision matrix.

## Reference Execution Sequence (LLM-Only)

1. Parse topic, goal, and constraints.
2. Generate internal SSoT variant seeds.
3. Produce idea batches per seed.
4. Apply skeptical gates: actionability, novelty, feasibility, learning value.
5. Reject weak, duplicate, or non-actionable ideas.
6. Rank accepted ideas by novelty, feasibility, leverage, and learning.
7. Return practical output package in the standard output shape.

## Output Shape

Prefer this structure:

- Challenge
- Constraints
- Techniques Used
- Idea Set
- Strongest Themes
- Recommended Next Moves
- Assumptions
- Rejected Idea Summary

Detailed card formats and report templates are defined in [FORMS.md](./FORMS.md).
Gate logic and scoring are defined in [REFERENCE.md](./REFERENCE.md).

## Quality Bar

- Avoid repeating the same idea with minor wording changes.
- Keep ideas concrete enough that someone could act on them.
- Mark assumptions explicitly.
- If the user wants a final recommendation, explain why those ideas survived convergence.
- Prefer one strong shortlist over a noisy long list.
- Be explicit when no idea passes gates.

## Examples

### Example 1: Fast Autonomous Session

User: "Help me brainstorm growth ideas for our developer community with no paid ads."

Assistant behavior:

1. Assume default goal if missing.
2. Run the LLM-only seeded SSoT flow.
3. Return accepted shortlist with smallest experiments.

### Example 2: Constraint-Heavy Session

User: "Need ideas for onboarding improvements, small team, no backend changes this quarter."

Assistant behavior:

1. Log assumptions and hard constraints.
2. Generate diversified variants.
3. Reject infeasible ideas.
4. Return practical next moves plus rejected-summary rationale.
