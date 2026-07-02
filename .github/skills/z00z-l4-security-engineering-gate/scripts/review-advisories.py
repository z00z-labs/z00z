#!/usr/bin/env python3

import argparse
import json
import pathlib
import subprocess
import sys
import tomllib
from collections import defaultdict, deque


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Classify cargo-audit findings for Z00Z.")
    parser.add_argument("--root", required=True)
    parser.add_argument("--vendor-root", required=True)
    parser.add_argument("--review-file", required=True)
    parser.add_argument("--summary-out", required=True)
    parser.add_argument("--project-report", required=True)
    parser.add_argument("--vendor-report", required=True)
    return parser.parse_args()


def run_json(cmd: list[str], cwd: pathlib.Path) -> dict:
    proc = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    stdout = proc.stdout.strip()
    if proc.returncode not in (0, 1):
        sys.stderr.write(proc.stderr)
        raise RuntimeError(f"command failed: {' '.join(cmd)}")
    if not stdout:
        sys.stderr.write(proc.stderr)
        raise RuntimeError(f"command returned no JSON: {' '.join(cmd)}")
    try:
        return json.loads(stdout)
    except json.JSONDecodeError as exc:
        sys.stderr.write(proc.stderr)
        raise RuntimeError(f"could not parse JSON from {' '.join(cmd)}: {exc}") from exc


def load_reviews(path: pathlib.Path) -> dict[tuple[str, str, str, str], dict]:
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    reviews = {}
    for item in data.get("reviewed", []):
        key = (
            item["id"],
            item["crate"],
            item["version"],
            item.get("scope", "any"),
        )
        reviews[key] = item
    return reviews


def package_scope(
    package_id: str,
    workspace_members: set[str],
    package_paths: dict[str, pathlib.Path],
    vendor_root: pathlib.Path,
) -> str:
    if package_id not in workspace_members:
        return "external"
    manifest = package_paths[package_id]
    try:
        manifest.relative_to(vendor_root)
        return "vendor"
    except ValueError:
        return "project"


def first_workspace_ancestors(
    package_ids: list[str],
    reverse_edges: dict[str, set[str]],
    workspace_members: set[str],
) -> set[str]:
    queue = deque(package_ids)
    visited = set(package_ids)
    ancestors: set[str] = set()
    while queue:
        current = queue.popleft()
        for parent in reverse_edges.get(current, set()):
            if parent in visited:
                continue
            visited.add(parent)
            if parent in workspace_members:
                ancestors.add(parent)
                continue
            queue.append(parent)
    return ancestors


def scope_from_ancestors(
    ancestors: set[str],
    workspace_members: set[str],
    package_paths: dict[str, pathlib.Path],
    vendor_root: pathlib.Path,
) -> str:
    if not ancestors:
        return "external"
    scopes = {
        package_scope(package_id, workspace_members, package_paths, vendor_root)
        for package_id in ancestors
    }
    scopes.discard("external")
    if not scopes:
        return "external"
    if scopes == {"vendor"}:
        return "vendor"
    if scopes == {"project"}:
        return "project"
    return "mixed"


def report_for_scope(path: pathlib.Path, title: str, findings: list[dict]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    lines = [f"# {title}", ""]
    lines.append(f"- Findings: `{len(findings)}`")
    lines.append("")
    if not findings:
        lines.append("- None.")
        lines.append("")
        path.write_text("\n".join(lines), encoding="utf-8")
        return

    for finding in findings:
        lines.append(
            f"## {finding['crate']} {finding['version']} / {finding['id']}"
        )
        lines.append("")
        lines.append(f"- Status: `{finding['status']}`")
        lines.append(f"- Kind: `{finding['kind']}`")
        lines.append(f"- Title: {finding['title']}")
        lines.append(f"- Scope: `{finding['scope']}`")
        lines.append(f"- URL: {finding['url']}")
        if finding["ancestors"]:
            joined = ", ".join(
                f"`{item['name']}` ({item['scope']})" for item in finding["ancestors"]
            )
            lines.append(f"- First workspace ancestors: {joined}")
        else:
            lines.append("- First workspace ancestors: none")
        if finding["review"]:
            lines.append(f"- Decision: `{finding['review']['decision']}`")
            lines.append(f"- Reason: {finding['review']['reason']}")
            action = finding["review"].get("action")
            if action:
                lines.append(f"- Action: {action}")
        else:
            lines.append("- Decision: `unreviewed`")
            lines.append("- Reason: no repository review record matched this advisory.")
        lines.append("")

    path.write_text("\n".join(lines), encoding="utf-8")


def main() -> int:
    args = parse_args()
    root = pathlib.Path(args.root).resolve()
    vendor_root = pathlib.Path(args.vendor_root).resolve()
    review_file = pathlib.Path(args.review_file).resolve()
    summary_out = pathlib.Path(args.summary_out).resolve()
    project_report = pathlib.Path(args.project_report).resolve()
    vendor_report = pathlib.Path(args.vendor_report).resolve()

    if not review_file.exists():
        raise FileNotFoundError(f"review file does not exist: {review_file}")

    metadata = run_json(
        ["cargo", "metadata", "--format-version", "1", "--all-features"],
        cwd=root,
    )
    audit = run_json(["cargo", "audit", "-q", "--json"], cwd=root)
    reviews = load_reviews(review_file)

    packages = {pkg["id"]: pkg for pkg in metadata.get("packages", [])}
    package_paths = {
        pkg_id: pathlib.Path(pkg["manifest_path"]).resolve()
        for pkg_id, pkg in packages.items()
    }
    workspace_members = set(metadata.get("workspace_members", []))

    reverse_edges: dict[str, set[str]] = defaultdict(set)
    resolve = metadata.get("resolve") or {}
    for node in resolve.get("nodes", []):
        for dep in node.get("deps", []):
            reverse_edges[dep["pkg"]].add(node["id"])

    audit_items = []
    for vulnerability in audit.get("vulnerabilities", {}).get("list", []):
        tagged = dict(vulnerability)
        tagged.setdefault("_z00z_kind", "vulnerability")
        audit_items.append(tagged)
    for unmaintained in audit.get("warnings", {}).get("unmaintained", []):
        tagged = dict(unmaintained)
        tagged.setdefault(
            "_z00z_kind",
            tagged.get("kind")
            or tagged.get("advisory", {}).get("informational")
            or "unmaintained",
        )
        audit_items.append(tagged)

    findings = []
    counts = {
        "project": {"reviewed": 0, "unreviewed": 0},
        "vendor": {"reviewed": 0, "unreviewed": 0},
        "mixed": {"reviewed": 0, "unreviewed": 0},
        "external": {"reviewed": 0, "unreviewed": 0},
    }

    for item in audit_items:
        package = item["package"]
        advisory = item["advisory"]
        crate = package["name"]
        version = package["version"]
        package_ids = [
            pkg_id
            for pkg_id, pkg in packages.items()
            if pkg["name"] == crate and pkg["version"] == version
        ]
        ancestors = first_workspace_ancestors(package_ids, reverse_edges, workspace_members)
        scope = scope_from_ancestors(ancestors, workspace_members, package_paths, vendor_root)
        review = (
            reviews.get((advisory["id"], crate, version, scope))
            or reviews.get((advisory["id"], crate, version, "any"))
        )
        status = "reviewed" if review else "unreviewed"
        counts[scope][status] += 1
        findings.append(
            {
                "id": advisory["id"],
                "crate": crate,
                "version": version,
                "kind": item.get("kind")
                or item.get("_z00z_kind")
                or advisory.get("informational")
                or "unknown",
                "title": advisory["title"],
                "url": advisory.get("url") or "",
                "scope": scope,
                "status": status,
                "review": review,
                "ancestors": [
                    {
                        "name": packages[pkg_id]["name"],
                        "version": packages[pkg_id]["version"],
                        "scope": package_scope(
                            pkg_id, workspace_members, package_paths, vendor_root
                        ),
                        "manifest_path": str(package_paths[pkg_id]),
                    }
                    for pkg_id in sorted(
                        ancestors,
                        key=lambda value: (
                            packages[value]["name"],
                            packages[value]["version"],
                        ),
                    )
                ],
            }
        )

    project_findings = [item for item in findings if item["scope"] in {"project", "mixed"}]
    vendor_findings = [item for item in findings if item["scope"] == "vendor"]

    report_for_scope(project_report, "Z00Z Supply-Chain Project Report", project_findings)
    report_for_scope(vendor_report, "Z00Z Supply-Chain Vendor Report", vendor_findings)

    summary = {
        "project": counts["project"],
        "vendor": counts["vendor"],
        "mixed": counts["mixed"],
        "external": counts["external"],
        "project_report": str(project_report),
        "vendor_report": str(vendor_report),
        "review_file": str(review_file),
        "finding_count": len(findings),
    }
    summary_out.parent.mkdir(parents=True, exist_ok=True)
    summary_out.write_text(json.dumps(summary, indent=2, sort_keys=True), encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
