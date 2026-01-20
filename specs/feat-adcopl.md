# feat-adcopl: Copilot Adapter

Goal: Implement GitHub Copilot adapter to generate `.github/copilot-instructions.md` (required) and optional path-specific `*.instructions.md` outputs.

Depends on: feat-shared, feat-outputs
Unblocks: feat-adtest

## Deliverables
- `.agents/adapters/copilot/adapter.yaml`
- Templates for:
  - `.github/copilot-instructions.md`
  - optional scoped instruction files

## Implementation Plan
- [x] Confirm Copilot repository instruction support
  - [x] `.github/copilot-instructions.md` required
  - [x] Decide strategy for path-specific instructions:
    - [x] optional v1: emit from `.agents/scopes` into `.github/instructions/<scope>.instructions.md`

Notes:
- Repository-wide instructions live at `.github/copilot-instructions.md`.
- Path-specific instructions live under `.github/instructions/` and must be named `NAME.instructions.md`.
- Path-specific files require a YAML frontmatter block with `applyTo: "glob,glob"`.

- [x] Implement adapter YAML
  - [x] Output `.github/copilot-instructions.md`
    - [x] `format: md`
    - [x] stamp via comment
    - [x] include current mode banner
    - [x] collision: error (unique path)
  - [x] Optional outputs:
    - [x] `*.instructions.md` for selected scopes
    - [x] include `applyTo` frontmatter (stamp via `frontmatter`)
  - [x] Backend defaults: preferred `materialize`

- [x] Implement templates
  - [x] Base instructions file:
    - [x] composed prompts
    - [x] policy summary
    - [x] references to AGENTS.md shared surface if desired
  - [x] Scoped instructions:
    - [x] deterministic naming from scope id
    - [x] frontmatter applyTo list

- [x] Tests
  - [x] Golden fixture for copilot output
  - [x] Test deterministic naming and ordering of scope instruction files

## Verification
- [x] `agents preview --agent copilot` produces required outputs
