# Model Router Reference

📌 This file provides the conservative scoring baseline for task tiers, model
families, and provider recommendations.

## Evidence Order

1. Official model overview and model detail pages.
2. Official provider catalogs and capability APIs.
3. Official pricing and context-window documentation.
4. First-party evaluation or migration guidance.
5. Local benchmark or evaluation results from the current workspace.
6. Community claims only as weak supporting evidence, never as primary proof.

## Task Tiers

| Tier | Label | Typical tasks | Conservative rule |
| --- | --- | --- | --- |
| 0 | Mechanical | Reformatting, translation, text cleanup, schema fill | Cheapest model can be primary if inputs are clean and bounded |
| 1 | Bounded execution | Single-file edits, extraction, classification, short summaries | Small models are allowed if the scope is explicit and failure cost is low |
| 2 | Narrow implementation | Small feature patch, short code generation, multi-step but local reasoning | Mini and haiku models are allowed only with clear boundaries and a stronger fallback |
| 3 | Broad implementation | Multi-file changes, debugging with uncertainty, long-context analysis | Default to strong mid/high tier models |
| 4 | High-risk judgment | Security review, architecture, protocol design, ambiguous refactor, novel algorithm work | Use strongest documented model in the pool unless local evals prove otherwise |

## Model Baselines

📌 These baselines are intentionally stricter than vendor marketing.

| Model | First-party evidence summary | Safe default tier | Escalate only if | Conservative notes |
| --- | --- | --- | --- | --- |
| Claude Haiku 4.5 | Anthropic describes it as the fastest Claude 4 model with near-frontier intelligence, 200k context, fast latency, and no adaptive thinking | 1 | The task is clearly bounded, low-risk, and does not need deep judgment | Do not make it the primary model for architecture, security, or ambiguous multi-file work |
| Claude Sonnet 4.6 | Anthropic describes it as the best combination of speed and intelligence, with 1M context and adaptive thinking | 3 | The task is high-risk but the user prefers Claude and no stronger Claude is in scope | Strong default for non-trivial coding and long-context work |
| GPT-5.4 | OpenAI describes it as the flagship for complex reasoning and coding, with 1M context and strong tool support | 4 | Cost or latency constraints are severe and the task can be decomposed safely | Safest default in this candidate set for hard coding and review work |
| GPT-5.4 mini | OpenAI describes it as the strongest mini model for coding, computer use, and subagents, with 400k context | 2 | The task is tightly scoped and failure cost is low to moderate | Good budget default for bounded code work, but do not oversell it for high-risk judgment |
| Raptor mini (Preview) | No verified first-party evidence was established in the current research set | 0 | Official docs confirm fit for the target task or local evals show reliable success | Treat as unproven, cap to low-risk work, and require a strong fallback |

## Provider Guidance

| Provider | Best fit | Conservative recommendation |
| --- | --- | --- |
| Anthropic direct | Claude-first workloads that need current first-party model behavior and docs alignment | Prefer when Claude is the chosen family and you want first-party feature coverage |
| AWS Bedrock | Claude workloads with AWS governance, regional controls, or existing AWS platform standards | Recommend only after confirming the exact Claude snapshot is available |
| Google Vertex AI | Claude workloads inside GCP governance or regional routing requirements | Recommend only after confirming the exact Claude snapshot is available |
| OpenAI direct | GPT-first workloads needing Responses API features, tool use, and latest first-party guidance | Prefer when GPT-5.4 or GPT-5.4 mini is selected and first-party tools matter |
| GitHub Models | Prototyping, cross-model comparison, prompt testing, and lightweight evaluation across providers | Good neutral recommendation when comparison and evaluation matter more than first-party exclusivity |
| Foundry | Enterprise governance, deployment control, centralized catalog management, and production operations | Recommend when governance and platform standardization matter more than newest direct-provider features |

## Catalog Checks

📌 Use live catalog or docs checks before naming a provider as supported.

- Anthropic models overview: official model roles, pricing, context, and API
  identifiers.
- Anthropic Models API: official capability and token-limit check.
- OpenAI models docs: official model roles, tools, pricing, context, and
  latest guidance.
- GitHub Models docs and catalog: official comparison and evaluation surface.

## Conservative Rejection Triggers

- The task spans multiple files and the smaller model is only described as
  fast or cheap.
- The user asks for security, architecture, protocol, or review work.
- The cheaper model lacks documented tool support required by the task.
- The cheaper model has a smaller context window than the task demands.
- The model is preview-only or hosting availability is unclear.
- The evidence is indirect or stale.

## Safe Recommendation Pattern

1. Choose the lowest-cost model that clears the task tier.
2. Pair it with a stronger fallback for escalation.
3. Explain which cheaper models were rejected and why.
4. State evidence gaps explicitly.
5. If provider support is unclear, recommend verifying the live catalog before
   implementation.

## Current Grounding Snapshot

📌 The baseline above is grounded in official model-family guidance reviewed on
2026-04-01.

- Anthropic models overview states that Claude Sonnet 4.6 is the best balance
  of speed and intelligence, while Claude Haiku 4.5 is the fastest model and
  lacks adaptive thinking.
- OpenAI models docs state that GPT-5.4 is the flagship for complex reasoning
  and coding, while GPT-5.4 mini is the strongest mini model for coding and
  lower-cost execution.
- GitHub Models docs position GitHub Models as a catalog, prototyping, and
  evaluation surface rather than a claim that every provider-specific feature is
  mirrored equally.
