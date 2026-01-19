# feat-fsutil: Filesystem + Path Utilities

Goal: Provide cross-platform filesystem helpers used by all other features: repo discovery, stable reads/writes, atomic writes, and safe path handling.

Depends on: feat-cliapp
Unblocks: feat-loadag, feat-matwiz, feat-prevdf

## Deliverables
- `agents-core` (or a dedicated module) exposes a small `fsutil` API.
- Repo root discovery works from any subdirectory.
- Deterministic file IO helpers (newline normalization only where appropriate).

## Implementation Plan
- [x] Module skeleton
  - [x] Add `agents-core::fsutil` module file
  - [x] Define `FsError` + `FsResult` and export module via `crates/agents-core/src/lib.rs`

- [x] Repo root discovery
  - [x] Implement `discover_repo_root(start: Path) -> Result<PathBuf>`
    - [x] Walk parents until `.git/` OR `.agents/` is found
    - [x] Prefer the nearest parent containing `.agents/` when multiple `.git` ancestors exist
    - [x] If none found, default to `start` (or error) based on CLI behavior
  - [x] Implement `agents_dir(root) -> PathBuf` and `require_agents_dir(root) -> Result<()>`

- [x] Path normalization and safety
  - [x] Implement `repo_relpath(root, path) -> Result<RepoPath>`
    - [x] Reject paths that escape repo root after canonicalization
    - [x] Normalize separators to `/` for internal matching/printing
  - [x] Implement stable path display helpers for diagnostics
  - [x] Add Windows-specific tests (use `Path::components` not string hacks)

- [x] Directory walking helpers
  - [x] Implement `walk_repo_agents(root) -> Iterator<PathBuf>`
    - [x] Skip `.agents/state/state.yaml` if missing (optional)
    - [x] Skip `.agents/state/**` except `.gitignore` and optional state.yaml
  - [x] Provide a helper to ensure deterministic traversal ordering (sort by normalized path)

- [x] Stable file reads
  - [x] Implement `read_to_string(path) -> Result<String>`
    - [x] Preserve exact bytes where possible
    - [x] Normalize `
` to `
` for text formats (md/yaml/json/jsonc/text)
  - [x] Implement `read_bytes(path) -> Result<Vec<u8>>` for non-text (future)


- [x] Atomic file writes
  - [x] Implement `atomic_write(path, bytes)`
    - [x] Create parent directories if needed
    - [x] Write to a temp file in the same directory
    - [x] `fsync` best-effort (platform-dependent)
    - [x] Replace target via rename
  - [x] Implement `ensure_trailing_newline(text) -> String` for generated text files

- [x] Temp directory utilities
  - [x] Implement `temp_generation_dir(prefix) -> Result<TempDir>`
  - [x] Ensure temp dirs are cleaned up unless `--keep-temp` is introduced later

- [x] Tests
  - [x] Unit tests for:
    - [x] root discovery (nested directories)
    - [x] path escape rejection (`..` paths)
    - [x] deterministic traversal ordering
    - [x] atomic write round-trip
    - [x] newline normalization

## Verification
- [x] `cargo test` passes
- [x] In a fixture repo: discovery finds the correct root from `repo/sub/dir`
