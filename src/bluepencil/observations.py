"""BP-6. Evidence enters as an observations file."""
from __future__ import annotations

import json
from pathlib import Path

KINDS = ("misreading", "ignored", "followed")


class ObservationError(Exception):
    """A named refusal."""


def load_observations(path: Path) -> tuple[list[dict], list[dict]]:
    """Return (observations, supplied rewrites).

    The document is a JSON object with an "observations"
    list and optionally a "rewrites" list. Every entry is
    validated by name; a malformed entry refuses the run.
    """
    try:
        document = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise ObservationError(
            f"cannot read observations {path}: {exc}") from exc
    if not isinstance(document, dict) or not isinstance(
            document.get("observations"), list):
        raise ObservationError(
            "observations file must be a JSON object with an "
            "observations list")
    observations = []
    for i, entry in enumerate(document["observations"]):
        if not isinstance(entry, dict):
            raise ObservationError(f"observation {i} is not an object")
        rule, kind, quote = (entry.get("rule"), entry.get("kind"),
                             entry.get("quote"))
        if not isinstance(rule, str) or not rule:
            raise ObservationError(
                f"observation {i} names no rule text")
        if kind not in KINDS:
            raise ObservationError(
                f"observation {i} has unknown kind {kind!r}")
        if not isinstance(quote, str) or not quote:
            raise ObservationError(f"observation {i} has no quote")
        observations.append({"rule": rule, "kind": kind,
                             "quote": quote})
    rewrites = []
    raw_rewrites = document.get("rewrites", [])
    if not isinstance(raw_rewrites, list):
        raise ObservationError("rewrites must be a list")
    for i, entry in enumerate(raw_rewrites):
        if not isinstance(entry, dict):
            raise ObservationError(f"rewrite {i} is not an object")
        rule = entry.get("rule")
        misreading = entry.get("misreading_quote")
        replacement = entry.get("replacement")
        if not isinstance(rule, str) or not rule:
            raise ObservationError(f"rewrite {i} names no rule text")
        if not isinstance(misreading, str) or not misreading:
            raise ObservationError(
                f"rewrite {i} quotes no misreading")
        if not isinstance(replacement, str) or not replacement:
            raise ObservationError(
                f"rewrite {i} supplies no replacement text")
        rewrites.append({"rule": rule,
                         "misreading_quote": misreading,
                         "replacement": replacement})
    return observations, rewrites
