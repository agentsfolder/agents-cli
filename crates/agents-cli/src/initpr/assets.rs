#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitPreset {
    Conservative,
    Standard,
    CiSafe,
    Monorepo,
    AgentPack,
}

impl InitPreset {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "conservative" => Some(Self::Conservative),
            "standard" => Some(Self::Standard),
            "ci-safe" => Some(Self::CiSafe),
            "monorepo" => Some(Self::Monorepo),
            "agent-pack" => Some(Self::AgentPack),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Conservative => "conservative",
            Self::Standard => "standard",
            Self::CiSafe => "ci-safe",
            Self::Monorepo => "monorepo",
            Self::AgentPack => "agent-pack",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EmbeddedFile {
    pub rel_path: &'static str,
    pub contents: &'static str,
}

pub fn files_for_preset(preset: InitPreset) -> Vec<EmbeddedFile> {
    let mut out: Vec<EmbeddedFile> = vec![];
    out.extend(common_files());

    match preset {
        InitPreset::Conservative => {
            out.push(file(
                ".agents/manifest.yaml",
                include_str!("assets/conservative/manifest.yaml"),
            ));
        }
        InitPreset::Standard => {
            out.push(file(
                ".agents/manifest.yaml",
                include_str!("assets/standard/manifest.yaml"),
            ));
        }
        InitPreset::CiSafe => {
            out.push(file(
                ".agents/manifest.yaml",
                include_str!("assets/ci-safe/manifest.yaml"),
            ));
        }
        InitPreset::Monorepo => {
            out.push(file(
                ".agents/manifest.yaml",
                include_str!("assets/monorepo/manifest.yaml"),
            ));
            out.push(file(
                ".agents/scopes/packages.yaml",
                include_str!("assets/monorepo/scopes/packages.yaml"),
            ));
        }
        InitPreset::AgentPack => {
            out.push(file(
                ".agents/manifest.yaml",
                include_str!("assets/agent-pack/manifest.yaml"),
            ));
            out.extend(agent_pack_adapters());
        }
    }

    out
}

fn file(rel_path: &'static str, contents: &'static str) -> EmbeddedFile {
    EmbeddedFile { rel_path, contents }
}

fn common_files() -> Vec<EmbeddedFile> {
    vec![
        // Prompts
        file(
            ".agents/prompts/base.md",
            include_str!("assets/common/prompts/base.md"),
        ),
        file(
            ".agents/prompts/project.md",
            include_str!("assets/common/prompts/project.md"),
        ),
        file(
            ".agents/prompts/snippets/example.md",
            include_str!("assets/common/prompts/snippets/example.md"),
        ),
        // Modes
        file(
            ".agents/modes/default.md",
            include_str!("assets/common/modes/default.md"),
        ),
        file(
            ".agents/modes/readonly-audit.md",
            include_str!("assets/common/modes/readonly-audit.md"),
        ),
        // Policies
        file(
            ".agents/policies/safe.yaml",
            include_str!("assets/common/policies/safe.yaml"),
        ),
        file(
            ".agents/policies/conservative.yaml",
            include_str!("assets/common/policies/conservative.yaml"),
        ),
        file(
            ".agents/policies/ci-safe.yaml",
            include_str!("assets/common/policies/ci-safe.yaml"),
        ),
        // Schemas
        file(
            ".agents/schemas/manifest.schema.json",
            include_str!("assets/common/schemas/manifest.schema.json"),
        ),
        file(
            ".agents/schemas/policy.schema.json",
            include_str!("assets/common/schemas/policy.schema.json"),
        ),
        file(
            ".agents/schemas/adapter.schema.json",
            include_str!("assets/common/schemas/adapter.schema.json"),
        ),
        file(
            ".agents/schemas/scope.schema.json",
            include_str!("assets/common/schemas/scope.schema.json"),
        ),
        file(
            ".agents/schemas/skill.schema.json",
            include_str!("assets/common/schemas/skill.schema.json"),
        ),
        file(
            ".agents/schemas/state.schema.json",
            include_str!("assets/common/schemas/state.schema.json"),
        ),
        file(
            ".agents/schemas/mode-frontmatter.schema.json",
            include_str!("assets/common/schemas/mode-frontmatter.schema.json"),
        ),
        // State
        file(
            ".agents/state/.gitignore",
            include_str!("assets/common/state/.gitignore"),
        ),
    ]
}

fn agent_pack_adapters() -> Vec<EmbeddedFile> {
    vec![
        // Cursor
        file(
            ".agents/adapters/cursor/adapter.yaml",
            include_str!("assets/agent-pack/adapters/cursor/adapter.yaml"),
        ),
        file(
            ".agents/adapters/cursor/templates/00-current-mode.md.hbs",
            include_str!("assets/agent-pack/adapters/cursor/templates/00-current-mode.md.hbs"),
        ),
        file(
            ".agents/adapters/cursor/templates/10-guidance.md.hbs",
            include_str!("assets/agent-pack/adapters/cursor/templates/10-guidance.md.hbs"),
        ),
        file(
            ".agents/adapters/cursor/templates/20-policy.md.hbs",
            include_str!("assets/agent-pack/adapters/cursor/templates/20-policy.md.hbs"),
        ),
        // Copilot
        file(
            ".agents/adapters/copilot/adapter.yaml",
            include_str!("assets/agent-pack/adapters/copilot/adapter.yaml"),
        ),
        file(
            ".agents/adapters/copilot/templates/copilot-instructions.md.hbs",
            include_str!(
                "assets/agent-pack/adapters/copilot/templates/copilot-instructions.md.hbs"
            ),
        ),
        file(
            ".agents/adapters/copilot/templates/scope.instructions.md.hbs",
            include_str!("assets/agent-pack/adapters/copilot/templates/scope.instructions.md.hbs"),
        ),
        // OpenCode
        file(
            ".agents/adapters/opencode/adapter.yaml",
            include_str!("assets/agent-pack/adapters/opencode/adapter.yaml"),
        ),
        file(
            ".agents/adapters/opencode/templates/opencode.jsonc.hbs",
            include_str!("assets/agent-pack/adapters/opencode/templates/opencode.jsonc.hbs"),
        ),
        // Gemini CLI
        file(
            ".agents/adapters/gemini-cli/adapter.yaml",
            include_str!("assets/agent-pack/adapters/gemini-cli/adapter.yaml"),
        ),
        file(
            ".agents/adapters/gemini-cli/templates/settings.json.hbs",
            include_str!("assets/agent-pack/adapters/gemini-cli/templates/settings.json.hbs"),
        ),
        // Claude Code
        file(
            ".agents/adapters/claude/adapter.yaml",
            include_str!("assets/agent-pack/adapters/claude/adapter.yaml"),
        ),
        file(
            ".agents/adapters/claude/templates/settings.json.hbs",
            include_str!("assets/agent-pack/adapters/claude/templates/settings.json.hbs"),
        ),
        file(
            ".agents/adapters/claude/templates/CLAUDE.md.hbs",
            include_str!("assets/agent-pack/adapters/claude/templates/CLAUDE.md.hbs"),
        ),
        // Codex (AGENTS.md)
        file(
            ".agents/adapters/codex/adapter.yaml",
            include_str!("assets/agent-pack/adapters/codex/adapter.yaml"),
        ),
        file(
            ".agents/adapters/codex/templates/AGENTS.md.hbs",
            include_str!("assets/agent-pack/adapters/codex/templates/AGENTS.md.hbs"),
        ),
    ]
}
