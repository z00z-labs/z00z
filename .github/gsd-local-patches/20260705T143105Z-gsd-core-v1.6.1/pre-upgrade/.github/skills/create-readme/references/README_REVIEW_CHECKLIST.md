# README Review Checklist

Use this checklist during `review`, `refresh`, or final verification.

## First-Screen Check

- [ ] The first screen explains what the project is.
- [ ] The first screen explains why someone would use it.
- [ ] The first screen identifies the likely audience.
- [ ] The title and description are not generic or vague.
- [ ] The value statement is clear within the first few lines.

## Quickstart Check

- [ ] There is a fastest path to first success.
- [ ] Commands appear in runnable order.
- [ ] Required prerequisites appear before setup commands.
- [ ] The quickstart does not depend on hidden configuration.
- [ ] Installation works from a clean setup in principle, not just from maintainer state.

## Truthfulness Check

- [ ] Claims appear supported by visible project context.
- [ ] Links and file references look plausible.
- [ ] Version, badge, and status signals do not contradict the repo.
- [ ] Sections do not promise behavior the project does not expose.
- [ ] Diagrams and screenshots, if present, do not contradict the implementation.

## Configuration Check

- [ ] Required environment variables are explicit.
- [ ] Optional configuration is clearly marked as optional.
- [ ] Secrets, credentials, or external services are called out.
- [ ] Local development assumptions are stated.

## Usability Check

- [ ] There is at least one minimal example.
- [ ] Commands are copy-paste friendly.
- [ ] Troubleshooting exists when setup is non-trivial.
- [ ] A support or help path exists when a user can get stuck.
- [ ] The README helps both users and contributors where relevant.
- [ ] Technical terms are either common for the audience or explained.

## Alert Block Check

- [ ] `NOTE` blocks add context rather than noise.
- [ ] `TIP` blocks improve success rate without being mandatory.
- [ ] `IMPORTANT` blocks guard critical prerequisites.
- [ ] `WARNING` blocks are reserved for real risk.
- [ ] `CAUTION` blocks describe plausible negative consequences.

## Compression Check

- [ ] No section exists only because templates usually contain it.
- [ ] Repetition between `Quickstart`, `Installation`, and `Usage` is minimized.
- [ ] Dense prose is converted into steps, bullets, or examples where helpful.
- [ ] Diagrams, screenshots, and badges are present only when they add signal.
- [ ] Longer READMEs include a table of contents when navigation would otherwise suffer.

## Ship / No-Ship Rule

Fail the README if any of these are true:

- A new user cannot tell how to run the project.
- A critical prerequisite is missing.
- The README makes unsupported claims.
- The minimal example is absent or obviously inconsistent.
- Badges, links, or visual elements create a false trust signal.
