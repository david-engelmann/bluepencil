# bluepencil

An editor for rules files that works the way a copy editor works copy: strikes the rules no agent follows, rewrites the ones agents misread, and moves the buried ones to the top, receipts attached.

Give bluepencil a rules file and the observations gathered on the sessions that ran under it, and it answers with a proposal document. Rules the evidence shows ignored are proposed for deletion, rules agents misread are proposed rewritten with the misreading quoted, and followed rules buried deep in the file are proposed moved up. Every proposal carries the observations it rests on as evidence, and nothing is applied on its own. The tool never writes to the files it reads.

![ci](https://github.com/david-engelmann/bluepencil/actions/workflows/ci.yml/badge.svg)

```json
{
  "proposal_version": "1.0",
  "target": {
    "sha256": "68737b58e096e689f132059ef8a4ef3350e3d746386995918f9fb3117f25133b"
  },
  "counting_method": "counts are observations per rule by kind, matched by exact rule text; a rule edited since an observation was taken is a new rule and the observation is unmatched; an observation naming a text carried by two rules attaches to the first, and the later rule is flagged a duplicate",
  "inventory": [
    {
      "id": "rule-1",
      "section": [
        "Team rules"
      ],
      "text": "Never force push to main",
      "line_span": [
        3,
        3
      ],
      "counts": {
        "misreading": 0,
        "ignored": 2,
        "followed": 0,
        "exercised": 2
      }
    },
    {
      "id": "rule-2",
      "section": [
        "Team rules"
      ],
      "text": "Sign every commit",
      "line_span": [
        4,
        4
      ],
      "counts": {
        "misreading": 0,
        "ignored": 0,
        "followed": 1,
        "exercised": 1
      }
    }
  ],
  "unmatched_observations": [],
  "warnings": [],
  "proposals": [
    {
      "kind": "delete",
      "rule_id": "rule-1",
      "rule_text": "Never force push to main",
      "evidence": [
        {
          "rule": "Never force push to main",
          "kind": "ignored",
          "quote": "force pushed after a rebase tangle"
        },
        {
          "rule": "Never force push to main",
          "kind": "ignored",
          "quote": "force pushed over a stale branch"
        }
      ],
      "hunk": {
        "action": "delete",
        "section": [
          "Team rules"
        ],
        "text": "Never force push to main"
      }
    }
  ]
}
```

## Install

```bash
git clone https://github.com/david-engelmann/bluepencil
cd bluepencil
cargo install --path .
```

The build needs a Rust toolchain and leaves a single `bluepencil` binary in the cargo bin directory.

## Quickstart

Write a small rules file.

```markdown
# Team rules

- Never force push to main
- Sign every commit
```

Write the observations gathered under it, one entry per observed act, each naming its rule by exact text and its kind as followed, ignored, or a misreading.

```json
{"observations":[{"kind":"ignored","quote":"force pushed after a rebase tangle","rule":"Never force push to main"},{"kind":"ignored","quote":"force pushed over a stale branch","rule":"Never force push to main"},{"kind":"followed","quote":"commit created and signed","rule":"Sign every commit"}],"rewrites":[]}
```

Run propose to write the proposal document, then diff to read it rendered as edits.

```bash
bluepencil propose rules.md observations.json -o proposals.json
bluepencil diff rules.md proposals.json
```

```text
bluepencil diff, 1 proposals (delete 1, rewrite 0, promote 0)
target sha256 68737b58e096e689f132059ef8a4ef3350e3d746386995918f9fb3117f25133b

## delete rule-1 (Team rules)
- - Never force push to main
  evidence, ignored, "force pushed after a rebase tangle"
  evidence, ignored, "force pushed over a stale branch"
```

The rule ignored twice is proposed for deletion with both observations quoted, and the followed rule earns no proposal.

## Usage

The crate ships synthetic sample inputs under `tests/fixtures`. Propose over them and write the document to a file.

```bash
bluepencil propose tests/fixtures/rules_synthetic.md tests/fixtures/observations_synthetic.json -o proposals.json
```

The document carries the hash of the rules file it targets, the full inventory of rules with their counts, and the proposals with the observations each one rests on. The first screen of this page shows a complete document from the Quickstart run. This run over the synthetic fixtures writes a larger document of the same shape, and diff renders it as a readable list of edits.

```bash
bluepencil diff tests/fixtures/rules_synthetic.md proposals.json
```

```text
bluepencil diff, 3 proposals (delete 1, rewrite 1, promote 1)
target sha256 1fc29c7a84fe5d0f7c6d066563dd4f53f5b646020303c318e786273b79df3450

## rewrite rule-1 (Setup rules)
- - Always run the local gate before pushing
+ - Run the local gate and see it pass before any push happens
  evidence, misreading, "agent ran the gate after pushing, reading before as eventually"

## delete rule-2 (Setup rules)
- - Never force push to main
  evidence, ignored, "session 12 force pushed after a rebase tangle"
  evidence, ignored, "session 19 force pushed over a stale branch"

## promote rule-7 (Setup rules / Style to Setup rules)
- - Keep functions short
  evidence, followed, "session 22 split the parser before review"
  evidence, followed, "session 23 kept the new module under review size"
```

The proposals follow fixed thresholds. A rule earns a delete proposal when at least two observations mark it ignored and none mark it followed. A rule earns a promote proposal when at least two observations mark it followed, the rule stands in the second half of the file by rule order, and its section differs from the section of the first rule. Below either threshold a rule earns no proposal, however the rest of its counts read, so a rule with a single ignored observation is left alone.

A misreading observation on its own never produces a rewrite. Rewrites come from the rewrites entries in the observations file, each one naming a rule by its exact text, quoting a misreading observation recorded against that rule, and giving the replacement text. An entry whose quote matches no misreading observation on its rule fails the run with a named error.

The output path may not name an input, in any spelling. Point it at the rules file and the run refuses and exits 1.

```bash
bluepencil propose tests/fixtures/rules_synthetic.md tests/fixtures/observations_synthetic.json -o tests/fixtures/rules_synthetic.md
```

```text
error, bluepencil never writes to its inputs
```

## Status

The tool is at its first working state. Both commands run, the input guard holds in every spelling the contract pins, and output writes land atomically beside their target under a drawn temp name. The behavior contract lives in CONTRACT.md and is pinned against the Python reference implementation by a differential harness.

Closed perfection rounds, 7 of 100.

## License

MIT. See the [LICENSE](LICENSE) file.
