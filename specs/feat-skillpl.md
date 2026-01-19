# feat-skillpl: Skill Plan (Enabled Skills Computation)

Goal: Given an `EffectiveConfig`, compute the final enabled skill set, validate compatibility, and produce a deterministic list used by adapters/backends.

Depends on: feat-resolv
Unblocks: feat-runner, feat-compat

## Deliverables
- `SkillPlanner` that returns `EffectiveSkills`:
  - `enabled: Vec<SkillRef>` with stable order
  - `disabled: Vec<SkillId>` (optional diagnostics)
  - compatibility flags (agent/backends) and requirements summaries

## Implementation Plan
- [x] Define types
  - [x] `SkillRef { id, dir: PathBuf, skill: Skill }`
  - [x] `EffectiveSkills { enabled: Vec<SkillRef>, disabled: Vec<SkillId>, warnings: Vec<String> }`

- [x] Compute candidate skill set
  - [x] Start from manifest `enabled.skills`
  - [x] Apply mode `enableSkills`/`disableSkills`
  - [x] Apply scope override `enableSkills`/`disableSkills` in resolution order
  - [x] Deduplicate and stable-sort by skill ID

- [x] Validate skill existence and enablement
  - [x] Error if a referenced skill is missing
  - [x] Error if a referenced skill is not listed in manifest enabled set (unless explicitly allowed by design)

- [x] Check compatibility
  - [x] If `Skill.compatibility.agents` is non-empty:
    - [x] ensure target agent is included
  - [x] If `Skill.compatibility.backends` is non-empty:
    - [x] ensure resolved backend is included
  - [x] Decide behavior:
    - [x] default error (strict)
    - [ ] optionally warn+drop skill (future)

- [x] Summarize requirements for later policy checks
  - [x] Extract required capabilities (filesystem/exec/network)
  - [x] Extract required paths (needs/writes)
  - [x] Provide a `SkillRequirementsSummary` used by feat-runner and feat-compat

- [x] Deterministic ordering
  - [x] Ensure stable ordering by skill ID
  - [x] Preserve stable ordering of `tags` etc when printed

- [x] Tests
  - [x] Unit tests for enable/disable precedence
  - [x] Unit tests for compatibility filtering
  - [x] Determinism test: ordering stable despite input file ordering

## Verification
- [x] `agents status` (once wired) lists enabled skills deterministically
- [x] Fixture where a skill is incompatible fails with actionable diagnostics
