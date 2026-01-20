# feat-matwiz: Materialize Backend

Goal: Write planned outputs into the repo filesystem deterministically, honoring adapter `writePolicy` and optionally updating `.gitignore`.

Depends on: feat-outputs, feat-stamps
Unblocks: feat-syncer, feat-cleanup

## Deliverables
- `MaterializeBackend::apply(plan, rendered_outputs) -> Result<ApplyReport>`
- Honors write policy modes:
  - `always`, `if_generated`, `never`
- Optional `.gitignore` update when adapter requests.

## Implementation Plan
- [x] Define backend interface
  - [x] `Backend` trait:
    - [x] `prepare(plan) -> Result<BackendSession>` (optional)
    - [x] `apply(session, outputs) -> Result<ApplyReport>`
  - [x] `RenderedOutput { path, bytes, stamp_meta, drift_status }`
  - [x] `ApplyReport { written: Vec<Path>, skipped: Vec<Path>, conflicts: Vec<Path> }`


- [x] Implement safe write behavior
  - [x] Ensure parent directories exist
  - [x] Use feat-fsutil `atomic_write`
  - [x] Ensure newline normalization for text formats

- [x] Implement writePolicy handling
  - [x] `always`:
    - [x] overwrite unconditionally
  - [x] `if_generated`:
    - [x] if file does not exist: write
    - [x] if file exists and has valid stamp from this generator: overwrite
    - [x] if file exists and is unmanaged: refuse and report conflict
  - [x] `never`:
    - [x] never write; report skipped

- [ ] Implement conflict reporting
  - [ ] Provide actionable error:
    - [ ] which path
    - [ ] why blocked (unmanaged, drifted)
    - [ ] suggested next step (`agents diff`, `agents doctor`, or change collision/writePolicy)

- [ ] `.gitignore` integration
  - [ ] If output writePolicy sets `gitignore=true`:
    - [ ] Add entry for the output path (repo-relative)
    - [ ] Ensure deterministic ordering within `.gitignore`
    - [ ] Avoid duplicate entries
    - [ ] Do not modify `.gitignore` if it is unmanaged? (decide; default safe)

- [ ] Permissions and file modes
  - [ ] Preserve executable bit only if explicitly needed (likely no for generated config)
  - [ ] Ensure Windows compatibility (no unix-only perms assumptions)

- [ ] Tests
  - [ ] Write new file
  - [ ] Overwrite stamped file with `if_generated`
  - [ ] Refuse overwrite unmanaged file with `if_generated`
  - [ ] `.gitignore` update stable and idempotent

## Verification
- [ ] `agents sync --backend materialize` writes expected outputs
- [ ] Rerun sync yields no diffs (`agents diff` is clean)
