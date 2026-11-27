<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="media/lockup-dark.svg">
    <img src="media/lockup-light.svg" alt="bluepencil" width="780">
  </picture>
</p>

<p align="center">
  <a href="https://github.com/david-engelmann/bluepencil/actions/workflows/ci.yml"><img src="https://github.com/david-engelmann/bluepencil/actions/workflows/ci.yml/badge.svg" alt="ci"></a>
  <a href="https://github.com/david-engelmann/bluepencil/actions/workflows/clean-install.yml"><img src="https://github.com/david-engelmann/bluepencil/actions/workflows/clean-install.yml/badge.svg" alt="clean-install"></a>
  <a href="https://github.com/david-engelmann/bluepencil/actions/workflows/docs.yml"><img src="https://github.com/david-engelmann/bluepencil/actions/workflows/docs.yml/badge.svg" alt="docs"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-sky.svg" alt="MIT License"></a>
</p>

# bluepencil

An editor for rules files that works the way a copy editor works copy: strikes the rules no agent follows, rewrites the ones agents misread, and moves the buried ones to the top, receipts attached.

Give bluepencil a rules file and the observations gathered on the sessions that ran under it, and it answers with a proposal document. Rules the evidence shows ignored are proposed for deletion, supplied rewrites of rules agents misread are proposed with the misreading quoted, and followed rules buried deep in the file are proposed moved up. Every proposal carries the observations it rests on as evidence, and nothing is applied on its own. The tool never writes to the files it reads.

<p align="center">
  <img src="media/money-shot.gif" alt="bluepencil propose and diff CLI showcase" width="880">
</p>

## Install

### Prebuilt Binary (Linux & macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/david-engelmann/bluepencil/main/install.sh | sh
```

### Build from Source (Zero-Dependency Rust)

```bash
git clone https://github.com/david-engelmann/bluepencil
cd bluepencil
cargo install --path . --locked
```

> Note: Do not run bare `cargo install bluepencil` against crates.io until `bluepencil-rules` is published, as an unrelated crate holds the bare `bluepencil` name on crates.io.

## Quickstart

From a local checkout of this repository:

Run `propose` and `diff` directly against the shipped `examples/` fixtures:

```bash
bluepencil propose examples/rules.md examples/observations.json -o proposals.json
bluepencil diff examples/rules.md proposals.json
```

```text
bluepencil diff, 3 proposals (delete 1, rewrite 1, promote 1)
target sha256 76f1c52ad6df42cc79fd8a6845238eb7eb4a3e6fb57d76df9b08e370bd40ce92

## rewrite rule-2 (Repository rules)
- - Run the verification gate before opening a pull request
+ - Run the full verification gate and confirm it exits 0 before opening any draft or ready pull request
  evidence, misreading, "agent opened the draft pull request first and ran checks afterward"

## delete rule-1 (Repository rules)
- - Never commit directly to main
  evidence, ignored, "session 04 committed directly on local main during hotfix"
  evidence, ignored, "session 09 landed formatting commit straight on main"

## promote rule-3 (Repository rules / Architecture to Repository rules)
- - Keep error messages actionable and free of stack traces
  evidence, followed, "session 15 mapped parse failures to named stderr lines"
  evidence, followed, "session 18 returned exit code 1 with a descriptive refusal"
```

## Architecture

<p align="center">
  <img src="media/architecture.svg" alt="bluepencil threshold engine architecture" width="900">
</p>

## Usage & Safety Guarantees

- **Delete threshold**: `ignored >= 2` and `followed == 0`.
- **Rewrite verification**: Every rewrite entry must quote a recorded `misreading` observation on that exact rule, or the run exits `1`.
- **Promote threshold**: `followed >= 2`, rule stands in the second half of the rules inventory, and its section differs from the first rule's section.
- **Input write-refusal**: Pointing `-o` at any input file refuses and exits `1` (`error, bluepencil never writes to its inputs`). Atomic sibling tempfiles draw 16 bytes from the OS CSPRNG (`/dev/urandom` on Unix, `BCryptGenRandom` on Windows).

## Performance

<p align="center">
  <img src="media/benchmark-chart.svg" alt="benchmark profile" width="900">
</p>

Measured on Apple M3 Max (`--release`, `738.1 KiB` zero-dep binary; details in [docs/PERFORMANCE.md](PERFORMANCE.md)):

| Rules in `rules.md` | Session Observations | `propose` Latency (`-o`) | `diff` Latency | Observation Throughput (`propose`) |
| ---: | ---: | ---: | ---: | ---: |
| `25` | `250` | `12.48 ms` | `10.15 ms` | `20,032 obs/s` |
| `100` | `1,500` | `26.78 ms` | `18.94 ms` | `56,012 obs/s` |
| `500` | `10,000` | `147.08 ms` | `203.77 ms` | `67,990 obs/s` |

## Where `bluepencil` Fits in the Retrospective Line

<p align="center">
  <img src="media/retrospective-arc.svg" alt="Retrospective Line ten-repository architecture" width="900">
</p>

## Documentation

- [Documentation Index](README.md) & [Searchable Handbook](https://david-engelmann.github.io/bluepencil/)
- [Operator & Integrator Guide](GUIDE.md)
- [Technical Architecture](ARCHITECTURE.md)
- [Wire Formats & JSON Schemas](FORMATS.md)
- [Ecosystem Comparison](COMPARISON.md)
- [Performance Benchmarks](PERFORMANCE.md)
- [FAQ](FAQ.md)

## Status

The tool is at its first working state. Both commands run, the input guard holds in every spelling the contract pins, and output writes land atomically beside their target under a drawn temp name. The behavior contract lives in CONTRACT.md and is pinned against the Python reference implementation by a differential harness.

Closed perfection rounds, 47 of 100.

## License

MIT. See the [LICENSE](../LICENSE) file.
