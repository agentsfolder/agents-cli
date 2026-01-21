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
- [x] Define preset list (PRD)
  - [x] `conservative`
  - [x] `standard`
  - [x] `ci-safe`
  - [x] `monorepo`
  - [x] `agent-pack`

- [x] Bundle template assets in the binary
  - [x] Use `include_str!` / `include_bytes!` for each template file
  - [x] Maintain a manifest of embedded files with destination paths
  - [x] Ensure deterministic output bytes (LF newlines)

- [x] Implement init behavior
  - [x] Detect existing `.agents/`:
    - [x] if exists and non-empty, require `--force` (or error for v1)
  - [x] Create directory structure
  - [x] Write files for chosen preset
  - [x] Write `.agents/state/.gitignore` to ignore `state.yaml`

- [x] Preset content requirements
  - [x] Manifest:
    - [x] set `specVersion`
    - [x] defaults mode/policy/backend
    - [x] enabled modes/policies/skills/adapters appropriate for preset
  - [x] Modes:
    - [x] at least `default` and `readonly-audit` examples
  - [x] Policies:
    - [x] at least one safe default policy
  - [x] Adapters:
    - [x] minimal adapter.yaml per supported agent in agent-pack
    - [x] minimal templates so `agents preview` works later
  - [x] Schemas:
    - [x] embed PRD schemas into `.agents/schemas/`

- [x] Post-init validation
  - [x] After writing, run internal validate (same as `agents validate`)
  - [x] Print next steps (run `agents status`, `agents preview`)

- [x] Tests
  - [x] Init into temp dir creates expected structure
  - [x] Init is deterministic (run twice with same preset => same bytes or second run fails safely)

## Verification
- [x] `agents init --preset standard` creates `.agents/` and `agents validate` passes
