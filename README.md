# bluepencil

An editor for rules files that strikes the rules no agent follows, rewrites the ones agents misread, and moves the buried ones to the top, receipts attached.

## What this is

The rules file is the most edited artifact in any agent setup, a fossil record of lessons bought with specific failures, and nothing audits it against behaviour. Bluepencil takes a rules file and session transcripts and produces a diff for a human with three proposal kinds only. It deletes rules that are never exercised, rewrites rules that are exercised and misread with the misreading quoted, and promotes rules that are buried and load bearing. Nothing is applied automatically. The boundary with its sibling is one sentence. Corked judges the work, bluepencil judges the instructions.

## Status

Seeded in October 2026. Local setup is in progress on the setup
branch. The remote seeding pass is waiting on a token access fix, so
work is banked locally until pushes clear. This repo is in phase 3
of the Retrospective Line and its build waits on the phase gate, the
main chat confirmation that the Gate Truth evidence pack schema has
merged and passed acceptance. Design work may proceed, build does
not start before the gate.

## Programme status

- Closed perfection rounds, 0 of 100. A round counts only when a
  full cycle of work, audit, review, red team, and approval has
  closed.
- Contract, stub in place, not yet met.
- Local gate, python3 scripts/codepoint_scan.py and, once tests
  exist, python3 -m pytest tests -q. CI runs the same commands.

## Licence

MIT. See LICENSE.
