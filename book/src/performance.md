# Bluepencil Performance & Scale Benchmarks

<p align="center">
  <img src="media/benchmark-chart.svg" alt="bluepencil benchmark profile" width="920">
</p>

Measured from a thin-LTO symbol-stripped `--release` build (`564.5 KiB` binary, `O(R + O)` `HashMap` rule-join engine) on **Apple M3 Max** (macOS `14.6`, `rustc 1.99.1 (58cf8f9c3 2026-10-13) (Homebrew)`), median of 5 runs after warmup.

## Synthetic Scale Corpus Results

| Rules in `rules.md` | Session Observations | `propose` Latency (`-o`) | `diff` Latency | Observation Throughput (`propose`) |
| ---: | ---: | ---: | ---: | ---: |
| `25` | `250` | `12.48 ms` | `10.15 ms` | `20,032 obs/s` |
| `100` | `1,500` | `26.78 ms` | `18.94 ms` | `56,012 obs/s` |
| `500` | `10,000` | `147.08 ms` | `203.77 ms` | `67,990 obs/s` |
