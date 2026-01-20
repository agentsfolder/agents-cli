# feat-adcodx: Codex Adapter

Goal: Implement the Codex adapter outputs required by the PRD, primarily using the shared `AGENTS.md` surface.

Depends on: feat-shared, feat-outputs
Unblocks: feat-adtest

## Deliverables
- `.agents/adapters/codex/adapter.yaml` + templates that:
  - emit `AGENTS.md` via shared surface ownership rules
  - optionally emit additional Codex-specific files if needed later

## Implementation Plan
- [x] Research/confirm Codex surface expectations
  - [x] Ensure root `AGENTS.md` is sufficient for v1
  - [x] Document any layering behavior assumptions (root-to-leaf)

Notes:
- Codex reads `AGENTS.md` files from repo root down to the current working directory.
- Per-directory precedence: `AGENTS.override.md` then `AGENTS.md` (then optional fallbacks).
- Files are concatenated with blank lines; later (deeper) files override earlier guidance.

- [x] Implement adapter definition
  - [x] Create `.agents/adapters/codex/adapter.yaml` (fixture)
  - [x] Outputs:
    - [x] Reference shared surface `shared:AGENTS.md`
      - [x] Declare logical surface and rely on shared ownership (via `sharedSurfacesOwner=codex`)
    - [x] No additional outputs for v1
  - [x] Set backend defaults:
    - [x] preferred `vfs_container`
    - [x] fallback `materialize`

- [x] Implement templates (if any)
  - [x] Implement `templates/AGENTS.md.hbs` for Codex
  - [x] Keep deterministic and minimal

- [x] Validate collision behavior
  - [x] Ensure Codex adapter does not attempt to own shared surface unless configured as owner

- [x] Tests
  - [x] Add golden fixture output for `agents preview --agent codex`
  - [x] Ensure output list matches PRD requirements

## Verification
- [x] `agents preview --agent codex` produces required outputs without collisions
