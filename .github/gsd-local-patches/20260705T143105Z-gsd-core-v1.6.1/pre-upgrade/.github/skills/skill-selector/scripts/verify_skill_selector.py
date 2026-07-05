#!/usr/bin/env python3
"""Behavioral regression assertions for skill-selector routing and dispatch."""

from __future__ import annotations

from pathlib import Path
import sys


SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

import build_skill_index as index_builder
from executor import RecordingDispatcher
from router import build_chain, rerank_results, score_entry


def ranked_for_query(entries: list[index_builder.SkillEntry], query: str) -> list[tuple[int, index_builder.SkillEntry, list[str]]]:
    ranked: list[tuple[int, index_builder.SkillEntry, list[str]]] = []
    for entry in entries:
        score, reasons = score_entry(entry, query)
        if score > 0:
            ranked.append((score, entry, reasons))
    ranked.sort(key=lambda item: (-item[0], item[1].scope, item[1].name))
    return rerank_results(ranked, query)


def assert_case(
    entries: list[index_builder.SkillEntry],
    query: str,
    expected_top: str,
    expected_chain: list[str],
    expected_first_alternate: str | None = None,
) -> None:
    ranked = ranked_for_query(entries, query)
    if not ranked:
        raise AssertionError(f"No matches found for query: {query}")

    top_name = ranked[0][1].name
    chain = build_chain(ranked, query)
    alternates = index_builder.select_alternates(ranked, query)

    if top_name != expected_top:
        raise AssertionError(
            f"Top match mismatch for {query!r}: expected {expected_top!r}, got {top_name!r}"
        )
    if chain != expected_chain:
        raise AssertionError(
            f"Chain mismatch for {query!r}: expected {expected_chain!r}, got {chain!r}"
        )
    if expected_first_alternate is not None:
        actual_first = alternates[0] if alternates else None
        if actual_first != expected_first_alternate:
            raise AssertionError(
                f"First alternate mismatch for {query!r}: expected {expected_first_alternate!r}, got {actual_first!r}"
            )


def assert_equal_chain(query: str, label: str, actual: list[str], expected: list[str]) -> None:
    if actual != expected:
        raise AssertionError(
            f"{label} mismatch for {query!r}: expected {expected!r}, got {actual!r}"
        )


def assert_dispatch_trace(
    query: str,
    execution: index_builder.ChainExecution,
    dispatcher: RecordingDispatcher,
    expected_chain: list[str],
) -> None:
    executed_names = [record.target.skill_name for record in execution.records]
    dispatched_names = [step.target.skill_name for step in dispatcher.steps]

    assert_equal_chain(query, "Dispatched chain", execution.chain, expected_chain)
    assert_equal_chain(query, "Executed steps", executed_names, expected_chain)
    assert_equal_chain(query, "Dispatcher calls", dispatched_names, expected_chain)


def assert_step_payloads(
    query: str,
    execution: index_builder.ChainExecution,
    dispatcher: RecordingDispatcher,
) -> None:
    if dispatcher.steps[0].prior_output is not None:
        raise AssertionError(f"First step unexpectedly received prior output for {query!r}")
    if dispatcher.steps[0].input_text != query:
        raise AssertionError(
            f"First step payload mismatch for {query!r}: expected {query!r}, got {dispatcher.steps[0].input_text!r}"
        )

    for index, step in enumerate(dispatcher.steps[1:], start=1):
        expected_prior = execution.records[index - 1].output_text
        if step.prior_output != expected_prior:
            raise AssertionError(
                f"Prior output mismatch for {query!r} at step {index + 1}: expected {expected_prior!r}, got {step.prior_output!r}"
            )
        if step.input_text != execution.records[index].input_text:
            raise AssertionError(
                f"Payload mismatch for {query!r} at step {index + 1}: expected {execution.records[index].input_text!r}, got {step.input_text!r}"
            )


def assert_doublecheck_target(query: str, final_record: index_builder.DispatchRecord) -> None:
    if final_record.target.skill_name != "doublecheck":
        raise AssertionError(f"Final executed step must be doublecheck for {query!r}")
    if final_record.target.target_kind != "agent":
        raise AssertionError(f"Doublecheck target kind must be agent for {query!r}")
    if final_record.target.target_name != "Doublecheck":
        raise AssertionError(f"Doublecheck target name mismatch for {query!r}: {final_record.target.target_name!r}")


def assert_doublecheck_trace(query: str, final_record: index_builder.DispatchRecord) -> None:
    if "executed:agent:Doublecheck" not in final_record.output_text:
        raise AssertionError(f"Doublecheck output missing for {query!r}: {final_record.output_text!r}")


def assert_doublecheck_payload(query: str, final_record: index_builder.DispatchRecord) -> None:
    payload_checks = [
        ("Request points to verify:", "Doublecheck request points missing"),
        ("Verification checklist:", "Doublecheck checklist missing"),
        ("Do not validate only the last baton", "Doublecheck baton guard missing"),
        ("Final artifact to verify:", "Doublecheck final artifact label missing"),
    ]
    for required_text, message in payload_checks:
        if required_text not in final_record.input_text:
            raise AssertionError(f"{message} for {query!r}: {final_record.input_text!r}")


def assert_doublecheck_output(query: str, execution: index_builder.ChainExecution) -> None:
    final_record = execution.records[-1]
    assert_doublecheck_target(query, final_record)
    assert_doublecheck_trace(query, final_record)
    assert_doublecheck_payload(query, final_record)


def assert_doublecheck_request_points(entries: list[index_builder.SkillEntry]) -> None:
    dispatcher = RecordingDispatcher()
    execution = index_builder.dispatch_query(entries, "document this repo and write a README", dispatcher)
    final_input = execution.records[-1].input_text

    if "1. document this repo" not in final_input:
        raise AssertionError(f"First request point missing from doublecheck payload: {final_input!r}")
    if "2. write a README" not in final_input:
        raise AssertionError(f"Second request point missing from doublecheck payload: {final_input!r}")
    if "1. document-project" not in final_input:
        raise AssertionError(f"Execution trace missing first worker for doublecheck payload: {final_input!r}")
    if "2. create-readme" not in final_input:
        raise AssertionError(f"Execution trace missing second worker for doublecheck payload: {final_input!r}")


def assert_dispatch_case(entries: list[index_builder.SkillEntry], query: str, expected_chain: list[str]) -> None:
    dispatcher = RecordingDispatcher()
    execution = index_builder.dispatch_query(entries, query, dispatcher)

    if not dispatcher.steps:
        raise AssertionError(f"No dispatch steps recorded for {query!r}")

    assert_dispatch_trace(query, execution, dispatcher, expected_chain)
    assert_step_payloads(query, execution, dispatcher)
    assert_doublecheck_output(query, execution)


def assert_rank_cases(entries: list[index_builder.SkillEntry]) -> None:
    assert_case(
        entries,
        "document this repo and write a README",
        "document-project",
        ["document-project", "create-readme", "doublecheck"],
        "create-readme",
    )
    assert_case(
        entries,
        "fix review comments on the active pull request",
        "address-pr-comments",
        ["address-pr-comments", "doublecheck"],
        "gh-cli",
    )
    assert_case(
        entries,
        "create a new skill from this workflow and validate the structure",
        "skill-builder",
        ["skill-builder", "doublecheck"],
        "crypto-skill-builder",
    )
    assert_case(
        entries,
        "document this brownfield project for humans and AI context",
        "document-project",
        ["document-project", "create-readme", "doublecheck"],
        "smart-docs-fusion",
    )
    assert_case(
        entries,
        "review this Rust crate for security issues and missing tests",
        "code-reviewer",
        ["code-reviewer", "rust-fuzz-coverage", "doublecheck"],
        "code-recon",
    )
    assert_case(
        entries,
        "prover sobstvennuju rabotu na kachestvo sdelaj sebe polnocennoe review",
        "review-extreme-skepticism",
        ["review-extreme-skepticism", "doublecheck"],
        "code-reviewer",
    )


def assert_dispatch_cases(entries: list[index_builder.SkillEntry]) -> None:
    assert_dispatch_case(
        entries,
        "document this repo and write a README",
        ["document-project", "create-readme", "doublecheck"],
    )
    assert_dispatch_case(
        entries,
        "fix review comments on the active pull request",
        ["address-pr-comments", "doublecheck"],
    )
    assert_dispatch_case(
        entries,
        "prover sobstvennuju rabotu na kachestvo sdelaj sebe polnocennoe review",
        ["review-extreme-skepticism", "doublecheck"],
    )


def assert_unique_skill_names(entries: list[index_builder.SkillEntry]) -> None:
    names = [entry.name.casefold() for entry in entries]
    if len(names) != len(set(names)):
        raise AssertionError("Indexed skill names must be unique after collision resolution")


def assert_agent_skills_sources(entries: list[index_builder.SkillEntry]) -> None:
    found = next((entry for entry in entries if entry.name == "microsoft-foundry"), None)
    if found is None:
        raise AssertionError("Expected microsoft-foundry skill from ~/.agents/skills to be indexed")
    if found.scope != "user":
        raise AssertionError(f"microsoft-foundry scope mismatch: {found.scope!r}")
    if found.source_label != "user-agents":
        raise AssertionError(f"microsoft-foundry source label mismatch: {found.source_label!r}")


def assert_selector_metadata(entries: list[index_builder.SkillEntry]) -> None:
    selector = next((entry for entry in entries if entry.name == "skill-selector"), None)
    if selector is None:
        raise AssertionError("skill-selector must be present in the indexed skills")
    expected_hint = 'task="<task description>" | --rebuild-index'
    actual_hint = str(selector.metadata.get("argument-hint") or "")
    if actual_hint != expected_hint:
        raise AssertionError(
            f"skill-selector metadata argument-hint mismatch: expected {expected_hint!r}, got {actual_hint!r}"
        )


def main() -> None:
    entries = index_builder.scan_skills()
    discovered_entries = index_builder.discover_skills()
    if len(discovered_entries) < 400:
        raise AssertionError(f"Expected at least 400 discovered skills, got {len(discovered_entries)}")
    if len(entries) < 300:
        raise AssertionError(f"Expected at least 300 unique indexed skills after collision resolution, got {len(entries)}")

    assert_rank_cases(entries)
    assert_dispatch_cases(entries)
    assert_doublecheck_request_points(entries)
    assert_unique_skill_names(entries)
    assert_agent_skills_sources(entries)
    assert_selector_metadata(entries)

    print("skill-selector regression checks passed")
    print(f"indexed_skills={len(entries)}")


if __name__ == "__main__":
    main()