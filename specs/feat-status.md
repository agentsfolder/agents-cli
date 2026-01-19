# feat-status: `agents status`

Goal: Print the effective configuration (mode/profile/scopes/policy/skills/backend/agent) for transparency and debugging.

Depends on: feat-resolv, feat-skillpl
Unblocks: feat-compat

## Deliverables
- `agents status` prints:
  - repo root
  - effective mode, policy, profile
  - matched scopes
  - enabled skills
  - target agent + backend (if provided)

## Implementation Plan
- [ ] Define status output model
  - [ ] `StatusReport` struct that can be rendered as:
    - [ ] human-readable text
    - [ ] json (optional placeholder)
  - [ ] Include fields:
    - [ ] `repo_root`
    - [ ] `effective_mode`, `effective_policy`, `effective_profile`, `effective_backend`
    - [ ] `scopes_matched` (ordered)
    - [ ] `skills_enabled` (ordered)

- [ ] Wire resolver + skill planner
  - [ ] `status` loads repo config
  - [ ] validates schemas (or relies on validate already)
  - [ ] resolves effective config with optional CLI overrides
  - [ ] computes enabled skills

- [ ] Render output
  - [ ] Keep output stable and grep-friendly
  - [ ] Ensure deterministic ordering of lists
  - [ ] Add hints if:
    - [ ] `.agents/state/state.yaml` influences mode/profile
    - [ ] user overlay is enabled

- [ ] Tests
  - [ ] Snapshot test for status output on a fixture
  - [ ] Ensure ordering is stable

## Verification
- [ ] `agents status` matches expected output for fixture repos
