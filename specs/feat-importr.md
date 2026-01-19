# feat-importr: `agents import --from <agent>`

Goal: Provide an explicit workflow to convert existing agent-native config into canonical `.agents/` artifacts.

Depends on: feat-models, feat-loadag
Unblocks: adoption in existing repos

## Deliverables
- `agents import --from <agent> [--path <path>]`.
- For v1, support importing at least one agent surface (start with Copilot or Cursor) and keep others as stubs.

## Implementation Plan
- [ ] Define import framework
  - [ ] `Importer` trait:
    - [ ] `agent_id()`
    - [ ] `discover(repo_root) -> Option<ImportInputs>`
    - [ ] `convert(inputs) -> CanonicalArtifacts`
  - [ ] `CanonicalArtifacts` includes files to write under `.agents/**`

- [ ] Implement import target selection
  - [ ] Parse `--from <agent>`
  - [ ] Determine input location:
    - [ ] default known paths (e.g., `.github/copilot-instructions.md`)
    - [ ] allow `--path` override

- [ ] Implement first importer (recommend: Copilot)
  - [ ] Read `.github/copilot-instructions.md`
  - [ ] Convert to:
    - [ ] `.agents/prompts/project.md` (or a snippet)
    - [ ] create a mode that includes that snippet
  - [ ] Generate/update manifest enabled sets as needed
  - [ ] Do not overwrite existing `.agents` without confirmation

- [ ] Dry-run and preview
  - [ ] Add `--dry-run` to show what would be written
  - [ ] Print diff-like summary

- [ ] Validation
  - [ ] After import, run schema validation on produced artifacts

- [ ] Tests
  - [ ] Fixture with a copilot instructions file imports into `.agents` and validates

## Verification
- [ ] `agents import --from copilot` produces valid `.agents` and `agents validate` passes
