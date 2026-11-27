# Changelog

All notable changes to `bluepencil` are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-11-27

### Added

- Evidence-driven rules file analyzer and proposal engine (`bluepencil propose` and `bluepencil diff`) operating over Markdown instruction files (`CLAUDE.md`, `AGENTS.md`, team guides) and session observation receipts.
- Deterministic Markdown rule inventory parser supporting ATX and Setext headings, bulleted and numbered rules, task-list checkbox stripping (`[ ]`, `[x]`, `[X]`), indented multi-line continuations, fenced code blocks, HTML comments, and duplicate-rule detection.
- Versioned observations loader (`observations_version` `"1.0"`) compatible with `unheard` outputs, validating `followed`, `ignored`, and `misreading` observations plus operator-supplied `rewrites`.
- Target-pinned proposal generator (`proposal_version` `"1.0"`) enforcing SHA-256 target integrity, hunk consistency checks, and deterministic thresholds for `delete`, `rewrite`, and `promote` proposals.
- Read-only input safety guarantees: `bluepencil` never modifies input files, refuses aliased or hardlinked `-o` paths, and writes output files atomically via OS-entropy sibling temporary files.
- JSON Schemas (`schema/observations.schema.json`, `schema/proposal.schema.json`), standalone synthetic examples (`examples/`), and operator guide (`docs/GUIDE.md`).
