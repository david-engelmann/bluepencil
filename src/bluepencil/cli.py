"""BP-12. The propose command. Bluepencil never writes
to the rules file it judges."""
from __future__ import annotations

import argparse
import json
import os
import sys
import tempfile
from pathlib import Path

from .diff import (DiffError, check_target, load_proposals,
                   render_diff)
from .inventory import InventoryError, parse_rules
from .observations import ObservationError, load_observations
from .proposals import ProposalError, build_document


def _write_atomically(path: Path, text: str) -> None:
    fd, tmp = tempfile.mkstemp(dir=str(path.parent),
                               prefix=path.name + ".")
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as handle:
            handle.write(text)
        os.replace(tmp, path)
    except BaseException:
        try:
            os.unlink(tmp)
        except OSError:
            pass
        raise


def _propose(args: argparse.Namespace) -> int:
    rules_path = Path(args.rules)
    observations_path = Path(args.observations)
    out = Path(args.output) if args.output else None
    if out is not None and out.resolve() in (
            rules_path.resolve(), observations_path.resolve()):
        print("error, bluepencil never writes to its inputs",
              file=sys.stderr)
        return 1
    if not rules_path.is_file():
        print(f"error, no such rules file, {rules_path}",
              file=sys.stderr)
        return 1
    if not observations_path.is_file():
        print(f"error, no such observations file, "
              f"{observations_path}", file=sys.stderr)
        return 1
    try:
        rules_bytes = rules_path.read_bytes()
    except OSError:
        print(f"error, cannot read the rules file, {rules_path}",
              file=sys.stderr)
        return 1
    try:
        rules_text = rules_bytes.decode("utf-8-sig")
    except UnicodeDecodeError:
        print(f"error, the rules file is not utf-8, {rules_path}",
              file=sys.stderr)
        return 1
    try:
        rules, warnings = parse_rules(rules_text)
        observations, rewrites = load_observations(observations_path)
        document = build_document(rules_bytes, rules, observations,
                                  rewrites, warnings)
    except (InventoryError, ObservationError,
            ProposalError) as exc:
        print(f"error, {exc}", file=sys.stderr)
        return 1
    text = json.dumps(document, indent=2, ensure_ascii=False) + "\n"
    try:
        if out is None:
            sys.stdout.write(text)
            sys.stdout.flush()
        else:
            _write_atomically(out, text)
    except (OSError, UnicodeEncodeError):
        if out is not None:
            print(f"error, cannot write the proposal file, {out}",
                  file=sys.stderr)
        else:
            print("error, cannot write the proposal to stdout",
                  file=sys.stderr)
            # The broken stream would fail again at interpreter
            # shutdown and mask this refusal with exit 120 and
            # an ignored-exception dump, so it is replaced
            # with a sink before returning.
            try:
                sys.stdout.close()
            except OSError:
                pass
            sys.stdout = open(os.devnull, "w", encoding="utf-8")
        return 1
    return 0


def _diff(args: argparse.Namespace) -> int:
    rules_path = Path(args.rules)
    proposals_path = Path(args.proposals)
    out = Path(args.output) if args.output else None
    if out is not None and out.resolve() in (
            rules_path.resolve(), proposals_path.resolve()):
        print("error, bluepencil never writes to its inputs",
              file=sys.stderr)
        return 1
    if not rules_path.is_file():
        print(f"error, no such rules file, {rules_path}",
              file=sys.stderr)
        return 1
    if not proposals_path.is_file():
        print(f"error, no such proposals file, "
              f"{proposals_path}", file=sys.stderr)
        return 1
    try:
        rules_bytes = rules_path.read_bytes()
    except OSError:
        print(f"error, cannot read the rules file, {rules_path}",
              file=sys.stderr)
        return 1
    try:
        rules_text = rules_bytes.decode("utf-8-sig")
    except UnicodeDecodeError:
        print(f"error, the rules file is not utf-8, {rules_path}",
              file=sys.stderr)
        return 1
    try:
        proposals_bytes = proposals_path.read_bytes()
    except OSError:
        print(f"error, cannot read the proposals file, "
              f"{proposals_path}", file=sys.stderr)
        return 1
    try:
        document = load_proposals(proposals_bytes)
        check_target(rules_bytes, document)
        rules, _warnings = parse_rules(rules_text)
        text = render_diff(rules_text.splitlines(), rules,
                           document)
    except (DiffError, InventoryError,
            ProposalError) as exc:
        print(f"error, {exc}", file=sys.stderr)
        return 1
    try:
        if out is None:
            sys.stdout.write(text)
            sys.stdout.flush()
        else:
            _write_atomically(out, text)
    except (OSError, UnicodeEncodeError):
        if out is not None:
            print(f"error, cannot write the diff file, {out}",
                  file=sys.stderr)
        else:
            print("error, cannot write the diff to stdout",
                  file=sys.stderr)
            # The broken stream would fail again at interpreter
            # shutdown and mask this refusal with exit 120 and
            # an ignored-exception dump, so it is replaced
            # with a sink before returning.
            try:
                sys.stdout.close()
            except OSError:
                pass
            sys.stdout = open(os.devnull, "w", encoding="utf-8")
        return 1
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="bluepencil")
    sub = parser.add_subparsers(dest="command", required=True)
    propose = sub.add_parser(
        "propose", help="propose edits to a rules file")
    propose.add_argument("rules")
    propose.add_argument("observations")
    propose.add_argument("-o", "--output")
    diff = sub.add_parser(
        "diff", help="render proposals as a diff for a human")
    diff.add_argument("rules")
    diff.add_argument("proposals")
    diff.add_argument("-o", "--output")
    args = parser.parse_args(argv)
    if args.command == "propose":
        return _propose(args)
    if args.command == "diff":
        return _diff(args)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
