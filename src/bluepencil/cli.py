"""BP-12. The propose command. Bluepencil never writes
to the rules file it judges."""
from __future__ import annotations

import json
import os
import sys
import tempfile
from pathlib import Path

from .inventory import InventoryError, parse_rules
from .observations import ObservationError, load_observations
from .proposals import ProposalError, build_document


def _write_atomic(path: Path, text: str) -> None:
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


def propose(rules_path: Path, observations_path: Path,
            out: Path | None) -> int:
    if out is not None and out.resolve() in (
            rules_path.resolve(), observations_path.resolve()):
        raise ProposalError(
            "bluepencil never writes to its inputs")
    rules_bytes = rules_path.read_bytes()
    rules = parse_rules(rules_bytes.decode("utf-8"))
    observations, rewrites = load_observations(observations_path)
    document = build_document(rules_bytes, rules, observations,
                              rewrites)
    text = json.dumps(document, indent=2, ensure_ascii=False) + "\n"
    if out is None:
        try:
            sys.stdout.write(text)
            sys.stdout.flush()
        except (BrokenPipeError, OSError) as exc:
            devnull = os.open(os.devnull, os.O_WRONLY)
            os.dup2(devnull, sys.stdout.fileno())
            raise ProposalError(
                f"cannot write to stdout: {exc}") from exc
    else:
        try:
            _write_atomic(out, text)
        except OSError as exc:
            raise ProposalError(
                f"cannot write proposal file {out}: {exc}") from exc
    return 0


def main(argv: list[str] | None = None) -> int:
    argv = sys.argv[1:] if argv is None else argv
    try:
        if not argv or argv[0] != "propose":
            raise ProposalError("usage: bluepencil propose "
                                "RULES OBSERVATIONS [-o OUT]")
        args = argv[1:]
        out = None
        if "-o" in args:
            i = args.index("-o")
            if i + 1 >= len(args):
                raise ProposalError("-o needs a path")
            out = Path(args[i + 1])
            args = args[:i] + args[i + 2:]
        if len(args) != 2:
            raise ProposalError("usage: bluepencil propose "
                                "RULES OBSERVATIONS [-o OUT]")
        return propose(Path(args[0]), Path(args[1]), out)
    except (InventoryError, ObservationError, ProposalError,
            OSError, UnicodeDecodeError) as exc:
        print(f"bluepencil: error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
