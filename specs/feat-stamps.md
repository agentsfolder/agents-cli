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
- [ ] Define stamp metadata
  - [ ] `StampMeta` includes:
    - [ ] generator id ("agents")
    - [ ] adapter agentId
    - [ ] manifest specVersion
    - [ ] effective mode/profile/policy/backend
    - [ ] timestamp? (avoid; keep deterministic) => do NOT include wall clock time
    - [ ] content hash (sha256 of canonical content)

- [ ] Define stamp encodings by stamp type
  - [ ] `comment` (text/md):
    - [ ] a clearly delimited block at top of file
  - [ ] `frontmatter` (md):
    - [ ] inject into YAML frontmatter under a reserved key (e.g., `x_generated`)
  - [ ] `json_field` (json/jsonc):
    - [ ] inject an `"x_generated"` object field

- [ ] Implement stamping functions
  - [ ] `strip_existing_stamp(content) -> (stripped, found_stamp)`
  - [ ] `apply_stamp(content_stripped, meta, method) -> stamped_content`
  - [ ] Ensure stamping is idempotent and deterministic

- [ ] Implement hashing
  - [ ] Use `sha2::Sha256`
  - [ ] Hash canonical content *without* stamp
  - [ ] Normalize newlines before hashing

- [ ] Implement drift detection
  - [ ] `classify(path, planned_content_without_stamp) -> DriftStatus`
    - [ ] if file missing: `Missing`
    - [ ] if no valid stamp: `Unmanaged`
    - [ ] if stamp present and hash matches planned: `Clean`
    - [ ] if stamp present and hash differs: `Drifted`
  - [ ] Allow adapter setting `mtime_only` and `none` (honor but keep sha256 default)

- [ ] Tests
  - [ ] Stamp parse/apply round-trip tests for each method
  - [ ] Drift classification tests
  - [ ] Determinism test: stamping same inputs yields identical bytes

## Verification
- [ ] Generated files contain valid stamp blocks/fields
- [ ] Manual edits are detected as drift
