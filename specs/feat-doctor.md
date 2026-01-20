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
- [x] Define check framework
  - [x] `DoctorCheck` trait with:
    - [x] `name`
    - [x] `run(ctx) -> CheckResult`
    - [x] optional `fix(ctx) -> FixResult`
  - [x] `DoctorReport` with:
    - [x] errors, warnings, infos
    - [x] exit code mapping (`--ci` makes warnings fail if desired)


- [ ] Implement checks
  - [x] Schema check
    - [x] run the same validation as `agents validate`
  - [x] Collision check
    - [x] plan outputs for each enabled adapter (or selected agent)
    - [x] surface/path collisions reported with details
  - [x] Drift check
    - [x] for each adapter, compute planned outputs and compare to disk
    - [x] report drifted files and unmanaged conflicts
  - [x] Prerequisites check
    - [x] if any adapter/backends require docker, confirm docker available
  - [x] State file check
    - [x] ensure `.agents/state/.gitignore` exists and ignores `state.yaml`

- [x] Implement safe fixes (`--fix`)
  - [x] Create missing `.agents/state/.gitignore`
  - [x] Remove stale generated files that are no longer planned (optional; careful)
  - [x] Do NOT overwrite unmanaged files

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
