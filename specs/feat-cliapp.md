# feat-cliapp: CLI App Skeleton

Goal: Provide a working `agents` binary with all v1 subcommands wired, consistent UX (help, exit codes), and structured diagnostics. No business logic beyond wiring and basic "repo not initialized" checks.

Depends on: (none)
Unblocks: feat-models, feat-initpr, feat-status

## Deliverables
- A buildable Rust workspace producing an `agents` binary.
- Subcommands from the PRD present in `--help`.
- Centralized error handling and exit codes.

## Implementation Plan
- [x] Create Cargo workspace and crates
  - [x] Create workspace `Cargo.toml` (root)
  - [x] Create binary crate `crates/agents-cli`
  - [x] Create library crate `crates/agents-core` (empty placeholders are fine)
  - [x] Ensure `cargo build` works on macOS/Linux/Windows

- [x] Add CLI argument parsing (clap)
  - [x] Define top-level `Cli` struct with global flags
    - [x] `--repo <path>` (optional; default: auto-discover)
    - [x] `--json` (optional; machine-readable output placeholder)
    - [x] `-v/--verbose` and `-q/--quiet` (mutually aware)
  - [x] Define `Commands` enum with v1 subcommands and args (stubs):
    - [x] `init [--preset <name>]`
    - [x] `validate [--profile <name>]`
    - [x] `status`
    - [x] `set-mode <mode> [--profile <profile>]`
    - [x] `preview [--agent <id>] [--backend ...] [--mode ...] [--profile ...]`
    - [x] `diff [--agent <id>]`
    - [x] `sync [--agent <id>] [--backend ...]`
    - [x] `run <agent> [--mode ...] [--profile ...] [--backend ...] [--] <passthrough...>`
    - [x] `doctor [--fix] [--ci]`
    - [x] `clean [--agent <id>]`
    - [x] `import --from <agent> [--path ...]`
    - [x] `explain <path>`
    - [x] `compat`
    - [x] `test adapters [--agent <id>]`
  - [x] Add enums for:
    - [x] `Backend` (`vfs_container`, `materialize`, `vfs_mount`) (string parsing + display)


- [ ] Establish app context plumbing
  - [ ] Create `AppContext` with:
    - [ ] resolved repo root path
    - [ ] output mode (human/json)
    - [ ] logger handle / verbosity
  - [ ] Add a single `dispatch(ctx, command) -> Result<()>` entrypoint
  - [ ] For now, each command handler can return `NotImplemented` or `NotInitialized` errors

- [ ] Structured errors and exit codes
  - [ ] Implement error type(s) with categories:
    - [ ] `NotInitialized` (missing `.agents/manifest.yaml`)
    - [ ] `InvalidArgs`
    - [ ] `Io`
    - [ ] `SchemaInvalid`
    - [ ] `Conflict` (collisions)
    - [ ] `PolicyDenied`
    - [ ] `ExternalToolMissing` (docker/agent binary)
  - [ ] Map categories to stable exit codes (document in code):
    - [ ] `0` success
    - [ ] `2` invalid args
    - [ ] `3` not initialized
    - [ ] `4` validation/schema error
    - [ ] `5` operational failure (io, external tool)
  - [ ] Ensure errors print:
    - [ ] one-line summary
    - [ ] optional context lines (path, schema, hint)

- [ ] Logging/tracing setup
  - [ ] Add `tracing` + `tracing_subscriber`
  - [ ] Honor `--verbose` and `RUST_LOG`
  - [ ] Keep default output clean (no debug logs unless enabled)

- [ ] Basic command UX and placeholders
  - [ ] Implement `agents --help` and `agents <cmd> --help` output sanity
  - [ ] Implement `agents validate` to:
    - [ ] locate repo root
    - [ ] detect missing `.agents/manifest.yaml`
    - [ ] return `NotInitialized` with a hint to run `agents init`
  - [ ] Ensure all other commands also fail with `NotInitialized` until feat-loadag is done

- [ ] Tests
  - [ ] Add a minimal CLI snapshot test (help text stable) OR a smoke test that command parsing works
  - [ ] Add unit tests for exit code mapping

## Verification
- [ ] `cargo test` passes
- [ ] `cargo run -p agents-cli -- --help` lists all v1 commands
- [ ] In an empty temp directory: `agents validate` exits with `NotInitialized` and a clear hint
