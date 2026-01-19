# feat-adcodx: Codex Adapter

Goal: Implement the Codex adapter outputs required by the PRD, primarily using the shared `AGENTS.md` surface.

Depends on: feat-shared, feat-outputs
Unblocks: feat-adtest

## Deliverables
- `.agents/adapters/codex/adapter.yaml` + templates that:
  - emit `AGENTS.md` via shared surface ownership rules
  - optionally emit additional Codex-specific files if needed later

## Implementation Plan
- [ ] Research/confirm Codex surface expectations
  - [ ] Ensure root `AGENTS.md` is sufficient for v1
  - [ ] Document any layering behavior assumptions (root-to-leaf)

- [ ] Implement adapter definition
  - [ ] Create `.agents/adapters/codex/adapter.yaml` (via init preset)
  - [ ] Outputs:
    - [ ] Reference shared surface `shared:AGENTS.md`
      - [ ] Either copy/concat from core generator output OR declare same logical surface and rely on shared ownership
    - [ ] If adapter needs separate file(s), add them with `surface` names to avoid collisions
  - [ ] Set backend defaults:
    - [ ] preferred `vfs_container`
    - [ ] fallback `materialize`

- [ ] Implement templates (if any)
  - [ ] If Codex needs additional instruction formatting, implement `templates/codex-extra.md.hbs`
  - [ ] Keep deterministic and minimal

- [ ] Validate collision behavior
  - [ ] Ensure Codex adapter does not attempt to own shared surface unless configured as owner

- [ ] Tests
  - [ ] Add golden fixture output for `agents preview --agent codex`
  - [ ] Ensure output list matches PRD requirements

## Verification
- [ ] `agents preview --agent codex` produces required outputs without collisions
