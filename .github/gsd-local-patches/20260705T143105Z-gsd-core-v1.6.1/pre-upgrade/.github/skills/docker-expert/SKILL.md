---
name: docker-expert
description: Auto-invoked when user wants to design, review, fix, or optimize a Dockerfile, container image, docker-compose stack, build context, runtime configuration, or container hardening setup, and needs practical containerization guidance, image-size reduction, security checks, startup validation, or Docker workflow troubleshooting. Also triggers on Docker, Dockerfile, docker compose, container image, multi-stage build, .dockerignore, healthcheck, bind mount, volume, and non-root container.
---

# Docker Expert

Review or shape a Docker-based setup so container builds, runtime behavior,
image security, and operational validation are explicit before the setup is
treated as production-ready.

## When to Use

- User asks to create, review, fix, or optimize a Dockerfile, image, compose
  stack, or container workflow.
- The task involves build speed, image size, runtime reliability, startup
  behavior, bind mounts, volumes, networks, or container hardening.
- The user wants a focused Docker pass before broader deployment or
  architecture work.
- A change needs validation of container boundaries, build stages, or runtime
  assumptions.
- The artifact under review may have been written by a human, AI assistant, or
  mixed workflow. This skill is coder-agnostic: evaluate the container setup,
  evidence, and operational risks, not the author.

## When Not to Use

- The main task is Kubernetes orchestration, cluster networking, or service
  mesh behavior rather than Docker/container setup.
- The request is cloud-platform specific deployment strategy with little Docker
  design work in scope.
- There is no container artifact, config, or runtime behavior to inspect.
- The user only wants generic Docker theory detached from a real workload.

## How It Works

1. **Define the container scope**
   - State what is under review: Dockerfile, compose file, local dev setup,
     production image, CI build path, or runtime container behavior.
   - Record the intended outcome: smaller image, faster build, safer runtime,
     reproducible environment, or startup/debug fix.
   - Keep the review anchored to the actual container boundary and workflow.

2. **Map the build graph**
   - Identify base images, build stages, copied artifacts, package-manager
     steps, cache boundaries, and build context size.
   - Check whether dependency installation is separated cleanly from changing
     source layers.
   - Treat oversized build context and unstable cache boundaries as real
     findings, not cosmetic issues.

3. **Inspect runtime shape**
   - Check entrypoint or command behavior, environment assumptions, exposed
     ports, health checks, signal handling, writable paths, and shutdown model.
   - Identify whether the container is single-purpose or hides multiple runtime
     concerns that should be split.
   - Distinguish local-development convenience from production runtime needs.

4. **Evaluate security and isolation**
   - Check user identity, root usage, secret handling, package footprint,
     unnecessary tools, writable filesystem assumptions, and network exposure.
   - Prefer minimal runtime dependencies and explicit privilege boundaries.
   - Treat secrets baked into layers, shell history, or default env patterns as
     high-priority defects.

5. **Evaluate persistence and connectivity**
   - Review volume strategy, bind mounts, ephemeral state, database coupling,
     service discovery, network topology, and dependency ordering.
   - Check whether data durability and startup dependencies are explicit enough
     for repeatable environments.
   - Flag setups that rely on fragile startup ordering or hidden host-state
     assumptions.

6. **Classify the root problem shape**
   - Group issues into categories such as cache inefficiency, oversized image,
     mixed build/runtime concerns, secret exposure, root runtime, weak
     health-checking, brittle startup dependency, bad volume model, or
     environment drift.
   - Collapse repeated symptoms into one root cause where possible.
   - Prefer root-shape findings over a long list of Docker trivia.

7. **Recommend the narrowest effective fix**
   - Propose specific changes such as multi-stage build cleanup, `.dockerignore`
     tightening, non-root runtime, health check addition, dependency reorder,
     volume redesign, or compose dependency clarification.
   - State trade-offs like debug convenience, rebuild cost, image size, and
     runtime observability.
   - Keep Docker-specific fixes separate from larger deployment redesigns.

8. **Validate and guard against regression**
   - Define what to build, run, inspect, and smoke-test after the change.
   - Recommend checks such as clean build, image history inspection, startup
     validation, compose config validation, and container runtime smoke tests.
   - End with the smallest next validation step that proves the fix actually
     changed the container behavior.

## Review Lenses

- **Build lens**: cache boundaries, stages, base images, and copied artifacts.
- **Runtime lens**: entrypoint, health, signals, ports, and writable paths.
- **Security lens**: user privileges, secrets, package surface, and exposure.
- **State lens**: volumes, bind mounts, ephemeral data, and persistence model.
- **Dependency lens**: service startup ordering, network assumptions, and
  environment coupling.
- **Validation lens**: whether the current setup can be rebuilt and verified
  reproducibly.

## Evidence Rules

- Do not call a Docker setup production-ready without naming what was actually
  built, run, or inspected.
- Do not recommend image-size or cache optimizations without tying them to the
  current build graph.
- Do not let generic Docker best practices replace evidence from the actual
  Dockerfile, compose config, image history, or runtime behavior.
- If a finding comes from config inspection only, say so. If it comes from a
  build or runtime check, say that too.
- Prefer a smaller set of high-confidence container findings over a bloated
  list of generic container advice.

## Review Output

When using this skill, structure the output around these fields:

- Scope and container target
- Build and runtime assumptions
- Evidence reviewed
- Findings ordered by severity or expected impact
- Recommended fixes and trade-offs
- Validation plan and remaining gaps

## Examples

### Example 1: Dockerfile Cleanup

```text
User: Review this Dockerfile and make it production-ready.
Assistant: First map the build stages, runtime entrypoint, privilege model, and
copied artifacts, then report the highest-impact Docker issues with the
narrowest effective fixes and validation steps.
```

### Example 2: Compose Stack Problem

```text
User: My docker compose stack starts unreliably. Do a full pass.
Assistant: Inspect startup dependencies, health checks, network assumptions,
and volume/state design, then separate brittle orchestration inside compose
from broader platform problems.
```

### Example 3: Mixed Human And AI Changes

```text
User: Some of this container setup was written by AI and some by people. Review
it.
Assistant: Keep the review coder-agnostic, evaluate the Docker artifacts and
runtime evidence only, and report container findings with explicit confidence.
```

## Notes

- Good container work reduces ambiguity between build-time and runtime
  concerns.
- A smaller image is not automatically a better image if observability,
  startup safety, or operability gets worse.
- Missing validation steps are often as risky as the Docker misconfiguration
  itself.
- If the user asks for a review, container findings and trade-offs come first
  and broad Docker theory stays secondary.