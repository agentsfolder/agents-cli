# feat-syncer: `agents sync`

Goal: Apply a computed output plan using the selected backend (`materialize` or `vfs_container`), producing deterministic results.

Depends on: feat-matwiz, feat-vfsctr, feat-outputs
Unblocks: feat-runner, feat-doctor

## Deliverables
- `agents sync [--agent <id>] [--backend <backend>]`.
- Applies planned outputs via the backend.

## Implementation Plan
- [ ] Implement sync orchestration
  - [ ] Load repo config + validate
  - [ ] Resolve effective config
  - [ ] Plan outputs for agent
  - [ ] Render outputs and apply stamps
  - [ ] Select backend:
    - [ ] from CLI override
    - [ ] else from manifest defaults/byAgent
    - [ ] else from adapter backendDefaults
  - [ ] Call backend apply

- [ ] Handle conflicts
  - [ ] If materialize backend reports unmanaged conflicts:
    - [ ] return non-zero
    - [ ] print actionable hint (run diff, adjust writePolicy)

- [ ] Reporting
  - [ ] Print written/skipped/conflict counts
  - [ ] In verbose mode, print per-path actions

- [ ] Tests
  - [ ] Sync then diff yields no changes (requires feat-driftx integration)
  - [ ] Sync fails when unmanaged file exists at output path and writePolicy is `if_generated`

## Verification
- [ ] `agents sync` followed by `agents diff` yields no changes
