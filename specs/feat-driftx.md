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
- [ ] Define diff model
  - [ ] `DiffKind`: `Create`, `Update`, `Noop`, `UnmanagedExists`, `Drifted`, `Missing`
  - [ ] `DiffEntry { path, kind, details }`
  - [ ] `DiffReport { entries: Vec<DiffEntry> }`

- [ ] Compare logic
  - [ ] For each `PlannedOutput`:
    - [ ] Read existing file if present
    - [ ] Strip stamp if present
    - [ ] Compare against planned content (without stamp)
    - [ ] Use feat-stamps drift classification to categorize
  - [ ] Decide treatment for unmanaged existing file at target path:
    - [ ] show as conflict; require explicit action in sync

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
