use std::path::Path;

use crate::loadag::{load_repo_config, LoadError, LoaderOptions};

use super::{validate_repo_config, SchemaInvalid};

pub fn validate_repo(repo_root: &Path) -> Result<(), SchemaInvalid> {
    let opts = LoaderOptions {
        require_schemas_dir: true,
    };

    let (cfg, _report) = load_repo_config(repo_root, &opts).map_err(|e| match e {
        LoadError::Parse { path, message } => SchemaInvalid {
            path,
            schema: "yaml".to_string(),
            pointer: "".to_string(),
            message,
            hint: None,
        },
        other => SchemaInvalid {
            path: repo_root.join(".agents"),
            schema: "load".to_string(),
            pointer: "".to_string(),
            message: other.to_string(),
            hint: None,
        },
    })?;

    validate_repo_config(repo_root, &cfg)
}
