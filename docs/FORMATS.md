# Bluepencil Wire Formats & Schemas

## 1. Observations Input (`schema/observations.schema.json`)

Consumed by `bluepencil propose <rules.md> <observations.json>` (typically produced upstream by `unheard` or review harnesses):

```json
{
  "observations": [
    {
      "rule": "Never commit directly to main",
      "kind": "ignored",
      "quote": "session 04 committed directly on local main during hotfix"
    },
    {
      "rule": "Run the verification gate before opening a pull request",
      "kind": "misreading",
      "quote": "agent opened the draft pull request first and ran checks afterward"
    }
  ],
  "rewrites": [
    {
      "rule": "Run the verification gate before opening a pull request",
      "misreading_quote": "agent opened the draft pull request first and ran checks afterward",
      "replacement": "Run the full verification gate and confirm it exits 0 before opening any draft or ready pull request"
    }
  ]
}
```

## 2. Proposal Output (`proposal_version` `"1.0"`, `schema/proposal.schema.json`)

Emitted by `bluepencil propose` to `stdout` or `-o <path>`:

- `proposal_version`: `"1.0"`
- `target.sha256`: 64-hex SHA-256 digest of the target rules file
- `counting_method`: Self-documenting contract string stating exact matching and duplicate rules
- `inventory[]`: Every extracted rule (`id`, `section[]`, `text`, `line_span`, `counts.{misreading,ignored,followed,exercised}`)
- `unmatched_observations[]` and `warnings[]`
- `proposals[]`: Ordered editorial hunks (`kind` in `delete | rewrite | promote`, `rule_id`, `rule_text`, `evidence[]`, `hunk`)

## 3. Human-Readable Editorial Diff (`bluepencil diff <rules.md> <proposal.json>`)

Verifies `target.sha256` against `<rules.md>` and renders a reviewable Markdown-style diff header, `-` / `+` bullet lines, and indented `evidence, <kind>, "<quote>"` receipts.

## 4. Exit Codes

- `0`: Proposal or diff succeeded cleanly.
- `1`: Domain error (`target.sha256` mismatch, unmatched `misreading_quote` in `rewrites`, malformed JSON, or `-o` pointing at an input file).
- `2`: CLI argument/usage error.
