# feat-prevdf: `agents preview` + `agents diff`

Goal: Implement preview (render to temp dir) and diff (show changes vs materialized) workflows.

Depends on: feat-driftx
Unblocks: feat-syncer

## Deliverables
- `agents preview`:
  - renders planned outputs into a temp dir
  - prints output paths and where they would go
- `agents diff`:
  - prints a stable list of changes needed to sync
  - optionally prints unified diffs

## Implementation Plan
- [x] Implement preview pipeline
  - [x] Load repo config + validate schemas
  - [x] Resolve effective config (mode/profile/scope/backend)
  - [x] Plan outputs for selected agent
  - [x] Render outputs (template/concat/copy/json_merge)
  - [x] Apply stamps (but write only into temp dir)
  - [x] Print:
    - [x] temp directory path
    - [x] list of outputs with repo-relative destination

- [ ] Implement diff pipeline
  - [ ] Same as preview until planned stamped content is available
  - [ ] Use feat-driftx to compare planned vs on-disk
  - [ ] Print:
    - [ ] counts (create/update/noop/conflict)
    - [ ] per-path status lines
    - [ ] optional `--show` to print unified diffs

- [x] Temp directory handling
  - [x] Use feat-fsutil temp dir helper
  - [x] Add `--keep-temp` (optional) for debugging

- [ ] Tests
  - [ ] Preview produces files with expected names in temp dir
  - [ ] Diff report matches fixture expectations

## Verification
- [ ] `agents preview --agent <id>` renders deterministic temp outputs
- [ ] `agents diff --agent <id>` is stable across runs
