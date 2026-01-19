use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModeFrontmatter {
    #[serde(default)]
    pub id: Option<String>,

    #[serde(default)]
    pub title: Option<String>,

    #[serde(default)]
    pub policy: Option<String>,

    #[serde(default, rename = "enableSkills")]
    pub enable_skills: Vec<String>,

    #[serde(default, rename = "disableSkills")]
    pub disable_skills: Vec<String>,

    #[serde(default, rename = "includeSnippets")]
    pub include_snippets: Vec<String>,

    #[serde(default, rename = "toolIntent")]
    pub tool_intent: Option<ToolIntent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolIntent {
    #[serde(default)]
    pub allow: Vec<String>,

    #[serde(default)]
    pub deny: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ModeFile {
    pub frontmatter: Option<ModeFrontmatter>,
    pub body: String,
}

pub fn parse_frontmatter_markdown(
    text: &str,
) -> Result<(Option<ModeFrontmatter>, String), serde_yaml::Error> {
    let normalized = text.replace("\r\n", "\n");

    if !normalized.starts_with("---\n") {
        return Ok((None, normalized));
    }

    let rest = &normalized[4..];
    if let Some(end) = rest.find("\n---\n") {
        let (fm_str, body) = rest.split_at(end);
        let body = &body[5..];
        let fm: ModeFrontmatter = serde_yaml::from_str(fm_str)?;
        return Ok((Some(fm), body.to_string()));
    }

    // frontmatter started but not properly terminated; try to parse as YAML to get a good error.
    // This will fail with a YAML error, which is what we want.
    let _: serde_yaml::Value = serde_yaml::from_str(rest)?;
    Ok((None, normalized))
}
