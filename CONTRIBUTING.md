# Contributing

Thanks for helping. Small, focused pull requests are easiest to review.

## Setup

```bash
git clone https://github.com/Alimzy/stellar-vesting && cd stellar-vesting
make check
```

`make check` runs `cargo fmt --check`, `cargo clippy -D warnings` and all tests. CI runs the
same checks plus the WASM build, so a green `make check` is the bar for a pull request.

## Workflow

1. Pick an issue and comment so work is not duplicated.
2. Branch from `main`: `git switch -c feat/short-name`.
3. Make the change, with tests. Money-affecting changes need a test that would fail without them.
4. Update `FEATURE-STATUS.md` and `CHANGELOG.md` when behaviour changes.
5. Open a PR using the template and link the issue (`Closes #N`).

## Issue complexity labels

| Label | Meaning |
|---|---|
| `trivial` | A few lines or docs; under an hour |
| `medium` | A self-contained change with tests; a few hours |
| `high` | Touches contract logic, security, or needs design discussion first |

## Code standards

- No `unsafe`, no `unwrap()` or `expect()` in contract code; return an `Error`.
- Arithmetic on amounts uses checked operations.
- State is written before any token call.
- Every state-changing entrypoint extends instance and schedule TTL.
- Error codes are ABI: never renumber, only append.

## WASM size budget

The optimized release build (`target/wasm32v1-none/release/vesting.wasm`) has a size budget of **23,000 bytes** (current build is ~18.8 KB, leaving ~20% headroom). CI enforces this limit on every PR.

## Commit style

Short imperative subject, for example `Add event payload snapshot tests`.
