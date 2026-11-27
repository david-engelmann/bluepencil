# Security Policy

## Supported Versions

| Version | Supported |
| --- | --- |
| 0.1.x | Yes |

## Reporting a Vulnerability

Do not open a public GitHub issue for suspected security vulnerabilities.

Report vulnerabilities privately through [GitHub Security Advisories](https://github.com/david-engelmann/bluepencil/security/advisories/new) on `david-engelmann/bluepencil`. Include:

1. Affected `bluepencil` version or commit SHA and operating system.
2. Minimal reproduction steps and synthetic Markdown/JSON inputs demonstrating the issue.
3. Expected behavior versus observed behavior and security impact.

Reports receive an initial acknowledgment within 72 hours. Validated fixes are released alongside a coordinated advisory.

## Security Model and Guarantees

- **Strict read-only input discipline**: `bluepencil` never edits rules files, observations files, or proposal files in place. Before reading inputs, the `-o` / `--output` guard resolves canonical prefixes, dangling symlink chains, and Unix device/inode hardlink identities, refusing any output spelling that aliases an input or passes through a symlink loop (`error, bluepencil never writes to its inputs`).
- **Unpredictable atomic output writes**: When `-o` is used, `bluepencil` creates an exclusive sibling temporary file whose 8-hex-character suffix mixes 16 bytes of operating-system cryptographic randomness with process, timestamp, counter, and path inputs through SHA-256. If OS randomness is unavailable, `bluepencil` refuses the file write rather than falling back to a predictable temporary filename.
- **Bounded untrusted JSON parsing**: Custom JSON parsers enforce container nesting limits and 4,300-digit integer bounds so adversarial JSON inputs exit cleanly without stack exhaustion or unbounded allocation.
- **Registry name warning**: On `crates.io`, the bare crate name `bluepencil` belongs to an unrelated third-party project. Always install `david-engelmann/bluepencil` via `install.sh` (which verifies `SHA256SUMS` against official GitHub Releases) or `cargo install --git https://github.com/david-engelmann/bluepencil --locked`.
