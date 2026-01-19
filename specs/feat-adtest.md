# feat-adtest: Adapter Golden Fixture Tests

Goal: Implement `agents test adapters` to render outputs from fixture repos and compare against golden snapshots for regression protection.

Depends on: feat-schemas, feat-outputs, feat-stamps, all adapters in scope
Unblocks: CI gate for adapter changes

## Deliverables
- A fixtures directory layout and runner.
- CLI command `agents test adapters [--agent <id>]`.
- Snapshot comparison with clear diffs on failure.

## Implementation Plan
- [ ] Define fixture repository format
  - [ ] `fixtures/<name>/repo/**` contains a `.agents/` tree and any other files needed
  - [ ] `fixtures/<name>/expect/<agent-id>/**` contains expected rendered outputs
  - [ ] Optionally `fixtures/<name>/matrix.yaml` describing profiles/modes/backends to test

- [ ] Implement test runner API (agents-testutil)
  - [ ] `run_fixture(fixture_path, agent_filter) -> TestReport`
  - [ ] For each fixture case:
    - [ ] load repo config
    - [ ] validate schemas
    - [ ] resolve effective config for each matrix entry
    - [ ] plan + render outputs
    - [ ] write to temp dir
    - [ ] compare temp outputs to golden expected directory

- [ ] Snapshot comparison
  - [ ] Ensure comparison is deterministic:
    - [ ] stable file list
    - [ ] compare bytes exactly
  - [ ] On mismatch:
    - [ ] print which files differ
    - [ ] print a small unified diff for text files
    - [ ] write actual outputs to a temp dir and print path for inspection

- [ ] CLI integration
  - [ ] Add command `agents test adapters [--agent <id>]`
  - [ ] Add `--update` flag (optional) to rewrite goldens (guarded)

- [ ] CI integration guidance
  - [ ] Ensure exit code non-zero on any mismatch
  - [ ] Keep runtime reasonable (parallelize fixtures if possible)

- [ ] Tests
  - [ ] Unit tests for file discovery and compare logic
  - [ ] One self-contained fixture test (runs in CI)

## Verification
- [ ] `agents test adapters` passes on fixtures
- [ ] Intentional template change causes a snapshot failure with readable diff
