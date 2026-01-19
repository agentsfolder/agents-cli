# feat-loadag: Load `.agents/**` Into Memory

Goal: Discover and load all canonical `.agents` inputs into an in-memory `RepoConfig` with referential integrity checks (existence of required files, duplicate IDs, etc.).

Depends on: feat-fsutil, feat-models
Unblocks: feat-schemas, feat-resolv, feat-initpr

## Deliverables
- `RepoConfig` representing all loaded `.agents` artifacts.
- Clear diagnostics for missing files, duplicate IDs, unreadable YAML/MD.

## Implementation Plan
- [ ] Define configuration container types
  - [ ] `RepoConfig` containing:
    - [ ] repo root path
    - [ ] manifest
    - [ ] maps: policies, skills, scopes, profiles (if implemented), adapters
    - [ ] prompts (base/project/snippets)
    - [ ] modes (frontmatter + body)
    - [ ] optional state
  - [x] Define `LoadWarnings` vs hard errors (e.g., missing optional state)

- [ ] Load required core files
  - [ ] Require `.agents/manifest.yaml`
  - [ ] Require `.agents/prompts/base.md` and `.agents/prompts/project.md`
  - [ ] Require `.agents/schemas/**` presence OR define behavior (error until feat-initpr populates)

- [ ] Load collections with deterministic ordering
  - [ ] Policies: `.agents/policies/*.yaml`
    - [ ] Parse to `Policy`, key by `policy.id`
    - [ ] Error on duplicate IDs
  - [ ] Skills: `.agents/skills/*/skill.yaml`
    - [ ] Parse to `Skill`, key by `skill.id`
    - [ ] Record base directory for skill assets
  - [ ] Scopes: `.agents/scopes/*.yaml`
    - [ ] Parse to `Scope`, key by `scope.id`
  - [ ] Modes: `.agents/modes/*.md`
    - [ ] Parse frontmatter/body
    - [ ] Determine mode ID:
      - [ ] `frontmatter.id` if present else derive from filename
    - [ ] Error on duplicate mode IDs
  - [ ] Adapters: `.agents/adapters/*/adapter.yaml`
    - [ ] Parse to `Adapter`, key by `agentId`
    - [ ] Record templates directory path
  - [ ] Profiles (if present): `.agents/profiles/*.yaml`
    - [ ] Decide minimal schema (even if not in PRD, treat as freeform overrides)

- [ ] Load prompt snippet library
  - [ ] Load `.agents/prompts/snippets/*.md`
  - [ ] Key by filename stem (snippet id)
  - [ ] Preserve deterministic ordering for later composition

- [ ] Load optional state
  - [ ] If `.agents/state/state.yaml` exists, parse to `State`
  - [ ] If missing, treat as None

- [ ] Referential integrity checks (lightweight)
  - [ ] Ensure manifest enabled IDs refer to loaded entities
    - [ ] enabled.modes exist
    - [ ] enabled.policies exist
    - [ ] enabled.skills exist
    - [ ] enabled.adapters exist
  - [ ] Ensure manifest defaults refer to existing entities

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
