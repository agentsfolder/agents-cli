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

- [x] Implement adapter YAML
  - [x] Output `.gemini/settings.json`
    - [x] `format: json`
    - [x] stamp via `json_field`
    - [x] `writePolicy: if_generated`
  - [x] Backend defaults: preferred `vfs_container`, fallback `materialize`

- [x] Implement template
  - [x] Include:
    - [x] current mode banner (if Gemini surfaces support it) (via `AGENTS.md`)
    - [x] instruction content or references (via `context.fileName`)

- [x] Tests
  - [x] Golden fixture for gemini-cli output

## Verification
- [x] `agents preview --agent gemini-cli` produces `.gemini/settings.json`
