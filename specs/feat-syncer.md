# feat-syncer: `agents sync`

Goal: Apply a computed output plan using the selected backend (`materialize` or `vfs_container`), producing deterministic results.

Depends on: feat-matwiz, feat-vfsctr, feat-outputs
Unblocks: feat-runner, feat-doctor

## Deliverables
- `agents sync [--agent <id>] [--backend <backend>]`.
- Applies planned outputs via the backend.

## Implementation Plan
- [x] Implement sync orchestration
  - [x] Load repo config + validate
  - [x] Resolve effective config
  - [x] Plan outputs for agent
  - [x] Render outputs and apply stamps
  - [x] Select backend:
    - [x] from CLI override
    - [x] else from manifest defaults/byAgent
    - [x] else from adapter backendDefaults
  - [x] Call backend apply

- [x] Handle conflicts
  - [x] If materialize backend reports unmanaged conflicts:
    - [x] return non-zero
    - [x] print actionable hint (run diff, adjust writePolicy)

- [ ] Reporting
  - [ ] Print written/skipped/conflict counts
  - [ ] In verbose mode, print per-path actions

- [ ] Tests
  - [ ] Sync then diff yields no changes (requires feat-driftx integration)
  - [ ] Sync fails when unmanaged file exists at output path and writePolicy is `if_generated`

## Verification
- [ ] `agents sync` followed by `agents diff` yields no changes
