# feat-schemas: JSON Schema Validation

Goal: Validate all canonical `.agents` authoring files against JSON Schemas stored in `.agents/schemas/**`.

Depends on: feat-models, feat-loadag
Unblocks: feat-doctor, feat-adtest

## Deliverables
- `agents validate` validates:
  - `.agents/manifest.yaml`
  - `.agents/policies/*.yaml`
  - `.agents/skills/*/skill.yaml`
  - `.agents/scopes/*.yaml`
  - `.agents/adapters/*/adapter.yaml`
  - `.agents/state/state.yaml` (if present)
  - mode frontmatter (if present)
- Validation errors include file path + schema name + a helpful pointer.

## Implementation Plan
- [x] Choose schema validation library
  - [x] Evaluate `jsonschema` crate (draft 2020-12 support)
  - [x] Confirm it supports:
    - [x] basic types, required, enums
    - [x] additionalProperties false
    - [x] defaults are not required to be applied (OK)

- [x] Implement schema loader
  - [x] Load schema JSON from `.agents/schemas/*.schema.json`
  - [x] Cache compiled schemas in memory per process
  - [x] Map canonical file types to schema filenames

- [x] Implement YAML/MD -> JSON conversion
  - [x] For YAML files: parse YAML -> `serde_json::Value`
  - [x] For mode frontmatter: serialize `ModeFrontmatter` to JSON value
  - [x] For state: YAML -> JSON

- [ ] Validate each file type
  - [ ] Manifest: `.agents/manifest.yaml`
  - [ ] Policies: all `.agents/policies/*.yaml`
  - [ ] Skills: all `.agents/skills/*/skill.yaml`
  - [ ] Scopes: all `.agents/scopes/*.yaml`
  - [ ] Adapters: all `.agents/adapters/*/adapter.yaml`
  - [ ] State: `.agents/state/state.yaml` if present
  - [ ] Modes: validate frontmatter if present, skip if absent

- [ ] Error shaping
  - [ ] Translate validator output into:
    - [ ] `SchemaInvalid { path, schema, pointer, message }`
  - [ ] Prefer stable, readable pointers (JSON Pointer) for where the error occurred
  - [ ] Include "hint" lines for common issues (unknown enum, missing required field)

- [ ] Integrate into `agents validate`
  - [ ] `validate` runs:
    - [ ] load repo config (feat-loadag)
    - [ ] schema validation (this feature)
    - [ ] referential integrity checks (can remain in load stage or here)

- [ ] Tests
  - [ ] Valid fixture passes
  - [ ] Invalid fixture fails with:
    - [ ] correct file path
    - [ ] correct schema
    - [ ] a non-empty pointer/message

## Verification
- [ ] `agents validate` fails fast on a deliberately broken `.agents/policies/*.yaml`
- [ ] `agents validate` succeeds on the init preset output (once feat-initpr exists)
