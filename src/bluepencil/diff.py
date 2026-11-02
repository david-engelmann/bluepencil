"""BP-13 to BP-17. Render a proposal file as a diff
for a human. The diff applies nothing."""
from __future__ import annotations

import hashlib
import json
import re

from .proposals import validate_proposals

MARKER = re.compile(r"^(\s*(?:[-*+]|\d+[.)])\s+)")
KINDS = ("delete", "rewrite", "promote")
EVIDENCE_KINDS = ("misreading", "ignored", "followed")
HEX = frozenset("0123456789abcdef")


class DiffError(ValueError):
    """A named refusal."""


def _is_section(value) -> bool:
    return isinstance(value, list) and all(
        isinstance(name, str) for name in value)


def _check_proposal(proposal) -> None:
    """BP-13a: every field the renderer touches, shaped."""
    if not isinstance(proposal, dict):
        raise DiffError("a proposal is not an object")
    rule_id = proposal.get("rule_id")
    if not isinstance(rule_id, str) or not rule_id \
            or "\n" in rule_id or "\r" in rule_id:
        raise DiffError("a proposal names no rule id")
    if not isinstance(proposal.get("rule_text"), str):
        raise DiffError(
            f"proposal for {rule_id} carries no rule text")
    kind = proposal.get("kind")
    if kind not in KINDS:
        raise DiffError(
            f"proposal for {rule_id} has unknown kind "
            f"{kind!r}")
    evidence = proposal.get("evidence")
    if not isinstance(evidence, list):
        raise DiffError(
            f"proposal for {rule_id} carries no evidence "
            "list")
    for item in evidence:
        if not isinstance(item, dict) \
                or item.get("kind") not in EVIDENCE_KINDS \
                or not isinstance(item.get("quote"), str):
            raise DiffError(
                f"proposal for {rule_id} cites evidence "
                "that is not a quoted observation")
    hunk = proposal.get("hunk")
    if not isinstance(hunk, dict):
        raise DiffError(
            f"proposal for {rule_id} carries no hunk")
    if kind == "delete":
        shaped = (hunk.get("action") == "delete"
                  and _is_section(hunk.get("section"))
                  and isinstance(hunk.get("text"), str))
    elif kind == "rewrite":
        shaped = (hunk.get("action") == "rewrite"
                  and _is_section(hunk.get("section"))
                  and isinstance(hunk.get("old"), str)
                  and isinstance(hunk.get("new"), str)
                  and isinstance(proposal.get("replacement"),
                                 str))
    else:
        shaped = (hunk.get("action") == "move"
                  and _is_section(hunk.get("from_section"))
                  and _is_section(hunk.get("to_section"))
                  and isinstance(hunk.get("text"), str))
    if not shaped:
        raise DiffError(
            f"proposal for {rule_id} carries a hunk of "
            "the wrong shape")


def _check_shape(document) -> None:
    """BP-13a: the document the renderer stands over."""
    target = document.get("target")
    if not isinstance(target, dict):
        raise DiffError("the proposal file names no target")
    sha = target.get("sha256")
    if not isinstance(sha, str) or len(sha) != 64 \
            or any(char not in HEX for char in sha):
        raise DiffError(
            "the proposal file names no target sha256")
    for entry in document["inventory"]:
        if not isinstance(entry, dict) \
                or not isinstance(entry.get("id"), str) \
                or not isinstance(entry.get("text"), str) \
                or not _is_section(entry.get("section")):
            raise DiffError(
                "an inventory entry is not a rule")
        span = entry.get("line_span")
        if not (isinstance(span, list) and len(span) == 2
                and all(type(part) is int for part in span)):
            raise DiffError(
                "an inventory entry carries no line span")
    for proposal in document["proposals"]:
        _check_proposal(proposal)


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
    _check_shape(document)
    return document


def _flat(text: str) -> str:
    return " ".join(text.splitlines())


def _section(section) -> str:
    return " / ".join(section)


def _check_hunk(proposal, entry, rules) -> None:
    """BP-14a: the hunk must agree with the rule it names."""
    kind = proposal["kind"]
    hunk = proposal["hunk"]
    rule_id = proposal["rule_id"]
    if kind == "delete":
        agree = (hunk["section"] == entry["section"]
                 and hunk["text"] == proposal["rule_text"])
    elif kind == "rewrite":
        agree = (hunk["section"] == entry["section"]
                 and hunk["old"] == proposal["rule_text"]
                 and hunk["new"] == proposal["replacement"])
    else:
        first_section = rules[0]["section"] if rules else None
        agree = (hunk["from_section"] == entry["section"]
                 and hunk["text"] == proposal["rule_text"]
                 and hunk["to_section"] == first_section)
    if not agree:
        raise DiffError(
            f"proposal for {rule_id} carries a hunk that "
            "does not agree with its rule")


def render_diff(lines: list[str], rules: list[dict],
                document: dict) -> str:
    """The diff text for one rules file and one proposal
    file, cross-checked before anything renders (BP-14).
    """
    _check_shape(document)
    validate_proposals(document)
    target = document["target"]["sha256"]
    inventory = {entry["id"]: entry
                 for entry in document["inventory"]}
    parsed = {rule["id"]: rule for rule in rules}
    blocks = []
    counts = {"delete": 0, "rewrite": 0, "promote": 0}
    for proposal in document["proposals"]:
        rule_id = proposal["rule_id"]
        kind = proposal["kind"]
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
        _check_hunk(proposal, entry, rules)
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
            replacement = proposal["replacement"] \
                .splitlines() or [""]
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
