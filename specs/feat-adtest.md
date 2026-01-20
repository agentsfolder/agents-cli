# feat-adtest: Adapter Golden Fixture Tests

Goal: Implement `agents test adapters` to render outputs from fixture repos and compare against golden snapshots for regression protection.

Depends on: feat-schemas, feat-outputs, feat-stamps, all adapters in scope
Unblocks: CI gate for adapter changes

## Deliverables
- A fixtures directory layout and runner.
- CLI command `agents test adapters [--agent <id>]`.
- Snapshot comparison with clear diffs on failure.

## Implementation Plan
- [x] Define fixture repository format
  - [x] `fixtures/<name>/repo/**` contains a `.agents/` tree and any other files needed
  - [x] `fixtures/<name>/expect/<agent-id>/**` contains expected rendered outputs
  - [x] Optionally `fixtures/<name>/matrix.yaml` describing profiles/modes/backends to test
  - [x] Add `fixtures/README.md` describing the layout

- [x] Implement test runner API (agents-testutil)
  - [x] `run_fixture(fixture_path, agent_filter) -> TestReport`
  - [x] For each fixture case:
    - [x] load repo config
    - [x] validate schemas
    - [x] resolve effective config for each matrix entry
    - [x] plan + render outputs
    - [x] write to temp dir
    - [x] compare temp outputs to golden expected directory

- [x] Snapshot comparison
  - [x] Ensure comparison is deterministic:
    - [x] stable file list
    - [x] compare bytes exactly
  - [x] On mismatch:
    - [x] print which files differ
    - [x] print a small unified diff for text files
    - [x] write actual outputs to a temp dir and print path for inspection

- [x] CLI integration
  - [x] Add command `agents test adapters [--agent <id>]`
  - [ ] Add `--update` flag (optional) to rewrite goldens (guarded)

- [x] CI integration guidance
  - [x] Ensure exit code non-zero on any mismatch
  - [x] Keep runtime reasonable (parallelize fixtures if possible)

- [ ] Tests
  - [ ] Unit tests for file discovery and compare logic
  - [ ] One self-contained fixture test (runs in CI)

## Verification
- [ ] `agents test adapters` passes on fixtures
- [ ] Intentional template change causes a snapshot failure with readable diff
