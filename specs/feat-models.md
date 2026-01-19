# feat-models: Canonical Data Model

Goal: Define typed Rust structs for all `.agents` canonical files and mode frontmatter, matching the PRD schemas.

Depends on: feat-cliapp
Unblocks: feat-loadag, feat-schemas, feat-resolv

## Deliverables
- `agents-core` exposes strongly typed models with `serde` support.
- Unknown fields are rejected (defense-in-depth, aligns with `additionalProperties: false`).

## Implementation Plan
- [x] Create module layout
  - [x] `agents_core::model::{manifest, policy, skill, scope, adapter, state, mode}`
  - [x] Re-export a small prelude (`use agents_core::model::*`) to reduce churn

- [x] Implement core structs (serde)
  - [x] Manifest
    - [x] `spec_version`, `project`, `defaults`, `enabled`, `resolution`, `backends`, `x`
    - [x] Enums for backend values
  - [x] Policy
    - [x] capabilities (filesystem/exec/network/mcp)
    - [x] paths (allow/deny/redact)
    - [x] confirmations (requiredFor)
    - [x] limits
  - [x] Skill
    - [x] `activation` enum
    - [x] `interface` struct with `type`, `entrypoint`, `args`, `env`
    - [x] `requirements` with capabilities and optional paths
    - [x] `assets` and `compatibility`
  - [x] Scope
    - [x] `applyTo` list
    - [x] `priority`
    - [x] overrides (mode/policy/enableSkills/disableSkills/includeSnippets)
  - [x] Adapter
    - [x] backend defaults
    - [x] outputs: path/format/surface/collision/condition/renderer/writePolicy/driftDetection
    - [x] renderer enum (template/concat/copy/json_merge)
  - [x] State
    - [x] mode/profile/backend/scopes
  - [x] Mode frontmatter
    - [x] id/title/policy/enableSkills/disableSkills/includeSnippets/toolIntent

- [x] Enforce strict deserialization
  - [x] Use `#[serde(deny_unknown_fields)]` on structs
  - [x] Use `Option<T>` only where schema allows absence
  - [x] Add `Default` only where it helps and does not hide missing required fields

- [x] Markdown frontmatter parsing
  - [x] Define `ModeFile { frontmatter: ModeFrontmatter, body: String }`
  - [x] Implement `parse_frontmatter_markdown(text) -> Result<(Option<ModeFrontmatter>, String)>`
    - [x] YAML frontmatter delimited by `---` lines
    - [x] Fail if frontmatter is present but invalid YAML
    - [x] Preserve body with normalized `\n`

- [x] ID and path newtypes (optional but recommended)
  - [x] `ModeId`, `PolicyId`, `SkillId`, `AdapterId`, `ScopeId` as `String` wrappers
  - [x] Add lightweight validation helpers (non-empty, printable)

- [x] Tests
  - [x] Deserialize fixtures for each entity (minimal valid examples)
  - [x] Frontmatter parsing tests:
    - [x] no frontmatter
    - [x] valid frontmatter
    - [x] malformed frontmatter
  - [x] Unknown field rejection tests

## Verification
- [ ] `cargo test` passes
- [ ] Sample YAML from PRD schemas deserializes cleanly
