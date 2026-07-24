#!/usr/bin/env node

"use strict";

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const inputPath = process.argv[2];
if (inputPath === "--help" || inputPath === "-h") {
  console.log("Usage: node scripts/validate-findings.mjs <findings.json|->");
  console.log('Use "-" to read findings JSON from standard input.');
  process.exit(0);
}
if (inputPath === undefined) {
  console.error("Usage: node scripts/validate-findings.mjs <findings.json|->");
  process.exit(1);
}

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const schemaPath = path.join(
  scriptDirectory,
  "..",
  "references",
  "findings-schema.json",
);

function loadJson(filePath, label) {
  try {
    const fileDescriptor = filePath === "-" ? 0 : filePath;
    return JSON.parse(fs.readFileSync(fileDescriptor, "utf8"));
  } catch (error) {
    console.error(`Failed to load ${label} from ${filePath}: ${error.message}`);
    process.exit(1);
  }
}

const schema = loadJson(schemaPath, "schema");
const findings = loadJson(inputPath, "findings");

function valueType(value) {
  if (Array.isArray(value)) {
    return "array";
  }
  if (value === null) {
    return "null";
  }
  return typeof value;
}

function resolveReference(rootSchema, reference) {
  if (!reference.startsWith("#/")) {
    throw new Error(`Unsupported schema reference: ${reference}`);
  }

  return reference
    .slice(2)
    .split("/")
    .reduce((current, key) => current[key], rootSchema);
}

function discriminator(branch) {
  if (!branch.properties) {
    return null;
  }

  for (const [key, property] of Object.entries(branch.properties)) {
    if (Object.prototype.hasOwnProperty.call(property, "const")) {
      return { key, value: property.const };
    }
  }
  return null;
}

function collectErrors(value, node, location) {
  const errors = [];
  validate(value, node, location, errors);
  return errors;
}

function validate(value, node, location, errors) {
  if (node.$ref) {
    validate(value, resolveReference(schema, node.$ref), location, errors);
    return;
  }

  if (node.oneOf) {
    const matchingBranch = node.oneOf.find((branch) => {
      const marker = discriminator(branch);
      return (
        marker &&
        valueType(value) === "object" &&
        value[marker.key] === marker.value
      );
    });

    if (matchingBranch) {
      validate(value, matchingBranch, location, errors);
      return;
    }

    const passingBranches = node.oneOf.filter(
      (branch) => collectErrors(value, branch, location).length === 0,
    );
    if (passingBranches.length !== 1) {
      const allowed = node.oneOf
        .map(discriminator)
        .filter(Boolean)
        .map((marker) => JSON.stringify(marker.value))
        .join(", ");
      errors.push(
        `${location}: must match exactly one schema; allowed verdicts: ${allowed}`,
      );
    }
    return;
  }

  if (
    Object.prototype.hasOwnProperty.call(node, "const") &&
    value !== node.const
  ) {
    errors.push(
      `${location}: expected ${JSON.stringify(node.const)}, got ${JSON.stringify(value)}`,
    );
  }

  if (node.enum && !node.enum.includes(value)) {
    errors.push(
      `${location}: invalid value ${JSON.stringify(value)}; expected one of ${node.enum
        .map((item) => JSON.stringify(item))
        .join(", ")}`,
    );
  }

  switch (node.type) {
    case "object": {
      if (valueType(value) !== "object") {
        errors.push(`${location}: expected object, got ${valueType(value)}`);
        return;
      }

      for (const requiredKey of node.required || []) {
        if (!Object.prototype.hasOwnProperty.call(value, requiredKey)) {
          errors.push(`${location}: missing required field "${requiredKey}"`);
        }
      }

      for (const [key, childValue] of Object.entries(value)) {
        if (node.properties && Object.prototype.hasOwnProperty.call(node.properties, key)) {
          validate(
            childValue,
            node.properties[key],
            `${location}.${key}`,
            errors,
          );
        } else if (node.additionalProperties === false) {
          errors.push(`${location}: unexpected field "${key}"`);
        }
      }
      break;
    }

    case "array": {
      if (valueType(value) !== "array") {
        errors.push(`${location}: expected array, got ${valueType(value)}`);
        return;
      }

      if (node.minItems !== undefined && value.length < node.minItems) {
        errors.push(
          `${location}: expected at least ${node.minItems} item(s), got ${value.length}`,
        );
      }

      if (node.items) {
        value.forEach((item, index) => {
          validate(item, node.items, `${location}[${index}]`, errors);
        });
      }
      break;
    }

    case "integer": {
      if (!Number.isInteger(value)) {
        errors.push(`${location}: expected integer, got ${valueType(value)}`);
        return;
      }
      if (node.minimum !== undefined && value < node.minimum) {
        errors.push(`${location}: expected value >= ${node.minimum}, got ${value}`);
      }
      break;
    }

    case "string": {
      if (valueType(value) !== "string") {
        errors.push(`${location}: expected string, got ${valueType(value)}`);
        return;
      }
      if (node.minLength !== undefined && value.length < node.minLength) {
        errors.push(
          `${location}: expected at least ${node.minLength} character(s)`,
        );
      }
      break;
    }

    default:
      break;
  }
}

const errors = collectErrors(findings, schema, "$");

if (Array.isArray(findings)) {
  findings.forEach((finding, findingIndex) => {
    if (
      !finding ||
      finding.verdict !== "confirmed" ||
      !Array.isArray(finding.trace) ||
      finding.trace.length === 0
    ) {
      return;
    }

    const lastIndex = finding.trace.length - 1;
    if (finding.trace[0].kind !== "entrypoint") {
      errors.push(
        `$[${findingIndex}].trace[0].kind: expected "entrypoint"`,
      );
    }
    if (finding.trace[lastIndex].kind !== "sink") {
      errors.push(
        `$[${findingIndex}].trace[${lastIndex}].kind: expected "sink"`,
      );
    }
    for (let traceIndex = 1; traceIndex < lastIndex; traceIndex += 1) {
      if (finding.trace[traceIndex].kind !== "propagation") {
        errors.push(
          `$[${findingIndex}].trace[${traceIndex}].kind: expected "propagation"`,
        );
      }
    }
  });
}

if (errors.length > 0) {
  errors.forEach((error) => console.error(`ERROR: ${error}`));
  console.error(`FAIL: ${errors.length} validation error(s)`);
  process.exit(1);
}

console.log(`PASS: ${findings.length} finding record(s) are structurally valid`);
