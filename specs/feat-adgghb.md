# feat-adgghb: Gemini GitHub Adapter

Goal: Implement Gemini Code Assist for GitHub adapter output `.gemini/styleguide.md` (required).

Depends on: feat-templ
Unblocks: feat-adtest

## Deliverables
- `.agents/adapters/gemini-github/adapter.yaml`
- Template for `.gemini/styleguide.md`

## Implementation Plan
- [ ] Confirm GitHub Gemini expectations
  - [ ] Determine whether `.gemini/config.yaml` is needed (optional)

- [ ] Implement adapter YAML
  - [ ] Output `.gemini/styleguide.md`
    - [ ] `format: md`
    - [ ] stamp via comment
    - [ ] `writePolicy: if_generated`
  - [ ] Backend defaults: preferred `materialize` (GitHub reads repo files)

- [ ] Implement template
  - [ ] Include:
    - [ ] current mode banner
    - [ ] coding conventions + composed prompt content

- [ ] Tests
  - [ ] Golden fixture for `.gemini/styleguide.md`

## Verification
- [ ] `agents preview --agent gemini-github` produces `.gemini/styleguide.md`
