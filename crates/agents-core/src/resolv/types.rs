use std::path::PathBuf;

use crate::model::BackendKind;

#[derive(Debug, Clone)]
pub struct ResolutionRequest {
    pub repo_root: PathBuf,

    /// Repo-relative path used for scope matching.
    pub target_path: Option<String>,

    pub override_mode: Option<String>,
    pub override_policy: Option<String>,
    pub override_profile: Option<String>,
    pub override_backend: Option<BackendKind>,

    /// Explicitly selected scopes (optional; if empty, scope matching is automatic).
    pub override_scopes: Vec<String>,

    /// Whether to enable user overlay resolution if present.
    pub enable_user_overlay: bool,

    /// Optional override for where to look for the user overlay directory.
    ///
    /// When set, the overlay is loaded from `<user_overlay_root>/.agents/manifest.yaml`.
    pub user_overlay_root: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ScopeMatch {
    pub id: String,
    pub score: i64,
    pub priority: i64,
}

#[derive(Debug, Clone)]
pub struct EffectiveConfig {
    pub mode_id: String,
    pub policy_id: String,
    pub profile: Option<String>,
    pub backend: BackendKind,

    pub scopes_matched: Vec<ScopeMatch>,

    pub skill_ids_enabled: Vec<String>,
    pub snippet_ids_included: Vec<String>,
}
