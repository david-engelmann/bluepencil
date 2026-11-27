# Bluepencil Performance & Scale Benchmarks

Measured from a zero-dependency `--release` build (`738.1 KiB` binary) on **Apple M3 Max** (macOS `14.6`, `rustc 1.99.1 (58cf8f9c3 2026-10-13) (Homebrew)`), median of 5 runs after warmup.

## Synthetic Scale Corpus Results

| Rules in `rules.md` | Session Observations | `propose` Latency (`-o`) | `diff` Latency | Observation Throughput (`propose`) |
| ---: | ---: | ---: | ---: | ---: |
| `25` | `250` | `27.86 ms` | `19.17 ms` | `8,973 obs/s` |
| `100` | `1,500` | `27.54 ms` | `23.61 ms` | `54,466 obs/s` |
| `500` | `10,000` | `286.7 ms` | `128.96 ms` | `34,880 obs/s` |

Even an enterprise-scale `500`-rule handbook evaluated against `10,000` recorded session observations completes end-to-end (`propose` + atomic file write + `diff` verification) in **`415.66 ms`**.
