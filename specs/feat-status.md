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
- [x] Define status output model
  - [x] `StatusReport` struct that can be rendered as:
    - [x] human-readable text
    - [ ] json (optional placeholder)
  - [x] Include fields:
    - [x] `repo_root`
    - [x] `effective_mode`, `effective_policy`, `effective_profile`, `effective_backend`
    - [x] `scopes_matched` (ordered)
    - [x] `skills_enabled` (ordered)

- [x] Wire resolver + skill planner
  - [x] `status` loads repo config
  - [x] validates schemas (or relies on validate already)
  - [x] resolves effective config with optional CLI overrides
  - [x] computes enabled skills

- [x] Render output
  - [x] Keep output stable and grep-friendly
  - [x] Ensure deterministic ordering of lists
  - [x] Add hints if:
    - [x] `.agents/state/state.yaml` influences mode/profile
    - [x] user overlay is enabled

- [x] Tests
  - [x] Snapshot test for status output on a fixture
  - [x] Ensure ordering is stable

## Verification
- [x] `agents status` matches expected output for fixture repos
