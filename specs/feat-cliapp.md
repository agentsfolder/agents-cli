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


- [x] Establish app context plumbing
  - [x] Create `AppContext` with:
    - [x] resolved repo root path
    - [x] output mode (human/json)
    - [x] verbosity flags (verbose/quiet)
  - [x] Add a single `dispatch(ctx, command) -> Result<()>` entrypoint
  - [x] For now, each command handler can return `NotImplemented` or `NotInitialized` errors

- [x] Structured errors and exit codes
  - [x] Implement error type(s) with categories:
    - [x] `NotInitialized` (missing `.agents/manifest.yaml`)
    - [x] `InvalidArgs`
    - [x] `Io`
    - [x] `SchemaInvalid`
    - [x] `Conflict` (collisions)
    - [x] `PolicyDenied`
    - [x] `ExternalToolMissing` (docker/agent binary)
  - [x] Map categories to stable exit codes (document in code):
    - [x] `0` success
    - [x] `2` invalid args
    - [x] `3` not initialized
    - [x] `4` validation/schema error
    - [x] `5` operational failure (io, external tool)
  - [x] Ensure errors print:
    - [x] one-line summary
    - [x] optional context lines (path, schema, hint)

- [x] Logging/tracing setup
  - [x] Add `tracing` + `tracing_subscriber`
  - [x] Honor `--verbose` and `RUST_LOG`
  - [x] Keep default output clean (no debug logs unless enabled)

- [x] Basic command UX and placeholders
  - [x] Implement `agents --help` and `agents <cmd> --help` output sanity
  - [x] Implement `agents validate` to:
    - [x] locate repo root
    - [x] detect missing `.agents/manifest.yaml`
    - [x] return `NotInitialized` with a hint to run `agents init`
  - [x] Ensure all other commands also fail with `NotInitialized` until feat-loadag is done

- [x] Tests
  - [x] Add a minimal CLI snapshot test (help text stable) OR a smoke test that command parsing works
  - [x] Add unit tests for exit code mapping

## Verification
- [x] `cargo test` passes
- [x] `cargo run -p agents-cli -- --help` lists all v1 commands
- [x] In an empty temp directory: `agents validate` exits with `NotInitialized` and a clear hint
