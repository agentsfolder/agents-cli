# feat-initpr: `agents init` Presets

Goal: Implement `agents init` to create a valid `.agents/` directory with required structure and starter content using PRD presets.

Depends on: feat-loadag
Unblocks: feat-adbase (if used), feat-adtest

## Deliverables
- `agents init [--preset <name>]` writes:
  - `.agents/manifest.yaml`
  - `.agents/prompts/base.md`, `.agents/prompts/project.md`, `.agents/prompts/snippets/*.md`
  - `.agents/modes/*.md`
  - `.agents/policies/*.yaml`
  - `.agents/scopes/*.yaml` (for monorepo preset)
  - `.agents/adapters/<agent>/**` minimal templates
  - `.agents/schemas/*.schema.json`
  - `.agents/state/.gitignore`

## Implementation Plan
- [ ] Define preset list (PRD)
  - [ ] `conservative`
  - [ ] `standard`
  - [ ] `ci-safe`
  - [ ] `monorepo`
  - [ ] `agent-pack`

- [ ] Bundle template assets in the binary
  - [ ] Use `include_str!` / `include_bytes!` for each template file
  - [ ] Maintain a manifest of embedded files with destination paths
  - [ ] Ensure deterministic output bytes (LF newlines)

- [ ] Implement init behavior
  - [ ] Detect existing `.agents/`:
    - [ ] if exists and non-empty, require `--force` (or error for v1)
  - [ ] Create directory structure
  - [ ] Write files for chosen preset
  - [ ] Write `.agents/state/.gitignore` to ignore `state.yaml`

- [ ] Preset content requirements
  - [ ] Manifest:
    - [ ] set `specVersion`
    - [ ] defaults mode/policy/backend
    - [ ] enabled modes/policies/skills/adapters appropriate for preset
  - [ ] Modes:
    - [ ] at least `default` and `readonly-audit` examples
  - [ ] Policies:
    - [ ] at least one safe default policy
  - [ ] Adapters:
    - [ ] minimal adapter.yaml per supported agent in agent-pack
    - [ ] minimal templates so `agents preview` works later
  - [ ] Schemas:
    - [ ] embed PRD schemas into `.agents/schemas/`

- [ ] Post-init validation
  - [ ] After writing, run internal validate (same as `agents validate`)
  - [ ] Print next steps (run `agents status`, `agents preview`)

- [ ] Tests
  - [ ] Init into temp dir creates expected structure
  - [ ] Init is deterministic (run twice with same preset => same bytes or second run fails safely)

## Verification
- [ ] `agents init --preset standard` creates `.agents/` and `agents validate` passes
