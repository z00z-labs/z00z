#!/usr/bin/env python3
"""Validate the Phase 066 machine-readable artifact and host report contract."""

from __future__ import annotations

import argparse
import json
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any

ALLOWED_STATUSES = {
    "passed",
    "failed",
    "missing",
    "skipped",
    "completed",
    "completed-with-failures",
    "completed-with-missing-tools",
}
FINDING_STATUSES = {"confirmed", "false-positive", "unconfirmed", "skipped"}
SEVERITY_VALUES = {"critical", "high", "medium", "low", "info"}
REQUIRED_DIRS = [
    "sast",
    "rust",
    "secrets",
    "dast",
    "report",
    "logs",
    "raw",
    "normalized",
]


@dataclass
class ValidationResult:
    """Structured artifact-validation output."""

    status: str
    artifact_dir: str
    report_dir: str | None = None
    checked_paths: list[str] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)
    errors: list[str] = field(default_factory=list)

    def exit_code(self) -> int:
        return 0 if self.status == "OK" else 1


def parse_args() -> argparse.Namespace:
    """Parse CLI arguments."""

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("artifact_dir", help="path to .security-artifacts/<timestamp>")
    parser.add_argument("--json", action="store_true", help="emit machine-readable output")
    return parser.parse_args()


def load_json(path: Path) -> Any:
    """Load JSON from disk."""

    return json.loads(path.read_text(encoding="utf-8"))


def require_path(path: Path, errors: list[str], checked: list[str], *, kind: str) -> None:
    """Record required path checks."""

    checked.append(path.as_posix())
    if kind == "dir" and not path.is_dir():
        errors.append(f"required directory is missing: {path}")
    if kind == "file" and not path.is_file():
        errors.append(f"required file is missing: {path}")


def validate_report_metadata(report_metadata: dict[str, Any], errors: list[str]) -> None:
    """Reject confirmed findings that do not carry evidence."""

    for finding in report_metadata.get("findings", []):
        finding_id = finding.get("id", "<unknown>")
        severity = finding.get("severity")
        status_name = finding.get("status")

        if severity and severity not in SEVERITY_VALUES:
            errors.append(f"finding {finding_id} uses unsupported severity: {severity}")
        if status_name and status_name not in FINDING_STATUSES:
            errors.append(f"finding {finding_id} uses unsupported status: {status_name}")

        if status_name == "confirmed":
            missing_fields = [
                field_name
                for field_name in (
                    "source_evidence",
                    "scanner_artifact",
                    "proof",
                    "confidence",
                    "fix_recommendation",
                    "regression_test",
                )
                if not finding.get(field_name)
            ]
            if missing_fields:
                errors.append(
                    "confirmed finding is missing evidence fields: "
                    + ", ".join(missing_fields)
                )
            continue

        if status_name == "false-positive":
            for field_name in ("scanner_artifact", "false_positive_reason"):
                if not finding.get(field_name):
                    errors.append(f"false-positive finding {finding_id} is missing {field_name}")
        elif status_name == "unconfirmed":
            for field_name in ("scanner_artifact", "confidence"):
                if not finding.get(field_name):
                    errors.append(f"unconfirmed finding {finding_id} is missing {field_name}")
        elif status_name == "skipped" and not finding.get("skip_reason"):
            errors.append(f"skipped finding {finding_id} is missing skip_reason")


def validate_commands(
    artifact_dir: Path,
    manifest: dict[str, Any],
    warnings: list[str],
    errors: list[str],
    checked: list[str],
) -> None:
    """Validate command ledger entries in the manifest."""

    commands = manifest.get("commands")
    if not isinstance(commands, list):
        errors.append("manifest.commands must be a list")
        return

    if not commands:
        warnings.append("manifest.commands is empty")
        return

    for entry in commands:
        if not isinstance(entry, dict):
            errors.append("manifest.commands contains a non-object entry")
            continue

        tool_name = entry.get("tool", "unknown")
        status_name = entry.get("status")
        if status_name not in ALLOWED_STATUSES:
            errors.append(f"{tool_name}: unsupported status value: {status_name}")
            continue

        if status_name in {"passed", "failed"} and not entry.get("command_display"):
            errors.append(f"{tool_name}: command_display is required for executed commands")

        for field_name in ("stdout_path", "stderr_path", "exit_path", "status_path"):
            raw_value = entry.get(field_name)
            if not raw_value:
                if status_name in {"passed", "failed"}:
                    errors.append(f"{tool_name}: {field_name} is required for executed commands")
                continue
            path = artifact_dir / str(raw_value)
            checked.append(path.as_posix())
            if field_name != "raw_output_path" and not path.exists():
                errors.append(f"{tool_name}: referenced path does not exist: {path}")

        raw_output = entry.get("raw_output_path")
        if raw_output:
            raw_path = artifact_dir / str(raw_output)
            checked.append(raw_path.as_posix())
            if status_name in {"passed", "failed"} and not raw_path.exists():
                warnings.append(f"{tool_name}: raw output path is absent: {raw_path}")


def validate_manifest_identity(
    artifact_dir: Path,
    manifest: dict[str, Any],
    warnings: list[str],
    errors: list[str],
) -> Path | None:
    """Validate the manifest identity and paired host report location."""

    run_id = manifest.get("run_id")
    if run_id != artifact_dir.name:
        errors.append(
            f"manifest.run_id must match artifact directory name: {artifact_dir.name}"
        )

    report_dir_raw = manifest.get("report_dir")
    if not isinstance(report_dir_raw, str) or not report_dir_raw:
        errors.append("manifest.report_dir must be a non-empty string")
        return None

    report_dir = Path(report_dir_raw)
    expected_name = f"z00z-pentests_report-{artifact_dir.name}"
    if report_dir.name != expected_name:
        errors.append(
            f"manifest.report_dir must end with {expected_name}, found {report_dir.name}"
        )

    if not report_dir.is_dir():
        errors.append(f"host report directory is missing: {report_dir}")

    artifact_report = manifest.get("report_files", {}).get("artifact_report_markdown")
    host_report = manifest.get("report_files", {}).get("host_report_markdown")
    if artifact_report != "report/security-report.md":
        warnings.append("manifest.report_files.artifact_report_markdown should be report/security-report.md")
    if host_report and Path(host_report).name != "security-report.md":
        warnings.append("host report markdown should be security-report.md")
    return report_dir


def validate_artifact_tree(artifact_dir: Path) -> ValidationResult:
    """Validate the artifact tree and paired host report directory."""

    warnings: list[str] = []
    errors: list[str] = []
    checked: list[str] = []

    if not artifact_dir.is_dir():
        return ValidationResult(
            status="FAIL",
            artifact_dir=artifact_dir.as_posix(),
            errors=[f"artifact directory not found: {artifact_dir}"],
        )

    manifest_path = artifact_dir / "manifest.json"
    scope_path = artifact_dir / "scope.normalized.json"
    tool_status_path = artifact_dir / "tool-status.json"
    report_metadata_path = artifact_dir / "report" / "report-metadata.json"
    report_markdown_path = artifact_dir / "report" / "security-report.md"
    findings_path = artifact_dir / "normalized" / "findings.json"

    require_path(manifest_path, errors, checked, kind="file")
    require_path(scope_path, errors, checked, kind="file")
    require_path(tool_status_path, errors, checked, kind="file")
    require_path(report_metadata_path, errors, checked, kind="file")
    require_path(report_markdown_path, errors, checked, kind="file")
    checked.append(findings_path.as_posix())

    for directory_name in REQUIRED_DIRS:
        require_path(artifact_dir / directory_name, errors, checked, kind="dir")

    if errors:
        return ValidationResult(
            status="FAIL",
            artifact_dir=artifact_dir.as_posix(),
            checked_paths=checked,
            warnings=warnings,
            errors=errors,
        )

    try:
        manifest = load_json(manifest_path)
        scope_payload = load_json(scope_path)
        tool_status_payload = load_json(tool_status_path)
        report_metadata = load_json(report_metadata_path)
    except json.JSONDecodeError as exc:
        return ValidationResult(
            status="FAIL",
            artifact_dir=artifact_dir.as_posix(),
            checked_paths=checked,
            warnings=warnings,
            errors=[f"invalid JSON payload: {exc}"],
        )

    if not isinstance(tool_status_payload.get("tools"), list):
        errors.append("tool-status.json must contain a tools list")

    if scope_payload.get("status") not in {"OK", "SKIP"}:
        errors.append("scope.normalized.json must contain an OK or SKIP status")

    report_dir = validate_manifest_identity(artifact_dir, manifest, warnings, errors)
    validate_commands(artifact_dir, manifest, warnings, errors, checked)
    validate_report_metadata(report_metadata, errors)

    if findings_path.exists():
        findings_payload = load_json(findings_path)
        if not isinstance(findings_payload.get("findings"), list):
            errors.append("normalized/findings.json must contain a findings list")

    if report_dir is not None:
        host_report_md = report_dir / "security-report.md"
        host_report_meta = report_dir / "report-metadata.json"
        checked.extend([host_report_md.as_posix(), host_report_meta.as_posix()])
        if not host_report_md.is_file():
            errors.append(f"host report markdown is missing: {host_report_md}")
        if not host_report_meta.is_file():
            errors.append(f"host report metadata is missing: {host_report_meta}")

    if not list((artifact_dir / "normalized").glob("*.status.json")):
        warnings.append("normalized/ contains no per-tool status JSON files")

    if not (artifact_dir / "dast" / "summary.json").exists():
        warnings.append("dast/summary.json is missing")

    return ValidationResult(
        status="OK" if not errors else "FAIL",
        artifact_dir=artifact_dir.as_posix(),
        report_dir=report_dir.as_posix() if report_dir is not None else None,
        checked_paths=checked,
        warnings=warnings,
        errors=errors,
    )


def print_text_result(result: ValidationResult) -> None:
    """Emit a concise human-readable validation summary."""

    if result.status == "OK":
        print(f"OK: artifact contract validated for {result.artifact_dir}")
        if result.report_dir:
            print(f"report_dir={result.report_dir}")
        if result.warnings:
            for warning in result.warnings:
                print(f"WARNING: {warning}")
        return

    for error in result.errors:
        print(f"FAIL: {error}")


def main() -> int:
    """CLI entrypoint."""

    args = parse_args()
    result = validate_artifact_tree(Path(args.artifact_dir).resolve())
    if args.json:
        print(json.dumps(asdict(result), indent=2, sort_keys=True))
    else:
        print_text_result(result)
    return result.exit_code()


if __name__ == "__main__":
    raise SystemExit(main())
