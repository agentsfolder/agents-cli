# feat-adcurs: Cursor Adapter

Goal: Implement Cursor adapter to generate deterministic `.cursor/rules/*.md` files, avoiding diff churn.

Depends on: feat-templ, feat-outputs
Unblocks: feat-adtest

## Deliverables
- `.agents/adapters/cursor/adapter.yaml`
- One or more templates producing rule files under `.cursor/rules/`.

## Implementation Plan
- [x] Determine rule file breakdown
  - [x] Decide categories:
    - [x] `00-current-mode.md`
    - [x] `10-guidance.md`
    - [x] `20-policy.md`
  - [x] Ensure naming is deterministic and stable

- [x] Implement adapter YAML
  - [ ] Outputs:
    - [x] `.cursor/rules/00-current-mode.md`
    - [x] `.cursor/rules/10-guidance.md` (composed prompts)
    - [x] `.cursor/rules/20-policy.md` (policy summary)
  - [x] `writePolicy: if_generated`
  - [x] stamp via comment
  - [x] Backend defaults: preferred `materialize`

- [ ] Implement templates
  - [ ] Each file includes:
    - [ ] stamp
    - [ ] current mode banner
    - [ ] deterministic section headers
  - [ ] Ensure stable ordering of snippets and lists

- [ ] Tests
  - [ ] Golden fixture for cursor outputs
  - [ ] Determinism test: repeated renders produce identical filenames and content

## Verification
- [ ] `agents diff --agent cursor` is stable across runs
