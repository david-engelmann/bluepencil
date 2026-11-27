# Ecosystem Comparison

| Dimension | `bluepencil` | Automated Prompt Optimizers (DSPy, TextGrad, OPRO) | Static Markdown Linters (`markdownlint`, `vale`) | `retroforge` (Sibling Tool) |
| :--- | :--- | :--- | :--- | :--- |
| **Core mechanism** | Deterministic threshold rules over empirical session receipts | Stochastic LLM rewriting loops optimizing a benchmark metric | Syntax, spelling, and prose style linting | Multi-tape trace mining proposing rules, hooks, and roster hunks |
| **Hallucination risk** | Zero (never invents rule text; rewrites require caller-supplied wording + verified quote) | High (black-box prompt mutations can drop critical safety invariants) | Zero (syntactic only) | Trace-anchored hunk proposals |
| **Auto-mutation** | Never writes to input files; human reviews `diff` | Mutates prompt candidates automatically | `--fix` mutates formatting in place | Emits proposal file only |
| **Dependencies** | 1 Rust crate (`sha2` only, zero network/SDKs) | Python + LLM API keys | Node.js or Go | Retrospective Line Rust toolchain |

## Why `bluepencil` Never Invents Rewrites Automatically

Letting a language model silently rewrite `AGENTS.md` replaces human-authored engineering boundaries with unreviewed prose. `bluepencil` enforces a clean separation: `unheard` and session reviewers gather `followed` / `ignored` / `misreading` receipts, an engineer or upstream proposer supplies candidate `replacement` text tied to a specific `misreading_quote`, `bluepencil` verifies every receipt and produces the diff, and `secondtake` tests whether the edited rules file actually improved agent behavior.
