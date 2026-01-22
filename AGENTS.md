# AGENTS

This repository implements `agents`, a Rust CLI + core library for projecting canonical `.agents/` configuration into agent-native surfaces (Copilot, Cursor, Codex, Claude Code, OpenCode, etc.).

The source of truth for work is in `specs/*.md` (task lists). Agents should implement specs one checkbox at a time and keep those files accurate.

## How To Work In This Repo

Development loop (required)
- Never work directly on `main`.
- Create a feature branch per spec: `git checkout -b feat/<spec-slug>`.
- Pick the next unchecked item in the relevant `specs/feat-*.md`.
- Before coding user-requested changes: update the relevant `specs/` file(s) first (checklists/notes), then implement.
- When adding a new spec, update `specs/plan.md` and this AGENTS specs index.
- Implement exactly one task; update the corresponding checkbox(es) in the spec.
- Run the smallest relevant test set (`cargo test -p agents-core`, `cargo test -p agents-cli`, or a targeted `cargo test -p ... --test ...`).
- Commit after each completed task.
- Repeat until the entire spec file is fully complete.
- When done: squash-merge locally into `main` (no PR).

Suggested squash merge sequence
```bash
git checkout main
git merge --squash feat/<spec-slug>
git commit -m "<spec>: <short summary>"
git branch -d feat/<spec-slug>
```

## Specs Index (Where Things Live)

Reference docs
- `specs/prd.md`: product requirements and canonical schema/UX expectations (not a checklist)
- `specs/architecture.md`: overall architecture, crates, and key flows
- `specs/plan.md`: high-level sequencing across specs

Foundation
- `specs/feat-cliapp.md`: CLI skeleton, exit codes, global flags
- `specs/feat-fsutil.md`: deterministic path utilities + repo root discovery
- `specs/feat-models.md`: data model for manifest/policy/mode/scope/adapter
- `specs/feat-schemas.md`: JSON schema validation system (`.agents/schemas/*.schema.json`)
- `specs/feat-loadag.md`: load `.agents/**` into memory; referential integrity
- `specs/feat-resolv.md`: resolution engine (defaults + scopes + state + CLI overrides)
- `specs/feat-prompts.md`: prompt composition + redaction
- `specs/feat-templ.md`: handlebars template engine + helpers
- `specs/feat-skillpl.md`: compute enabled skills (ordering + backend compatibility)

Outputs, Drift, Sync
- `specs/feat-stamps.md`: stamp formats + hashing + drift classification
- `specs/feat-driftx.md`: diff engine (planned vs materialized)
- `specs/feat-outputs.md`: output planning (conditions, collisions, renderer sources)
- `specs/feat-prevdf.md`: `agents preview` + `agents diff`
- `specs/feat-matwiz.md`: materialize backend implementation
- `specs/feat-syncer.md`: `agents sync` command (apply plan via backend)
- `specs/feat-cleanup.md`: `agents clean` + cleanup identification/deletion

UX / Diagnostics
- `specs/feat-status.md`: `agents status` effective config reporting
- `specs/feat-doctor.md`: `agents doctor [--fix]` health checks
- `specs/feat-explnx.md`: `agents explain <path>` via source-map sidecars + stamp fallback

Bootstrap / Migration
- `specs/feat-initpr.md`: `agents init --preset ...` with embedded assets
- `specs/feat-importr.md`: `agents import --from <agent>` (currently: copilot)

Runtimes / Execution
- `specs/feat-runner.md`: `agents run <agent>` orchestration (backend + exec)
- `specs/feat-vfsctr.md`: `vfs_container` backend (docker overlay)
- `specs/feat-vfsmnt.md`: `vfs_mount` backend (v1.1)
- `specs/feat-compat.md`: `agents compat` compatibility matrix

Adapters (agent-native surfaces)
- `specs/feat-shared.md`: shared surfaces (AGENTS.md) generator
- `specs/feat-adtest.md`: adapter golden fixture runner (`agents test adapters`)
- `specs/feat-adopen.md`: OpenCode adapter (`opencode.jsonc` + shared surface)
- `specs/feat-adcodx.md`: Codex adapter (`AGENTS.md`)
- `specs/feat-adcopl.md`: Copilot adapter (`.github/copilot-instructions.md` + scoped instructions)
- `specs/feat-adclde.md`: Claude Code adapter (`.claude/settings.json` + `CLAUDE.md`)
- `specs/feat-adcurs.md`: Cursor adapter (`.cursor/rules/*.md`)
- `specs/feat-adgcli.md`: Gemini CLI adapter (planned)
- `specs/feat-adgghb.md`: Gemini GitHub adapter (planned)

Maintenance
- `specs/feat-prodci.md`: production CI hardening (fmt/clippy/tests)
- `specs/feat-npmpub.md`: npm packaging for CLI distribution

## Repo-Specific Learnings / Conventions

Output planning and collisions
- Logical shared surfaces use `surface: <name>` with `collision: shared_owner`; ownership is controlled by `manifest.defaults.sharedSurfacesOwner` (defaults to `core`).
- Physical path collisions always error.
- Renderer `sources` are validated during planning (supports `template:<name>`, `file:`/`repo:` repo-relative existing files, and `prompt:*`/`snippet:*`).

Shared `AGENTS.md`
- A built-in `core` adapter exists in code (not on disk) and can render `AGENTS.md` via an inline template.

Explain sidecars
- `agents preview` and `agents sync` persist explain maps under `.agents/state/explain/<sha256(repo_rel_path)>.json`.

Scopes and per-scope outputs
- Output paths may include `{{scopeId}}`; planning expands these to one output per scope id in deterministic order.
- Templates can use `scope.id` and `scope.applyTo` when rendered for a scope-expanded output.

Fixture runner
- `agents test adapters` runs golden fixtures under `fixtures/*`.
- Update goldens with: `AGENTS_UPDATE_GOLDENS=1 cargo run -p agents-cli -- test adapters --update --agent <id>`.
- Fixture matrices support `targetPath`, `mode`, `profile`, `backend`.

Write safety
- Prefer `writePolicy.mode: if_generated` for agent-native surfaces; materialize will refuse to overwrite unmanaged or drifted files.
- Keep outputs deterministic: stable ordering, stable whitespace, and avoid CRLF.
