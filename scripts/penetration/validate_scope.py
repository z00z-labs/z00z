#!/usr/bin/env python3
"""Validate the Phase 066 local-only pentest scope contract."""

from __future__ import annotations

import argparse
import ipaddress
import json
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any
from urllib.parse import SplitResult, urlsplit, urlunsplit

import yaml

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_DENYLIST_PATH = ROOT / ".security" / "denied-tools.txt"
DEFAULT_ALLOWED_TARGETS_PATH = ROOT / ".security" / "allowed-targets.txt"
REQUIRED_KEYS = {
    "mode",
    "allowed_paths",
    "excluded_paths",
    "allowed_hosts",
    "allowed_urls",
    "forbidden",
    "rate_limits",
    "evidence_required",
}
ALLOWED_MODES = {"local-only"}
ALLOWED_URL_SCHEMES = {"http", "https"}
WILDCARD_CHARS = {"*", "?", "["}
EXIT_OK = 0
EXIT_FAIL = 1
EXIT_SKIP = 3
LOOPBACK_V4 = ipaddress.ip_network("127.0.0.0/8")
LOOPBACK_V6 = ipaddress.ip_network("::1/128")


@dataclass
class ValidationResult:
    """Structured scope validation result."""

    status: str
    mode: str | None
    scope_path: str
    normalized_paths: list[str] = field(default_factory=list)
    normalized_hosts: list[str] = field(default_factory=list)
    normalized_urls: list[str] = field(default_factory=list)
    requested_tool: str | None = None
    dast_targets_present: bool = False
    skip_reason: str | None = None
    errors: list[str] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)

    def exit_code(self) -> int:
        if self.status == "OK":
            return EXIT_OK
        if self.status == "SKIP":
            return EXIT_SKIP
        return EXIT_FAIL


def fail(scope_path: Path, errors: list[str], *, requested_tool: str | None = None) -> ValidationResult:
    """Build a failed validation result."""

    return ValidationResult(
        status="FAIL",
        mode=None,
        scope_path=scope_path.as_posix(),
        requested_tool=requested_tool,
        errors=errors,
    )


def load_yaml(path: Path) -> Any:
    """Load a YAML file into Python objects."""

    return yaml.safe_load(path.read_text(encoding="utf-8")) or {}


def load_denylist(path: Path) -> set[str]:
    """Load the denylist from a newline-delimited text file."""

    denylist: set[str] = set()
    for line in path.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        denylist.add(stripped.casefold())
    return denylist


def load_allowed_targets_mirror(path: Path) -> list[str]:
    """Load the human-readable allowed host mirror."""

    hosts: list[str] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        hosts.append(stripped)
    return hosts


def ensure_mapping(value: Any, context: str, errors: list[str]) -> dict[str, Any]:
    """Validate that a loaded object is a mapping."""

    if isinstance(value, dict):
        return value
    errors.append(f"{context} must be a YAML mapping")
    return {}


def ensure_string_list(value: Any, field_name: str, errors: list[str]) -> list[str]:
    """Validate that a field is a list of non-empty strings."""

    if not isinstance(value, list):
        errors.append(f"{field_name} must be a list")
        return []

    items: list[str] = []
    for index, item in enumerate(value):
        if not isinstance(item, str):
            errors.append(f"{field_name}[{index}] must be a string")
            continue
        stripped = item.strip()
        if not stripped:
            errors.append(f"{field_name}[{index}] must not be empty")
            continue
        items.append(stripped)
    return items


def ensure_rate_limits(value: Any, errors: list[str]) -> dict[str, int]:
    """Validate rate-limit configuration."""

    if not isinstance(value, dict):
        errors.append("rate_limits must be a mapping")
        return {}

    normalized: dict[str, int] = {}
    for key in ("requests_per_second", "max_concurrency", "timeout_seconds"):
        raw_value = value.get(key)
        if not isinstance(raw_value, int) or raw_value <= 0:
            errors.append(f"rate_limits.{key} must be a positive integer")
            continue
        normalized[key] = raw_value
    return normalized


def normalize_repo_path(raw_path: str) -> str:
    """Normalize a repo-scoped path and reject escapes or glob patterns."""

    if any(char in raw_path for char in WILDCARD_CHARS):
        raise ValueError(f"path contains wildcard characters: {raw_path}")

    candidate = Path(raw_path)
    resolved = candidate.resolve(strict=False) if candidate.is_absolute() else (ROOT / candidate).resolve(strict=False)
    try:
        relative = resolved.relative_to(ROOT)
    except ValueError as exc:
        raise ValueError(f"path escapes repository root: {raw_path}") from exc
    return relative.as_posix() or "."


def normalize_host(raw_host: str) -> str:
    """Normalize a host or CIDR entry for local-only scope."""

    value = raw_host.strip()
    if any(char in value for char in WILDCARD_CHARS):
        raise ValueError(f"wildcard hosts are not allowed: {raw_host}")

    if "/" in value:
        try:
            network = ipaddress.ip_network(value, strict=False)
        except ValueError as exc:
            raise ValueError(f"invalid CIDR target: {raw_host}") from exc
        loopback = LOOPBACK_V4 if network.version == 4 else LOOPBACK_V6
        if not network.subnet_of(loopback):
            raise ValueError(f"CIDR target is broader than loopback: {raw_host}")
        return str(network)

    try:
        address = ipaddress.ip_address(value)
    except ValueError:
        hostname = value.rstrip(".").casefold()
        if hostname != "localhost":
            raise ValueError(f"public DNS names are not allowed in local-only mode: {raw_host}")
        return hostname

    if not address.is_loopback:
        raise ValueError(f"public or non-loopback IP targets are not allowed: {raw_host}")
    return str(address)


def normalize_url(raw_url: str) -> str:
    """Normalize a URL and ensure it resolves to a local-only target."""

    parsed = urlsplit(raw_url)
    if parsed.scheme.casefold() not in ALLOWED_URL_SCHEMES:
        raise ValueError(f"unsupported URL scheme for DAST target: {raw_url}")
    if parsed.username or parsed.password:
        raise ValueError(f"target URLs must not embed credentials: {raw_url}")
    if not parsed.hostname:
        raise ValueError(f"URL is missing a hostname: {raw_url}")
    try:
        port = parsed.port
    except ValueError as exc:
        raise ValueError(f"invalid URL port: {raw_url}") from exc

    normalized_host = normalize_host(parsed.hostname)
    netloc = normalized_host
    if port is not None:
        netloc = f"{netloc}:{port}"
    normalized = SplitResult(
        scheme=parsed.scheme.casefold(),
        netloc=netloc,
        path=parsed.path,
        query=parsed.query,
        fragment="",
    )
    return urlunsplit(normalized)


def maybe_validate_allowed_targets_mirror(scope_path: Path, normalized_hosts: list[str], errors: list[str]) -> None:
    """Keep the human-readable mirror aligned with the authoritative scope file."""

    if scope_path.resolve() != (ROOT / ".security" / "scope.yaml").resolve():
        return
    if not DEFAULT_ALLOWED_TARGETS_PATH.exists():
        errors.append(f"allowed target mirror is missing: {DEFAULT_ALLOWED_TARGETS_PATH}")
        return

    mirror_hosts = sorted(load_allowed_targets_mirror(DEFAULT_ALLOWED_TARGETS_PATH))
    if mirror_hosts != sorted(normalized_hosts):
        errors.append(
            "allowed target mirror does not match scope.yaml normalized hosts"
        )


def validate_scope(
    scope_path: Path,
    *,
    denylist_path: Path,
    requested_tool: str | None,
    require_dast_targets: bool,
) -> ValidationResult:
    """Validate the local-only scope contract."""

    if not scope_path.exists():
        return fail(scope_path, [f"scope file not found: {scope_path}"], requested_tool=requested_tool)
    if not denylist_path.exists():
        return fail(
            scope_path,
            [f"denylist file not found: {denylist_path}"],
            requested_tool=requested_tool,
        )

    errors: list[str] = []
    warnings: list[str] = []

    try:
        loaded = load_yaml(scope_path)
    except yaml.YAMLError as exc:
        return fail(scope_path, [f"invalid YAML: {exc}"], requested_tool=requested_tool)

    scope = ensure_mapping(loaded, "scope", errors)
    missing_keys = sorted(REQUIRED_KEYS - set(scope))
    if missing_keys:
        errors.append(f"scope is missing required keys: {', '.join(missing_keys)}")

    mode = scope.get("mode")
    if not isinstance(mode, str):
        errors.append("mode must be a string")
        mode = None
    elif mode not in ALLOWED_MODES:
        errors.append(
            f"mode '{mode}' is not authorized; explicit future authorization modes are not live in Phase 066"
        )

    allowed_paths_raw = ensure_string_list(scope.get("allowed_paths"), "allowed_paths", errors)
    excluded_paths_raw = ensure_string_list(scope.get("excluded_paths"), "excluded_paths", errors)
    allowed_hosts_raw = ensure_string_list(scope.get("allowed_hosts"), "allowed_hosts", errors)
    allowed_urls_raw = ensure_string_list(scope.get("allowed_urls"), "allowed_urls", errors)
    forbidden_raw = ensure_string_list(scope.get("forbidden"), "forbidden", errors)
    ensure_rate_limits(scope.get("rate_limits"), errors)

    evidence_required = scope.get("evidence_required")
    if not isinstance(evidence_required, bool):
        errors.append("evidence_required must be a boolean")
    elif not evidence_required:
        errors.append("evidence_required must remain true for the default workflow")

    normalized_paths: list[str] = []
    for raw_path in allowed_paths_raw:
        try:
            normalized_paths.append(normalize_repo_path(raw_path))
        except ValueError as exc:
            errors.append(str(exc))

    for raw_path in excluded_paths_raw:
        try:
            normalize_repo_path(raw_path)
        except ValueError as exc:
            errors.append(str(exc))

    normalized_hosts: list[str] = []
    for raw_host in allowed_hosts_raw:
        try:
            normalized_hosts.append(normalize_host(raw_host))
        except ValueError as exc:
            errors.append(str(exc))

    normalized_urls: list[str] = []
    for raw_url in allowed_urls_raw:
        try:
            normalized_urls.append(normalize_url(raw_url))
        except ValueError as exc:
            errors.append(str(exc))

    if not normalized_paths:
        errors.append("allowed_paths must contain at least one repository path")

    maybe_validate_allowed_targets_mirror(scope_path, normalized_hosts, errors)

    denylist = load_denylist(denylist_path)
    forbidden = {entry.casefold() for entry in forbidden_raw}
    if requested_tool:
        normalized_tool = Path(requested_tool).name.casefold()
        if normalized_tool in denylist or normalized_tool in forbidden:
            errors.append(f"requested tool is denied by policy: {normalized_tool}")

    if errors:
        return ValidationResult(
            status="FAIL",
            mode=mode if isinstance(mode, str) else None,
            scope_path=scope_path.as_posix(),
            normalized_paths=sorted(set(normalized_paths)),
            normalized_hosts=sorted(set(normalized_hosts)),
            normalized_urls=sorted(set(normalized_urls)),
            requested_tool=requested_tool,
            errors=errors,
            warnings=warnings,
        )

    dast_targets_present = bool(normalized_hosts or normalized_urls)
    if require_dast_targets and not dast_targets_present:
        return ValidationResult(
            status="SKIP",
            mode=mode,
            scope_path=scope_path.as_posix(),
            normalized_paths=sorted(set(normalized_paths)),
            normalized_hosts=[],
            normalized_urls=[],
            requested_tool=requested_tool,
            dast_targets_present=False,
            skip_reason="no allowed local DAST targets are present in scope",
            warnings=warnings,
        )

    return ValidationResult(
        status="OK",
        mode=mode,
        scope_path=scope_path.as_posix(),
        normalized_paths=sorted(set(normalized_paths)),
        normalized_hosts=sorted(set(normalized_hosts)),
        normalized_urls=sorted(set(normalized_urls)),
        requested_tool=requested_tool,
        dast_targets_present=dast_targets_present,
        warnings=warnings,
    )


def parse_args() -> argparse.Namespace:
    """Parse CLI arguments."""

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("scope_path", help="path to scope.yaml")
    parser.add_argument(
        "--denylist",
        default=str(DEFAULT_DENYLIST_PATH),
        help="path to denied-tools.txt",
    )
    parser.add_argument(
        "--tool",
        help="requested dynamic tool name to validate against the denylist",
    )
    parser.add_argument(
        "--require-dast-targets",
        action="store_true",
        help="return SKIP when the scope does not admit any local DAST targets",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="emit machine-readable validation output",
    )
    return parser.parse_args()


def print_text_result(result: ValidationResult) -> None:
    """Emit a human-readable validation summary."""

    if result.status == "OK":
        print("OK: scope validated for local-only testing")
        print(f"hosts={len(result.normalized_hosts)} urls={len(result.normalized_urls)}")
        return
    if result.status == "SKIP":
        print(f"SKIP: {result.skip_reason}")
        return
    for error in result.errors:
        print(f"FAIL: {error}")


def main() -> int:
    """CLI entrypoint."""

    args = parse_args()
    result = validate_scope(
        Path(args.scope_path),
        denylist_path=Path(args.denylist),
        requested_tool=args.tool,
        require_dast_targets=args.require_dast_targets,
    )

    if args.json:
        print(json.dumps(asdict(result), indent=2, sort_keys=True))
    else:
        print_text_result(result)
    return result.exit_code()


if __name__ == "__main__":
    raise SystemExit(main())
