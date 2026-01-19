# feat-doctor: `agents doctor`

Goal: Provide a diagnostic command that checks for invalid schemas, drift, collisions, missing prerequisites, and can fix safe issues.

Depends on: feat-schemas, feat-driftx, feat-cleanup
Unblocks: feat-adtest

## Deliverables
- `agents doctor [--fix] [--ci]` with checks:
  - schema validity
  - collision detection (via planning)
  - drift detection (via stamps + diff)
  - backend prerequisites (docker installed for vfs_container)
  - safe cleanup suggestions

## Implementation Plan
- [ ] Define check framework
  - [ ] `DoctorCheck` trait with:
    - [ ] `name`
    - [ ] `run(ctx) -> CheckResult`
    - [ ] optional `fix(ctx) -> FixResult`
  - [ ] `DoctorReport` with:
    - [ ] errors, warnings, infos
    - [ ] exit code mapping (`--ci` makes warnings fail if desired)

- [ ] Implement checks
  - [ ] Schema check
    - [ ] run the same validation as `agents validate`
  - [ ] Collision check
    - [ ] plan outputs for each enabled adapter (or selected agent)
    - [ ] surface/path collisions reported with details
  - [ ] Drift check
    - [ ] for each adapter, compute planned outputs and compare to disk
    - [ ] report drifted files and unmanaged conflicts
  - [ ] Prerequisites check
    - [ ] if any adapter/backends require docker, confirm docker available
  - [ ] State file check
    - [ ] ensure `.agents/state/.gitignore` exists and ignores `state.yaml`

- [ ] Implement safe fixes (`--fix`)
  - [ ] Create missing `.agents/state/.gitignore`
  - [ ] Remove stale generated files that are no longer planned (optional; careful)
  - [ ] Do NOT overwrite unmanaged files

- [ ] `--ci` semantics
  - [ ] In `--ci` mode:
    - [ ] drift is an error
    - [ ] collisions are errors
    - [ ] schema errors are errors

- [ ] Tests
  - [ ] Doctor passes on clean fixture
  - [ ] Doctor reports drift when a generated file is edited
  - [ ] Doctor `--fix` creates missing gitignore

## Verification
- [ ] `agents doctor --ci` fails on drift/collisions and succeeds when clean
