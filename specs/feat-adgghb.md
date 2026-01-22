# feat-adgghb: Gemini GitHub Adapter

Goal: Implement Gemini Code Assist for GitHub adapter output `.gemini/styleguide.md` (required).

Depends on: feat-templ
Unblocks: feat-adtest

## Deliverables
- `.agents/adapters/gemini-github/adapter.yaml`
- Template for `.gemini/styleguide.md`

## Implementation Plan
- [x] Confirm GitHub Gemini expectations
  - [x] Determine whether `.gemini/config.yaml` is needed (optional; not required for v1)

- [x] Implement adapter YAML
  - [x] Output `.gemini/styleguide.md`
    - [x] `format: md`
    - [x] stamp via comment
    - [x] `writePolicy: if_generated`
  - [x] Backend defaults: preferred `materialize` (GitHub reads repo files)

- [x] Implement template
  - [x] Include:
    - [x] current mode banner
    - [x] coding conventions + composed prompt content

- [x] Tests
  - [x] Golden fixture for `.gemini/styleguide.md`

## Verification
- [x] `agents preview --agent gemini-github` produces `.gemini/styleguide.md`
