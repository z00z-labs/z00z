#!/usr/bin/env python3
"""Execution helpers for skill-selector dispatch chains."""

from __future__ import annotations

from dataclasses import dataclass
import re
from typing import Callable, Mapping, Protocol, Sequence


@dataclass(frozen=True)
class DispatchTarget:
    """Resolved execution target for one skill in the chain."""

    skill_name: str
    target_kind: str
    target_name: str


@dataclass(frozen=True)
class DispatchStep:
    """One prepared invocation inside an ordered chain."""

    target: DispatchTarget
    query: str
    position: int
    total: int
    input_text: str
    prior_output: str | None = None


@dataclass(frozen=True)
class DispatchRecord:
    """Recorded output from one dispatched step."""

    target: DispatchTarget
    position: int
    input_text: str
    output_text: str


@dataclass(frozen=True)
class ChainExecution:
    """Completed execution trace for a routed chain."""

    query: str
    chain: list[str]
    records: list[DispatchRecord]

    @property
    def final_output(self) -> str | None:
        if not self.records:
            return None
        return self.records[-1].output_text


class SkillDispatcher(Protocol):
    def dispatch(self, step: DispatchStep) -> str:
        """Execute one step and return its output."""


def normalize_request_points(query: str) -> list[str]:
    """Split a freeform request into compact verification points."""

    compact = re.sub(r"\s+", " ", query.strip())
    if not compact:
        return []

    parts = re.split(r"\b(?:and then|then|and|after that)\b|[;\n]+", compact, flags=re.IGNORECASE)
    points: list[str] = []
    seen: set[str] = set()

    for raw_part in parts:
        point = raw_part.strip(" ,.-")
        if not point:
            continue
        lowered = point.casefold()
        if lowered in seen:
            continue
        points.append(point)
        seen.add(lowered)

    if not points:
        return [compact]

    return points


def render_request_points(query: str) -> str:
    """Render the extracted request points as a numbered list."""

    points = normalize_request_points(query)
    if not points:
        return "1. Verify the full original task as written"
    return "\n".join(f"{index}. {point}" for index, point in enumerate(points, start=1))


def render_execution_trace(chain: Sequence[str], completed_steps: int) -> str:
    """Render the already completed worker steps before the current dispatch."""

    if completed_steps <= 0:
        return "1. No prior worker output yet"

    return "\n".join(
        f"{index}. {skill_name}" for index, skill_name in enumerate(chain[:completed_steps], start=1)
    )


def format_step_input(
    target: DispatchTarget,
    query: str,
    prior_output: str | None,
    chain: Sequence[str],
    completed_steps: int,
) -> str:
    """Render the input payload that each step receives."""

    if prior_output is None:
        return query

    if target.skill_name == "doublecheck":
        return (
            "Original task:\n"
            f"{query}\n\n"
            "Request points to verify:\n"
            f"{render_request_points(query)}\n\n"
            "Verification checklist:\n"
            "1. Confirm the created artifact answers the full original task.\n"
            "2. Confirm every listed request point is satisfied explicitly.\n"
            "3. Flag each missing, partial, or contradicted request point.\n"
            "4. Do not validate only the last baton; validate the final artifact against the original request.\n\n"
            "Completed execution trace:\n"
            f"{render_execution_trace(chain, completed_steps)}\n\n"
            "Final artifact to verify:\n"
            f"{prior_output}"
        )

    return (
        "Original task:\n"
        f"{query}\n\n"
        "Previous step output:\n"
        f"{prior_output}"
    )


def execute_chain(chain: Sequence[DispatchTarget], query: str, dispatcher: SkillDispatcher) -> ChainExecution:
    """Execute a chain through the supplied dispatcher."""

    records: list[DispatchRecord] = []
    prior_output: str | None = None
    chain_names = [target.skill_name for target in chain]

    for position, target in enumerate(chain, start=1):
        input_text = format_step_input(
            target,
            query,
            prior_output,
            chain_names,
            position - 1,
        )
        step = DispatchStep(
            target=target,
            query=query,
            position=position,
            total=len(chain),
            input_text=input_text,
            prior_output=prior_output,
        )
        output_text = dispatcher.dispatch(step)
        records.append(
            DispatchRecord(
                target=target,
                position=position,
                input_text=input_text,
                output_text=output_text,
            )
        )
        prior_output = output_text

    return ChainExecution(
        query=query,
        chain=chain_names,
        records=records,
    )


class RecordingDispatcher:
    """Dispatcher used for behavior tests and CLI smoke checks."""

    def __init__(
        self,
        handlers: Mapping[str, Callable[[DispatchStep], str]] | None = None,
    ) -> None:
        self.handlers = dict(handlers or {})
        self.steps: list[DispatchStep] = []

    def dispatch(self, step: DispatchStep) -> str:
        self.steps.append(step)
        handler = self.handlers.get(step.target.skill_name)
        if handler is not None:
            return handler(step)

        return (
            f"executed:{step.target.target_kind}:"
            f"{step.target.target_name}:input={step.input_text}"
        )