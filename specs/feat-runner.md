# feat-runner: `agents run <agent>`

Goal: Run the selected agent with the resolved configuration, applying backend projection and starting any skill runtime components.

Depends on: feat-vfsctr, feat-skillpl, feat-syncer
Unblocks: end-to-end workflows

## Deliverables
- `agents run <agent> [--mode ..] [--profile ..] [--backend ..] [--] ...`.
- For v1:
  - uses `vfs_container` by default for CLI agents
  - ensures generated outputs are available to the agent
  - starts MCP servers for `mcp_tool` skills (if implemented)

## Implementation Plan
- [ ] Define agent registry
  - [ ] Map agent IDs to:
    - [ ] executable name or docker entrypoint
    - [ ] default backend preference
    - [ ] required outputs (from adapter)
  - [ ] For v1, allow `--agent <id>` to pick adapter and `run <agent>` to run actual binary

- [ ] Implement run orchestration
  - [ ] Load + validate repo config
  - [ ] Resolve effective config
  - [ ] Plan + render outputs
  - [ ] Determine backend and prepare environment

- [ ] Skill runtime (incremental)
  - [ ] instruction_only:
    - [ ] already handled by prompt composition/templates
  - [ ] mcp_tool:
    - [ ] read skill interface entrypoint/args/env
    - [ ] validate policy allows MCP
    - [ ] start process(es), capture stdout/stderr
    - [ ] propagate connection info to agent (env vars)
  - [ ] cli_shim:
    - [ ] ensure skill assets are available in runtime (via vfs container mounts or materialize)

- [ ] Policy warnings
  - [ ] If policy enables network or unrestricted exec:
    - [ ] print warning and require confirmation (if configured)

- [ ] Exec the agent
  - [ ] Support passthrough args after `--`
  - [ ] Propagate exit code of agent as CLI exit code
  - [ ] Ensure cleanup:
    - [ ] stop MCP servers
    - [ ] remove temp dirs

- [ ] Tests
  - [ ] Unit tests for argument construction
  - [ ] Integration test with a dummy "agent" executable (script) in fixtures

## Verification
- [ ] `agents run opencode -- --help` starts the backend and runs the binary (or container entry)
