# Client-Side and Browser Security Patterns

Use this reference for SPAs, browser extensions, embedded WebViews, and code
that renders untrusted content, receives cross-window messages, opens
WebSockets, or exposes credentialed cross-origin responses.

## Source-to-Sink Discipline

Require both an attacker-controlled browser source and a security-relevant
sink. Typical sources include URL fragments and query strings, `window.name`,
`document.referrer`, message events, cookies, and storage. Typical sinks include
HTML insertion, code evaluation, navigation, script/resource URLs, and
privileged client APIs.

Framework interpolation that is escaped by default is not a finding. Focus on
escape hatches and custom rendering paths.

## High-Value Classes

### DOM Injection

Trace client-only sources into `innerHTML`, `outerHTML`, `document.write`,
string-based timers, `eval`, `Function`, `javascript:` navigation, jQuery HTML
methods, and framework trust-bypass APIs. Server-side validation cannot inspect
fragment, `window.name`, or cross-window message data.

### Messaging and Origin Trust

For `postMessage`, verify exact origin allowlists and security-relevant use of
`event.data`; inspect senders that use an unrestricted target origin. For
WebSockets authenticated by ambient credentials, verify origin/session binding
and show what another origin can read or change.

### Credentialed Cross-Origin Reads

Inspect CORS origin matching, credential use, `null` origins, suffix/prefix
matching, and reflected origins. Prove that a hostile origin can read a
credentialed sensitive response; a permissive header alone is a hardening note.

### UI Redress and Navigation

Require a sensitive framed action for clickjacking. For new-window navigation,
verify actual opener behavior in the supported browser/runtime. For open
redirect or script-scheme claims, trace attacker-controlled navigation data to
the client sink.

### Prototype Pollution

Prove both parts: an attacker-controlled recursive or path-based write reaches a
prototype, and a reachable gadget consumes the polluted property to cause XSS,
authorization bypass, or code execution. Pollution without a gadget is not an
exploitable finding.

## Validation Gate

1. cite the controllable client source and executing sink
2. prove sanitization or framework escaping does not break the path
3. identify whose session or origin is affected
4. verify browser/WebView behavior for the supported runtime
5. separate missing headers with no sensitive action into hardening notes

## Primary References

- [OWASP DOM-based XSS Prevention](https://cheatsheetseries.owasp.org/cheatsheets/DOM_based_XSS_Prevention_Cheat_Sheet.html)
- [OWASP HTML5 Security](https://cheatsheetseries.owasp.org/cheatsheets/HTML5_Security_Cheat_Sheet.html)
