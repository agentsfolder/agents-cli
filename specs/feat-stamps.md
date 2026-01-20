# feat-stamps: Generated Stamps + Drift Metadata

Goal: Provide deterministic stamping of generated files and a drift model based on stamp + sha256.

Depends on: feat-outputs
Unblocks: feat-driftx, feat-cleanup, feat-doctor

## Deliverables
- A canonical stamp format and functions:
  - `stamp_rendered_output(format, content, meta) -> content_with_stamp`
  - `parse_stamp(content) -> Option<Stamp>`
  - `compute_hash(content_without_stamp) -> sha256`
- Drift detection that can classify files as:
  - not-generated
  - generated-and-clean
  - generated-but-drifted

## Implementation Plan
- [x] Define stamp metadata
  - [x] `StampMeta` includes:
    - [x] generator id ("agents")
    - [x] adapter agentId
    - [x] manifest specVersion
    - [x] effective mode/profile/policy/backend
    - [x] timestamp? (avoid; keep deterministic) => do NOT include wall clock time
    - [x] content hash (sha256 of canonical content)

- [x] Define stamp encodings by stamp type
  - [x] `comment` (text/md):
    - [x] a clearly delimited block at top of file
  - [x] `frontmatter` (md):
    - [x] inject into YAML frontmatter under a reserved key (e.g., `x_generated`)
  - [x] `json_field` (json/jsonc):
    - [x] inject an `"x_generated"` object field

- [x] Implement stamping functions
  - [x] `strip_existing_stamp(content) -> (stripped, found_stamp)`
  - [x] `apply_stamp(content_stripped, meta, method) -> stamped_content`
  - [x] Ensure stamping is idempotent and deterministic

- [x] Implement hashing
  - [x] Use `sha2::Sha256`
  - [x] Hash canonical content *without* stamp
  - [x] Normalize newlines before hashing

- [x] Implement drift detection
  - [x] `classify(path, planned_content_without_stamp) -> DriftStatus`
    - [x] if file missing: `Missing`
    - [x] if no valid stamp: `Unmanaged`
    - [x] if stamp present and hash matches planned: `Clean`
    - [x] if stamp present and hash differs: `Drifted`
  - [x] Allow adapter setting `mtime_only` and `none` (honor but keep sha256 default)

- [x] Tests
  - [x] Stamp parse/apply round-trip tests for each method
  - [x] Drift classification tests
  - [x] Determinism test: stamping same inputs yields identical bytes

## Verification
- [x] Generated files contain valid stamp blocks/fields
- [x] Manual edits are detected as drift
