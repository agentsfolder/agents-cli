<p align="center">
  <img src="agents-folder.PNG" alt="(dot)agents folder logo" width="500"/>
</p>

# agents

`agents` is a Rust CLI + core library that projects canonical `.agents/` configuration into agent-native surfaces (Copilot, Cursor, Codex, Claude Code, OpenCode, Gemini, etc.). It gives you deterministic outputs, drift detection, and a single source of truth for agent instructions.

## Features
- Canonical `.agents/` schema with deterministic resolution, templating, and output planning.
- Multi-backend rendering: `materialize`, `vfs_container`, and `vfs_mount` (workspace copy).
- Drift detection and cleanup for generated outputs.
- Adapter fixtures and CLI verification tools.

## Quickstart
```bash
cargo run -p agents-cli -- init --preset standard
cargo run -p agents-cli -- status
cargo run -p agents-cli -- preview --agent opencode
cargo run -p agents-cli -- sync --agent opencode --backend materialize
```

## Common Commands
```bash
# Validate configuration
agents validate

# Inspect effective config
agents status [--mode <id>] [--profile <id>] [--json]

# Preview planned outputs (no writes)
agents preview --agent <id> [--backend <backend>] [--keep-temp]

# Apply outputs via backend
agents sync --agent <id> [--backend <backend>]

# Run a CLI agent with resolved config
agents run <agent-binary> --adapter <id> [--backend <backend>] -- [agent args]

# Show drift
agents diff --agent <id>

# Clean generated artifacts
agents clean --agent <id> [--dry-run]

# Explain a generated file
agents explain <path>

# Compatibility matrix
agents compat [--json]

# Import from existing config
agents import --from copilot [--path <file>]

# Adapter golden fixtures
agents test adapters [--agent <id>] [--update]
```

## Supported Adapters
- `opencode` (opencode.jsonc + shared AGENTS.md)
- `codex` (AGENTS.md)
- `copilot` (.github/copilot-instructions.md + scoped instructions)
- `claude` (.claude/settings.json + optional CLAUDE.md)
- `cursor` (.cursor/rules/*.md)
- `gemini-cli` (.gemini/settings.json)
- `gemini-github` (.gemini/styleguide.md)
- `core` shared AGENTS.md surface

## Configuration Layout
```
.agents/
  manifest.yaml
  prompts/
    base.md
    project.md
    snippets/
  modes/
  policies/
  adapters/
  schemas/
  state/
```

## Development
The source of truth for work lives in `specs/*.md` checklists. When adding a new spec, update both `specs/plan.md` and the specs index in `AGENTS.md`.

### Tests
```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Notes
- Use `--backend vfs_container` to avoid writing to the repo when supported by your environment.
- Adapter fixtures live under `fixtures/` and are exercised by `agents test adapters`.
