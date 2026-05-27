# Benchmarks

These numbers are reproducible artifacts, not headline marketing. Quote them
only with the environment block alongside, or rerun the bench on your own
hardware.

## Methodology

- Harness: [`criterion`](https://crates.io/crates/criterion) 0.5, 100 samples
  per case, 3 s warmup, 5 s measurement window (criterion defaults).
- Workload: `derive(DeriveDomain::User, &[0xAB; N])` for `N ∈ {16, 64, 256, 1024, 65536}`,
  plus `Did::to_string()` (the `did:agid:<base58>` formatter).
- Profile: `cargo bench --bench throughput` (release, default codegen).
- All inputs and outputs are wrapped in `criterion::black_box` to defeat
  constant folding.
- Reported intervals are criterion's lower / point / upper estimates
  (typically ~95 % CI on the median).

## Environment

| Field | Value |
|---|---|
| Date (UTC) | 2026-05-27T05:01Z |
| Crate version | `ag_id 0.1.0` |
| Dependency | `blake3 1.8.5`, default features (SIMD enabled) |
| CPU | Apple M2, 8 cores |
| CPU SIMD available | NEON (Apple Silicon — no AVX) |
| OS / kernel | Darwin 25.5.0, arm64 (macOS 26.5) |
| `rustc` | 1.95.0 (59807616e 2026-04-14), host `aarch64-apple-darwin` |
| Profile | `release` (codegen-units default, no `RUSTFLAGS`) |

The CPU was not pinned, frequency scaling was left at default, and the
machine was running a desktop session. Numbers will be tighter on a
quiesced host with `cpupower frequency-set --governor performance`.

## Results

### `derive` — input bytes → 32-byte hash

| Input size | Median time | Throughput |
|---:|---:|---:|
| 16 B    | 91.59 ns | 166.60 MiB/s |
| 64 B    | 162.97 ns | 374.51 MiB/s |
| 256 B   | 374.76 ns | 651.46 MiB/s |
| 1024 B  | 1.30 µs | 749.80 MiB/s |
| 65536 B | 37.32 µs | 1.64 GiB/s |

Asymptote is BLAKE3's chunked SIMD path (NEON on Apple Silicon); small-input
cost is dominated by hasher init and the 8-byte prefix update. AVX-512
hardware shows higher asymptotic throughput; see footnote on hardware spread.

### `display/did_string` — `Did → "did:agid:<base58>"`

| Operation | Median time |
|---:|---:|
| `Did::to_string()` (≤44-char base58 + `did:agid:` prefix) | 1.25 µs |

The base58 encoder is the dominant cost — it does ~170 modulo-58 operations
on a 44-digit accumulator (~2× longer here than on AVX-512 hosts). If you only need byte equality (`Did::eq`,
`as_bytes`, hex), do not pay for the DID string.

## Reproducing

```sh
cargo bench --bench throughput
# HTML report: target/criterion/report/index.html
```

To compare against this baseline:

```sh
# To pin a baseline for comparison across changes:
cargo bench --bench throughput -- --save-baseline reference
# (then later)
cargo bench --bench throughput -- --baseline reference
```

## Interpretation guidance

- **Do not quote sub-microsecond numbers without an environment block.**
  A 10× swing between hardware classes is normal; AVX-512 vs scalar BLAKE3
  alone explains a ~3× spread.
- **Throughput converges at ~1 KiB.** Below that, hasher init dominates and
  per-byte numbers are misleading.
- **DID-string formatting is ~7× the cost of a 16-byte derive.** If you
  cache derived IDs in raw form and only stringify on display, the hot path
  cost is what `derive/N` reports, not what `display/did_string` reports.
