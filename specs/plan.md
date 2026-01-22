# Master Plan: Multi-phase Implementation

This plan decomposes the PRD into independently buildable and verifiable features. Each feature has an identifier `feat-<shortcode>` (6-10 chars) and lists its dependencies and what it unblocks.

Conventions:
- Verification commands are illustrative; the exact CLI flags may evolve.
- "Unblocks" means subsequent features can be implemented with low rework.

## Phase 0: CLI Foundation
### feat-cliapp
- Goal: CLI skeleton with subcommands, help text, structured errors, logging.
- Depends on: (none)
- Unblocks: feat-models, feat-initpr, feat-status
- Verify:
  - `agents --help` lists commands
  - `agents validate` returns a clear "not initialized" error on empty repo

### feat-fsutil
- Goal: Cross-platform path utilities, repo root discovery, stable read/write helpers.
- Depends on: feat-cliapp
- Unblocks: feat-loadag, feat-matwiz, feat-prevdf
- Verify:
  - Unit tests for path normalization and stable newline handling

## Phase 1: Canonical Spec Loading + Validation
### feat-models
- Goal: Typed Rust data model for manifest/policy/skill/scope/adapter/state + mode frontmatter.
- Depends on: feat-cliapp
- Unblocks: feat-loadag, feat-schemas, feat-resolv
- Verify:
  - Deserialize representative YAML/MD frontmatter fixtures

### feat-loadag
- Goal: Load `.agents/**` tree and build an in-memory `RepoConfig`.
- Depends on: feat-fsutil, feat-models
- Unblocks: feat-schemas, feat-resolv, feat-initpr
- Verify:
  - `agents status` prints repo discovery results (even before full resolution)

### feat-schemas
- Goal: JSON Schema validation for all canonical files; fail fast on invalid shapes.
- Depends on: feat-models, feat-loadag
- Unblocks: feat-doctor, feat-adtest
- Verify:
  - `agents validate` fails on a broken fixture and points to file+schema

## Phase 2: Resolution Engine
### feat-resolv
- Goal: Implement PRD precedence + deterministic deep-merge + scope specificity.
- Depends on: feat-loadag, feat-schemas
- Unblocks: feat-prompts, feat-skillpl, feat-status
- Verify:
  - Unit tests for precedence ordering and tie-breaking
  - `agents status --mode X --profile Y` matches expected effective config for fixtures

### feat-prompts
- Goal: Prompt composition (base/project/snippets) + policy redaction application.
- Depends on: feat-resolv
- Unblocks: feat-templ, feat-shared
- Verify:
  - Golden test: composed prompt text is deterministic and redacts configured globs

### feat-skillpl
- Goal: Compute enabled skills for a resolved mode/profile/scope.
- Depends on: feat-resolv
- Unblocks: feat-runner, feat-compat
- Verify:
  - `agents status` lists enabled/disabled skills deterministically

## Phase 3: Rendering + Planning
### feat-templ
- Goal: Handlebars rendering with required helpers + stable serialization helpers.
- Depends on: feat-prompts
- Unblocks: feat-outputs, feat-shared, all adapters
- Verify:
  - Render a minimal template fixture; helper outputs are byte-stable

### feat-outputs
- Goal: Adapter output planning: select outputs, evaluate conditions, plan collisions.
- Depends on: feat-templ, feat-resolv
- Unblocks: feat-prevdf, feat-syncer, feat-stamps
- Verify:
  - A fixture with colliding surfaces fails with actionable diagnostics

### feat-stamps
- Goal: Stamping + drift detection metadata + "generated" identification.
- Depends on: feat-outputs
- Unblocks: feat-driftx, feat-cleanup, feat-doctor
- Verify:
  - Generated file contains deterministic stamp
  - Drift detection flags manual edits

### feat-driftx
- Goal: Diff engine comparing planned outputs vs on-disk materialized outputs.
- Depends on: feat-stamps
- Unblocks: feat-prevdf, feat-doctor
- Verify:
  - `agents diff` shows clean/no-op after `agents sync`

## Phase 4: Backends
### feat-matwiz
- Goal: `materialize` backend with writePolicy rules and optional `.gitignore` updates.
- Depends on: feat-outputs, feat-stamps
- Unblocks: feat-syncer, feat-cleanup
- Verify:
  - `agents sync --backend materialize` writes files
  - `writePolicy=if_generated` refuses to overwrite unstamped files

### feat-vfsctr
- Goal: `vfs_container` backend to run agents with injected generated outputs (no host writes).
- Depends on: feat-outputs, feat-stamps
- Unblocks: feat-runner
- Verify:
  - `agents run <agent> --backend vfs_container` mounts a temp output dir and starts the agent

### feat-cleanup
- Goal: Safe cleanup of generated artifacts for a target agent.
- Depends on: feat-matwiz, feat-stamps
- Unblocks: feat-doctor
- Verify:
  - `agents clean` deletes only stamped files and never deletes user-owned files

## Phase 5: Core UX Commands
### feat-initpr
- Goal: `agents init` with presets from PRD; creates required `.agents/` skeleton.
- Depends on: feat-loadag
- Unblocks: feat-adbase, feat-adtest
- Verify:
  - `agents init --preset standard` creates `.agents/` layout

### feat-status
- Goal: `agents status` prints effective mode/profile/scopes/policy/skills/backend/agent.
- Depends on: feat-resolv, feat-skillpl
- Unblocks: feat-compat
- Verify:
  - `agents status` output matches fixture expectations

### feat-prevdf
- Goal: `agents preview` (temp render) and `agents diff` (no writes).
- Depends on: feat-driftx
- Unblocks: feat-syncer
- Verify:
  - Preview renders to temp and prints output paths
  - Diff is stable across runs

### feat-syncer
- Goal: `agents sync` applies selected backend (materialize or VFS).
- Depends on: feat-matwiz, feat-vfsctr, feat-outputs
- Unblocks: feat-runner, feat-doctor
- Verify:
  - `agents sync` followed by `agents diff` yields no changes

### feat-doctor
- Goal: `agents doctor [--fix] [--ci]` checks schemas, drift, collisions, prerequisites.
- Depends on: feat-schemas, feat-driftx, feat-cleanup
- Unblocks: feat-adtest
- Verify:
  - `agents doctor --ci` exits non-zero on drift/collisions

## Phase 6: Adapter Set (v1)
Adapters are implemented after the shared rendering/planning pipeline is stable.

### feat-shared
- Goal: Shared surfaces generator (recommended owner of `shared:AGENTS.md`).
- Depends on: feat-templ, feat-prompts, feat-stamps
- Unblocks: feat-adcodx, feat-adopen, feat-adcopl
- Verify:
  - Renders `AGENTS.md` with current mode/profile banner and stable ordering

### feat-adcodx
- Goal: Codex adapter outputs using shared `AGENTS.md` surface.
- Depends on: feat-shared, feat-outputs
- Unblocks: feat-adtest
- Verify:
  - `agents preview --agent codex` produces required files

### feat-adopen
- Goal: OpenCode adapter (`opencode.jsonc` + one shared rule surface).
- Depends on: feat-shared, feat-templ
- Unblocks: feat-adtest
- Verify:
  - `agents preview --agent opencode` produces `opencode.jsonc` with stamp

### feat-adclde
- Goal: Claude Code adapter (`.claude/settings.json` and optional `CLAUDE.md`).
- Depends on: feat-templ, feat-outputs
- Unblocks: feat-adtest
- Verify:
  - `agents preview --agent claude` produces `.claude/settings.json`

### feat-adgcli
- Goal: Gemini CLI adapter (`.gemini/settings.json`).
- Depends on: feat-templ, feat-outputs
- Unblocks: feat-adtest
- Verify:
  - `agents preview --agent gemini-cli` produces `.gemini/settings.json`

### feat-adgghb
- Goal: Gemini GitHub adapter (`.gemini/styleguide.md`).
- Depends on: feat-templ
- Unblocks: feat-adtest
- Verify:
  - `agents preview --agent gemini-github` produces `.gemini/styleguide.md`

### feat-adcurs
- Goal: Cursor adapter (`.cursor/rules/*.md`) with deterministic filenames.
- Depends on: feat-templ, feat-outputs
- Unblocks: feat-adtest
- Verify:
  - `agents diff --agent cursor` is stable across runs (no churn)

### feat-adcopl
- Goal: Copilot adapter (`.github/copilot-instructions.md`, optional scoped instructions).
- Depends on: feat-shared, feat-outputs
- Unblocks: feat-adtest
- Verify:
  - `agents preview --agent copilot` produces required GitHub path outputs

### feat-adtest
- Goal: Golden fixture test runner: `agents test adapters [--agent ...]`.
- Depends on: feat-schemas, feat-outputs, feat-stamps, all adapters in scope
- Unblocks: CI gate for adapter changes
- Verify:
  - `agents test adapters` passes on a fixture matrix (single repo + monorepo)

## Phase 7: Run-time Skills + Import + Explainability
### feat-runner
- Goal: `agents run <agent>` executes the chosen agent with backend + optional skill runtime.
- Depends on: feat-vfsctr, feat-skillpl, feat-syncer
- Unblocks: end-to-end workflows
- Verify:
  - Running an agent uses the resolved mode/policy and injects generated surfaces

### feat-explnx
- Goal: `agents explain <path>` prints the source map for a generated output.
- Depends on: feat-outputs, feat-prompts
- Unblocks: better debugging and trust
- Verify:
  - Explain lists contributing prompt/snippets/mode/policy/adapter template

### feat-compat
- Goal: `agents compat` prints compatibility matrix from adapters and enforcement coverage.
- Depends on: feat-skillpl, feat-outputs
- Unblocks: onboarding UX
- Verify:
  - Matrix includes recommended backend and limitations per adapter

### feat-importr
- Goal: `agents import --from <agent>` converts native config into `.agents` artifacts (explicit workflow).
- Depends on: feat-models, feat-loadag
- Unblocks: adoption in existing repos
- Verify:
  - Import produces valid `.agents/` files and `agents validate` passes

## Phase 8 (Later / v1.1): Advanced Backend
### feat-vfsmnt
- Goal: Optional `vfs_mount` backend for IDE workflows.
- Depends on: feat-outputs, feat-stamps
- Unblocks: IDE integration that avoids materializing into repo
- Verify:
  - Mount workflow produces a usable composite workspace path on supported platforms

## Phase 9: Production Hardening
### feat-prodci
- Goal: production CI hardening (fmt, clippy, tests, warning-free builds).
- Depends on: feat-cliapp
- Unblocks: CI stability, release readiness
- Verify:
  - `cargo fmt -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`

## Phase 10: Distribution
### feat-npmpub
- Goal: npm packaging for CLI distribution.
- Depends on: feat-cliapp
- Unblocks: npm install workflow
- Verify:
  - `npm pack`
  - `npm install -g agents && agents --help`
