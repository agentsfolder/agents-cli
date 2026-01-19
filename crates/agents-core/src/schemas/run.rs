use std::path::Path;

use crate::loadag::RepoConfig;
use crate::model::parse_frontmatter_markdown;

use super::validate::{
    frontmatter_to_json_value, validate_json, yaml_to_json_value, SchemaInvalid, SchemaKind,
    SchemaStore,
};

pub fn validate_repo_config(repo_root: &Path, cfg: &RepoConfig) -> Result<(), SchemaInvalid> {
    let mut store = SchemaStore::load(repo_root)?;

    // Manifest
    validate_yaml_file(
        &mut store,
        SchemaKind::Manifest,
        &repo_root.join(".agents/manifest.yaml"),
    )?;

    // Policies
    for (id, _) in &cfg.policies {
        let path = repo_root.join(format!(".agents/policies/{id}.yaml"));
        if path.is_file() {
            validate_yaml_file(&mut store, SchemaKind::Policy, &path)?;
        }
    }

    // Skills
    for (_id, dir) in &cfg.skill_dirs {
        let path = dir.join("skill.yaml");
        if path.is_file() {
            validate_yaml_file(&mut store, SchemaKind::Skill, &path)?;
        }
    }

    // Scopes
    for (id, _) in &cfg.scopes {
        let path = repo_root.join(format!(".agents/scopes/{id}.yaml"));
        if path.is_file() {
            validate_yaml_file(&mut store, SchemaKind::Scope, &path)?;
        }
    }

    // Adapters
    for (_id, adapter) in &cfg.adapters {
        // Adapter file path is not stored in cfg, so derive from agent_id.
        let path = repo_root.join(format!(
            ".agents/adapters/{}/adapter.yaml",
            adapter.agent_id
        ));
        if path.is_file() {
            validate_yaml_file(&mut store, SchemaKind::Adapter, &path)?;
        }
    }

    // State
    let state_path = repo_root.join(".agents/state/state.yaml");
    if state_path.is_file() {
        validate_yaml_file(&mut store, SchemaKind::State, &state_path)?;
    }

    // Modes: validate frontmatter only if present
    for (id, _mode) in &cfg.modes {
        let path = repo_root.join(format!(".agents/modes/{id}.md"));
        if !path.is_file() {
            continue;
        }

        let text = crate::fsutil::read_to_string(&path).map_err(|e| SchemaInvalid {
            path: path.clone(),
            schema: SchemaKind::ModeFrontmatter.schema_file_name().to_string(),
            pointer: "".to_string(),
            message: e.to_string(),
        })?;

        let (frontmatter, _body) =
            parse_frontmatter_markdown(&text).map_err(|e| SchemaInvalid {
                path: path.clone(),
                schema: SchemaKind::ModeFrontmatter.schema_file_name().to_string(),
                pointer: "".to_string(),
                message: e.to_string(),
            })?;

        if let Some(fm) = frontmatter {
            let json = frontmatter_to_json_value(&fm).map_err(|e| SchemaInvalid {
                path: path.clone(),
                schema: SchemaKind::ModeFrontmatter.schema_file_name().to_string(),
                pointer: "".to_string(),
                message: e,
            })?;
            validate_json(&mut store, SchemaKind::ModeFrontmatter, &path, &json)?;
        }
    }

    Ok(())
}

fn validate_yaml_file(
    store: &mut SchemaStore,
    kind: SchemaKind,
    path: &Path,
) -> Result<(), SchemaInvalid> {
    let text = crate::fsutil::read_to_string(path).map_err(|e| SchemaInvalid {
        path: path.to_path_buf(),
        schema: kind.schema_file_name().to_string(),
        pointer: "".to_string(),
        message: e.to_string(),
    })?;

    let json = yaml_to_json_value(&text).map_err(|e| SchemaInvalid {
        path: path.to_path_buf(),
        schema: kind.schema_file_name().to_string(),
        pointer: "".to_string(),
        message: e,
    })?;

    validate_json(store, kind, path, &json)
}
