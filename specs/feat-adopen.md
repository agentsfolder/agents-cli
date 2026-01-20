# feat-adopen: OpenCode Adapter

Goal: Implement OpenCode adapter outputs: `opencode.jsonc` plus one shared rule surface (`AGENTS.md`/`CLAUDE.md`/`CONTEXT.md`) compatible with OpenCode discovery.

Depends on: feat-shared, feat-templ
Unblocks: feat-adtest

## Deliverables
- `.agents/adapters/opencode/adapter.yaml`
- Templates:
  - `templates/opencode.jsonc.hbs` (or json_merge sources)
  - shared surface reference

## Implementation Plan
- [x] Confirm OpenCode discovery order and supported files
  - [x] Decide which shared surface to emit for OpenCode (prefer `AGENTS.md` to share)

- [x] Implement adapter YAML
  - [x] Outputs:
    - [x] `opencode.jsonc`
      - [x] `format: jsonc`
      - [x] `driftDetection.stamp: json_field` (chosen)
    - [x] Shared surface reference:
      - [x] use `surface: shared:AGENTS.md` and path `AGENTS.md`
  - [x] Backend defaults: preferred `vfs_container`

- [x] Implement opencode.jsonc rendering
  - [x] Decide configuration mapping for v1:
    - [x] embed effective mode name
    - [x] embed policy/tool intent as advisory text fields
    - [x] include pointers to generated rule file(s) if OpenCode supports
  - [x] Ensure deterministic JSON serialization ordering

- [ ] Tests
  - [x] Golden fixture: `agents preview --agent opencode` outputs
  - [x] Validate stamp and drift detection for jsonc

## Verification
- [ ] `agents preview --agent opencode` produces `opencode.jsonc` and shared rule surface
