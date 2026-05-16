#!/usr/bin/env python3

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent

REQUIRED_FILES = [
    "AGENTS.md",
    "docs/README.md",
    "docs/agent/README.md",
    "docs/agent/08-documentation-system.md",
    "docs/memory/README.md",
    "docs/memory/roadmaps/README.md",
    "docs/memory/adr/README.md",
    "docs/specs/README.md",
]

NAV_FILES = [
    "AGENTS.md",
    "README.md",
    "docs/README.md",
    "docs/agent/README.md",
    "docs/agent/08-documentation-system.md",
    "docs/memory/README.md",
    "docs/memory/roadmaps/README.md",
    "docs/memory/adr/README.md",
    "docs/specs/README.md",
]

INDEX_SPECS = {
    "docs/README.md": (
        "## Start Here",
        lambda: sorted(
            ["AGENTS.md", "docs/agent/README.md", "docs/memory/README.md", "docs/specs/README.md"]
        ),
    ),
    "docs/agent/README.md": (
        "## Guides",
        lambda: sorted(
            f"docs/agent/{path.name}"
            for path in (ROOT / "docs/agent").glob("*.md")
            if path.name != "README.md"
        ),
    ),
    "docs/memory/README.md": (
        "## Index",
        lambda: sorted(
            [path.name for path in (ROOT / "docs/memory").glob("*.md") if path.name != "README.md"]
            + ["adr/README.md", "roadmaps/README.md"]
        ),
    ),
    "docs/memory/roadmaps/README.md": (
        "## Index",
        lambda: sorted(
            path.name
            for path in (ROOT / "docs/memory/roadmaps").glob("*.md")
            if path.name != "README.md"
        ),
    ),
    "docs/memory/adr/README.md": (
        "## Index",
        lambda: sorted(
            path.name
            for path in (ROOT / "docs/memory/adr").glob("*.md")
            if path.name != "README.md"
        ),
    ),
    "docs/specs/README.md": (
        "## Index",
        lambda: sorted(
            path.name
            for path in (ROOT / "docs/specs").glob("*.md")
            if path.name != "README.md"
        ),
    ),
}

FORBIDDEN_DUPLICATE_PHRASES = {
    "docs/agent/README.md": [
        "required entrypoint",
        "## Read this before any task",
        "## Required workflow",
        "## Source of truth",
    ],
}

GRAPHIFY_REQUIRED_SNIPPETS = [
    "graphify query",
    "graphify explain",
    "graphify path",
    "graphify update .",
    "missing, stale, or insufficient",
    "graphify-out/GRAPH_REPORT.md",
]

LOCAL_PATH_RE = re.compile(r"`([^`]+)`|\[[^\]]+\]\(([^)]+)\)")
SECTION_PATH_RE = re.compile(r"`([^`]+)`")


def read_text(rel_path: str) -> str:
    return (ROOT / rel_path).read_text(encoding="utf-8")


def error(errors: list[str], message: str) -> None:
    errors.append(message)


def is_local_path(token: str) -> bool:
    if not token or token.startswith(("http://", "https://", "file://", "#")):
        return False
    if any(ch in token for ch in "<>{}|$*?"):
        return False
    if " " in token:
        return False
    prefixes = (
        "docs/",
        "src/",
        "ui/",
        "migrations/",
        "config/",
        "graphify-out/",
        ".github/",
        "AGENTS.md",
        "README.md",
        "CLAUDE.md",
        "Makefile",
    )
    return token.startswith(prefixes)


def extract_local_paths(text: str) -> set[str]:
    paths: set[str] = set()
    for match in LOCAL_PATH_RE.finditer(text):
        token = match.group(1) or match.group(2) or ""
        token = token.strip()
        if is_local_path(token):
            paths.add(token.rstrip("/"))
    return paths


def extract_section(text: str, heading: str) -> str | None:
    lines = text.splitlines()
    capture = False
    collected: list[str] = []
    for line in lines:
        if line.strip() == heading:
            capture = True
            continue
        if capture and line.startswith("## "):
            break
        if capture:
            collected.append(line)
    if not capture:
        return None
    return "\n".join(collected)


def extract_section_paths(text: str, heading: str) -> list[str]:
    section = extract_section(text, heading)
    if section is None:
        raise ValueError(f"missing section {heading}")
    paths: list[str] = []
    for line in section.splitlines():
        stripped = line.strip()
        if not stripped.startswith("- "):
            continue
        matches = SECTION_PATH_RE.findall(stripped)
        paths.extend(match.rstrip("/") for match in matches if is_local_path(match) or match.endswith(".md"))
    return sorted(paths)


def validate_required_files(errors: list[str]) -> None:
    for rel_path in REQUIRED_FILES:
        if not (ROOT / rel_path).exists():
            error(errors, f"required file missing: {rel_path}")


def validate_local_references(errors: list[str]) -> None:
    for rel_path in NAV_FILES:
        text = read_text(rel_path)
        for token in sorted(extract_local_paths(text)):
            if not (ROOT / token).exists():
                error(errors, f"broken local reference in {rel_path}: {token}")


def validate_indexes(errors: list[str]) -> None:
    for rel_path, (heading, expected_fn) in INDEX_SPECS.items():
        text = read_text(rel_path)
        try:
            actual = extract_section_paths(text, heading)
        except ValueError as exc:
            error(errors, f"{rel_path}: {exc}")
            continue
        expected = expected_fn()
        if actual != expected:
            error(
                errors,
                f"{rel_path}: index drift\n  expected: {expected}\n  actual:   {actual}",
            )


def validate_duplicate_phrases(errors: list[str]) -> None:
    for rel_path, phrases in FORBIDDEN_DUPLICATE_PHRASES.items():
        text = read_text(rel_path)
        for phrase in phrases:
            if phrase in text:
                error(errors, f"{rel_path}: forbidden duplicated entrypoint language: {phrase}")


def validate_graphify_guidance(errors: list[str]) -> None:
    text = read_text("AGENTS.md")
    for snippet in GRAPHIFY_REQUIRED_SNIPPETS:
        if snippet not in text:
            error(errors, f"AGENTS.md missing Graphify guidance snippet: {snippet}")


def main() -> int:
    errors: list[str] = []
    validate_required_files(errors)
    validate_local_references(errors)
    validate_indexes(errors)
    validate_duplicate_phrases(errors)
    validate_graphify_guidance(errors)

    if errors:
        print("docs validation failed:")
        for item in errors:
            print(f"- {item}")
        return 1

    print("docs validation passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
