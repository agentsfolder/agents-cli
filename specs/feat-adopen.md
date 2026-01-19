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
- [ ] Confirm OpenCode discovery order and supported files
  - [ ] Decide which shared surface to emit for OpenCode (prefer `AGENTS.md` to share)

- [ ] Implement adapter YAML
  - [ ] Outputs:
    - [ ] `opencode.jsonc`
      - [ ] `format: jsonc`
      - [ ] `driftDetection.stamp: comment` or `json_field` (choose)
    - [ ] Shared surface reference:
      - [ ] use `surface: shared:AGENTS.md` and path `AGENTS.md`
  - [ ] Backend defaults: preferred `vfs_container`

- [ ] Implement opencode.jsonc rendering
  - [ ] Decide configuration mapping for v1:
    - [ ] embed effective mode name
    - [ ] embed policy/tool intent as advisory text fields
    - [ ] include pointers to generated rule file(s) if OpenCode supports
  - [ ] Ensure deterministic JSON serialization ordering

- [ ] Tests
  - [ ] Golden fixture: `agents preview --agent opencode` outputs
  - [ ] Validate stamp and drift detection for jsonc

## Verification
- [ ] `agents preview --agent opencode` produces `opencode.jsonc` and shared rule surface
