---
name: api-builder
description: "Canonical stack-agnostic API builder for designing, reviewing, and securing APIs. Use when the user wants to build an API, choose REST vs GraphQL vs tRPC, define contracts, responses, versioning, pagination, authentication, authorization, input validation, rate limiting, or OWASP-style API security controls. Triggers on API design, endpoint planning, auth/authz, validation, throttling, and API review."
risk: unknown
source: community
date_added: "2026-02-27"
---

# API Builder

> API design principles and decision-making for 2025.
> **Learn to THINK, not copy fixed patterns.**

## ✅ Scope

This skill is the **canonical top-level API skill** for design, review, and security planning.

It is **stack-agnostic**. Use it to plan, review, or compare API choices even when no language, framework, or codebase is specified.

If the user's stack is unknown, stay at the level of API contracts, trade-offs, and compatibility constraints first. Only narrow into framework-specific guidance when the user explicitly provides that context.

Treat security as a built-in lane of API design, not as a separate top-level workflow.

## 🎯 Selective Reading Rule

**Read ONLY files relevant to the request!** Check the content map, find what you need.

---

## 📑 Content Map

| File | Description | When to Read |
|------|-------------|--------------|
| `api-style.md` | REST vs GraphQL vs tRPC decision tree | Choosing API type |
| `rest.md` | Resource naming, HTTP methods, status codes | Designing REST API |
| `response.md` | Envelope pattern, error format, pagination | Response structure |
| `graphql.md` | Schema design, when to use, security | Considering GraphQL |
| `trpc.md` | TypeScript monorepo, type safety | TS fullstack projects |
| `versioning.md` | URI/Header/Query versioning | API evolution planning |
| `auth.md` | JWT, OAuth, Passkey, API Keys | Auth pattern selection |
| `rate-limiting.md` | Token bucket, sliding window | API protection |
| `documentation.md` | OpenAPI/Swagger best practices | Documentation |
| `security-testing.md` | OWASP API Top 10, auth/authz testing | Security audits |

---

## 🔗 Related Skills

These are optional follow-ons, not prerequisites for using this skill.

| Need | Skill |
|------|-------|
| API implementation | `@[skills/backend-development]` |
| Data structure | `@[skills/database-design]` |
| Security details | `@[skills/security-hardening]` |

---

## ✅ Decision Checklist

Before designing an API:

- [ ] **Asked user about API consumers?**
- [ ] **Chosen API style for THIS context?** (REST/GraphQL/tRPC)
- [ ] **Defined consistent response format?**
- [ ] **Planned versioning strategy?**
- [ ] **Considered authentication needs?**
- [ ] **Defined authorization rules?**
- [ ] **Planned input validation and sanitization?**
- [ ] **Planned rate limiting?**
- [ ] **Planned sensitive-data protection and error hygiene?**
- [ ] **Defined security verification approach?**
- [ ] **Documentation approach defined?**

---

## How It Works

1. Identify the API consumers, trust boundaries, and change pressure.
2. Choose the API style that fits the actual context.
3. Define contracts: resources or operations, responses, errors, pagination, and versioning.
4. Activate the security lane when auth, exposure, abuse resistance, or sensitive data are in scope.
5. Produce implementation guidance that matches the user's stack, or stay stack-neutral if the stack is unknown.

---

## Security Lane

Use this lane whenever the request involves exposed endpoints, authentication, authorization, abuse resistance, sensitive data, or API audit readiness.

### Security Steps

1. **Authentication and authorization**
	- Choose the auth method that matches the client and trust model.
	- Separate authentication from authorization.
	- Check object-level and function-level authorization explicitly.

2. **Input validation and sanitization**
	- Validate all request inputs.
	- Sanitize untrusted values.
	- Prefer schema validation and parameterized access patterns.

3. **Rate limiting and abuse resistance**
	- Apply per-user, per-token, or per-IP throttling as appropriate.
	- Protect authentication and high-cost endpoints more aggressively.
	- Define graceful rate-limit errors and monitoring hooks.

4. **Data protection and error hygiene**
	- Require HTTPS or TLS in transit.
	- Protect sensitive data at rest when applicable.
	- Avoid leaking secrets, internals, or stack traces in API errors.

5. **Security verification**
	- Test authentication and authorization paths.
	- Review against OWASP API Top 10 style risks.
	- Test validation failures and rate-limit behavior.

If the user's stack is unknown, respond with controls, threat coverage, validation rules, and test cases in generic terms instead of assuming a framework.

---

## Examples

- "Build a public REST API for mobile and web clients with pagination, versioning, and OAuth."
- "Should this internal TypeScript API be REST, GraphQL, or tRPC?"
- "Review this API design for broken authz, missing validation, and rate-limit gaps."
- "Design consistent error responses and OpenAPI-ready endpoint contracts for this service."

---

## ❌ Anti-Patterns

**DON'T:**
- Default to REST for everything
- Use verbs in REST endpoints (/getUsers)
- Return inconsistent response formats
- Expose internal errors to clients
- Treat authentication as authorization
- Skip input validation because the client already validates
- Skip rate limiting
- Treat API security as an afterthought after contract design

**DO:**
- Choose API style based on context
- Ask about client requirements
- Document thoroughly
- Use appropriate status codes
- Define auth, validation, throttling, and verification as part of the first design pass

---

## Script

Treat this helper as optional. If it is absent, continue with the design/review workflow using the guidance in this file alone.

| Script | Purpose | Command |
|--------|---------|---------|
| `scripts/api_validator.py` | API endpoint validation | `python scripts/api_validator.py <project_path>` |

## When to Use

- Use when the user wants to build, design, review, or compare an API.
- Use when the user asks which API style to choose: REST, GraphQL, or tRPC.
- Use when the user needs endpoint contracts, response formats, pagination, versioning, or documentation strategy.
- Use when the user wants authentication, authorization, validation, throttling, or API abuse protection designed into the API.
- Use when the user asks to secure an API, prepare for an API security review, or check for OWASP API Top 10 style risks.
- Trigger on natural language such as "build an API", "design endpoints", "secure this API", "add auth", "add rate limiting", "review my API", or "choose REST vs GraphQL".
- Trigger on technical language such as API contract, endpoint design, versioning, pagination, auth/authz, input validation, throttling, rate limiting, OpenAPI, and OWASP API Top 10.
