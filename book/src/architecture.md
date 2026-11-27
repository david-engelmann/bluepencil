# Bluepencil Technical Architecture

<p align="center">
  <img src="media/architecture.svg" alt="bluepencil technical architecture" width="920">
</p>

Agent instruction files (`AGENTS.md`, `CLAUDE.md`) rot in predictable ways: teams append rules after every incident, never delete rules agents systematically ignore, leave ambiguous wording that agents misread, and bury high-leverage rules at the bottom of 500-line files where attention degrades. `bluepencil` turns empirical session observations into an auditable editorial diff without ever touching an input file.

## 1. Zero-Dependency Pipeline

`bluepencil` has **zero runtime crate dependencies** (`Cargo.toml` carries an empty `[dependencies]` table; SHA-256, JSON parsing/serialization, Markdown heading/list scanning, and OS CSPRNG tempfile generation are implemented directly in standard Rust).

1. **Markdown Inventory & Target Hash (`rules.md`)**:
   - Computes the exact SHA-256 hex digest of the raw `rules.md` bytes (`target.sha256`) so `bluepencil diff` refuses stale proposals if `rules.md` changed by even a single byte.
   - Walks Markdown headings (`#`, `##`, `###`, ...) into a section path stack and extracts bullet rules in document order (`rule-1` .. `rule-N`) with exact 1-based `[start_line, end_line]` spans.
2. **Exact-Text Observation Matching**:
   - Matches each observation (`followed`, `ignored`, `misreading`) by **exact rule text**. If a rule was edited since an observation was recorded, that observation lands transparently in `unmatched_observations[]` rather than silently attaching to the wrong rule.
   - Duplicate rule texts attach observations to the first occurrence and emit an explicit duplicate warning in `warnings[]`.
3. **Fixed Editorial Thresholds**:
   - **`delete`**: `ignored >= 2` AND `followed == 0`. A single ignored act never triggers deletion; a rule that is sometimes followed and sometimes ignored is a compliance problem, not dead weight.
   - **`rewrite`**: Never synthesized by `bluepencil` on its own. Emitted only when `rewrites[]` supplies a replacement whose `misreading_quote` matches an actual `misreading` observation recorded against that exact rule (otherwise `propose` exits `1` with a named error).
   - **`promote`**: `followed >= 2` AND the rule sits in the **second half** of the rule inventory (`index >= N / 2`) AND its section path differs from `rule-1`'s section path.
4. **Input Write-Refusal Guard & CSPRNG Atomic Output**:
   - `-o` is checked against all input files by path and filesystem identity (`dev`/`ino` on Unix, Win32 file handles on Windows); attempting `-o rules.md` exits `1` with `error, bluepencil never writes to its inputs`.
   - File writes draw 16 cryptographic random bytes from `/dev/urandom` (Unix) or `BCryptGenRandom` (Windows) for a sibling temporary filename and rename atomically into place.

## 2. Position in the Retrospective Line

<p align="center">
  <img src="media/retrospective-arc.svg" alt="bluepencil in the Retrospective Line" width="920">
</p>
