# feat-adcurs: Cursor Adapter

Goal: Implement Cursor adapter to generate deterministic `.cursor/rules/*.md` files, avoiding diff churn.

Depends on: feat-templ, feat-outputs
Unblocks: feat-adtest

## Deliverables
- `.agents/adapters/cursor/adapter.yaml`
- One or more templates producing rule files under `.cursor/rules/`.

## Implementation Plan
- [ ] Determine rule file breakdown
  - [ ] Decide categories (e.g., `00-mode.md`, `10-style.md`, `20-testing.md`, etc.)
  - [ ] Ensure naming is deterministic and stable

- [ ] Implement adapter YAML
  - [ ] Outputs:
    - [ ] `.cursor/rules/00-current-mode.md`
    - [ ] `.cursor/rules/10-guidance.md` (composed prompts)
    - [ ] `.cursor/rules/20-policy.md` (policy summary)
  - [ ] `writePolicy: if_generated`
  - [ ] stamp via comment
  - [ ] Backend defaults: preferred `materialize`

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
