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

- [ ] Implement helpers
  - [ ] `indent(text, n)`
    - [ ] preserve trailing newline behavior
    - [ ] handle multi-line input deterministically
  - [ ] `join(list, sep)`
    - [ ] stringify list items deterministically
  - [ ] `toJson(obj)`
    - [ ] stable ordering (convert maps to BTreeMap before serialize)
    - [ ] compact or pretty? decide and standardize
  - [ ] `toJsonc(obj)`
    - [ ] output JSON text; JSONC differences are usually stamping comments
  - [ ] `toYaml(obj)`
    - [ ] stable key ordering
    - [ ] no anchors, deterministic formatting
  - [ ] `frontmatter(obj)`
    - [ ] emit `---\n<yaml>---\n` deterministically
  - [ ] `generatedStamp(meta)`
    - [ ] produce deterministic stamp block used by feat-stamps

- [ ] Template loading and caching
  - [ ] Load template files under `.agents/adapters/<id>/templates/**`
  - [ ] Cache compiled templates per adapter per run
  - [ ] Ensure deterministic partial registration order (sorted paths)

- [ ] Output normalization
  - [ ] Normalize newlines to `\n` after render
  - [ ] Ensure trailing newline for text/markdown outputs

- [ ] Tests
  - [ ] Unit tests for each helper with snapshot outputs
  - [ ] Integration test: render a minimal adapter template against a fixture context
  - [ ] Determinism test: render twice and compare bytes

## Verification
- [ ] `agents preview` (once available) renders templates without missing-variable surprises
- [ ] Helper outputs are byte-identical across runs
