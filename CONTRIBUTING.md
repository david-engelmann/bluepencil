# Contributing to bluepencil

Thank you for contributing to `bluepencil`. Because `bluepencil` sits between `unheard` observation harvesters and human code review, deterministic output and exact CLI refusals are strictly tested.

## Prerequisites

- Rust stable toolchain (`1.74` or newer) with `rustfmt` and `clippy`
- Python `3.12+` (used by `scripts/codepoint_scan.py`)

## Local Verification Gate

Run all four checks from the repository root before submitting changes:

```sh
python3 scripts/codepoint_scan.py
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

All checks must pass with zero warnings and zero test failures.

## Engineering Rules

1. **Zero em-dashes or en-dashes**: `scripts/codepoint_scan.py` scans `.md`, `.txt`, `.py`, `.rs`, `.toml`, and `.lock` files for `U+2014` and `U+2013`. Zero occurrences of either code point are permitted anywhere in the repository.
2. **Zero extra crate dependencies**: `sha2` is the sole external dependency. Do not introduce `serde`, `clap`, `regex`, `rand`, or sibling workspace crates.
3. **Behavioral contract**: `CONTRACT.md` defines the normative specification for Markdown rule extraction, observation matching, proposal generation, diff verification, `argparse` CLI compatibility, and atomic file output. Changes to observable CLI output or exit codes must be accompanied by contract review and integration tests under `tests/`.
4. **Never mutate inputs**: Any feature or fix must preserve the invariant that `bluepencil` only reads its positional inputs and never modifies them under any path alias.
