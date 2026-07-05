# Output Contracts

These are compact shape contracts, not mandatory serialization formats.
Use them when you want repeatable, terse answers that are easy to scan or evaluate.

## Compact answer

```yaml
answer: "direct answer"
commands:
  - "optional command"
risk: "only if relevant"
```

## Coding result

```yaml
changed_files:
  - "path"
commands_run:
  - "command"
tests_result:
  status: "pass|fail|not_run"
  command: "command"
notes:
  - "max 5 bullets"
```

## Review result

```yaml
findings:
  - severity: "critical|high|medium|low"
    file: "path"
    issue: "short"
    fix: "short"
```

## Architecture result

```yaml
recommendation: "one strong option"
why:
  - "reason"
tradeoffs:
  - "tradeoff"
next_step: "single next action"
```

## Failure result

```yaml
status: fail
failing_check: "command or assertion"
reason: "short root cause"
next_step: "single discriminating action"
```
