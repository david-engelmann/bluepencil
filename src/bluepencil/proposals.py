"""BP-7 to BP-11. Join evidence to rules, generate and
validate proposals, assemble the proposal file."""
from __future__ import annotations

import hashlib

COUNTING_METHOD = (
    "counts are observations per rule by kind, matched by "
    "exact rule text; a rule edited since an observation "
    "was taken is a new rule and the observation is "
    "unmatched; an observation naming a text carried by "
    "two rules attaches to the first, and the later rule "
    "is flagged a duplicate")


class ProposalError(ValueError):
    """A named refusal."""


def join(rules: list[dict], observations: list[dict]):
    """Attach observations to rules by exact text (BP-6).

    Returns (joined rules with counts, unmatched list).
    """
    first_position: dict[str, int] = {}
    for position, rule in enumerate(rules):
        first_position.setdefault(rule["text"], position)
    joined = []
    seen_texts: dict[str, str] = {}
    for rule in rules:
        entry = dict(rule)
        entry["observations"] = []
        entry["counts"] = {"misreading": 0, "ignored": 0,
                           "followed": 0, "exercised": 0}
        if rule["text"] in seen_texts:
            entry["duplicate_of"] = seen_texts[rule["text"]]
        else:
            seen_texts[rule["text"]] = rule["id"]
        joined.append(entry)
    unmatched = []
    for obs in observations:
        position = first_position.get(obs["rule"])
        if position is None:
            unmatched.append(obs)
            continue
        entry = joined[position]
        entry["observations"].append(obs)
        entry["counts"][obs["kind"]] += 1
        entry["counts"]["exercised"] += 1
    return joined, unmatched


def generate(joined: list[dict], rewrites: list[dict]):
    """Mechanical generation (BP-10) plus supplied rewrites
    carried and validated (BP-9)."""
    proposals = []
    n = len(joined)
    first_section = joined[0]["section"] if joined else None
    seen_rewrite_texts: set[str] = set()
    for i, entry in enumerate(joined):
        counts = entry["counts"]
        citations = entry["observations"]
        if counts["ignored"] >= 2 and counts["followed"] == 0:
            proposals.append({
                "kind": "delete",
                "rule_id": entry["id"],
                "rule_text": entry["text"],
                "evidence": [o for o in citations
                             if o["kind"] == "ignored"],
                "hunk": {"action": "delete",
                         "section": entry["section"],
                         "text": entry["text"]},
            })
        for rewrite in (r for r in rewrites
                        if r["rule"] == entry["text"]
                        and entry["text"] not in seen_rewrite_texts):
            matching = [o for o in citations
                        if o["kind"] == "misreading"
                        and o["quote"] == rewrite["misreading_quote"]]
            if not matching:
                raise ProposalError(
                    f"rewrite for {entry['id']} quotes a "
                    "misreading that is not among its "
                    "observations")
            proposals.append({
                "kind": "rewrite",
                "rule_id": entry["id"],
                "rule_text": entry["text"],
                "evidence": matching,
                "misreading_quote": rewrite["misreading_quote"],
                "replacement": rewrite["replacement"],
                "hunk": {"action": "rewrite",
                         "section": entry["section"],
                         "old": entry["text"],
                         "new": rewrite["replacement"]},
            })
        seen_rewrite_texts.add(entry["text"])
        if (2 * i >= n and counts["followed"] >= 2
                and entry["section"] != first_section):
            proposals.append({
                "kind": "promote",
                "rule_id": entry["id"],
                "rule_text": entry["text"],
                "evidence": [o for o in citations
                             if o["kind"] == "followed"],
                "hunk": {"action": "move",
                         "from_section": entry["section"],
                         "to_section": first_section,
                         "text": entry["text"]},
            })
    known_texts = {entry["text"] for entry in joined}
    for rewrite in rewrites:
        if rewrite["rule"] not in known_texts:
            raise ProposalError(
                "supplied rewrite names a rule that is not in "
                f"the file: {rewrite['rule'][:60]!r}")
    return proposals


def validate_proposals(document: dict) -> None:
    """BP-8: every proposal rests on at least one quoted
    observation, and the target names the rules file."""
    target = document.get("target", {}).get("sha256", "")
    if len(target) != 64:
        raise ProposalError("proposal file names no target sha256")
    for proposal in document["proposals"]:
        if proposal.get("kind") not in ("delete", "rewrite",
                                        "promote"):
            raise ProposalError(
                f"proposal for {proposal.get('rule_id')} has "
                f"unknown kind {proposal.get('kind')!r}")
        if not proposal.get("evidence"):
            raise ProposalError(
                f"proposal for {proposal.get('rule_id')} rests "
                "on no observation")
        for item in proposal["evidence"]:
            if not item.get("quote"):
                raise ProposalError(
                    f"proposal for {proposal.get('rule_id')} "
                    "cites evidence with no quote")


def build_document(rules_bytes: bytes, rules: list[dict],
                   observations: list[dict],
                   rewrites: list[dict],
                   warnings: list[str] | None = None) -> dict:
    joined, unmatched = join(rules, observations)
    proposals = generate(joined, rewrites)
    document = {
        "proposal_version": "1.0",
        "target": {"sha256": hashlib.sha256(rules_bytes).hexdigest()},
        "counting_method": COUNTING_METHOD,
        "inventory": [
            {key: entry[key] for key in
             ("id", "section", "text", "line_span", "counts",
              "duplicate_of") if key in entry}
            for entry in joined
        ],
        "unmatched_observations": unmatched,
        "warnings": list(warnings or []),
        "proposals": proposals,
    }
    validate_proposals(document)
    return document
