use agents_core::model::BackendKind;

#[derive(Debug, Clone, serde::Serialize)]
pub struct StatusReport {
    pub repo_root: String,

    pub effective_mode: String,
    pub effective_policy: String,
    pub effective_profile: Option<String>,
    pub effective_backend: BackendKind,

    pub scopes_matched: Vec<String>,
    pub skills_enabled: Vec<String>,

    pub agent_id: Option<String>,

    pub hints: Vec<String>,
}

impl StatusReport {
    pub fn render_human(&self) -> String {
        let mut out = String::new();

        out.push_str(&format!("repo: {}\n", self.repo_root));
        out.push_str(&format!("mode: {}\n", self.effective_mode));
        out.push_str(&format!("policy: {}\n", self.effective_policy));
        out.push_str(&format!(
            "profile: {}\n",
            self.effective_profile.as_deref().unwrap_or("<none>")
        ));
        out.push_str(&format!("backend: {:?}\n", self.effective_backend));

        out.push_str("scopes:\n");
        if self.scopes_matched.is_empty() {
            out.push_str("  - <none>\n");
        } else {
            for s in &self.scopes_matched {
                out.push_str(&format!("  - {s}\n"));
            }
        }

        out.push_str("skills:\n");
        if self.skills_enabled.is_empty() {
            out.push_str("  - <none>\n");
        } else {
            for s in &self.skills_enabled {
                out.push_str(&format!("  - {s}\n"));
            }
        }

        if let Some(agent) = &self.agent_id {
            out.push_str(&format!("agent: {agent}\n"));
        }

        if !self.hints.is_empty() {
            out.push_str("hints:\n");
            for h in &self.hints {
                out.push_str(&format!("  - {h}\n"));
            }
        }

        out
    }
}
