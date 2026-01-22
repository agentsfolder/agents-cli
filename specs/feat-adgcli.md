# feat-adgcli: Gemini CLI Adapter

Goal: Implement Gemini CLI adapter output `.gemini/settings.json`.

Depends on: feat-templ, feat-outputs
Unblocks: feat-adtest

## Deliverables
- `.agents/adapters/gemini-cli/adapter.yaml`
- Template for `.gemini/settings.json`

## Implementation Plan
- [x] Confirm Gemini CLI project settings format
  - [x] Identify minimal config keys used by Gemini CLI (`context.fileName` + `$schema`)
  - [x] Decide whether to embed instructions directly or point to a file (point to shared `AGENTS.md`)

- [ ] Implement adapter YAML
  - [ ] Output `.gemini/settings.json`
    - [ ] `format: json`
    - [ ] stamp via `json_field`
    - [ ] `writePolicy: if_generated`
  - [ ] Backend defaults: preferred `vfs_container`, fallback `materialize`

- [ ] Implement template
  - [ ] Include:
    - [ ] current mode banner (if Gemini surfaces support it)
    - [ ] instruction content or references

- [ ] Tests
  - [ ] Golden fixture for gemini-cli output

## Verification
- [ ] `agents preview --agent gemini-cli` produces `.gemini/settings.json`
