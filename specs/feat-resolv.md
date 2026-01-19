# feat-resolv: Resolution Engine (Profiles, Scopes, Overrides)

Goal: Compute the effective configuration for a run given repo config, optional user overlay, optional state, and CLI overrides. Must be deterministic.

Depends on: feat-loadag, feat-schemas
Unblocks: feat-prompts, feat-skillpl, feat-status

## Deliverables
- `Resolver` that returns `EffectiveConfig` with:
  - effective mode/policy/profile/backend
  - matched scopes (ordered)
  - enabled skills
  - included snippets
- Deterministic deep-merge and scope specificity.

## Implementation Plan
- [x] Define resolution inputs/outputs
  - [x] `ResolutionRequest`:
    - [x] repo root
    - [x] optional target path (repo-relative) for scope matching (default: ".")
    - [x] cli overrides: mode/policy/profile/backend/scopes
    - [x] enable/disable user overlay (from manifest)
  - [x] `EffectiveConfig`:
    - [x] `mode_id`, `policy_id`, `profile` (optional), `backend`
    - [x] `scopes_matched: Vec<ScopeMatch>` (sorted)
    - [x] `skill_ids_enabled: Vec<SkillId>`
    - [x] `snippet_ids_included: Vec<String>`

- [x] Implement scope matching
  - [x] Use `globset` to compile `Scope.applyTo` patterns
  - [x] Decide match input:
    - [x] scope matches if any `applyTo` matches the target path
  - [x] Implement specificity scoring:
    - [x] higher score for more literal segments
    - [x] lower score for `*`, `**`, and `?` wildcards
    - [x] tie-breakers: `priority` desc, then `scope.id` asc
  - [x] Return sorted list of matches with their score

- [ ] Implement precedence layering (PRD)
  - [ ] Build a layered set of "overrides" in order:
    - [ ] user overlay (lowest) (if enabled)
    - [ ] repo base (manifest defaults and repo artifacts)
    - [ ] best matching scope override(s) (apply most specific)
    - [ ] repo state (if used) (decide exact placement; document)
    - [ ] CLI overrides (highest)
  - [ ] Decide rule for multiple matching scopes:
    - [ ] apply in deterministic order (least specific to most specific) so most specific wins

- [ ] Implement deep merge semantics
  - [ ] Define merge for:
    - [ ] scalar fields (mode/policy/profile/backend): last writer wins
    - [ ] lists (enableSkills/disableSkills/includeSnippets): union + stable sort
    - [ ] policies: "deny beats allow" evaluation rule
  - [ ] For authoritative singletons (e.g., specVersion conflicts across overlays): error by default

- [ ] Validate resolved references
  - [ ] Ensure resolved mode exists and is enabled
  - [ ] Ensure resolved policy exists and is enabled
  - [ ] Ensure profile exists if specified (once profile model exists)
  - [ ] Ensure referenced skills/snippets exist (warn vs error; decide and document)

- [ ] Expose public API
  - [ ] `Resolver::resolve(req) -> Result<EffectiveConfig>`
  - [ ] `Resolver::resolve_for_agent(req, agent_id) -> Result<EffectiveConfig>` (optional helper)

- [ ] Tests
  - [ ] Precedence tests:
    - [ ] CLI override beats scope
    - [ ] scope beats repo default
    - [ ] user overlay is lowest
  - [ ] Specificity tests:
    - [ ] `apps/web/**` beats `apps/**`
    - [ ] `priority` breaks ties
  - [ ] Determinism tests:
    - [ ] stable ordering of skills and snippets

## Verification
- [ ] Fixture matrix produces expected effective mode/policy/skills
- [ ] `agents status --mode <x> --profile <y>` matches expected effective config
