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

- [x] Implement precedence layering (PRD)
  - [x] Build a layered set of "overrides" in order:
    - [ ] user overlay (lowest) (if enabled)
    - [x] repo base (manifest defaults and repo artifacts)
    - [x] best matching scope override(s) (apply most specific)
    - [x] repo state (if used) (decide exact placement; document)
    - [x] CLI overrides (highest)
  - [x] Decide rule for multiple matching scopes:
    - [x] apply in deterministic order (least specific to most specific) so most specific wins

- [x] Implement deep merge semantics
  - [x] Define merge for:
    - [x] scalar fields (mode/policy/profile/backend): last writer wins
    - [x] lists (enableSkills/disableSkills/includeSnippets): union + stable sort
    - [x] policies: "deny beats allow" evaluation rule
  - [x] For authoritative singletons (e.g., specVersion conflicts across overlays): error by default

- [x] Validate resolved references
  - [x] Ensure resolved mode exists and is enabled
  - [x] Ensure resolved policy exists and is enabled
  - [x] Ensure profile exists if specified (once profile model exists)
  - [x] Ensure referenced skills/snippets exist (warn vs error; decide and document)

- [x] Expose public API
  - [x] `Resolver::resolve(req) -> Result<EffectiveConfig>`
  - [ ] `Resolver::resolve_for_agent(req, agent_id) -> Result<EffectiveConfig>` (optional helper)

- [x] Tests
  - [x] Precedence tests:
    - [x] CLI override beats scope
    - [x] scope beats repo default
    - [x] user overlay is lowest
  - [x] Specificity tests:
    - [x] `apps/web/**` beats `apps/**`
    - [x] `priority` breaks ties
  - [x] Determinism tests:
    - [x] stable ordering of skills and snippets

## Verification
- [ ] Fixture matrix produces expected effective mode/policy/skills
- [ ] `agents status --mode <x> --profile <y>` matches expected effective config
