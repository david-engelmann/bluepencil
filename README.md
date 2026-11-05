# bluepencil

An editor for rules files that strikes the rules no agent follows, rewrites the ones agents misread, and moves the buried ones to the top, receipts attached.

## What this is

The rules file is the most edited artifact in any agent setup, a fossil record of lessons bought with specific failures, and nothing audits it against behaviour. Bluepencil takes a rules file and an observations file, the evidence gathered from what agents actually did, and produces a proposal file for a human with three proposal kinds only. It deletes rules the evidence shows are ignored, rewrites rules that are exercised and misread with the misreading quoted, and promotes rules that are buried and load bearing. Every proposal rests on quoted observations; a rule with no evidence earns no proposal. Nothing is applied automatically. The boundary with its sibling is one sentence. Stickler judges the work, bluepencil judges the instructions.

## Status

Round 1 is closed. The propose command parses a rules
file into an inventory, joins the observations to it by
exact rule text, and writes the proposal file.

Round 2 is closed. The diff command renders a proposal
file against its rules file as a diff for a human, each
proposal with the rule's own lines and its cited
evidence, cross-checked against the inventory and the
parsed file before anything renders, and applies
nothing. Later rounds work from the remaining contract
backlog and the sibling formats as they pin.

## Programme status

- Closed perfection rounds, 6 of 100. A round counts only when a
  full cycle of work, audit, review, red team, and approval has
  closed.
- Contract, BP-1 to BP-12 with amendments BP-5a, BP-5b,
  BP-5c, BP-8a, and BP-10a, written ahead of the code and
  met in Round 1. BP-13 to BP-17 with amendments
  BP-13a, BP-14a, BP-15a, and BP-16a, written ahead of
  the code and met in Round 2. BP-18 to BP-26, the Rust
  port obligations, written ahead of the port and met in
  Round 3.
- Local gate, python3 scripts/codepoint_scan.py and
  cargo test. CI runs the same commands, with fmt and
  clippy ahead of the tests.

## Licence

MIT. See LICENSE.
