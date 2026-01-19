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
- [ ] Confirm Copilot repository instruction support
  - [ ] `.github/copilot-instructions.md` required
  - [ ] Decide strategy for path-specific instructions:
    - [ ] optional v1: emit from `.agents/scopes` if supported

- [ ] Implement adapter YAML
  - [ ] Output `.github/copilot-instructions.md`
    - [ ] `format: md`
    - [ ] stamp via comment
    - [ ] include current mode banner
    - [ ] collision: error (unique path)
  - [ ] Optional outputs:
    - [ ] `*.instructions.md` for selected scopes
    - [ ] include `applyTo` frontmatter if required by Copilot format
  - [ ] Backend defaults: preferred `materialize`

- [ ] Implement templates
  - [ ] Base instructions file:
    - [ ] composed prompts
    - [ ] policy summary
    - [ ] references to AGENTS.md shared surface if desired
  - [ ] Scoped instructions:
    - [ ] deterministic naming from scope id
    - [ ] frontmatter applyTo list

- [ ] Tests
  - [ ] Golden fixture for copilot output
  - [ ] Test deterministic naming and ordering of scope instruction files

## Verification
- [ ] `agents preview --agent copilot` produces required outputs
