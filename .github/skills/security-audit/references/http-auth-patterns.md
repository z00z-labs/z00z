# HTTP Protocol and Authentication Patterns

Use this reference for proxies, gateways, caches, custom HTTP parsing, JWT,
OAuth/OIDC, SAML, sessions, password reset, and account recovery.

## HTTP Framing and Cache Classes

### Request Smuggling and Desynchronization

Find two components that parse or forward the same bytes and identify the exact
framing difference. Test duplicate or conflicting length metadata,
transfer-coding normalization, HTTP/2-to-HTTP/1 translation, and ambiguous line
or header handling. A single parser with no downstream peer is not a confirmed
smuggling finding.

### Cache Poisoning and Deception

Compare response-varying inputs with the cache key. Trace unkeyed headers,
cookies, normalized query parameters, path extensions, and routing differences
into a response stored for another user. Prove shared-cache storage and
cross-user retrieval.

### Request-Derived Authority

Trace `Host`, `Forwarded`, `X-Forwarded-*`, scheme, and request-derived absolute
URLs into reset links, routing, redirects, cache keys, cookie attributes, and
authorization decisions. Prove the affected value reaches a victim or protected
decision.

## Authentication Protocol Classes

### JWT

Locate the verification call and confirm:

- allowed algorithms are pinned by the verifier
- signatures are verified before claims are trusted
- issuer, audience, expiration, and other flow-required claims are checked
- attacker-controlled key selectors cannot choose arbitrary files, URLs, or keys
- secret/key strength and trust-domain separation are evidence-backed

### OAuth and OIDC

Establish whether the target is an authorization server, relying-party client,
or resource server before assigning responsibility. Inspect exact redirect URI
matching, PKCE, transaction/session binding, issuer and audience checks, nonce
handling where applicable, mix-up defenses, and open redirects.

### SAML

Verify that the signed element is the same element used for identity and
authorization. Check parser configuration, assertion freshness, audience and
recipient binding, response correlation, and replay handling.

### Sessions and Recovery

Walk the entire lifecycle: issue, store, transmit, privilege change, refresh,
revoke, logout, password change, and recovery. Compare every path that can mint
or upgrade a session. Check rotation, invalidation, token binding, single use,
expiry, and reset-link construction.

## Source-Visibility Gate

Protocol exploitability often depends on a proxy chain, cache policy, identity
provider, runtime/library version, or production configuration outside the
repository. If the decisive component is unavailable:

- record the exact in-repo behavior and test bytes
- label the candidate `requires deployment testing`
- do not assign it a confirmed vulnerability severity

Verify framework and library defaults for the actual version before concluding
that a defense is absent.

## Primary References

- [RFC 9112: HTTP/1.1](https://www.rfc-editor.org/rfc/rfc9112.html)
- [RFC 8725: JSON Web Token Best Current Practices](https://www.rfc-editor.org/rfc/rfc8725.html)
- [RFC 9700: OAuth 2.0 Security Best Current Practice](https://www.rfc-editor.org/rfc/rfc9700.html)
