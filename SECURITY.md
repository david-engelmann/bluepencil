# Security Policy

## Supported Versions

Security fixes land on `main` and the latest tagged release.

## Security Boundaries

1. **Input write-refusal guard**: `bluepencil` checks `-o` against every input file (`rules.md`, `observations.json`, `proposal.json`) and exits `1` rather than mutating an input.
2. **Unpredictable sibling tempfiles**: Output file writes draw 16 bytes from the OS randomness source (`/dev/urandom` on Unix; `BCryptGenRandom` in cross-platform builds) and refuse `-o` if OS entropy is unavailable.
3. **Structured JSON bounds**: The custom JSON reader enforces container-nesting and integer-digit limits without executing code or making network calls (callers handling multi-gigabyte untrusted inputs should enforce outer file-size quotas at the OS/CI boundary).

## Reporting a Vulnerability

Report security vulnerabilities privately via [GitHub Security Advisories](https://github.com/david-engelmann/bluepencil/security/advisories/new).
