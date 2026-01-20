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

- [x] Implement conflict reporting
  - [x] Provide actionable error:
    - [x] which path
    - [x] why blocked (unmanaged, drifted)
    - [x] suggested next step (`agents diff`, `agents doctor`, or change collision/writePolicy)

- [x] `.gitignore` integration
  - [x] If output writePolicy sets `gitignore=true`:
    - [x] Add entry for the output path (repo-relative)
    - [x] Ensure deterministic ordering within `.gitignore`
    - [x] Avoid duplicate entries
    - [x] Do not modify `.gitignore` if it is unmanaged? (decide; default safe)

- [x] Permissions and file modes
  - [x] Preserve executable bit only if explicitly needed (likely no for generated config)
  - [x] Ensure Windows compatibility (no unix-only perms assumptions)

- [x] Tests
  - [x] Write new file
  - [x] Overwrite stamped file with `if_generated`
  - [x] Refuse overwrite unmanaged file with `if_generated`
  - [x] `.gitignore` update stable and idempotent

## Verification
- [ ] `agents sync --backend materialize` writes expected outputs
- [ ] Rerun sync yields no diffs (`agents diff` is clean)
