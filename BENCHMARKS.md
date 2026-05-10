# Benchmarks

These numbers are reproducible artifacts, not headline marketing. Quote them
only with the environment block alongside, or rerun the bench on your own
hardware.

## Methodology

- Harness: [`criterion`](https://crates.io/crates/criterion) 0.5, 100 samples
  per case, 3 s warmup, 5 s measurement window (criterion defaults).
- Workload: `derive(Domain::User, &[0xAB; N])` for `N ∈ {16, 64, 256, 1024, 65536}`,
  plus `Did::to_string()` (the `did:agid:<base58>` formatter).
- Profile: `cargo bench --bench throughput` (release, default codegen).
- All inputs and outputs are wrapped in `criterion::black_box` to defeat
  constant folding.
- Reported intervals are criterion's lower / point / upper estimates
  (typically ~95 % CI on the median).

## Environment

| Field | Value |
|---|---|
| Date (UTC) | 2026-05-06T07:06Z (pre-rename, protocol-prefix-equivalent measurement) |
| Crate version | `ag_id 0.1.0` (formerly `determin-id 0.1.0`) |
| Dependency | `blake3 1.8.5`, default features (SIMD enabled) |
| CPU | AMD Ryzen 9 7900X, 12 cores / 24 threads |
| CPU SIMD available | SSE4.1, SSE4.2, AVX2, AVX-512F, BMI2 |
| OS / kernel | Linux 6.17.0-23-generic, x86\_64 |
| `rustc` | 1.94.0 (4a4ef493e 2026-03-02), host `x86_64-unknown-linux-gnu` |
| Profile | `release` (codegen-units default, no `RUSTFLAGS`) |

The CPU was not pinned, frequency scaling was left at default, and the
machine was running a desktop session. Numbers will be tighter on a
quiesced host with `cpupower frequency-set --governor performance`.

## Results

### `derive` — input bytes → 32-byte hash

| Input size | Median time | Throughput |
|---:|---:|---:|
| 16 B    | 67.21 ns | 227.0 MiB/s |
| 64 B    | 110.29 ns | 553.4 MiB/s |
| 256 B   | 241.82 ns | 1.01 GiB/s |
| 1024 B  | 833.44 ns | 1.14 GiB/s |
| 65536 B | 9.96 µs   | 6.13 GiB/s |

Asymptote is BLAKE3's chunked SIMD path; small-input cost is dominated by
hasher init and the 8-byte prefix update.

### `display/did_string` — `Did → "did:agid:<base58>"`

| Operation | Median time |
|---:|---:|
| `Did::to_string()` (≤44-char base58 + `did:agid:` prefix) | 594.60 ns |

The base58 encoder is the dominant cost — it does ~170 modulo-58 operations
on a 44-digit accumulator. If you only need byte equality (`Did::eq`,
`as_bytes`, hex), do not pay for the DID string.

## Reproducing

```sh
cargo bench --bench throughput
# HTML report: target/criterion/report/index.html
```

To compare against this baseline:

```sh
git checkout e126a1009599967293aa5f1e7101a255b1ad7f60
cargo bench --bench throughput -- --save-baseline reference
git checkout your-branch
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
