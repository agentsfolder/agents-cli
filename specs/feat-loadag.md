# feat-loadag: Load `.agents/**` Into Memory

Goal: Discover and load all canonical `.agents` inputs into an in-memory `RepoConfig` with referential integrity checks (existence of required files, duplicate IDs, etc.).

Depends on: feat-fsutil, feat-models
Unblocks: feat-schemas, feat-resolv, feat-initpr

## Deliverables
- `RepoConfig` representing all loaded `.agents` artifacts.
- Clear diagnostics for missing files, duplicate IDs, unreadable YAML/MD.

## Implementation Plan
- [x] Define configuration container types
  - [x] `RepoConfig` containing:
    - [x] repo root path
    - [x] manifest
    - [x] maps: policies, skills, scopes, profiles (if implemented), adapters
    - [x] prompts (base/project/snippets)
    - [x] modes (frontmatter + body)
    - [x] optional state
  - [x] Define `LoadWarnings` vs hard errors (e.g., missing optional state)

- [x] Load required core files
  - [x] Require `.agents/manifest.yaml`
  - [x] Require `.agents/prompts/base.md` and `.agents/prompts/project.md`
  - [x] Require `.agents/schemas/**` presence OR define behavior (error until feat-initpr populates)

- [x] Load collections with deterministic ordering
  - [x] Policies: `.agents/policies/*.yaml`
    - [x] Parse to `Policy`, key by `policy.id`
    - [x] Error on duplicate IDs
  - [x] Skills: `.agents/skills/*/skill.yaml`
    - [x] Parse to `Skill`, key by `skill.id`
    - [x] Record base directory for skill assets
  - [x] Scopes: `.agents/scopes/*.yaml`
    - [x] Parse to `Scope`, key by `scope.id`
  - [x] Modes: `.agents/modes/*.md`
    - [x] Parse frontmatter/body
    - [x] Determine mode ID:
      - [x] `frontmatter.id` if present else derive from filename
    - [x] Error on duplicate mode IDs
  - [x] Adapters: `.agents/adapters/*/adapter.yaml`
    - [x] Parse to `Adapter`, key by `agentId`
    - [x] Record templates directory path
  - [x] Profiles (if present): `.agents/profiles/*.yaml`
    - [x] Decide minimal schema (even if not in PRD, treat as freeform overrides)

- [x] Load prompt snippet library
  - [x] Load `.agents/prompts/snippets/*.md`
  - [x] Key by filename stem (snippet id)
  - [x] Preserve deterministic ordering for later composition

- [x] Load optional state
  - [x] If `.agents/state/state.yaml` exists, parse to `State`
  - [x] If missing, treat as None

- [x] Referential integrity checks (lightweight)
  - [x] Ensure manifest enabled IDs refer to loaded entities
    - [x] enabled.modes exist
    - [x] enabled.policies exist
    - [x] enabled.skills exist
    - [x] enabled.adapters exist
  - [x] Ensure manifest defaults refer to existing entities

- [ ] Error reporting
  - [ ] Include file path in all load/parse errors
  - [ ] Distinguish YAML parse error vs schema invalid (schema is feat-schemas)

- [ ] Tests
  - [ ] Create a minimal fixture `.agents` tree under `crates/agents-core/tests/fixtures/...`
  - [ ] Unit tests:
    - [ ] missing manifest => NotInitialized
    - [ ] duplicate IDs => error
    - [ ] deterministic ordering of loaded collections

## Verification
- [ ] `agents validate` can load `.agents` and reach the next stage (schema validation comes in feat-schemas)
