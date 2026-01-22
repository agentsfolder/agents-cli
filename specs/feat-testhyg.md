# feat-testhyg: CLI Test Harness Hygiene

Goal: Remove deprecated test helpers and keep CLI integration tests compatible with custom cargo build directories.

Depends on: feat-cliapp
Unblocks: CI stability

## Deliverables
- CLI integration tests use `cargo::cargo_bin_cmd` instead of deprecated `Command::cargo_bin`.
- Shared helper for spawning the `agents` binary in tests.

## Implementation Plan
- [x] Add shared test helper for `agents` command construction.
- [x] Replace deprecated `Command::cargo_bin` in CLI tests.
- [x] Verify CLI test suite runs without `cargo_bin` deprecation warnings.

## Verification
- [x] `cargo test -p agents-cli` passes.
