# feat-prodci: Production CI Hardening

Goal: Add CI checks for formatting/clippy/tests and enforce warning-free builds in production crates.

Depends on: feat-cliapp
Unblocks: production readiness

## Implementation Plan
- [x] Update repo workflow guidance to require adding new specs to `specs/plan.md` and the AGENTS spec index.
- [x] Add `feat-prodci` to `specs/plan.md` and the AGENTS spec index.
- [x] Add CI workflow with `cargo fmt -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`.
- [x] Enforce `#![deny(warnings)]` for `agents-core` and `agents-cli` crates.

## Verification
- [x] `cargo fmt -- --check` passes.
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [x] `cargo test` passes.
