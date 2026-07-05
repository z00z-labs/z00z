---
name: model-router
description: Auto-invoked when user wants to choose the cheapest safe model, save tokens without breaking quality, decide if a smaller model can handle a task, or compare a shortlist of models before routing work. Also triggers on adaptive model selection, conservative model routing, task-model fit, capability gating, model fallback, provider choice, cost-aware inference, and token budget planning.
argument-hint: "task=\"<task description>\" | model_set=\"<model1,model2,...>\" | providers=\"<optional providers>\" | constraints=\"<optional constraints>\""
metadata:
   argument-hint: "task=\"<task description>\" | model_set=\"<model1,model2,...>\" | providers=\"<optional providers>\" | constraints=\"<optional constraints>\""
---

# Model Router

📌 Use this skill to pick the least expensive model that is still likely to
finish the task successfully.

⚠️ This skill is deliberately conservative: it must not overstate model
capability, especially for mini, haiku, or preview models.

## When to Use

- The user asks which model is good enough for a task.
- The user wants to save tokens or reduce cost without a quality collapse.
- The user provides a shortlist of models and wants a grounded comparison.
- The user asks whether a small model can replace a stronger one.
- The user asks for provider guidance for a known model family.
- The task involves model routing, fallback logic, capability gating, or
  task-model fit.

## Required Inputs

- A task description with expected output quality.
- An explicit `model_set` listing the candidate models to compare.
- Optional provider constraints such as `OpenAI`, `Anthropic`, `GitHub`,
  `Foundry`, `Bedrock`, or `Vertex`.
- Optional limits such as max cost, latency target, context size, tool use,
  or risk tolerance.

## How It Works

1. Classify the task before touching the model list.
   - Determine whether the task is low-risk transformation, bounded coding,
     multi-step reasoning, or high-risk architecture/security work.
   - Estimate failure cost. A wrong answer in a security review or complex
     refactor is more expensive than a wrong answer in simple formatting.

2. Ground every candidate with first-party evidence.
   - Use official model pages first.
   - Prefer provider catalogs and model overview pages over forum claims.
   - Capture explicit vendor wording about coding, reasoning, context,
     latency, tools, and price.
   - Use the rules in `REFERENCE.md` when the official pages are incomplete.

3. Apply conservative capability gates.
   - Do not upgrade a mini-class model into a hard reasoning role unless the
     official evidence clearly supports it.
   - Treat preview models as unproven by default.
   - If the task is high-risk or ambiguous, prefer the stronger model even if
     the cheaper one might work sometimes.
   - Only recommend the cheaper model when the task is bounded and the model's
     documented strengths match the task shape.

4. Recommend a provider only after checking hosting fit.
   - Recommend only providers that actually expose the requested model.
   - Prefer direct providers for first-party feature coverage.
   - Prefer platform providers such as GitHub Models or Foundry when the user
     values model comparison, governance, or unified access more than absolute
     first-party freshness.

5. Produce a strict decision with fallback.
   - Name the primary model.
   - Name the fallback escalation model.
   - Explain why smaller candidates were rejected.
   - Call out missing evidence explicitly instead of guessing.

## Decision Rules

1. Start from task risk, not from model marketing.
2. If two models look close, choose the cheaper one only when the task is
   tightly scoped and low-consequence.
3. If the task requires deep reasoning, broad code changes, security review,
   or architecture judgment, bias upward.
4. If the task requires very long context, check documented context windows
   before making any cost recommendation.
5. If the task needs tools such as web search, file search, or computer use,
   prefer models and providers that document those capabilities.
6. If evidence for a preview model is weak, cap it to low-risk work and keep a
   stronger fallback ready.
7. If the available evidence is mixed, say `insufficient confidence` and route
   to the safer model.

## Output Format

Use this exact structure:

```text
Task class: <tier and short label>
Primary model: <model>
Why: <2-4 sentences>
Fallback model: <model>
Rejected cheaper options: <comma-separated list with short reason>
Provider recommendation: <provider or provider shortlist>
Confidence: <high|medium|low>
Evidence gaps: <none or explicit gaps>
```

## Examples

### Example 1: Cost-Sensitive Coding

```text
User: Compare Claude Haiku 4.5, Claude Sonnet 4.6, GPT-5.4, GPT-5.4 mini, and Raptor mini (Preview) for a bounded bug fix in one file. I want to save tokens.
Assistant: Classify the task as bounded coding, ground each model with official sources, reject Raptor mini as unproven, and choose GPT-5.4 mini or Claude Haiku 4.5 only if the task stays local and low-risk. Keep Claude Sonnet 4.6 or GPT-5.4 as the fallback.
```

### Example 2: High-Risk Review

```text
User: Pick the cheapest safe model for an auth flow security review across multiple files.
Assistant: Classify the task as high-risk reasoning and review, reject mini and preview models as primary choices, and recommend GPT-5.4 or Claude Sonnet 4.6 depending on provider and tool constraints.
```

## Notes

📌 Use `REFERENCE.md` for task tiers, model baselines, provider guidance, and
evidence order.

⚠️ Never claim that a model is suitable for a hard task only because it is
faster or cheaper.

⚠️ Never treat a preview label as proof of strength.

✅ When uncertainty remains, recommend a stronger fallback instead of pretending
the smaller model is reliable.

## Invocation Examples

📌 Use these examples when invoking `model-router` directly.

### Example 1: Budget Routing

```text
/model-router task="bounded one-file Rust bug fix" model_set="Claude Haiku 4.5,Claude Sonnet 4.6,GPT-5.4,GPT-5.4 mini,Raptor mini (Preview)" providers="Anthropic,OpenAI,GitHub Models" constraints="minimize cost, keep safe fallback"
```

### Example 2: High-Risk Review

```text
/model-router task="security review for auth flow across multiple files" model_set="Claude Sonnet 4.6,GPT-5.4,GPT-5.4 mini" providers="Anthropic,OpenAI,Foundry" constraints="prefer safest primary model, long-context support"
```