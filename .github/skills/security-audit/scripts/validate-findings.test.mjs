import assert from "node:assert/strict";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { describe, test } from "node:test";
import { fileURLToPath } from "node:url";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const validatorPath = path.join(scriptDirectory, "validate-findings.mjs");

function runValidator(findings) {
  return spawnSync(process.execPath, [validatorPath, "-"], {
    encoding: "utf8",
    input: JSON.stringify(findings),
  });
}

function createFinding() {
  const score = {
    reason: "The path is directly reachable",
    score: "high",
  };

  return {
    category: "access-control",
    conditions: [],
    confidence: {
      reason: "The complete source path was verified",
      score: "high",
    },
    description: "A user can read another user's record",
    execution: {
      attacker_perspective: "Authenticated low-privilege user",
      expected_result: "Another user's record is returned",
      instructions: ["Request a record owned by another user"],
      payloads: ["GET /records/2"],
    },
    intended_behavior: "Restrict record reads to the owner",
    remediation: {
      code_changes: [
        {
          description: "Use an owner-scoped lookup",
          file: "src/api.js",
        },
      ],
      strategy: "Authorize ownership at the data access boundary",
    },
    residual_risk: "Parallel record lookup paths still require review",
    root_cause: "handler in src/api.js omits owner authorization",
    severity: {
      impact: score,
      likelihood: score,
      overall: "high",
    },
    title: "Missing record ownership check",
    trace: [
      {
        description: "Reads the attacker-selected record identifier",
        file: "src/api.js",
        kind: "entrypoint",
        line: 10,
        scope: "handler",
      },
      {
        description: "Returns the selected record without an owner predicate",
        file: "src/store.js",
        kind: "sink",
        line: 20,
        scope: "load_record",
      },
    ],
    verdict: "confirmed",
    verification: {
      evidence: "No authorization check exists on the reachable source path",
      mode: "source-confirmed",
    },
  };
}

describe("validate-findings", () => {
  test("accepts an empty findings array", () => {
    const result = runValidator([]);

    assert.equal(result.status, 0);
    assert.match(result.stdout, /PASS: 0 finding record/);
  });

  test("accepts a complete confirmed finding", () => {
    const result = runValidator([createFinding()]);

    assert.equal(result.status, 0);
    assert.match(result.stdout, /PASS: 1 finding record/);
  });

  test("rejects missing required fields", () => {
    const result = runValidator([{ title: "", verdict: "confirmed" }]);

    assert.equal(result.status, 1);
    assert.match(result.stderr, /missing required field "category"/);
    assert.match(result.stderr, /expected at least 1 character/);
  });

  test("rejects an invalid trace order", () => {
    const finding = createFinding();
    finding.trace[0].kind = "sink";
    finding.trace[1].kind = "entrypoint";

    const result = runValidator([finding]);

    assert.equal(result.status, 1);
    assert.match(result.stderr, /trace\[0\]\.kind: expected "entrypoint"/);
    assert.match(result.stderr, /trace\[1\]\.kind: expected "sink"/);
  });

  test("rejects low-confidence confirmed findings", () => {
    const finding = createFinding();
    finding.confidence.score = "low";

    const result = runValidator([finding]);

    assert.equal(result.status, 1);
    assert.match(result.stderr, /confidence\.score: invalid value "low"/);
  });
});
