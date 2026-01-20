# feat-outputs: Adapter Output Planning

Goal: Given an adapter definition and an effective configuration, compute a concrete list of outputs to render, including collision checks, conditional outputs, and renderer selection.

Depends on: feat-templ, feat-resolv
Unblocks: feat-prevdf, feat-syncer, feat-stamps

## Deliverables
- `OutputPlan` listing outputs with:
  - target path
  - logical surface (optional)
  - renderer type + inputs
  - write policy and drift detection settings
  - source map skeleton
- Deterministic collision handling per PRD.

## Implementation Plan
- [x] Define planning types
  - [x] `PlannedOutput`:
    - [x] `path: RepoPath` (repo-relative)
    - [x] `format`
    - [x] `surface: Option<String>`
    - [x] `collision`
    - [x] `renderer`
    - [x] `write_policy`
    - [x] `drift_detection`
    - [x] `render_context: RenderContext` (or pointer)
  - [x] `OutputPlan`:
    - [x] `agent_id`
    - [x] `backend`
    - [x] `outputs: Vec<PlannedOutput>`

- [x] Evaluate output conditions
  - [x] Implement `condition.backendIn` filtering
  - [x] Implement `condition.profileIn` filtering
  - [x] Stable ordering of outputs (by `path`, then `surface`)
  - [x] Planned output paths do not need to exist on disk

- [x] Collision detection
  - [x] Detect physical path collisions within the same plan
  - [x] Detect logical surface collisions within the same plan
  - [x] Apply collision policies:
    - [x] `error`: fail with details
    - [x] `overwrite`: allow a single winner; define deterministic winner or require explicit ordering
    - [x] `merge`: produce a combined planned output (concat) with stable ordering
    - [x] `shared_owner`: enforce manifest `defaults.sharedSurfacesOwner`

- [x] Renderer dispatch specification
  - [x] Validate renderer config fields:
    - [x] `template` requires `template` path
    - [x] `concat` requires `sources`
    - [x] `copy` requires `sources` (or a `source`)
    - [x] `json_merge` requires `sources` and `jsonMergeStrategy`
  - [ ] Validate sources resolve to known canonical inputs or adapter templates

- [x] Source map skeleton
  - [x] Track for each planned output:
    - [x] adapter id, template path
    - [ ] contributing prompt/snippet file paths
    - [x] mode/policy/skill IDs
  - [x] Store enough info to later implement `agents explain` (feat-explnx)

- [ ] Tests
  - [ ] Condition filtering tests
  - [ ] Collision tests:
    - [ ] physical path collision error
    - [ ] logical surface collision with shared_owner
    - [ ] merge collision produces deterministic output ordering

## Verification
- [ ] A fixture adapter renders a plan with deterministic output ordering
- [ ] Colliding outputs fail with actionable diagnostics
