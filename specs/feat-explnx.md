# feat-explnx: `agents explain <path>` (Source Map)

Goal: Explain how a generated output was produced: which prompts/snippets/mode/policy/skills and which adapter template contributed.

Depends on: feat-outputs, feat-prompts
Unblocks: debugging and trust

## Deliverables
- `agents explain <path>` prints a source map:
  - selected adapter
  - output definition (surface/path)
  - template used
  - contributing prompt files and snippet IDs
  - effective mode/policy/profile/scopes

## Implementation Plan
- [ ] Decide where to store source maps
  - [ ] Option A: embed in stamp metadata (limited)
  - [x] Option B: write sidecar file under `.agents/state/` (gitignored) during preview/sync
  - [x] Prefer B: `.agents/state/explain/<hash>.json` keyed by output path

- [ ] Implement source map generation
  - [ ] During planning/rendering, build `SourceMap`:
    - [x] output path
    - [x] adapter id
    - [x] renderer type and template path
    - [x] mode/policy/profile/backend
    - [x] scopes matched
    - [x] prompt source file paths
    - [x] enabled skills
  - [ ] Persist source map when:
    - [x] `agents preview`
    - [x] `agents sync`

- [ ] Implement explain lookup
  - [x] Input: a path in repo
  - [x] Find matching stored source map by normalized repo-relative path
  - [ ] If not found:
    - [x] attempt to parse stamp and provide minimal explanation
    - [x] else report "unmanaged file"

- [ ] Render explain output
  - [ ] Human-readable stable format
  - [ ] Optionally `--json`

- [ ] Tests
  - [ ] After preview/sync, explain returns expected components
  - [ ] Unmanaged path returns helpful message

## Verification
- [ ] `agents explain AGENTS.md` prints contributing sources
