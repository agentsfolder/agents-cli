# feat-driftx: Diff Engine (Planned vs Materialized)

Goal: Compute and present differences between planned outputs and current on-disk state, without writing changes.

Depends on: feat-stamps
Unblocks: feat-prevdf, feat-doctor

## Deliverables
- `DiffReport` per output:
  - create/update/delete/noop
  - drift classification
  - optional unified diff text
- CLI integration for `agents diff` and `agents preview` (later)

## Implementation Plan
- [x] Define diff model
  - [x] `DiffKind`: `Create`, `Update`, `Delete`, `Noop`, `UnmanagedExists`, `Drifted`
  - [x] `DiffEntry { path, kind, details }`
  - [x] `DiffReport { entries: Vec<DiffEntry> }`

- [x] Compare logic
  - [x] For each `PlannedOutput`:
    - [x] Read existing file if present
    - [x] Strip stamp if present
    - [x] Compare against planned content (without stamp)
    - [x] Use feat-stamps drift classification to categorize
  - [x] Decide treatment for unmanaged existing file at target path:
    - [x] show as conflict; require explicit action in sync

- [ ] Unified diff generation
  - [ ] Choose a diff library or implement simple line diff
  - [ ] Ensure stable diff output (normalize newlines)
  - [ ] Include context lines, but keep output bounded (later add `--full`)

- [ ] Deletion detection (optional for v1)
  - [ ] Identify stamped generated outputs that are no longer planned
    - [ ] used by `clean` and `doctor`

- [ ] Tests
  - [ ] Fixture where planned == existing => Noop
  - [ ] Fixture where existing differs => Update + diff
  - [ ] Fixture where unmanaged file exists => UnmanagedExists

## Verification
- [ ] `agents diff` shows clean/no-op after `agents sync`
- [ ] Diff output is deterministic across runs
