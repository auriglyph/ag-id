# Publishing checklist

Run through this before every release. Keep it short, keep it real.

## Pre-flight

- [ ] `cargo fmt --check` is clean.
- [ ] `cargo clippy --release --all-features --all-targets -- -D warnings` is clean.
- [ ] `cargo test --release --all-features` is green.
- [ ] `cargo test --release --no-default-features --lib` is green (no_std).
- [ ] `cargo test --release --test vectors_json` is green (JSON ↔ implementation parity).
- [ ] `cargo bench --bench throughput` runs without panics. (Numbers don't have to be re-published every release; only update `BENCHMARKS.md` when the protocol or the Rust toolchain changes meaningfully.)
- [ ] `cargo build --release --examples` builds both `basic` and `parsing`.
- [ ] `cargo doc --release --all-features` produces no warnings.
- [ ] Working tree is clean: `git status` shows nothing.

## Version bump

- [ ] Decide the bump using semver. **Patch** for fixes that don't change output. **Minor** for new APIs / new `Domain` variants. **Major** only for wire-format changes (extremely rare).
- [ ] Update `Cargo.toml` `version`.
- [ ] Update `Cargo.lock`: `cargo update -p ag_id`.
- [ ] Update `CHANGELOG.md`:
  - Move `[Unreleased]` items into a new `[X.Y.Z] — YYYY-MM-DD` section.
  - Leave a fresh empty `[Unreleased]` block at the top.

## Tag

- [ ] `git add -A && git commit -m "release: vX.Y.Z"`.
- [ ] `git tag -a vX.Y.Z -m "ag_id vX.Y.Z"`.
- [ ] `git push origin main && git push origin vX.Y.Z`.

## Publish to crates.io

- [ ] Confirm logged in: `cargo login` (if needed) — token from <https://crates.io/me>.
- [ ] Dry-run: `cargo publish --dry-run`. Read the file list — it should include `src/`, `examples/`, `tests/`, `benches/`, `test-vectors/`, all `*.md`, `Cargo.toml`. It should NOT include `target/`, `.git/`, IDE config.
- [ ] Real publish: `cargo publish`.
- [ ] Watch <https://crates.io/crates/ag_id> for the new version to appear (≤2 minutes).
- [ ] Watch <https://docs.rs/ag_id> for the docs build (≤10 minutes).

## Post-publish

- [ ] Create a GitHub release at the tag, paste the `CHANGELOG.md` section as the description.
- [ ] If this is the first 1.0 release: announce in places that match the audience (rust subreddit, Lobsters, distributed-systems forums). Honest framing, no superlatives — let the spec and tests speak.
- [ ] If a vulnerability prompted the release: post a Security Advisory on GitHub referencing the affected versions and the fix commit.

## Yank policy

- A version is yanked only if it is materially broken (panics in safe API, produces different bytes than the spec, has a security defect). Yanking is reversible (`cargo yank --undo`) but should be rare.
- Never yank a version because a successor exists. Cargo handles that already.

## Rollback

If `cargo publish` succeeds but the published artefact is broken:

1. `cargo yank --version X.Y.Z`
2. Fix on `main`. Bump to `X.Y.(Z+1)` and re-run this checklist.
3. Update `CHANGELOG.md` to note the yank and the replacement version.

There is no `cargo unpublish`. Plan accordingly.
