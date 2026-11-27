<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="media/lockup-dark.svg">
    <img src="media/lockup-light.svg" alt="bluepencil" width="760">
  </picture>
</p>

<p align="center">
  <img src="media/money-shot.gif" alt="bluepencil propose and diff showcase" width="880">
</p>

<p align="center">
  <img src="media/architecture.svg" alt="bluepencil threshold engine architecture" width="900">
</p>

# Bluepencil Operator Manual

Bluepencil is an evidence-driven copy editor for AI agent instruction files (`CLAUDE.md`, `AGENTS.md`, `.cursorrules`, and team Markdown guides). Given a Markdown rules file and a JSON file of session observations (`followed`, `ignored`, `misreading`) plus candidate `rewrites`, `bluepencil` inventories every rule, computes evidence-backed `delete`, `rewrite`, and `promote` proposals (`bluepencil propose`), and renders a human-reviewable edit summary (`bluepencil diff`) without ever modifying its input files.

Within the broader agent-quality portfolio, `bluepencil` judges the instructions whereas `stickler` judges the code, and `unheard` harvests the `observations_version` `"1.0"` input from `flighttape` session corpora.

## Contents

1. [Installation (Avoiding the crates.io Collision)](#1-installation-avoiding-the-cratesio-collision)
2. [Quick Walkthrough with `examples/`](#2-quick-walkthrough-with-examples)
3. [CLI Reference (`propose` and `diff`)](#3-cli-reference-propose-and-diff)
4. [Markdown Rule Extraction Grammar](#4-markdown-rule-extraction-grammar)
5. [Observations Format (`1.0`) and `unheard` Integration](#5-observations-format-10-and-unheard-integration)
6. [Proposal Thresholds, Ordering, and Hunk Verification](#6-proposal-thresholds-ordering-and-hunk-verification)
7. [UTF-8 Byte Order Mark (BOM) Rules](#7-utf-8-byte-order-mark-bom-rules)
8. [CI Recipes](#8-ci-recipes)

---

## 1. Installation (Avoiding the crates.io Collision)

> **Important**: Do **not** run bare `cargo install bluepencil` from `crates.io`. The `bluepencil` package name on `crates.io` is occupied by an unrelated third-party prose/fiction linter. Always install from `david-engelmann/bluepencil` GitHub Releases (`install.sh`) or directly from the Git repository.

### Prebuilt Binary via `install.sh` (Recommended)

On Linux (`x86_64`, `aarch64`) and macOS (`x86_64`, `aarch64`), `install.sh` downloads the matching archive from `david-engelmann/bluepencil` GitHub Releases, verifies its SHA-256 checksum against `SHA256SUMS` (`sha256sum` or `shasum -a 256`), and installs `bluepencil` into `$HOME/.local/bin`:

```sh
curl -fsSL https://raw.githubusercontent.com/david-engelmann/bluepencil/main/install.sh | sh
```

Pin a specific release tag or custom install directory with environment variables:

```sh
curl -fsSL https://raw.githubusercontent.com/david-engelmann/bluepencil/main/install.sh | \
  BLUEPENCIL_VERSION=v0.1.0 BLUEPENCIL_INSTALL_DIR=/usr/local/bin sh
```

On Windows (`x86_64-pc-windows-msvc`), download `bluepencil-x86_64-pc-windows-msvc.zip` and `SHA256SUMS` from [GitHub Releases](https://github.com/david-engelmann/bluepencil/releases), verify the SHA-256 hash with `Get-FileHash -Algorithm SHA256`, and place `bluepencil.exe` on `PATH`.

### Build from Source with Cargo

```sh
cargo install --git https://github.com/david-engelmann/bluepencil --locked
```

Or from a local clone:

```sh
git clone https://github.com/david-engelmann/bluepencil.git
cd bluepencil
cargo install --path . --locked
```

---

## 2. Quick Walkthrough with `examples/`

The repository ships standalone sample inputs in `examples/rules.md` and `examples/observations.json`:

```sh
# 1. Generate a target-pinned proposal document
bluepencil propose examples/rules.md examples/observations.json -o proposals.json

# 2. Render a human-readable diff verified against examples/rules.md
bluepencil diff examples/rules.md proposals.json
```

`bluepencil diff` verifies that `proposals.json` matches the exact SHA-256 of `examples/rules.md`, cross-checks every proposal hunk against both the embedded inventory and freshly parsed rules, and prints:

```text
bluepencil diff, 3 proposals (delete 1, rewrite 1, promote 1)
target sha256 c6e6571273e93b6a525da154beb77f16bc709b4f08fd0b091593371374565938

## rewrite rule-1 (Repository rules)
- - Run the verification gate before opening a pull request
+ - Run the full verification gate and confirm it exits 0 before opening any draft or ready pull request
  evidence, misreading, "agent opened the draft pull request first and ran checks afterward"

## delete rule-2 (Repository rules)
- - Never commit directly to main
  evidence, ignored, "session 04 committed directly on local main during hotfix"
  evidence, ignored, "session 09 landed formatting commit straight on main"

## promote rule-7 (Repository rules / Architecture to Repository rules)
- - Keep error messages actionable and free of stack traces
  evidence, followed, "session 15 mapped parse failures to named stderr lines"
  evidence, followed, "session 18 returned exit code 1 with a descriptive refusal"
```

---

## 3. CLI Reference (`propose` and `diff`)

### Exit Codes

- `0`: Proposal JSON or diff text emitted cleanly (note: `propose` and `diff` exit `0` whether `proposals` is empty or non-empty).
- `1`: Operational refusal (missing or unreadable file, structural Markdown defect, invalid JSON/schema, unmatched rewrite quote, target SHA-256 mismatch, input-overwrite guard refusal, or output write failure). Named errors are printed to `stderr` prefixed with `error, `.
- `2`: Command-line usage error (`argparse`-compatible option and positional validation).

### `bluepencil propose`

```sh
bluepencil propose [-h] [-o OUTPUT] rules observations
```

- Reads `rules` (Markdown) and `observations` (JSON, schema at `schema/observations.schema.json`).
- Emits a `proposal_version` `"1.0"` JSON document (schema at `schema/proposal.schema.json`) to `stdout` when `-o` is omitted, or atomically to `OUTPUT` when `-o` / `--output` is given.

### `bluepencil diff`

```sh
bluepencil diff [-h] [-o OUTPUT] rules proposals
```

- Reads `rules` (Markdown) and `proposals` (JSON generated by `bluepencil propose`).
- Verifies `proposals.target.sha256` against the raw bytes of `rules`; if `rules` was edited after `proposals` was generated, `diff` exits `1` with `error, the proposal file names a different rules file`.
- Cross-checks every proposal against `inventory` and the freshly parsed `rules` list, then renders the human diff to `stdout` or `-o OUTPUT`.

### Input-Overwrite Protection and Atomic `-o` Writes

- Passing an `-o` path that resolves to either positional input (directly, via relative segments, via symlinks, via symlink loops, or via Unix hardlinks) is refused before any input is processed:
  ```text
  error, bluepencil never writes to its inputs
  ```
- When `-o OUTPUT` is used on Unix, `bluepencil` draws 16 bytes from `/dev/urandom`, mixes them through SHA-256 to pick an 8-hex-character sibling temp filename (`<name>.<hex8>`), writes with mode `0600`, and renames atomically onto `OUTPUT`. Note that on builds without an OS entropy implementation (unpatched non-Unix targets), `-o` refuses the write (`error, cannot write the proposal file, ...`), whereas omitting `-o` and redirecting `stdout` works everywhere.
- In `-o` file mode (`Mode::Strict`), unpaired UTF-16 surrogates in JSON strings fail the write cleanly (`exit 1`); on `stdout` (`Mode::Stdout`), CPython `surrogateescape` output rules apply.

---

## 4. Markdown Rule Extraction Grammar

Understanding how `bluepencil` parses Markdown instruction files prevents unexpected refusals on real-world `CLAUDE.md`, `AGENTS.md`, and `.cursorrules` files. Decisions are evaluated line-by-line in strict precedence order:

1. **Mandatory leading heading (`rule outside any section`)**:
   Every rule must belong to at least one heading section. **If a bullet or numbered item appears before the first heading in the file, `bluepencil` refuses the file and exits `1`**:
   ```text
   error, rule outside any section at line 1
   ```
   If your `.cursorrules` or `AGENTS.md` starts directly with `- ...` bullets, add a top-level `# Rules` heading on line 1 before running `bluepencil`.

2. **Headings and section paths**:
   - **ATX headings**: 1 to 6 `#` characters followed by whitespace and heading text (`# Top`, `## Sub`). Trailing `#` runs preceded by whitespace (`## Style ##`) are stripped from the section name.
   - **Setext headings**: A single line of prose immediately followed by a line of all `=` characters (level 1) or all `-` characters (level 2) becomes a section heading. Note that `---` after a blank line is **not** a Setext heading (and a single `-` alone on a line is treated as an empty bullet!).
   - Nested headings maintain a `section` path array on each rule, such as `["Repository rules", "Testing"]`.

3. **Rule markers and empty-bullet refusal**:
   - Unordered markers: `-`, `*`, or `+` (after any leading indentation) followed by whitespace and rule text.
   - Ordered markers: One or more Unicode decimal digits followed by `.` or `)` and whitespace (`1. `, `2) `).
   - **Empty markers fail the run**: A line containing only `-`, `*`, `+`, `1.`, or `1)` with no rule text after it exits `1` with `error, empty rule at line N` (watch out for a lone `-` horizontal separator; use `***` or `___` instead, or keep `---` only as a Setext underline under prose).

4. **Task-list checkbox stripping**:
   If a rule's text begins with `[ ] `, `[x] `, or `[X] ` (followed by whitespace and remaining text), the checkbox prefix is stripped from `rule.text` so observations match the clean rule text. When `bluepencil diff` renders the edit, it preserves the original source line in `- - [x] ...` and uses the list marker prefix (`- `) on `+ - ...`.

5. **Indented multi-line continuations**:
   While a rule is open, any subsequent non-blank line that is not a comment, fence, heading, or list marker and whose leading indentation (in Unicode code points) is **strictly greater** than the opening marker's indentation is appended to `rule.text` separated by `\n`, extending `line_span` to `[start_line, end_line]`. A blank line or unindented prose line closes the open rule.

6. **Fenced code blocks and HTML comments**:
   - **Fences**: A line beginning (after optional whitespace) with 3 or more backticks (`` ``` ``) or tildes (`~~~`) closes any open rule and ignores all lines until a closing fence of at least the same length using the same character (` ``` ` or `~~~`) with nothing else on the line. Bullets inside code examples are therefore never mistaken for rules. If a fence remains unclosed at EOF, `bluepencil` records `"unclosed fence at line N, the rest of the file was read as code"` in `warnings`.
   - **HTML comments**: Any line whose trimmed text starts with `<!--` closes any open rule and skips lines until `-->` is encountered (on the same line or a later line).

7. **Exact-text matching and duplicate rules**:
   - Rules receive sequential positional IDs: `rule-1`, `rule-2`, ..., `rule-N`.
   - Observations match rules by **exact `rule.text`** (including `\n` between continuation lines, without leading bullet/checkbox prefixes).
   - If a rule was edited since an observation was recorded, the old observation lands in `unmatched_observations`.
   - If two rules in the file have identical `rule.text`, observations attach to the **first** occurrence, and each subsequent occurrence carries `"duplicate_of": "rule-K"` in `inventory`.

---

## 5. Observations Format (`1.0`) and `unheard` Integration

In an automated fleet pipeline, `flighttape` converts raw agent sessions into canonical `1.0` tapes, `unheard` evaluates those tapes against your rules file to emit an `observations_version` `"1.0"` JSON document, and `bluepencil propose` turns those observations into concrete edits:

```text
session.jsonl -> [flighttape convert] -> run.tape.json -> [unheard] -> observations.json -> [bluepencil propose] -> proposals.json
```

### Schema Summary (`schema/observations.schema.json`)

```json
{
  "observations_version": "1.0",
  "observations": [
    {
      "rule": "Never force push to main",
      "kind": "ignored",
      "quote": "session 12 force pushed after a rebase tangle"
    },
    {
      "rule": "Always run the local gate before pushing",
      "kind": "misreading",
      "quote": "agent ran the gate after pushing, reading before as eventually"
    }
  ],
  "rewrites": [
    {
      "rule": "Always run the local gate before pushing",
      "misreading_quote": "agent ran the gate after pushing, reading before as eventually",
      "replacement": "Run the local gate and see it pass before any push happens"
    }
  ]
}
```

Validation rules enforced by `load_observations`:

- `observations_version`: Optional for pre-versioned inputs; when present, **must** be the exact string `"1.0"`. Any other string, number, or type exits `1` with `error, unsupported observations_version ...`.
- `observations`: Required JSON array. Every element must be an object with non-empty string `rule`, `kind` in `["misreading", "ignored", "followed"]`, and non-empty string `quote`.
- `rewrites`: Optional JSON array (defaults to `[]`). Every element must be an object with non-empty strings `rule`, `misreading_quote`, and `replacement`.
  - If `rewrite.rule` matches no rule in the rules file, `propose` exits `1` (`error, supplied rewrite names a rule that is not in the file: ...`).
  - If `rewrite.misreading_quote` does not match the exact `quote` of at least one `"misreading"` observation attached to that rule, `propose` exits `1` (`error, rewrite for rule-N quotes a misreading that is not among its observations`).

---

## 6. Proposal Thresholds, Ordering, and Hunk Verification

For each inventoried rule `rule-1 .. rule-N` (0-based `index` in `0 .. total`), `bluepencil propose` evaluates three deterministic rules in order (`delete`, then `rewrite` entries in document order, then `promote`):

| Proposal `kind` | Exact Trigger Condition | Hunk `action` & Shape |
| --- | --- | --- |
| `"delete"` | `counts.ignored >= 2` **and** `counts.followed == 0` (even a single `followed` observation blocks deletion). | `{"action": "delete", "section": [...], "text": "<rule_text>"}` |
| `"rewrite"` | First occurrence of `rule.text` has one or more `rewrites` entries whose `misreading_quote` matches a recorded `"misreading"` observation on that rule. | `{"action": "rewrite", "section": [...], "old": "<rule_text>", "new": "<replacement>"}` |
| `"promote"` | `2 * index >= total` (rule stands in the second half of the file by rule count), `counts.followed >= 2`, **and** `rule.section != rules[0].section` (not already in the opening section). | `{"action": "move", "from_section": [...], "to_section": "<first_rule_section>", "text": "<rule_text>"}` |

Every generated proposal embeds the filtered `evidence` array of observations that triggered it; `validate_proposals` and `diff` refuse any proposal with an empty `evidence` list or an unquoted observation.

---

## 7. UTF-8 Byte Order Mark (BOM) Rules

Input surfaces differ intentionally in how they treat a leading UTF-8 BOM (`0xEF 0xBB 0xBF`):

- **`rules.md` (`propose` and `diff`)**: Decoded with `utf-8-sig` semantics (one leading UTF-8 BOM is stripped before parsing headings and rules). However, `target.sha256` hashes the **raw on-disk bytes** of `rules.md` (including the BOM if present), so `diff` still detects if a BOM is added or removed later.
- **`proposals.json` (`diff`)**: Decoded with `utf-8-sig` semantics (one leading UTF-8 BOM is accepted).
- **`observations.json` (`propose`)**: Decoded as **strict UTF-8 without BOM stripping**. A leading UTF-8 BOM in `observations.json` is rejected with exit `1` (`Unexpected UTF-8 BOM (decode using utf-8-sig)`). On Windows PowerShell, avoid `Out-File -Encoding utf8` in Windows PowerShell 5.1 (which writes a BOM); use `[System.IO.File]::WriteAllBytes` or PowerShell 7 `utf8NoBOM`.

---

## 8. CI Recipes

Because `bluepencil propose` exits `0` when it successfully writes a proposal document (even when actionable proposals or unclosed-fence warnings are present), pair `bluepencil` with `jq` in GitHub Actions to fail pull requests when instruction drift or Markdown structural warnings are detected:

```yaml
name: rules-hygiene
on:
  pull_request:
permissions:
  contents: read
jobs:
  check-rules:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          persist-credentials: false
      - name: Install bluepencil
        run: ./install.sh
      - name: Run bluepencil propose and diff
        run: |
          export PATH="$HOME/.local/bin:$PATH"
          bluepencil propose CLAUDE.md observations.json -o /tmp/proposals.json
          bluepencil diff CLAUDE.md /tmp/proposals.json | tee -a "$GITHUB_STEP_SUMMARY"
          jq -e '(.warnings | length == 0) and (.proposals | length == 0)' /tmp/proposals.json >/dev/null
```

---

## Shell Completions

Static shell completions ship under `completions/` (and inside GitHub Release archives):

```sh
source completions/bluepencil.bash                 # Bash
fpath=(./completions $fpath) && compinit           # Zsh
cp completions/bluepencil.fish ~/.config/fish/completions/  # Fish
. ./completions/bluepencil.ps1                     # PowerShell
```

