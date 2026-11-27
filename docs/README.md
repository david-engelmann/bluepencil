# Bluepencil Documentation

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="media/lockup-dark.svg">
    <img src="media/lockup-light.svg" alt="bluepencil" width="760">
  </picture>
</p>

Welcome to the documentation suite for `bluepencil`, the evidence-backed rules file editor (`AGENTS.md`, `CLAUDE.md`, `.cursorrules`) in the Retrospective Line.

## Documentation Index

| Document | Audience | Contents |
| :--- | :--- | :--- |
| [Operator & Integrator Guide](GUIDE.md) | Prompt & platform engineers | Installation, `propose` and `diff` workflows, threshold rules, input write-guard semantics |
| [Technical Architecture](ARCHITECTURE.md) | Systems engineers & contributors | Markdown rule extraction, midpoint promotion math, duplicate handling, OS CSPRNG atomic writes |
| [Wire Formats & Schemas](FORMATS.md) | Upstream/downstream integrators | `observations.json` (`unheard` output), `proposal.json` (`1.0`), `diff` rendering grammar, exit codes |
| [Ecosystem Comparison](COMPARISON.md) | Evaluators & leads | How `bluepencil` differs from LLM prompt optimizers (DSPy/TextGrad), static linters, and `retroforge` |
| [Performance & Scale](PERFORMANCE.md) | Infrastructure teams | Measured benchmarks up to `500` rules x `10,000` observations (`54,466` obs/sec on Apple M3 Max) |
| [Frequently Asked Questions](FAQ.md) | All users | Crates.io naming note (`bluepencil` collision), why edits are never auto-applied, threshold rationale |
| [Storefront README Draft](STOREFRONT_README_DRAFT.md) | Maintainers & Muse seat | Drop-in visual storefront upgrade preserving all contracted `README.md` pins and `47 of 100` rounds line |

## Machine-Readable Contracts

- Observations Schema: [`../schema/observations.schema.json`](../schema/observations.schema.json)
- Proposal Schema: [`../schema/proposal.schema.json`](../schema/proposal.schema.json)
- Runnable Examples: [`../examples/rules.md`](../examples/rules.md) & [`../examples/observations.json`](../examples/observations.json)
