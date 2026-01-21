# feat-importr: `agents import --from <agent>`

Goal: Provide an explicit workflow to convert existing agent-native config into canonical `.agents/` artifacts.

Depends on: feat-models, feat-loadag
Unblocks: adoption in existing repos

## Deliverables
- `agents import --from <agent> [--path <path>]`.
- For v1, support importing at least one agent surface (start with Copilot or Cursor) and keep others as stubs.

## Implementation Plan
- [x] Define import framework
  - [ ] `Importer` trait:
    - [x] `agent_id()`
    - [x] `discover(repo_root) -> Option<ImportInputs>`
    - [x] `convert(inputs) -> CanonicalArtifacts`
  - [x] `CanonicalArtifacts` includes files to write under `.agents/**`

- [x] Implement import target selection
  - [x] Parse `--from <agent>`
  - [x] Determine input location:
    - [x] default known paths (e.g., `.github/copilot-instructions.md`)
    - [x] allow `--path` override

- [x] Implement first importer (recommend: Copilot)
  - [x] Read `.github/copilot-instructions.md`
  - [x] Convert to:
    - [x] `.agents/prompts/project.md` (or a snippet)
    - [x] create a mode that includes that snippet
  - [x] Generate/update manifest enabled sets as needed
  - [x] Do not overwrite existing `.agents` without confirmation

- [x] Dry-run and preview
  - [x] Add `--dry-run` to show what would be written
  - [x] Print diff-like summary

- [x] Validation
  - [x] After import, run schema validation on produced artifacts

- [ ] Tests
  - [ ] Fixture with a copilot instructions file imports into `.agents` and validates

## Verification
- [ ] `agents import --from copilot` produces valid `.agents` and `agents validate` passes
