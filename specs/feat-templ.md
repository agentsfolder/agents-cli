# feat-templ: Template Rendering Engine (Handlebars + Helpers)

Goal: Provide deterministic rendering of adapter templates using a stable render context and required helpers from the PRD.

Depends on: feat-prompts
Unblocks: feat-outputs, feat-shared, all adapters

## Deliverables
- `TemplateEngine` wrapping Handlebars in strict mode.
- Required helpers implemented:
  - `indent`, `join`, `toJson`, `toJsonc`, `toYaml`, `frontmatter`, `generatedStamp`
- Stable serialization for JSON/YAML.

## Implementation Plan
- [x] Choose and configure template engine
  - [x] Use `handlebars` crate
  - [x] Enable strict mode / error on missing variables
  - [x] Register partials from adapter `templates/**` directory
  - [x] Define template lookup conventions (paths relative to templates dir)

- [x] Define standard render context
  - [x] `RenderContext` struct serializable via `serde`:
    - [x] `effective.mode` (frontmatter + body)
    - [x] `effective.policy`
    - [x] `effective.skills` (IDs + optional summaries)
    - [x] `effective.prompts` (base/project/snippets/composed)
    - [x] `profile`
    - [x] `scopesMatched` (ids + metadata)
    - [x] `generation.stamp` (meta)
    - [x] `adapter.agentId`
  - [x] Ensure fields are stable (avoid hashmaps; prefer BTreeMap)

- [x] Implement helpers
  - [x] `indent(text, n)`
    - [x] preserve trailing newline behavior
    - [x] handle multi-line input deterministically
  - [x] `join(list, sep)`
    - [x] stringify list items deterministically
  - [x] `toJson(obj)`
    - [x] stable ordering (convert maps to BTreeMap before serialize)
    - [x] compact or pretty? decide and standardize
  - [x] `toJsonc(obj)`
    - [x] output JSON text; JSONC differences are usually stamping comments
  - [x] `toYaml(obj)`
    - [x] stable key ordering
    - [x] no anchors, deterministic formatting
  - [x] `frontmatter(obj)`
    - [x] emit `---
<yaml>---
` deterministically
  - [x] `generatedStamp(meta)`
    - [x] produce deterministic stamp block used by feat-stamps

- [x] Template loading and caching
  - [x] Load template files under `.agents/adapters/<id>/templates/**`
  - [x] Cache compiled templates per adapter per run
  - [x] Ensure deterministic partial registration order (sorted paths)

- [x] Output normalization
  - [x] Normalize newlines to `
` after render
  - [x] Ensure trailing newline for text/markdown outputs

- [x] Tests
  - [x] Unit tests for each helper with snapshot outputs
  - [x] Integration test: render a minimal adapter template against a fixture context
  - [x] Determinism test: render twice and compare bytes


## Verification
- [x] `agents preview` (once available) renders templates without missing-variable surprises (covered by preview missing-var test)
- [x] Helper outputs are byte-identical across runs
