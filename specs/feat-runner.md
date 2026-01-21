# feat-runner: `agents run <agent>`

Goal: Run the selected agent with the resolved configuration, applying backend projection and starting any skill runtime components.

Depends on: feat-vfsctr, feat-skillpl, feat-syncer
Unblocks: end-to-end workflows

## Deliverables
- `agents run <agent> [--mode ..] [--profile ..] [--backend ..] [--] ...`.
- For v1:
  - uses `vfs_container` by default for CLI agents
  - ensures generated outputs are available to the agent
  - skill runtime beyond `instruction_only` is deferred (no MCP server orchestration yet)

## Implementation Plan
- [x] Define agent registry
  - [x] Map agent IDs to:
    - [x] executable name or docker entrypoint
    - [x] default backend preference
    - [x] required outputs (derived from adapter plan at runtime)
  - [x] For v1, allow `--adapter <id>` (alias: `--agent`) to pick adapter and `run <agent>` to run actual binary

- [x] Implement run orchestration
  - [x] Load + validate repo config
  - [x] Resolve effective config
  - [x] Plan + render outputs
  - [x] Determine backend and prepare environment

- [x] Skill runtime (v1)
  - [x] instruction_only:
    - [x] already handled by prompt composition/templates

Future (not in v1)
- MCP server orchestration for `mcp_tool` skills
- CLI shim runtime for `cli_shim` skills

- [x] Policy warnings
  - [x] If policy enables network or unrestricted exec:
    - [x] print warning (no interactive confirmation in v1)

- [x] Exec the agent
  - [x] Support passthrough args after `--`
  - [x] Propagate exit code of agent as CLI exit code
  - [x] Ensure cleanup:
    - [x] stop MCP servers (no-op in v1; MCP runtime not started)
    - [x] remove temp dirs

- [x] Tests
  - [x] Unit tests for argument construction
  - [x] Integration test with a dummy "agent" executable (script) in fixtures

## Verification
- [x] `agents run opencode -- --help` starts the backend and runs the binary (or container entry) (covered by dummy agent run test)
