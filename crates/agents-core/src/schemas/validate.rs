use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use jsonschema::{Draft, JSONSchema};

use crate::fsutil;

#[derive(Debug, Clone)]
pub struct SchemaInvalid {
    pub path: PathBuf,
    pub schema: String,
    pub pointer: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy)]
pub enum SchemaKind {
    Manifest,
    Policy,
    Skill,
    Scope,
    Adapter,
    State,
    ModeFrontmatter,
}

impl SchemaKind {
    pub fn schema_file_name(&self) -> &'static str {
        match self {
            SchemaKind::Manifest => "manifest.schema.json",
            SchemaKind::Policy => "policy.schema.json",
            SchemaKind::Skill => "skill.schema.json",
            SchemaKind::Scope => "scope.schema.json",
            SchemaKind::Adapter => "adapter.schema.json",
            SchemaKind::State => "state.schema.json",
            SchemaKind::ModeFrontmatter => "mode-frontmatter.schema.json",
        }
    }
}

#[derive(Debug)]
pub struct SchemaStore {
    repo_root: PathBuf,
    compiled: BTreeMap<String, JSONSchema>,
}

impl SchemaStore {
    pub fn load(repo_root: &Path) -> Result<Self, SchemaInvalid> {
        Ok(Self {
            repo_root: repo_root.to_path_buf(),
            compiled: BTreeMap::new(),
        })
    }

    pub fn get(&mut self, kind: SchemaKind) -> Result<&JSONSchema, SchemaInvalid> {
        let schema_path = self
            .repo_root
            .join(".agents/schemas")
            .join(kind.schema_file_name());

        let key = schema_path.to_string_lossy().to_string();
        if self.compiled.contains_key(&key) {
            return Ok(self.compiled.get(&key).unwrap());
        }

        let schema_text = fsutil::read_to_string(&schema_path).map_err(|e| SchemaInvalid {
            path: schema_path.clone(),
            schema: kind.schema_file_name().to_string(),
            pointer: "".to_string(),
            message: e.to_string(),
        })?;

        let schema_json: serde_json::Value =
            serde_json::from_str(&schema_text).map_err(|e| SchemaInvalid {
                path: schema_path.clone(),
                schema: kind.schema_file_name().to_string(),
                pointer: "".to_string(),
                message: format!("invalid schema json: {e}"),
            })?;

        let compiled = JSONSchema::options()
            .with_draft(Draft::Draft7)
            .compile(&schema_json)
            .map_err(|e| SchemaInvalid {
                path: schema_path.clone(),
                schema: kind.schema_file_name().to_string(),
                pointer: "".to_string(),
                message: format!("failed to compile schema: {e}"),
            })?;

        self.compiled.insert(key.clone(), compiled);
        Ok(self.compiled.get(&key).unwrap())
    }
}

pub fn yaml_to_json_value(text: &str) -> Result<serde_json::Value, String> {
    let y: serde_yaml::Value = serde_yaml::from_str(text).map_err(|e| e.to_string())?;
    serde_json::to_value(y).map_err(|e| e.to_string())
}

pub fn validate_json(
    store: &mut SchemaStore,
    kind: SchemaKind,
    path: &Path,
    json: &serde_json::Value,
) -> Result<(), SchemaInvalid> {
    let schema = store.get(kind)?;

    let result = schema.validate(json);
    if let Err(errors) = result {
        // Choose the first error for v1 (fail-fast).
        if let Some(err) = errors.into_iter().next() {
            return Err(SchemaInvalid {
                path: path.to_path_buf(),
                schema: kind.schema_file_name().to_string(),
                pointer: err.instance_path.to_string(),
                message: err.to_string(),
            });
        }
    }

    Ok(())
}
