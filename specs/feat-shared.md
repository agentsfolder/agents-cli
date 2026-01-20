# feat-shared: Shared Surfaces Generator

Goal: Implement the core shared surface generator, primarily owning `shared:AGENTS.md`, used by Codex/OpenCode/Copilot (and optionally others). Must be deterministic and include a "current mode/profile" banner.

Depends on: feat-templ, feat-prompts, feat-stamps
Unblocks: feat-adcodx, feat-adopen, feat-adcopl

## Deliverables
- A built-in/shared adapter (or a special-cased generator) that can emit `AGENTS.md`.
- Stable template(s) and a clear collision policy (`shared_owner`).

## Implementation Plan
- [ ] Decide implementation approach
  - [ ] Option A: Create a special adapter id (e.g., `core`) under `.agents/adapters/core` embedded by init preset
  - [x] Option B: Hardcode shared surface generation in code
  - [ ] Prefer Option A to keep everything in `.agents/adapters/**` and testable via golden fixtures.

- [ ] Define shared surface template requirements
  - [ ] Top banner includes:
    - [ ] generator stamp (feat-stamps)
    - [ ] current mode, profile, policy, backend
    - [ ] matched scopes summary
  - [ ] Body includes:
    - [ ] composed prompt guidance (base + project + selected snippets)
    - [ ] safety policy summary (allow/deny highlights)
    - [ ] optional setup/test commands section (placeholder)

- [ ] Implement adapter YAML for shared surface
  - [ ] Output:
    - [ ] `path: AGENTS.md`
    - [ ] `surface: shared:AGENTS.md`
    - [ ] `collision: shared_owner`
    - [ ] `renderer: template` pointing to `templates/AGENTS.md.hbs`
  - [ ] Ensure adapter is enabled in manifest for presets that need it

- [ ] Rendering integration
  - [ ] Ensure render context contains everything needed for shared surface:
    - [ ] effective config
    - [ ] composed prompts
    - [ ] policy summary helper (optional)

- [ ] Collision ownership enforcement
  - [ ] Ensure manifest `defaults.sharedSurfacesOwner` is honored
  - [ ] If `sharedSurfacesOwner != core`, ensure either:
    - [ ] another adapter owns the shared surface, or
    - [ ] planning fails with actionable error

- [ ] Tests
  - [ ] Golden test rendering `AGENTS.md` for:
    - [ ] single repo
    - [ ] monorepo with scopes
    - [ ] different modes/profiles
  - [ ] Collision test: two adapters try to own `shared:AGENTS.md` => error

## Verification
- [ ] `agents preview --agent codex` (later) uses shared `AGENTS.md`
- [ ] `AGENTS.md` output is byte-identical across runs
