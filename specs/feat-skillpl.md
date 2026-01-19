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
- [ ] Define types
  - [ ] `SkillRef { id, dir: PathBuf, skill: Skill }`
  - [ ] `EffectiveSkills { enabled: Vec<SkillRef>, disabled: Vec<SkillId>, warnings: Vec<String> }`

- [ ] Compute candidate skill set
  - [ ] Start from manifest `enabled.skills`
  - [ ] Apply mode `enableSkills`/`disableSkills`
  - [ ] Apply scope override `enableSkills`/`disableSkills` in resolution order
  - [ ] Deduplicate and stable-sort by skill ID

- [ ] Validate skill existence and enablement
  - [ ] Error if a referenced skill is missing
  - [ ] Error if a referenced skill is not listed in manifest enabled set (unless explicitly allowed by design)

- [ ] Check compatibility
  - [ ] If `Skill.compatibility.agents` is non-empty:
    - [ ] ensure target agent is included
  - [ ] If `Skill.compatibility.backends` is non-empty:
    - [ ] ensure resolved backend is included
  - [ ] Decide behavior:
    - [ ] default error (strict)
    - [ ] optionally warn+drop skill (future)

- [ ] Summarize requirements for later policy checks
  - [ ] Extract required capabilities (filesystem/exec/network)
  - [ ] Extract required paths (needs/writes)
  - [ ] Provide a `SkillRequirementsSummary` used by feat-runner and feat-compat

- [ ] Deterministic ordering
  - [ ] Ensure stable ordering by skill ID
  - [ ] Preserve stable ordering of `tags` etc when printed

- [ ] Tests
  - [ ] Unit tests for enable/disable precedence
  - [ ] Unit tests for compatibility filtering
  - [ ] Determinism test: ordering stable despite input file ordering

## Verification
- [ ] `agents status` (once wired) lists enabled skills deterministically
- [ ] Fixture where a skill is incompatible fails with actionable diagnostics
