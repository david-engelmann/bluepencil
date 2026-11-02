"""BP-5, BP-5a. Parse a rules file into an inventory."""
from __future__ import annotations

import re

HEADING = re.compile(r"^(#{1,6})\s+(.*\S)\s*$")
BULLET = re.compile(r"^(\s*)[-*+]\s+(.*\S)\s*$")
NUMBERED = re.compile(r"^(\s*)\d+[.)]\s+(.*\S)\s*$")
FENCE = re.compile(r"^\s*(```|~~~)")


class InventoryError(Exception):
    """A named refusal."""


def parse_rules(text: str) -> list[dict]:
    """Every bullet or numbered line under a heading is a rule.

    Prose is context. Lines inside fenced code blocks are
    never rules. A rule line before the first heading is a
    named refusal, per BP-5a.
    """
    rules: list[dict] = []
    section_stack: list[tuple[int, str]] = []
    in_fence = False
    current = None  # the rule currently open for continuations

    def close_current(end_line: int) -> None:
        nonlocal current
        if current is not None:
            current["line_span"][1] = end_line
            current["text"] = "\n".join(current.pop("_lines"))
            rules.append(current)
            current = None

    lines = text.splitlines()
    for idx, line in enumerate(lines, start=1):
        if FENCE.match(line):
            close_current(idx - 1)
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        heading = HEADING.match(line)
        if heading:
            close_current(idx - 1)
            level = len(heading.group(1))
            while section_stack and section_stack[-1][0] >= level:
                section_stack.pop()
            section_stack.append((level, heading.group(2)))
            continue
        marker = BULLET.match(line) or NUMBERED.match(line)
        if marker:
            if not section_stack:
                raise InventoryError(
                    f"rule outside any section at line {idx}")
            close_current(idx - 1)
            current = {
                "id": f"rule-{len(rules) + 1}",
                "section": [name for _, name in section_stack],
                "line_span": [idx, idx],
                "_lines": [marker.group(2)],
                "_indent": len(marker.group(1)),
            }
            continue
        if current is not None and line.strip():
            indent = len(line) - len(line.lstrip())
            if indent > current["_indent"]:
                current["_lines"].append(line.strip())
                continue
        close_current(idx - 1)
    close_current(len(lines))
    for rule in rules:
        rule.pop("_indent", None)
    return rules
