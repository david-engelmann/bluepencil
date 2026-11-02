"""BP-13 to BP-17. Render a proposal file as a diff
for a human. The diff applies nothing."""
from __future__ import annotations

import hashlib
import json
import re

from .proposals import validate_proposals

MARKER = re.compile(r"^(\s*(?:[-*+]|\d+[.)])\s+)")


class DiffError(ValueError):
    """A named refusal."""


def load_proposals(data: bytes) -> dict:
    """Decode and shape-check a proposal file (BP-13)."""
    try:
        text = data.decode("utf-8-sig")
    except UnicodeDecodeError:
        raise DiffError("the proposal file is not utf-8") \
            from None
    try:
        document = json.loads(text)
    except json.JSONDecodeError:
        raise DiffError("the proposal file is not JSON") \
            from None
    if not isinstance(document, dict) \
            or not isinstance(document.get("proposals"), list) \
            or not isinstance(document.get("inventory"), list) \
            or not isinstance(document.get("target"), dict):
        raise DiffError(
            "the proposal file is not a proposal document")
    return document


def _flat(text: str) -> str:
    return " ".join(text.splitlines())


def _section(section) -> str:
    return " / ".join(section)


def render_diff(lines: list[str], rules: list[dict],
                document: dict) -> str:
    """The diff text for one rules file and one proposal
    file, cross-checked before anything renders (BP-14).
    """
    target = document["target"].get("sha256", "")
    validate_proposals(document)
    inventory = {entry["id"]: entry
                 for entry in document["inventory"]}
    parsed = {rule["id"]: rule for rule in rules}
    blocks = []
    counts = {"delete": 0, "rewrite": 0, "promote": 0}
    for proposal in document["proposals"]:
        rule_id = proposal.get("rule_id")
        kind = proposal.get("kind")
        counts[kind] += 1
        entry = inventory.get(rule_id)
        if entry is None or entry["text"] != \
                proposal["rule_text"]:
            raise DiffError(
                f"proposal for {rule_id} does not match "
                "the inventory it travels with")
        rule = parsed.get(rule_id)
        if rule is None or rule["text"] != \
                proposal["rule_text"] or \
                rule["section"] != entry["section"]:
            raise DiffError(
                f"proposal for {rule_id} does not match "
                "the rules file as parsed")
        start, end = rule["line_span"]
        raw = lines[start - 1:end]
        hunk = proposal["hunk"]
        if kind == "promote":
            header = (f"## promote {rule_id} "
                      f"({_section(hunk['from_section'])} to "
                      f"{_section(hunk['to_section'])})")
        else:
            header = (f"## {kind} {rule_id} "
                      f"({_section(entry['section'])})")
        block = [header]
        block += [f"- {line}" for line in raw]
        if kind == "rewrite":
            marker = MARKER.match(raw[0])
            prefix = marker.group(1) if marker else "- "
            replacement = proposal["replacement"].split("\n")
            block.append(f"+ {prefix}{replacement[0]}")
            block += [f"+ {line}" for line in replacement[1:]]
        for item in proposal["evidence"]:
            block.append(
                f"  evidence, {item['kind']}, "
                f"\"{_flat(item['quote'])}\"")
        blocks.append("\n".join(block))
    header = (
        f"bluepencil diff, {len(document['proposals'])} "
        f"proposals (delete {counts['delete']}, rewrite "
        f"{counts['rewrite']}, promote {counts['promote']})\n"
        f"target sha256 {target}")
    if not blocks:
        return header + "\n"
    return header + "\n\n" + "\n\n".join(blocks) + "\n"


def check_target(rules_bytes: bytes, document: dict) -> None:
    """BP-13: the proposal file must name this rules
    file by the SHA-256 of its bytes."""
    digest = hashlib.sha256(rules_bytes).hexdigest()
    if document["target"].get("sha256") != digest:
        raise DiffError(
            "the proposal file names a different rules file")
