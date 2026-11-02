"""BP-6. Evidence enters as an observations file."""
from __future__ import annotations

import json
from pathlib import Path

KINDS = ("misreading", "ignored", "followed")


class ObservationError(ValueError):
    """A named refusal."""


def _entries(document: dict, key: str) -> list:
    entries = document.get(key, [])
    if not isinstance(entries, list):
        raise ObservationError(f"{key} must be a list")
    return entries


def _text_field(entry: dict, field: str, label: str) -> str:
    value = entry.get(field)
    if not isinstance(value, str) or not value:
        raise ObservationError(f"{label} has no {field}")
    return value


def load_observations(path: Path) -> tuple[list[dict], list[dict]]:
    """Return (observations, supplied rewrites).

    The document is a JSON object with an "observations"
    list and optionally a "rewrites" list. Every entry is
    validated by name; a malformed entry refuses the run.
    """
    try:
        document = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, ValueError,
            RecursionError) as exc:
        raise ObservationError(
            f"cannot read observations {path}: {exc}") from exc
    if not isinstance(document, dict) or not isinstance(
            document.get("observations"), list):
        raise ObservationError(
            "observations file must be a JSON object with an "
            "observations list")
    observations = []
    for i, entry in enumerate(_entries(document, "observations")):
        if not isinstance(entry, dict):
            raise ObservationError(f"observation {i} is not an object")
        label = f"observation {i}"
        kind = entry.get("kind")
        if kind not in KINDS:
            raise ObservationError(
                f"{label} has unknown kind {kind!r}")
        observations.append({
            "rule": _text_field(entry, "rule", label),
            "kind": kind,
            "quote": _text_field(entry, "quote", label)})
    rewrites = []
    for i, entry in enumerate(_entries(document, "rewrites")):
        if not isinstance(entry, dict):
            raise ObservationError(f"rewrite {i} is not an object")
        label = f"rewrite {i}"
        rewrites.append({
            "rule": _text_field(entry, "rule", label),
            "misreading_quote": _text_field(
                entry, "misreading_quote", label),
            "replacement": _text_field(
                entry, "replacement", label)})
    return observations, rewrites
