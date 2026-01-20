use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::fsutil;
use crate::loadag::{LoadError, LoadReport, RepoConfig};
use crate::model::{
    parse_frontmatter_markdown, Adapter, Manifest, ModeFile, Policy, Scope, Skill, State,
};

#[derive(Debug, Clone)]
pub struct LoaderOptions {
    /// If true, missing `.agents/schemas` is an error.
    pub require_schemas_dir: bool,
}

impl Default for LoaderOptions {
    fn default() -> Self {
        Self {
            require_schemas_dir: false,
        }
    }
}

pub fn load_repo_config(
    repo_root: &Path,
    opts: &LoaderOptions,
) -> Result<(RepoConfig, LoadReport), LoadError> {
    let mut report = LoadReport::new();

    let agents_dir = fsutil::agents_dir(repo_root);

    // Required: manifest
    let manifest_path = agents_dir.join("manifest.yaml");
    if !manifest_path.is_file() {
        return Err(LoadError::NotInitialized {
            path: manifest_path,
        });
    }

    let manifest_str = fsutil::read_to_string(&manifest_path).map_err(|e| LoadError::Io {
        path: manifest_path.clone(),
        source: io_from_fs_error(e),
    })?;

    let manifest: Manifest = serde_yaml::from_str(&manifest_str).map_err(|e| LoadError::Parse {
        path: manifest_path.clone(),
        message: e.to_string(),
    })?;

    // Required: prompts base/project
    let prompts_dir = agents_dir.join("prompts");
    let base_path = prompts_dir.join("base.md");
    let project_path = prompts_dir.join("project.md");
    if !base_path.is_file() {
        return Err(LoadError::NotInitialized { path: base_path });
    }
    if !project_path.is_file() {
        return Err(LoadError::NotInitialized { path: project_path });
    }

    let base_md = fsutil::read_to_string(&base_path).map_err(|e| LoadError::Io {
        path: base_path.clone(),
        source: io_from_fs_error(e),
    })?;
    let project_md = fsutil::read_to_string(&project_path).map_err(|e| LoadError::Io {
        path: project_path.clone(),
        source: io_from_fs_error(e),
    })?;

    // Schemas directory behavior
    let schemas_dir = agents_dir.join("schemas");
    if !schemas_dir.is_dir() {
        if opts.require_schemas_dir {
            return Err(LoadError::NotInitialized { path: schemas_dir });
        } else {
            report.warn(
                Some(schemas_dir),
                "missing .agents/schemas; schema validation will not be available until `agents init`",
            );
        }
    }

    // Load snippets
    let mut snippets: BTreeMap<String, String> = BTreeMap::new();
    let snippets_dir = prompts_dir.join("snippets");
    if snippets_dir.is_dir() {
        for entry in std::fs::read_dir(&snippets_dir).map_err(|e| LoadError::Io {
            path: snippets_dir.clone(),
            source: e,
        })? {
            let entry = entry.map_err(|e| LoadError::Io {
                path: snippets_dir.clone(),
                source: e,
            })?;

            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();
            let text = fsutil::read_to_string(&path).map_err(|e| LoadError::Io {
                path: path.clone(),
                source: io_from_fs_error(e),
            })?;
            snippets.insert(stem, text);
        }
    }

    // Collections
    let policies =
        load_yaml_dir::<Policy>(&agents_dir.join("policies"), "policies", |p| p.id.clone())?;

    let (skills, skill_dirs) = load_skills_dir(&agents_dir.join("skills"))?;

    let scopes = load_yaml_dir::<Scope>(&agents_dir.join("scopes"), "scopes", |s| s.id.clone())?;

    let (modes, _mode_sources) = load_modes_dir(&agents_dir.join("modes"))?;

    let (mut adapters, adapter_template_dirs) = load_adapters_dir(&agents_dir.join("adapters"))?;

    crate::shared::inject_builtin_adapters(&mut adapters);

    let profiles = load_profiles_dir(&agents_dir.join("profiles"))?;

    let state = load_optional_state(&agents_dir.join("state/state.yaml"))?;

    let cfg = RepoConfig {
        repo_root: repo_root.to_path_buf(),
        manifest,
        policies,
        skills,
        skill_dirs,
        scopes,
        modes,
        adapters,
        adapter_template_dirs,
        profiles,
        prompts: crate::loadag::PromptLibrary {
            base_md,
            project_md,
            snippets,
        },
        state,
    };

    crate::loadag::check_referential_integrity(&cfg)?;

    Ok((cfg, report))
}

fn load_yaml_dir<T: serde::de::DeserializeOwned>(
    dir: &Path,
    kind: &'static str,
    id_fn: impl Fn(&T) -> String,
) -> Result<BTreeMap<String, T>, LoadError> {
    let mut out: BTreeMap<String, T> = BTreeMap::new();

    if !dir.is_dir() {
        return Ok(out);
    }

    let mut entries: Vec<PathBuf> = vec![];
    for entry in std::fs::read_dir(dir).map_err(|e| LoadError::Io {
        path: dir.to_path_buf(),
        source: e,
    })? {
        let entry = entry.map_err(|e| LoadError::Io {
            path: dir.to_path_buf(),
            source: e,
        })?;
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) == Some("yaml") {
            entries.push(p);
        }
    }
    entries.sort();

    for path in entries {
        let text = fsutil::read_to_string(&path).map_err(|e| LoadError::Io {
            path: path.clone(),
            source: io_from_fs_error(e),
        })?;
        let obj: T = serde_yaml::from_str(&text).map_err(|e| LoadError::Parse {
            path: path.clone(),
            message: e.to_string(),
        })?;

        let id = id_fn(&obj);
        if out.contains_key(&id) {
            return Err(LoadError::DuplicateId { kind, id });
        }
        out.insert(id, obj);
    }

    Ok(out)
}

fn load_skills_dir(
    dir: &Path,
) -> Result<(BTreeMap<String, Skill>, BTreeMap<String, PathBuf>), LoadError> {
    let mut skills = BTreeMap::new();
    let mut dirs = BTreeMap::new();

    if !dir.is_dir() {
        return Ok((skills, dirs));
    }

    let mut entries: Vec<PathBuf> = vec![];
    for entry in std::fs::read_dir(dir).map_err(|e| LoadError::Io {
        path: dir.to_path_buf(),
        source: e,
    })? {
        let entry = entry.map_err(|e| LoadError::Io {
            path: dir.to_path_buf(),
            source: e,
        })?;
        let p = entry.path();
        if p.is_dir() {
            entries.push(p);
        }
    }
    entries.sort();

    for skill_dir in entries {
        let skill_yaml = skill_dir.join("skill.yaml");
        if !skill_yaml.is_file() {
            continue;
        }
        let text = fsutil::read_to_string(&skill_yaml).map_err(|e| LoadError::Io {
            path: skill_yaml.clone(),
            source: io_from_fs_error(e),
        })?;

        let skill: Skill = serde_yaml::from_str(&text).map_err(|e| LoadError::Parse {
            path: skill_yaml.clone(),
            message: e.to_string(),
        })?;

        if skills.contains_key(&skill.id) {
            return Err(LoadError::DuplicateId {
                kind: "skills",
                id: skill.id,
            });
        }
        dirs.insert(skill.id.clone(), skill_dir);
        skills.insert(skill.id.clone(), skill);
    }

    Ok((skills, dirs))
}

fn load_modes_dir(
    dir: &Path,
) -> Result<(BTreeMap<String, ModeFile>, BTreeMap<String, PathBuf>), LoadError> {
    let mut modes = BTreeMap::new();
    let mut sources = BTreeMap::new();

    if !dir.is_dir() {
        return Ok((modes, sources));
    }

    let mut entries: Vec<PathBuf> = vec![];
    for entry in std::fs::read_dir(dir).map_err(|e| LoadError::Io {
        path: dir.to_path_buf(),
        source: e,
    })? {
        let entry = entry.map_err(|e| LoadError::Io {
            path: dir.to_path_buf(),
            source: e,
        })?;
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) == Some("md") {
            entries.push(p);
        }
    }
    entries.sort();

    for path in entries {
        let text = fsutil::read_to_string(&path).map_err(|e| LoadError::Io {
            path: path.clone(),
            source: io_from_fs_error(e),
        })?;

        let (frontmatter, body) =
            parse_frontmatter_markdown(&text).map_err(|e| LoadError::Parse {
                path: path.clone(),
                message: e.to_string(),
            })?;

        let id = frontmatter
            .as_ref()
            .and_then(|fm| fm.id.clone())
            .or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "mode".to_string());

        if modes.contains_key(&id) {
            return Err(LoadError::DuplicateId { kind: "modes", id });
        }

        modes.insert(id.clone(), ModeFile { frontmatter, body });
        sources.insert(id, path);
    }

    Ok((modes, sources))
}

fn load_adapters_dir(
    dir: &Path,
) -> Result<(BTreeMap<String, Adapter>, BTreeMap<String, PathBuf>), LoadError> {
    let mut adapters = BTreeMap::new();
    let mut template_dirs = BTreeMap::new();

    if !dir.is_dir() {
        return Ok((adapters, template_dirs));
    }

    let mut entries: Vec<PathBuf> = vec![];
    for entry in std::fs::read_dir(dir).map_err(|e| LoadError::Io {
        path: dir.to_path_buf(),
        source: e,
    })? {
        let entry = entry.map_err(|e| LoadError::Io {
            path: dir.to_path_buf(),
            source: e,
        })?;
        let p = entry.path();
        if p.is_dir() {
            entries.push(p);
        }
    }
    entries.sort();

    for adapter_dir in entries {
        let adapter_yaml = adapter_dir.join("adapter.yaml");
        if !adapter_yaml.is_file() {
            continue;
        }

        let text = fsutil::read_to_string(&adapter_yaml).map_err(|e| LoadError::Io {
            path: adapter_yaml.clone(),
            source: io_from_fs_error(e),
        })?;

        let adapter: Adapter = serde_yaml::from_str(&text).map_err(|e| LoadError::Parse {
            path: adapter_yaml.clone(),
            message: e.to_string(),
        })?;

        if adapters.contains_key(&adapter.agent_id) {
            return Err(LoadError::DuplicateId {
                kind: "adapters",
                id: adapter.agent_id,
            });
        }

        template_dirs.insert(adapter.agent_id.clone(), adapter_dir.join("templates"));
        adapters.insert(adapter.agent_id.clone(), adapter);
    }

    Ok((adapters, template_dirs))
}

fn load_profiles_dir(dir: &Path) -> Result<BTreeMap<String, serde_yaml::Value>, LoadError> {
    let mut out = BTreeMap::new();
    if !dir.is_dir() {
        return Ok(out);
    }

    let mut entries: Vec<PathBuf> = vec![];
    for entry in std::fs::read_dir(dir).map_err(|e| LoadError::Io {
        path: dir.to_path_buf(),
        source: e,
    })? {
        let entry = entry.map_err(|e| LoadError::Io {
            path: dir.to_path_buf(),
            source: e,
        })?;
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) == Some("yaml") {
            entries.push(p);
        }
    }
    entries.sort();

    for path in entries {
        let text = fsutil::read_to_string(&path).map_err(|e| LoadError::Io {
            path: path.clone(),
            source: io_from_fs_error(e),
        })?;
        let v: serde_yaml::Value = serde_yaml::from_str(&text).map_err(|e| LoadError::Parse {
            path: path.clone(),
            message: e.to_string(),
        })?;
        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        if out.contains_key(&id) {
            return Err(LoadError::DuplicateId {
                kind: "profiles",
                id,
            });
        }
        out.insert(id, v);
    }

    Ok(out)
}

fn load_optional_state(path: &Path) -> Result<Option<State>, LoadError> {
    if !path.is_file() {
        return Ok(None);
    }

    let text = fsutil::read_to_string(path).map_err(|e| LoadError::Io {
        path: path.to_path_buf(),
        source: io_from_fs_error(e),
    })?;

    let state: State = serde_yaml::from_str(&text).map_err(|e| LoadError::Parse {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;

    Ok(Some(state))
}

fn io_from_fs_error(err: fsutil::FsError) -> std::io::Error {
    match err {
        fsutil::FsError::Io { source, .. } => source,
        fsutil::FsError::PathEscapesRepo { .. } => {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, err.to_string())
        }
    }
}
