# Bluepencil Frequently Asked Questions

### Why does `cargo install bluepencil` on crates.io install a different tool?
An unrelated third-party prose linter registered the bare crate name `bluepencil` on `crates.io` earlier. Until the published crate package name (`bluepencil-rules`, keeping the `bluepencil` binary name) lands on `crates.io`, install prebuilt binaries via `install.sh` / GitHub Releases or from source with `cargo install --git https://github.com/david-engelmann/bluepencil`.

### Why did an observation land in `unmatched_observations`?
`bluepencil` matches observations to rules by **exact rule text**. Once a rule in `AGENTS.md` is edited, observations gathered against the old wording intentionally stop matching so stale failure counts from before the edit never penalize the new wording.

### Why wasn't a rule with one `ignored` observation proposed for deletion?
A single missed instruction in one session is noise. `bluepencil` requires at least **two** `ignored` observations and **zero** `followed` observations before proposing `delete`.

### What happens if `rules.md` is edited between `bluepencil propose` and `bluepencil diff`?
`proposal.json` pins `target.sha256`. `bluepencil diff` hashes `rules.md` first and exits `1` if the file changed, preventing out-of-date line numbers or hunks from being misread.
