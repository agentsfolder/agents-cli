PRD: .agents/ Unified Agent Spec + Projection CLI

One-line summary

Define a single canonical .agents/ folder that standardizes project guidance, skills, modes, and policies for AI coding agents, plus a CLI that projects (physically or virtually) agent-native configuration surfaces so each agent behaves consistently without bespoke per-agent dotfolders.

⸻

1. Background and problem statement

Teams increasingly use multiple coding agents across terminal and IDE environments (Codex, Gemini, Claude Code, OpenCode, Cursor, Copilot). Each agent expects different repository files and structures for custom instructions, configuration, and behavior controls. This produces:
	•	Multiple competing “sources of truth” across .cursor/, .claude/, .gemini/, .github/, opencode.jsonc, and instruction files.
	•	Duplicated, inconsistent, and stale agent guidance.
	•	Uneven safety posture (what’s allowed/denied differs by agent).
	•	Poor ergonomics when switching agents, onboarding contributors, or scaling across monorepos.

⸻

2. Goals and non-goals

Goals (v1)
	1.	Canonical spec: .agents/ is the repository’s canonical, version-controlled definition of:
	•	prompts and instruction blocks
	•	modes (behavior profiles)
	•	policies (permissions, path controls, confirmation gates)
	•	skills (tool/harness definitions)
	•	adapters (how to compile into each agent’s native surfaces)
	2.	Seamless compatibility via projection: Agents consume their native config surfaces (or a faithful virtual equivalent) without learning .agents.
	3.	VFS-first for CLI agents: Prefer a virtual filesystem projection via container overlay for CLI-first agents; fall back to materialization where required.
	4.	Monorepo-ready: Support scoping by path (“applyTo” style) and deterministic resolution.
	5.	Explainability and lifecycle UX: Provide preview, diff, drift detection, cleanup, and import flows.
	6.	Adapter reliability: Golden fixture tests, compatibility matrix generation, and regression guardrails.

Non-goals (v1)
	•	Full sandbox/VM security product or strong isolation guarantees.
	•	Implicit two-way syncing from agent-native files back into .agents/ (only explicit import).
	•	Public marketplace/registry/signing for skills/plugins (local-first only).

⸻

3. Personas and key use cases

Personas
	•	Platform/DevEx: owns org-wide agent consistency and onboarding.
	•	Repo maintainers: want stable, reviewable, low-churn configuration.
	•	Individual developers: want quick mode switching and predictable behavior across agents.

Core use cases
	1.	Standardize agent behavior across tools: one set of project rules applied to all supported agents.
	2.	Switch agents with no rewrite: toggle --agent codex|claude|gemini|opencode|cursor|copilot.
	3.	Monorepo scoping: different guidance for apps/web/** vs services/api/** without nested dotfolders.
	4.	Repo hygiene: keep generated artifacts out of the repo by default for CLI workflows.
	5.	CI enforcement: validate schemas, detect drift, and ensure deterministic outputs.

⸻

4. Supported agents and their native surfaces (v1)

Adapters must support the following native surfaces (at minimum). These are “projection targets” that .agents compiles into:

Codex (OpenAI)
	•	Codex reads AGENTS.md files for project instructions, supporting layered guidance across directories.  ￼
	•	Codex CLI also uses ~/.codex/config.toml and allows invocation-level overrides.  ￼

Claude Code (Anthropic)
	•	Claude Code supports hierarchical settings.json configuration (user and project), as the official mechanism.  ￼

Gemini
	•	Gemini CLI uses JSON settings files (settings.json) and supports user/project layering (official docs).  ￼
	•	Gemini Code Assist for GitHub supports repository customization via .gemini/styleguide.md and related files.  ￼

OpenCode
	•	OpenCode supports rules discovery via AGENTS.md / CLAUDE.md / CONTEXT.md and a global rules file.  ￼
	•	OpenCode supports JSON/JSONC configuration.  ￼

Cursor
	•	Cursor supports rule files in .cursor/rules.  ￼

GitHub Copilot
	•	Repository-wide custom instructions via .github/copilot-instructions.md.  ￼
	•	Path-specific instructions via *.instructions.md with applyTo are supported in certain Copilot Chat/coding agent contexts.  ￼

⸻

5. Product requirements

5.1 Canonical repo structure

Required (v1):

.agents/
  manifest.yaml
  prompts/
    base.md
    project.md
    snippets/
      *.md
  modes/
    *.md
  policies/
    *.yaml
  skills/
    <skill-id>/
      skill.yaml
      README.md
      harness/
      scripts/
      examples/
  scopes/
    *.yaml
  adapters/
    <agent-id>/
      adapter.yaml
      templates/
        **
      mappings/
        **   (optional)
  profiles/
    *.yaml
  state/
    .gitignore
    state.yaml         (optional, non-committed)
  schemas/
    manifest.schema.json
    policy.schema.json
    skill.schema.json
    adapter.schema.json
    scope.schema.json
    mode-frontmatter.schema.json
    state.schema.json

Repo hygiene rules
	•	.agents/** is canonical and must be committed (except .agents/state/state.yaml).
	•	Generated artifacts written outside .agents/ must be clearly stamped and deterministically reproducible.

⸻

5.2 Resolution model (profiles, scopes, overlays)

Inputs
	•	Repo base: .agents/ (canonical)
	•	User overlay (optional): ~/.agents/ for personal defaults
	•	Profiles: .agents/profiles/<name>.yaml (e.g., dev, ci)
	•	Scopes: .agents/scopes/<scope-id>.yaml with applyTo glob patterns
	•	State: .agents/state/state.yaml (selected mode/profile for IDE workflows)
	•	CLI overrides: --mode, --profile, --policy, --scope, --backend

Precedence (highest wins)
	1.	CLI flags / env overrides
	2.	Repo .agents base
	3.	Repo scopes (most specific match wins; ties resolved deterministically)
	4.	User overlay (if enabled)

Merge semantics
	•	Deterministic deep-merge with “deny beats allow” for policy fields.
	•	Conflicts for authoritative singletons (e.g., specVersion, unknown mode IDs) default to error.
	•	Non-conflicting settings are preserved.

⸻

5.3 Modes (UX-first behavior profiles)

Purpose

A mode is the primary user-facing control for agent behavior (examples: default, readonly-audit, refactor, release).

Required capabilities
	•	Mode selects:
	•	prompt composition (base + project + snippets)
	•	policy binding
	•	enabled skills (add/remove)
	•	tool intent (advisory + enforceable where possible)

Mode switching UX
	•	CLI agents: agents run <agent> --mode <mode> --profile <profile>
	•	IDE agents: agents set-mode <mode> [--profile <profile>] writes to .agents/state/state.yaml (gitignored) and triggers sync.
	•	“Current mode” must be visible:
	•	agents status prints effective mode/profile/scope
	•	adapters may inject a “Current Mode” banner at top of generated instruction surfaces

⸻

5.4 Policies (minimal but practical controls)

Policy model (v1)
	•	Filesystem: read/write/delete/rename
	•	Exec: enabled + allow/deny command patterns
	•	Network: enabled + allow/deny host patterns
	•	Redaction globs for sensitive files
	•	Confirmation gates: delete/overwrite/publish/deploy/push/rebase
	•	Limits: max files changed, max patch lines, max runtime

Enforcement approach
	•	Where enforceable: CLI wrapper and container environment implement restrictions (best-effort).
	•	Where not enforceable: policies compile into instructions (“advisory”) for the agent.

⸻

5.5 Skills (tools/harnesses)

Skill activation modes (v1)

Each skill declares an activation mode so users understand “what it does” operationally:
	1.	instruction_only
	•	Skill contributes guidance/instructions (no runtime tooling)
	2.	mcp_tool
	•	Skill is exposed as an MCP server/tooling integration (launched by CLI)
	3.	cli_shim
	•	Skill provides commands/scripts available within the container/VFS environment

Runtime assembly requirements

When running an agent (agents run), the CLI must:
	•	Determine effective mode/policy/profile/scope
	•	Compute enabled skills
	•	Start required MCP servers (if any)
	•	Mount or materialize skill assets per backend
	•	Apply execution constraints (timeouts, allow/deny patterns) where feasible

⸻

5.6 Adapters and projection outputs

Adapter responsibilities
	•	Map canonical .agents content into agent-native files and formats.
	•	Declare output paths, formats, stamping/drift methods, and backend conditions.
	•	Define collision behavior for shared surfaces like AGENTS.md.

Shared instruction surfaces and collisions

Many ecosystems can consume AGENTS.md or similar surfaces; OpenCode explicitly searches for AGENTS.md / CLAUDE.md / CONTEXT.md.  ￼

PRD requirement:
	•	Outputs declare one of:
	•	collision: error (default)
	•	collision: overwrite
	•	collision: merge (deterministic concatenation with stable ordering)
	•	collision: shared_owner (only one designated generator may own this surface)
	•	Recommended v1 default: a core “shared surfaces generator” owns AGENTS.md; other adapters reference it.

⸻

5.7 Projection backends

Backends (v1)
	1.	vfs_container (default for CLI agents)
	•	Container overlay that injects generated agent-native files without host writes.
	2.	materialize (default for IDE agents)
	•	Writes deterministic generated files into the repo filesystem.
	3.	vfs_mount (optional advanced)
	•	Mounts a composite workspace path for IDEs that can operate on mounted paths.

Backend selection defaults
	•	Codex / Claude / Gemini CLI / OpenCode → vfs_container
	•	Cursor / Copilot → materialize (unless user opts into mount workflow)

⸻

5.8 Generated output lifecycle (critical UX)

Commands (v1)
	•	agents preview [--agent <id>] [--mode ...] [--profile ...]
Renders outputs to a temp directory; prints paths and diffs.
	•	agents diff [--agent <id>]
Shows what changes would be materialized (no writes).
	•	agents sync [--agent <id>] [--backend ...]
Materializes or prepares projections (depending on backend).
	•	agents clean [--agent <id>]
Removes generated artifacts that are safe to delete.
	•	agents doctor [--fix] [--ci]
Detects missing prerequisites, drift, collisions, invalid schemas; can fix safe items.
	•	agents import --from <agent> [--path ...]
Converts existing agent-native config into canonical .agents artifacts (explicit workflow).

Drift model
	•	Drift is detected via stamp + content hash (default sha256).
	•	Drift is surfaced in doctor, and optionally fails CI under strict mode.

⸻

5.9 Explainability and transparency

Requirements
	•	agents status shows effective: mode, profile, scope(s), policy, enabled skills, backend, target agent.
	•	agents explain <output-path> prints a “source map”:
	•	which prompts/snippets contributed
	•	which mode/policy/skills affected it
	•	which adapter template rendered it
	•	agents render --agent <id> --path <target> --stdout for inspectability and debugging.

⸻

5.10 Adapter reliability and compatibility UX

Compatibility matrix
	•	agents compat prints a matrix per agent:
	•	supported surfaces
	•	recommended backend
	•	enforceable vs advisory policy mapping coverage
	•	known limitations

Adapter test strategy (v1 requirement)
	•	Golden fixture tests:
	•	agents test adapters [--agent <id>]
	•	Renders deterministic outputs from fixtures and compares to golden snapshots.
	•	Regression gate for adapter changes:
	•	CI required for .agents/adapters/** modifications.

⸻

6. CLI specification

6.1 Primary commands (v1)
	•	agents init [--preset <name>]
	•	agents validate [--profile ci]
	•	agents status
	•	agents set-mode <mode> [--profile <profile>]
	•	agents preview [--agent <id>] [--backend ...]
	•	agents diff [--agent <id>]
	•	agents sync [--agent <id>] [--backend ...]
	•	agents run <agent> [--mode ...] [--profile ...] [--backend ...]
	•	agents doctor [--fix] [--ci]
	•	agents clean [--agent <id>]
	•	agents import --from <agent>
	•	agents explain <path>
	•	agents compat
	•	agents test adapters [--agent <id>]

6.2 Guided init presets (UX)

agents init offers presets:
	•	conservative (readonly-ish, strict confirmations)
	•	standard (balanced)
	•	ci-safe (no network/exec by default)
	•	monorepo (enables scopes and example applyTo)
	•	agent-pack (enables all v1 agents + shared surfaces)

⸻

7. Data model and schema definitions (v1)

Authoring format: YAML/Markdown. Validation format: JSON Schema stored in .agents/schemas/.

Below are the canonical schemas (representative and complete for v1). Implementations may include additional constraints (e.g., enums, patterns) as needed.

7.1 manifest.yaml (schema)

{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": ".agents manifest",
  "type": "object",
  "required": ["specVersion", "defaults", "enabled"],
  "additionalProperties": false,
  "properties": {
    "specVersion": { "type": "string" },
    "project": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "name": { "type": "string" },
        "description": { "type": "string" },
        "languages": { "type": "array", "items": { "type": "string" } },
        "frameworks": { "type": "array", "items": { "type": "string" } }
      }
    },
    "defaults": {
      "type": "object",
      "required": ["mode", "policy"],
      "additionalProperties": false,
      "properties": {
        "mode": { "type": "string" },
        "policy": { "type": "string" },
        "profile": { "type": "string" },
        "backend": { "enum": ["vfs_container", "materialize", "vfs_mount"] },
        "sharedSurfacesOwner": { "type": "string", "default": "core" }
      }
    },
    "enabled": {
      "type": "object",
      "required": ["modes", "policies", "skills", "adapters"],
      "additionalProperties": false,
      "properties": {
        "modes": { "type": "array", "items": { "type": "string" } },
        "policies": { "type": "array", "items": { "type": "string" } },
        "skills": { "type": "array", "items": { "type": "string" } },
        "adapters": { "type": "array", "items": { "type": "string" } }
      }
    },
    "resolution": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "enableUserOverlay": { "type": "boolean", "default": true },
        "denyOverridesAllow": { "type": "boolean", "default": true },
        "onConflict": { "enum": ["error", "warn"], "default": "error" }
      }
    },
    "backends": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "default": { "enum": ["vfs_container", "materialize", "vfs_mount"] },
        "byAgent": {
          "type": "object",
          "additionalProperties": { "enum": ["vfs_container", "materialize", "vfs_mount"] }
        }
      }
    },
    "x": { "type": "object", "additionalProperties": true }
  }
}

7.2 policy.yaml (schema)

{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": ".agents policy",
  "type": "object",
  "required": ["id", "description", "capabilities", "paths", "confirmations"],
  "additionalProperties": false,
  "properties": {
    "id": { "type": "string" },
    "description": { "type": "string" },
    "capabilities": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "filesystem": {
          "type": "object",
          "additionalProperties": false,
          "properties": {
            "read": { "type": "boolean", "default": true },
            "write": { "type": "boolean", "default": true },
            "delete": { "type": "boolean", "default": false },
            "rename": { "type": "boolean", "default": false }
          }
        },
        "exec": {
          "type": "object",
          "additionalProperties": false,
          "properties": {
            "enabled": { "type": "boolean", "default": true },
            "allow": { "type": "array", "items": { "type": "string" }, "default": [] },
            "deny": { "type": "array", "items": { "type": "string" }, "default": [] }
          }
        },
        "network": {
          "type": "object",
          "additionalProperties": false,
          "properties": {
            "enabled": { "type": "boolean", "default": false },
            "allowHosts": { "type": "array", "items": { "type": "string" }, "default": [] },
            "denyHosts": { "type": "array", "items": { "type": "string" }, "default": ["*"] }
          }
        },
        "mcp": {
          "type": "object",
          "additionalProperties": false,
          "properties": {
            "enabled": { "type": "boolean", "default": true },
            "allowServers": { "type": "array", "items": { "type": "string" }, "default": [] },
            "denyServers": { "type": "array", "items": { "type": "string" }, "default": [] }
          }
        }
      }
    },
    "paths": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "allow": { "type": "array", "items": { "type": "string" }, "default": ["**"] },
        "deny": { "type": "array", "items": { "type": "string" }, "default": [] },
        "redact": { "type": "array", "items": { "type": "string" }, "default": ["**/.env", "**/.env.*", "**/secrets/**"] }
      }
    },
    "confirmations": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "requiredFor": {
          "type": "array",
          "items": { "enum": ["delete", "overwrite", "publish", "deploy", "push", "rebase"] },
          "default": ["delete", "overwrite", "publish", "deploy"]
        }
      }
    },
    "limits": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "maxFilesChanged": { "type": "integer", "default": 200 },
        "maxPatchLines": { "type": "integer", "default": 5000 },
        "maxCommandRuntimeSec": { "type": "integer", "default": 600 }
      }
    },
    "x": { "type": "object", "additionalProperties": true }
  }
}

7.3 skill.yaml (schema)

{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": ".agents skill",
  "type": "object",
  "required": ["id", "version", "title", "description", "activation", "interface", "contract", "requirements"],
  "additionalProperties": false,
  "properties": {
    "id": { "type": "string" },
    "version": { "type": "string" },
    "title": { "type": "string" },
    "description": { "type": "string" },
    "tags": { "type": "array", "items": { "type": "string" }, "default": [] },

    "activation": { "enum": ["instruction_only", "mcp_tool", "cli_shim"] },

    "interface": {
      "type": "object",
      "required": ["type"],
      "additionalProperties": false,
      "properties": {
        "type": { "enum": ["mcp", "cli", "script", "library"] },
        "entrypoint": { "type": "string" },
        "args": { "type": "array", "items": { "type": "string" }, "default": [] },
        "env": { "type": "object", "additionalProperties": { "type": "string" }, "default": {} }
      }
    },

    "contract": {
      "type": "object",
      "required": ["inputs", "outputs"],
      "additionalProperties": false,
      "properties": {
        "inputs": { "type": "object" },
        "outputs": { "type": "object" }
      }
    },

    "requirements": {
      "type": "object",
      "required": ["capabilities"],
      "additionalProperties": false,
      "properties": {
        "capabilities": {
          "type": "object",
          "additionalProperties": false,
          "properties": {
            "filesystem": { "enum": ["none", "read", "write"] },
            "exec": { "enum": ["none", "restricted", "full"] },
            "network": { "enum": ["none", "restricted", "full"] }
          }
        },
        "paths": {
          "type": "object",
          "additionalProperties": false,
          "properties": {
            "needs": { "type": "array", "items": { "type": "string" }, "default": [] },
            "writes": { "type": "array", "items": { "type": "string" }, "default": [] }
          }
        }
      }
    },

    "assets": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "mount": { "type": "array", "items": { "type": "string" }, "default": [] },
        "materialize": { "type": "array", "items": { "type": "string" }, "default": [] }
      }
    },

    "compatibility": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "agents": { "type": "array", "items": { "type": "string" }, "default": [] },
        "backends": { "type": "array", "items": { "enum": ["vfs_container", "materialize", "vfs_mount"] }, "default": ["vfs_container", "materialize"] }
      }
    },

    "x": { "type": "object", "additionalProperties": true }
  }
}

7.4 scope.yaml (schema)

{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": ".agents scope",
  "type": "object",
  "required": ["id", "applyTo", "overrides"],
  "additionalProperties": false,
  "properties": {
    "id": { "type": "string" },
    "applyTo": { "type": "array", "items": { "type": "string" } },
    "priority": { "type": "integer", "default": 0, "description": "Higher wins when multiple scopes match with equal specificity." },
    "overrides": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "mode": { "type": "string" },
        "policy": { "type": "string" },
        "enableSkills": { "type": "array", "items": { "type": "string" }, "default": [] },
        "disableSkills": { "type": "array", "items": { "type": "string" }, "default": [] },
        "includeSnippets": { "type": "array", "items": { "type": "string" }, "default": [] }
      }
    }
  }
}

7.5 Mode frontmatter (schema)

Mode files are Markdown; optional YAML frontmatter is validated.

{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": ".agents mode frontmatter",
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "id": { "type": "string" },
    "title": { "type": "string" },
    "policy": { "type": "string" },
    "enableSkills": { "type": "array", "items": { "type": "string" }, "default": [] },
    "disableSkills": { "type": "array", "items": { "type": "string" }, "default": [] },
    "includeSnippets": { "type": "array", "items": { "type": "string" }, "default": [] },
    "toolIntent": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "allow": { "type": "array", "items": { "type": "string" }, "default": [] },
        "deny": { "type": "array", "items": { "type": "string" }, "default": [] }
      }
    }
  }
}

7.6 .agents/state/state.yaml (schema)

This file is non-committed and supports IDE mode switching.

{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": ".agents state",
  "type": "object",
  "required": ["mode"],
  "additionalProperties": false,
  "properties": {
    "mode": { "type": "string" },
    "profile": { "type": "string" },
    "backend": { "enum": ["vfs_container", "materialize", "vfs_mount"] },
    "scopes": { "type": "array", "items": { "type": "string" }, "default": [] }
  }
}


⸻

8. Adapter template format (v1)

8.1 Adapter layout
	•	.agents/adapters/<agentId>/adapter.yaml
	•	.agents/adapters/<agentId>/templates/** (rendered)
	•	.agents/adapters/<agentId>/mappings/** (optional data helpers)

8.2 Adapter schema

{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": ".agents adapter",
  "type": "object",
  "required": ["agentId", "version", "backendDefaults", "outputs"],
  "additionalProperties": false,
  "properties": {
    "agentId": { "type": "string" },
    "version": { "type": "string" },

    "backendDefaults": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "preferred": { "enum": ["vfs_container", "materialize", "vfs_mount"] },
        "fallback": { "enum": ["vfs_container", "materialize", "vfs_mount"] }
      }
    },

    "capabilityMapping": { "type": "object", "additionalProperties": true },

    "outputs": {
      "type": "array",
      "minItems": 1,
      "items": {
        "type": "object",
        "required": ["path", "renderer"],
        "additionalProperties": false,
        "properties": {
          "path": { "type": "string" },
          "format": { "enum": ["text", "md", "yaml", "json", "jsonc"], "default": "text" },

          "surface": {
            "type": "string",
            "description": "Optional logical surface name (e.g., shared:AGENTS.md) for collision control."
          },

          "collision": { "enum": ["error", "overwrite", "merge", "shared_owner"], "default": "error" },

          "condition": {
            "type": "object",
            "additionalProperties": false,
            "properties": {
              "backendIn": { "type": "array", "items": { "enum": ["vfs_container", "materialize", "vfs_mount"] } },
              "profileIn": { "type": "array", "items": { "type": "string" } }
            }
          },

          "renderer": {
            "type": "object",
            "required": ["type"],
            "additionalProperties": false,
            "properties": {
              "type": { "enum": ["template", "concat", "copy", "json_merge"] },
              "template": { "type": "string" },
              "sources": { "type": "array", "items": { "type": "string" } },
              "jsonMergeStrategy": { "enum": ["deep", "shallow"], "default": "deep" }
            }
          },

          "writePolicy": {
            "type": "object",
            "additionalProperties": false,
            "properties": {
              "mode": { "enum": ["always", "if_generated", "never"], "default": "if_generated" },
              "gitignore": { "type": "boolean", "default": false }
            }
          },

          "driftDetection": {
            "type": "object",
            "additionalProperties": false,
            "properties": {
              "method": { "enum": ["sha256", "mtime_only", "none"], "default": "sha256" },
              "stamp": { "enum": ["comment", "frontmatter", "json_field"], "default": "comment" }
            }
          }
        }
      }
    },

    "tests": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "goldenFixturesDir": { "type": "string" },
        "goldenCommand": { "type": "string" }
      }
    },

    "x": { "type": "object", "additionalProperties": true }
  }
}

8.3 Template engine requirements (v1)
	•	Handlebars-compatible syntax ({{var}}, {{#if}}, {{#each}}, partials).
	•	Required helpers:
	•	indent(text, n)
	•	join(list, sep)
	•	toJson(obj) / toJsonc(obj) (stable ordering)
	•	toYaml(obj) (stable ordering)
	•	frontmatter(obj)
	•	generatedStamp(meta) (deterministic stamp)

8.4 Standard render context

Adapters render against a stable computed context:
	•	effective.mode, effective.policy, effective.skills, effective.prompts
	•	profile, scopesMatched, generation.stamp, adapter.agentId

⸻

9. Agent adapter requirements (v1)

This section defines required output targets and key mapping expectations.

9.1 Codex adapter
	•	Required outputs:
	•	AGENTS.md (shared surface owner recommended)
	•	Behavior:
	•	Must support layered instructions (root-to-leaf injection) consistent with Codex guidance.  ￼
	•	CLI integration:
	•	When using vfs_container, Codex runs in overlay workspace with injected AGENTS.md.
	•	When materializing, writes AGENTS.md with generated stamp.

9.2 Copilot adapter
	•	Required outputs:
	•	.github/copilot-instructions.md  ￼
	•	Optional outputs:
	•	*.instructions.md for path-specific instructions with applyTo (where supported)  ￼
	•	AGENTS.md (shared surface)
	•	Notes:
	•	Adapter should emit a small “current mode” banner for clarity in IDE contexts.

9.3 Cursor adapter
	•	Required outputs:
	•	.cursor/rules/*.md  ￼
	•	Notes:
	•	Deterministic filenames and ordering are required to avoid diff churn.

9.4 Claude Code adapter
	•	Required outputs:
	•	.claude/settings.json (project) as the official settings mechanism.  ￼
	•	Optional outputs:
	•	CLAUDE.md (human-readable instructions aligned with effective mode)

9.5 Gemini CLI adapter
	•	Required outputs:
	•	.gemini/settings.json (project settings) consistent with official docs.  ￼
	•	Optional:
	•	documentation snippet for ~/.gemini/settings.json (CLI prints; does not write by default)

9.6 Gemini Code Assist for GitHub adapter
	•	Required outputs:
	•	.gemini/styleguide.md (and optionally .gemini/config.yaml when applicable).  ￼

9.7 OpenCode adapter
	•	Required outputs:
	•	opencode.jsonc (JSONC config)  ￼
	•	One of: AGENTS.md / CLAUDE.md / CONTEXT.md (OpenCode rule discovery order includes these)  ￼

⸻

10. Quality attributes

Cross-platform
	•	macOS, Linux, Windows supported.
	•	Avoid symlink-only solutions by default.
	•	Docker Desktop supported for vfs_container workflows.

Performance
	•	validate and diff should complete quickly (target: <2 seconds typical repo).
	•	sync should be deterministic and incremental where possible.

Determinism
	•	Stable sorting, stable serialization, stable stamping.
	•	Golden fixtures enforce deterministic output.

⸻

11. Security and privacy
	•	“Deny beats allow” in policy evaluation.
	•	Default policies require confirmation for destructive operations.
	•	Redaction globs prevent accidental prompt inclusion of secrets.
	•	CLI should warn when agent is run with policies that enable network or unrestricted exec.
	•	Telemetry (if added) must be opt-in and never collect source code content.

⸻

12. Testing and CI requirements

Required CI checks
	•	agents validate
	•	agents test adapters
	•	Optional strictness:
	•	agents doctor --ci to fail on drift/collisions

Golden fixtures
	•	Fixtures include representative repo shapes (single repo, monorepo, scoped overrides).
	•	Fixtures include representative profiles (dev, ci) and modes.

⸻

13. Rollout plan

Phase 1 (v1 MVP)
	•	.agents schema + folder structure
	•	Backends: vfs_container, materialize
	•	Adapters: Codex, Claude Code, Gemini CLI, Gemini GitHub, OpenCode, Cursor, Copilot
	•	CLI: init/validate/status/set-mode/preview/diff/sync/run/doctor/clean/import/explain/compat/test

Phase 2 (v1.1)
	•	vfs_mount backend (advanced IDE workflow)
	•	Improved enforceability mappings per agent
	•	Skill packaging bundles (local lockfile)
	•	IDE integration (optional VS Code extension for schema completion + mode switching)

⸻

14. Risks and mitigations
	1.	IDE agents don’t work well with VFS container overlays
Mitigation: default to materialize for Cursor/Copilot; provide vfs_mount later.
	2.	Agent surfaces evolve over time
Mitigation: compatibility matrix + adapter golden tests + doc references for each adapter.
	3.	User confusion about “current mode”
Mitigation: agents status, mode banners in generated outputs, and a gitignored .agents/state/state.yaml.
	4.	Collision and shared-surface complexity
Mitigation: explicit collision policy and a “shared surface owner” concept in the manifest.

⸻

15. Appendix: Minimum viable templates (illustrative)

Shared AGENTS.md template concept (for Codex/OpenCode/Copilot shared surface)
	•	Includes:
	•	setup commands
	•	test/lint commands
	•	coding conventions
	•	safety policy summary
	•	current mode/profile banner
	•	Designed to be compatible with Codex’s AGENTS behavior.  ￼

Cursor rules generation
	•	One file per logical category (style, testing, architecture) for readability.
	•	Stored under .cursor/rules per Cursor docs.  ￼

⸻
