# feat-shared: Shared Surfaces Generator

Goal: Implement the core shared surface generator, primarily owning `shared:AGENTS.md`, used by Codex/OpenCode/Copilot (and optionally others). Must be deterministic and include a "current mode/profile" banner.

Depends on: feat-templ, feat-prompts, feat-stamps
Unblocks: feat-adcodx, feat-adopen, feat-adcopl

## Deliverables
- A built-in/shared adapter (or a special-cased generator) that can emit `AGENTS.md`.
- Stable template(s) and a clear collision policy (`shared_owner`).

## Implementation Plan
- [x] Decide implementation approach
  - [x] Option A: Create a special adapter id (e.g., `core`) under `.agents/adapters/core` embedded by init preset (rejected)
  - [x] Option B: Hardcode shared surface generation in code
  - [x] Prefer Option A to keep everything in `.agents/adapters/**` and testable via golden fixtures (rejected)

- [x] Define shared surface template requirements
  - [x] Top banner includes:
    - [x] generator stamp (feat-stamps)
    - [x] current mode, profile, policy, backend
    - [x] matched scopes summary
  - [x] Body includes:
    - [x] composed prompt guidance (base + project + selected snippets)
    - [x] safety policy summary (allow/deny highlights)
    - [x] optional setup/test commands section (placeholder)

- [x] Implement built-in adapter for shared surface
  - [x] Output:
    - [x] `path: AGENTS.md`
    - [x] `surface: shared:AGENTS.md`
    - [x] `collision: shared_owner`
    - [x] `renderer: template` (inline) using `AGENTS.md.hbs`
  - [x] Built-in `core` adapter is always available (manifest may still list it)

- [x] Rendering integration
  - [x] Ensure render context contains everything needed for shared surface:
    - [x] effective config
    - [x] composed prompts
    - [x] policy summary helper (optional; template prints a basic summary)

- [x] Collision ownership enforcement
  - [x] Ensure manifest `defaults.sharedSurfacesOwner` is honored
  - [x] If `sharedSurfacesOwner != core`, ensure either:
    - [x] another adapter owns the shared surface, or
    - [x] planning fails with actionable error

- [x] Tests
  - [x] Golden test rendering `AGENTS.md` for:
    - [x] single repo
    - [x] monorepo with scopes
    - [x] different modes/profiles
  - [x] Collision test: two adapters try to own `shared:AGENTS.md` => error

## Verification
- [x] `agents preview --agent core` uses shared `AGENTS.md`
- [x] `AGENTS.md` output is byte-identical across runs
