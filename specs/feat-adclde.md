# feat-adclde: Claude Code Adapter

Goal: Implement Claude Code adapter outputs: `.claude/settings.json` (required) and optional `CLAUDE.md`.

Depends on: feat-templ, feat-outputs
Unblocks: feat-adtest

## Deliverables
- `.agents/adapters/claude/adapter.yaml`
- Templates:
  - `.claude/settings.json` (template or json_merge)
  - optional `CLAUDE.md.hbs`

## Implementation Plan
- [ ] Confirm required Claude Code settings surface
  - [ ] Document expected keys in `.claude/settings.json` for v1

- [ ] Implement adapter YAML
  - [ ] Output `.claude/settings.json`
    - [ ] `format: json`
    - [ ] `renderer: json_merge` or `template`
    - [ ] `writePolicy: if_generated`
    - [ ] stamp via `json_field`
  - [ ] Optional output `CLAUDE.md`
    - [ ] `format: md`
    - [ ] includes current mode banner and composed prompts
  - [ ] Backend defaults: preferred `vfs_container`, fallback `materialize`

- [ ] Implement settings mapping
  - [ ] Minimal v1 mapping:
    - [ ] project instructions path or embedded instructions if supported
    - [ ] any official project settings needed (keep conservative)
  - [ ] Keep unknown keys out; deterministic formatting

- [ ] Tests
  - [ ] Golden fixture for `.claude/settings.json`
  - [ ] Ensure stamp present and drift detectable

## Verification
- [ ] `agents preview --agent claude` produces `.claude/settings.json` deterministically
