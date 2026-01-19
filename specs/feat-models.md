# feat-models: Canonical Data Model

Goal: Define typed Rust structs for all `.agents` canonical files and mode frontmatter, matching the PRD schemas.

Depends on: feat-cliapp
Unblocks: feat-loadag, feat-schemas, feat-resolv

## Deliverables
- `agents-core` exposes strongly typed models with `serde` support.
- Unknown fields are rejected (defense-in-depth, aligns with `additionalProperties: false`).

## Implementation Plan
- [ ] Create module layout
  - [ ] `agents_core::model::{manifest, policy, skill, scope, adapter, state, mode}`
  - [ ] Re-export a small prelude (`use agents_core::model::*`) to reduce churn

- [ ] Implement core structs (serde)
  - [ ] Manifest
    - [ ] `spec_version`, `project`, `defaults`, `enabled`, `resolution`, `backends`, `x`
    - [ ] Enums for backend values
  - [ ] Policy
    - [ ] capabilities (filesystem/exec/network/mcp)
    - [ ] paths (allow/deny/redact)
    - [ ] confirmations (requiredFor)
    - [ ] limits
  - [ ] Skill
    - [ ] `activation` enum
    - [ ] `interface` struct with `type`, `entrypoint`, `args`, `env`
    - [ ] `requirements` with capabilities and optional paths
    - [ ] `assets` and `compatibility`
  - [ ] Scope
    - [ ] `applyTo` list
    - [ ] `priority`
    - [ ] overrides (mode/policy/enableSkills/disableSkills/includeSnippets)
  - [ ] Adapter
    - [ ] backend defaults
    - [ ] outputs: path/format/surface/collision/condition/renderer/writePolicy/driftDetection
    - [ ] renderer enum (template/concat/copy/json_merge)
  - [ ] State
    - [ ] mode/profile/backend/scopes
  - [ ] Mode frontmatter
    - [ ] id/title/policy/enableSkills/disableSkills/includeSnippets/toolIntent

- [ ] Enforce strict deserialization
  - [ ] Use `#[serde(deny_unknown_fields)]` on structs
  - [ ] Use `Option<T>` only where schema allows absence
  - [ ] Add `Default` only where it helps and does not hide missing required fields

- [ ] Markdown frontmatter parsing
  - [ ] Define `ModeFile { frontmatter: ModeFrontmatter, body: String }`
  - [ ] Implement `parse_frontmatter_markdown(text) -> Result<(Option<ModeFrontmatter>, String)>`
    - [ ] YAML frontmatter delimited by `---` lines
    - [ ] Fail if frontmatter is present but invalid YAML
    - [ ] Preserve body with normalized `\n`

- [ ] ID and path newtypes (optional but recommended)
  - [ ] `ModeId`, `PolicyId`, `SkillId`, `AdapterId`, `ScopeId` as `String` wrappers
  - [ ] Add lightweight validation helpers (non-empty, printable)

- [ ] Tests
  - [ ] Deserialize fixtures for each entity (minimal valid examples)
  - [ ] Frontmatter parsing tests:
    - [ ] no frontmatter
    - [ ] valid frontmatter
    - [ ] malformed frontmatter
  - [ ] Unknown field rejection tests

## Verification
- [ ] `cargo test` passes
- [ ] Sample YAML from PRD schemas deserializes cleanly
