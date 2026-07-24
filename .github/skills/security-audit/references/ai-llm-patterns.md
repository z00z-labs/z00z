# AI, LLM, and Agent Security Patterns

Use this reference when untrusted text can influence a model, retrieval system,
agent loop, MCP connection, tool call, or downstream renderer.

## Map the Boundary

Inventory:

- every prompt/context source and who can write it
- whose session later consumes each source
- tool identity, permissions, and resource scope
- secrets or cross-tenant data present in model context
- every sink reached by model-generated text or arguments

## High-Value Classes

### Indirect Prompt Injection

Trace attacker-written documents, web pages, email, issues, files, filenames,
tool responses, or retrieved chunks into another principal's model context.
A model changing its answer is not enough; prove that the injected content can
cross a session or authority boundary and reach protected data or a capability.

### Tool-Argument Injection

Treat model-generated arguments as untrusted input. Trace them into SQL, shell,
filesystem, HTTP, messaging, and destructive API sinks. Validate and authorize
inside the tool handler against the requesting principal and the specific
resource.

### Excessive Agency and Confused Deputy

Compare what the user may do directly with what the agent's identity may do.
Look for broad service credentials, unnecessary tools, missing per-resource
authorization, irreversible actions, and model-controlled loops without action
budgets or confirmation gates.

### Context and Tenant Isolation

Inspect conversation, retrieval, embedding, and cache keys. Confirm tenant and
ACL filters are applied at query time, not merely stored as metadata. Check what
context sub-agents and MCP servers inherit and treat their responses as
untrusted.

### Insecure Output Handling

Trace model output into HTML/Markdown rendering, commands, queries, templates,
URLs, files, and code execution. Confirm the real client and its sanitization,
CSP, remote-resource policy, and sink behavior before claiming XSS or data
exfiltration.

## Validation Gate

For every AI/LLM finding:

1. name the attacker, victim, executing identity, and boundary crossed
2. cite the code that assembles trusted and untrusted context
3. cite the tool or output sink and prove model-influenced data reaches it
4. prove the user lacks the resulting capability or data through normal access
5. distinguish code-enforced authorization from prompt-only instructions
6. mark deployment-only capabilities as unverified until confirmed

Do not report prompt injection merely because the model can be influenced.
Report the missing code-level mediation that turns that influence into impact.

## Primary References

- [OWASP Prompt Injection](https://owasp.org/www-community/attacks/PromptInjection)
- [OWASP LLM Prompt Injection Prevention](https://cheatsheetseries.owasp.org/cheatsheets/LLM_Prompt_Injection_Prevention_Cheat_Sheet.html)
- [OWASP LLM06:2025 Excessive Agency](https://genai.owasp.org/llmrisk/llm062025-excessive-agency/)
