"""Routing helpers for scoring and chaining skill-selector results."""

from __future__ import annotations

import re
from typing import Iterable, Protocol


STOP_WORDS = {
    "a",
    "about",
    "after",
    "all",
    "also",
    "an",
    "and",
    "any",
    "are",
    "as",
    "at",
    "be",
    "before",
    "best",
    "both",
    "build",
    "by",
    "can",
    "code",
    "create",
    "current",
    "do",
    "does",
    "for",
    "from",
    "get",
    "guide",
    "help",
    "how",
    "if",
    "in",
    "into",
    "is",
    "it",
    "its",
    "me",
    "mode",
    "new",
    "of",
    "on",
    "or",
    "out",
    "over",
    "project",
    "repo",
    "repository",
    "request",
    "right",
    "run",
    "should",
    "skill",
    "skills",
    "so",
    "task",
    "that",
    "the",
    "then",
    "this",
    "to",
    "use",
    "user",
    "using",
    "want",
    "when",
    "which",
    "with",
    "workflow",
    "workflows",
    "work",
    "you",
    "your",
}

INTENT_BUNDLES = {
    "review": {"review", "reviewer", "audit", "bug", "risk"},
    "security": {"security", "secure", "vulnerability", "owasp"},
    "testing": {"test", "testing", "coverage", "fuzz", "regression"},
    "planning": {"plan", "planner", "planning", "architecture", "spec", "specification"},
    "docs": {"readme", "docs", "documentation", "document", "write", "tutorial"},
    "frontend": {"ui", "ux", "frontend", "react", "layout", "design", "dashboard"},
    "pr_feedback": {"comment", "feedback", "pull", "thread", "reviewer"},
    "rust": {"rust", "cargo", "crate", "clippy", "fmt"},
}

SELF_SCOPE_TOKENS = {
    "self",
    "own",
    "current",
    "sobstvennuju",
    "sobstvennuyu",
    "sobstvennyi",
    "sobstvennyj",
    "sebe",
    "svoi",
    "svoĭ",
    "svoe",
    "svoju",
    "svoj",
}

ACTION_TOKENS = {
    "review": {"review", "audit", "inspect", "analyze"},
    "create": {"create", "build", "generate", "make", "scaffold"},
    "document": {"document", "write", "explain"},
    "improve": {"improve", "fix", "enhance", "refine", "redesign"},
}

CHAIN_HINTS = {
    "review": ("code-reviewer", "review-extreme-skepticism"),
    "testing": ("rust-fuzz-coverage", "gsd-add-tests"),
    "docs_scan": ("document-project",),
    "docs_write": ("create-readme",),
    "frontend": ("frontend-ui",),
    "pr_feedback": ("address-pr-comments",),
    "skill_create": ("skill-builder",),
}

SPECIALTY_TOKENS = {
    "smart-rename": {"rename", "naming", "identifier", "abbreviation", "clarity"},
    "z00z-chat-init": {"init", "startup", "boot", "begin"},
    "sql-code-review": {"sql", "mysql", "postgresql", "postgres", "oracle", "database"},
    "ai-prompt-engineering-safety-review": {"prompt", "llm", "ai", "model"},
}

PROMOTION_RULES = (
    ("code-reviewer", {"review", "security"}, set(), set(), 24),
    ("rust-fuzz-coverage", {"review", "testing"}, set(), set(), 12),
    ("document-project", {"docs"}, {"document"}, set(), 18),
    ("create-readme", set(), {"readme"}, set(), 20),
    ("frontend-ui", {"frontend"}, set(), set(), 24),
    ("address-pr-comments", {"review", "pr_feedback"}, {"comment"}, {"improve"}, 34),
    ("crypto-skill-builder", set(), {"skill"}, {"create"}, 18),
    ("skill-builder", set(), {"skill"}, {"create"}, 24),
)


class SkillLike(Protocol):
    name: str
    description: str
    scope: str
    source_priority: int
    keywords: list[str]
    trigger_phrases: list[str]
    meta_skill: bool


RankedSkill = tuple[int, SkillLike, list[str]]


def entry_sort_key(entry: SkillLike) -> tuple[int, str]:
    return (entry.source_priority, entry.name.casefold())


def split_words(text: str) -> list[str]:
    camel_split = re.sub(r"([a-z0-9])([A-Z])", r"\1 \2", text)
    normalized = re.sub(r"[^a-zA-Z0-9]+", " ", camel_split)
    return [part.lower() for part in normalized.split() if part]


def singularize(word: str) -> str:
    if len(word) > 4 and word.endswith("ies"):
        return word[:-3] + "y"
    if len(word) > 3 and word.endswith("s") and not word.endswith(("ss", "us", "is")):
        return word[:-1]
    return word


def tokenize(text: str) -> list[str]:
    normalized_words = [singularize(word) for word in split_words(text)]
    return [word for word in normalized_words if len(word) > 2 and word not in STOP_WORDS]


def detect_query_intents(query_tokens: set[str]) -> set[str]:
    return {label for label, bundle in INTENT_BUNDLES.items() if query_tokens & bundle}


def detect_primary_actions(query_tokens: set[str]) -> set[str]:
    return {label for label, bundle in ACTION_TOKENS.items() if query_tokens & bundle}


def is_self_review_query(query_words: set[str], query_intents: set[str], query_actions: set[str]) -> bool:
    if "review" not in query_intents and "review" not in query_actions:
        return False
    return bool(query_words & SELF_SCOPE_TOKENS)


def add_score(score: int, reasons: list[str], points: int, reason: str) -> int:
    reasons.append(reason)
    return score + points


def score_name_match(score: int, reasons: list[str], entry: SkillLike, query_text: str) -> int:
    if entry.name.lower() in query_text:
        return add_score(score, reasons, 40, "exact skill-name mention")
    return score


def score_phrase_match(score: int, reasons: list[str], entry: SkillLike, query_text: str) -> int:
    phrase_hits = [phrase for phrase in entry.trigger_phrases if phrase and phrase in query_text]
    if not phrase_hits:
        return score
    return add_score(
        score,
        reasons,
        30 + (5 * min(len(phrase_hits), 3)),
        f"trigger phrase match: {phrase_hits[0]}",
    )


def score_name_overlap(score: int, reasons: list[str], entry: SkillLike, query_tokens: set[str]) -> int:
    overlap = query_tokens & set(split_words(entry.name))
    if not overlap:
        return score
    return add_score(score, reasons, 8 * len(overlap), f"name overlap: {', '.join(sorted(overlap))}")


def score_keyword_overlap(score: int, reasons: list[str], entry: SkillLike, query_tokens: set[str]) -> int:
    overlap = query_tokens & set(entry.keywords)
    if not overlap:
        return score
    preview = ", ".join(sorted(list(overlap))[:4])
    return add_score(score, reasons, 4 * len(overlap), f"keyword overlap: {preview}")


def score_review_action(score: int, reasons: list[str], entry_tokens: set[str], query_actions: set[str]) -> int:
    if "review" in query_actions and entry_tokens & {"review", "reviewer", "audit"}:
        return add_score(score, reasons, 18, "primary action fit: review")
    return score


def score_create_action(score: int, reasons: list[str], entry_tokens: set[str], query_actions: set[str]) -> int:
    if "create" in query_actions and entry_tokens & {"create", "builder", "build", "scaffold"}:
        return add_score(score, reasons, 16, "primary action fit: create")
    return score


def score_document_action(score: int, reasons: list[str], entry_tokens: set[str], query_actions: set[str]) -> int:
    if "document" in query_actions and entry_tokens & {"document", "documentation", "readme", "writer"}:
        return add_score(score, reasons, 16, "primary action fit: document")
    return score


def score_improve_action(score: int, reasons: list[str], entry_tokens: set[str], query_actions: set[str]) -> int:
    if "improve" in query_actions and entry_tokens & {"improve", "frontend", "ui", "refactor", "cleanup"}:
        return add_score(score, reasons, 14, "primary action fit: improve")
    return score


def score_docs_artifact_fit(
    score: int, reasons: list[str], entry_tokens: set[str], query_tokens: set[str]
) -> int:
    if "readme" in query_tokens and "readme" in entry_tokens:
        score = add_score(score, reasons, 14, "artifact fit: readme")
    if "dashboard" in query_tokens and entry_tokens & {"dashboard", "ui", "frontend"}:
        score = add_score(score, reasons, 14, "artifact fit: dashboard")
    return score


def score_pr_artifact_fit(
    score: int, reasons: list[str], entry_tokens: set[str], query_tokens: set[str]
) -> int:
    if "comment" in query_tokens and entry_tokens & {"comment", "feedback", "thread", "reviewer"}:
        score = add_score(score, reasons, 16, "artifact fit: review comments")
    if "pull" in query_tokens and entry_tokens & {"pull", "pr", "thread"}:
        score = add_score(score, reasons, 12, "artifact fit: pull request")
    return score


def score_artifact_fit(score: int, reasons: list[str], entry_tokens: set[str], query_tokens: set[str]) -> int:
    score = score_docs_artifact_fit(score, reasons, entry_tokens, query_tokens)
    return score_pr_artifact_fit(score, reasons, entry_tokens, query_tokens)


def score_action_fit(
    score: int,
    reasons: list[str],
    entry: SkillLike,
    query_tokens: set[str],
    query_actions: set[str],
) -> int:
    entry_tokens = set(entry.keywords) | set(split_words(entry.name))
    score = score_review_action(score, reasons, entry_tokens, query_actions)
    score = score_create_action(score, reasons, entry_tokens, query_actions)
    score = score_document_action(score, reasons, entry_tokens, query_actions)
    score = score_improve_action(score, reasons, entry_tokens, query_actions)
    return score_artifact_fit(score, reasons, entry_tokens, query_tokens)


def score_intent_alignment(score: int, reasons: list[str], entry: SkillLike, query_intents: set[str]) -> int:
    entry_tokens = set(entry.keywords) | set(split_words(entry.name))
    matched_intents = 0
    for label in sorted(query_intents):
        if entry_tokens & INTENT_BUNDLES[label]:
            score = add_score(score, reasons, 12, f"intent match: {label}")
            matched_intents += 1
    if matched_intents > 1:
        score = add_score(score, reasons, 8 * (matched_intents - 1), "multi-intent coverage")
    return score


def score_critical_intent_gap(score: int, reasons: list[str], entry: SkillLike, query_intents: set[str]) -> int:
    entry_tokens = set(entry.keywords) | set(split_words(entry.name))
    for label in sorted(query_intents & {"review", "security", "docs", "frontend"}):
        if entry_tokens & INTENT_BUNDLES[label]:
            continue
        score = add_score(score, reasons, -18, f"missing critical intent: {label}")
    return score


def score_routing_bias(score: int, reasons: list[str], entry: SkillLike) -> int:
    if score <= 0:
        return score
    if entry.scope == "workspace":
        score = add_score(score, reasons, 20, "workspace-local bonus")
    elif entry.scope == "user":
        score = add_score(score, reasons, 8, "user-level skill bonus")
    if entry.meta_skill:
        score = add_score(score, reasons, -4, "meta-skill penalty")
    return score


def score_entry(entry: SkillLike, query: str) -> tuple[int, list[str]]:
    query_text = query.lower()
    query_tokens = set(tokenize(query))
    query_intents = detect_query_intents(query_tokens)
    query_actions = detect_primary_actions(query_tokens)
    reasons: list[str] = []
    score = 0

    score = score_name_match(score, reasons, entry, query_text)
    score = score_phrase_match(score, reasons, entry, query_text)
    score = score_name_overlap(score, reasons, entry, query_tokens)
    score = score_keyword_overlap(score, reasons, entry, query_tokens)
    score = score_action_fit(score, reasons, entry, query_tokens, query_actions)
    score = score_intent_alignment(score, reasons, entry, query_intents)
    score = score_critical_intent_gap(score, reasons, entry, query_intents)
    return score_routing_bias(score, reasons, entry), reasons


def self_review_bonus(entry: SkillLike, self_review: bool) -> int:
    if not self_review:
        return 0
    if entry.name == "review-extreme-skepticism":
        return 36
    if entry.name == "code-reviewer":
        return 20
    return 0


def promotion_rule_bonus(
    entry: SkillLike,
    query_tokens: set[str],
    query_intents: set[str],
    query_actions: set[str],
) -> int:
    bonus = 0
    for name, required_intents, required_tokens, required_actions, points in PROMOTION_RULES:
        if rule_matches(
            entry,
            name,
            required_intents,
            required_tokens,
            required_actions,
            query_intents,
            query_tokens,
            query_actions,
        ):
            bonus += points
    return bonus


def has_repo_docs_bonus(entry: SkillLike, query_words: set[str], query_intents: set[str], query_actions: set[str]) -> bool:
    return bool(
        entry.name == "document-project"
        and "docs" in query_intents
        and "document" in query_actions
        and query_words & {"repo", "repository", "project", "brownfield"}
    )


def promotion_for_entry(
    entry: SkillLike,
    query_tokens: set[str],
    query_words: set[str],
    query_intents: set[str],
    query_actions: set[str],
) -> int:
    self_review = is_self_review_query(query_words, query_intents, query_actions)
    bonus = self_review_bonus(entry, self_review)
    bonus += promotion_rule_bonus(entry, query_tokens, query_intents, query_actions)
    if has_repo_docs_bonus(entry, query_words, query_intents, query_actions):
        bonus += 24
    return bonus


def rule_matches(
    entry: SkillLike,
    name: str,
    required_intents: set[str],
    required_tokens: set[str],
    required_actions: set[str],
    query_intents: set[str],
    query_tokens: set[str],
    query_actions: set[str],
) -> bool:
    return (
        entry.name == name
        and (not required_intents or required_intents <= query_intents)
        and (not required_tokens or required_tokens <= query_tokens)
        and (not required_actions or required_actions <= query_actions)
    )


def demotion_for_entry(
    entry: SkillLike,
    query_tokens: set[str],
    query_words: set[str],
    query_intents: set[str],
    query_actions: set[str],
) -> int:
    specialty = SPECIALTY_TOKENS.get(entry.name)
    penalty = 0
    if specialty and not (query_tokens & specialty):
        penalty -= 24
    if is_self_review_query(query_words, query_intents, query_actions) and entry.name == "review":
        penalty -= 28
    return penalty


def rerank_results(ranked: list[RankedSkill], query: str) -> list[RankedSkill]:
    query_tokens = set(tokenize(query))
    query_words = set(split_words(query))
    query_intents = detect_query_intents(query_tokens)
    query_actions = detect_primary_actions(query_tokens)
    adjusted: list[RankedSkill] = []

    for score, entry, reasons in ranked:
        bonus = promotion_for_entry(entry, query_tokens, query_words, query_intents, query_actions)
        penalty = demotion_for_entry(entry, query_tokens, query_words, query_intents, query_actions)
        adjusted_score = score + bonus + penalty
        adjusted_reasons = list(reasons)
        if bonus:
            adjusted_reasons.append(f"query-aware promotion: {bonus}")
        if penalty:
            adjusted_reasons.append(f"query-aware demotion: {penalty}")
        adjusted.append((adjusted_score, entry, adjusted_reasons))

    adjusted.sort(key=lambda item: (-item[0], *entry_sort_key(item[1])))
    return adjusted


def find_ranked_entry(ranked: list[RankedSkill], names: Iterable[str], minimum_score: int = 1) -> SkillLike | None:
    target_names = set(names)
    for score, entry, _reasons in ranked:
        if score >= minimum_score and entry.name in target_names:
            return entry
    return None


def append_unique(chain: list[str], skill_name: str | None) -> None:
    if skill_name and skill_name not in chain:
        chain.append(skill_name)


def add_docs_chain(chain: list[str], ranked: list[RankedSkill], query_tokens: set[str], query_intents: set[str]) -> None:
    if "docs" not in query_intents:
        return
    if not ({"document", "readme"} & query_tokens):
        return
    append_unique(chain, find_ranked_entry(ranked, CHAIN_HINTS["docs_scan"]).name if find_ranked_entry(ranked, CHAIN_HINTS["docs_scan"]) else None)
    append_unique(chain, find_ranked_entry(ranked, CHAIN_HINTS["docs_write"]).name if find_ranked_entry(ranked, CHAIN_HINTS["docs_write"]) else None)


def add_review_chain(chain: list[str], ranked: list[RankedSkill], query_intents: set[str]) -> None:
    if "review" not in query_intents:
        return
    if "pr_feedback" in query_intents:
        return
    append_unique(chain, find_ranked_entry(ranked, CHAIN_HINTS["review"]).name if find_ranked_entry(ranked, CHAIN_HINTS["review"]) else None)
    if "testing" in query_intents:
        append_unique(chain, find_ranked_entry(ranked, CHAIN_HINTS["testing"]).name if find_ranked_entry(ranked, CHAIN_HINTS["testing"]) else None)


def add_frontend_chain(chain: list[str], ranked: list[RankedSkill], query_intents: set[str]) -> None:
    if "frontend" not in query_intents:
        return
    append_unique(chain, find_ranked_entry(ranked, CHAIN_HINTS["frontend"]).name if find_ranked_entry(ranked, CHAIN_HINTS["frontend"]) else None)


def add_pr_feedback_chain(chain: list[str], ranked: list[RankedSkill], query_intents: set[str]) -> None:
    if "pr_feedback" not in query_intents:
        return
    append_unique(chain, find_ranked_entry(ranked, CHAIN_HINTS["pr_feedback"]).name if find_ranked_entry(ranked, CHAIN_HINTS["pr_feedback"]) else None)


def add_skill_creation_chain(chain: list[str], ranked: list[RankedSkill], query_actions: set[str], query_tokens: set[str]) -> None:
    if "create" not in query_actions or "skill" not in query_tokens:
        return
    append_unique(chain, find_ranked_entry(ranked, CHAIN_HINTS["skill_create"]).name if find_ranked_entry(ranked, CHAIN_HINTS["skill_create"]) else None)


def build_chain(ranked: list[RankedSkill], query: str) -> list[str]:
    if not ranked:
        return ["doublecheck"]

    query_tokens = set(tokenize(query))
    query_intents = detect_query_intents(query_tokens)
    query_actions = detect_primary_actions(query_tokens)
    chain: list[str] = []

    add_docs_chain(chain, ranked, query_tokens, query_intents)
    add_pr_feedback_chain(chain, ranked, query_intents)
    add_review_chain(chain, ranked, query_intents)
    add_frontend_chain(chain, ranked, query_intents)
    add_skill_creation_chain(chain, ranked, query_actions, query_tokens)

    if not chain:
        append_unique(chain, ranked[0][1].name)

    append_unique(chain, "doublecheck")
    return chain