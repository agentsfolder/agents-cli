use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use agents_core::driftx::unified_diff_for;
use agents_core::loadag::{load_repo_config, LoaderOptions};
use agents_core::outputs::{plan_outputs, render_planned_output};
use agents_core::resolv::{ResolutionRequest, Resolver};
use agents_core::{fsutil, schemas};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct FileMismatch {
    pub path: String,
    pub kind: String,
    pub diff: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FixtureFailure {
    pub fixture: String,
    pub agent_id: String,
    pub case: String,
    pub actual_dir: PathBuf,
    pub mismatches: Vec<FileMismatch>,
}

impl FixtureFailure {
    pub fn render_human(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "fixture {} (agent {}, case {}): {} mismatches\n",
            self.fixture,
            self.agent_id,
            self.case,
            self.mismatches.len()
        ));
        for m in &self.mismatches {
            out.push_str(&format!("- {}: {}\n", m.kind, m.path));
            if let Some(d) = &m.diff {
                out.push_str(d);
                if !d.ends_with('\n') {
                    out.push('\n');
                }
            }
        }
        out.push_str(&format!("actual outputs: {}\n", self.actual_dir.display()));
        out
    }
}

#[derive(Debug, Clone, Default)]
pub struct TestReport {
    pub passed: usize,
    pub failed: usize,
    pub failures: Vec<FixtureFailure>,
}

#[derive(Debug, thiserror::Error)]
pub enum TestError {
    #[error("fs error: {0}")]
    Fs(#[from] fsutil::FsError),

    #[error("load error: {0}")]
    Load(String),

    #[error("resolve error: {0}")]
    Resolve(String),

    #[error("plan error: {0}")]
    Plan(String),

    #[error("render error: {0}")]
    Render(String),
}

pub fn run_fixture(fixture_root: &Path, agent_filter: Option<&str>) -> Result<TestReport, TestError> {
    let repo_root = fixture_root.join("repo");
    let expect_root = fixture_root.join("expect");
    let matrix_path = fixture_root.join("matrix.yaml");

    let (repo, _report) = load_repo_config(
        &repo_root,
        &LoaderOptions {
            require_schemas_dir: false,
        },
    )
    .map_err(|e| TestError::Load(e.to_string()))?;

    // Validate schemas best-effort.
    let _ = schemas::validate_repo(&repo_root);

    let resolver = Resolver::new(repo.clone());

    let (cases, use_case_subdir) = load_matrix(&matrix_path)?;

    let mut agent_ids: Vec<String> = repo.manifest.enabled.adapters.clone();
    agent_ids.sort();
    if let Some(filter) = agent_filter {
        agent_ids.retain(|a| a == filter);
    }

    let mut out = TestReport::default();
    for agent_id in agent_ids {
        for case in &cases {
            let mut req = ResolutionRequest::default();
            req.repo_root = repo_root.clone();
            req.override_mode = case.mode.clone();
            req.override_profile = case.profile.clone();
            req.override_backend = case.backend;

            let effective = resolver
                .resolve(&req)
                .map_err(|e| TestError::Resolve(e.to_string()))?;

            let plan = plan_outputs(&repo_root, repo.clone(), &effective, &agent_id)
                .map_err(|e| TestError::Plan(e.to_string()))?
                .plan;

            let tmp = fsutil::temp_generation_dir("agents-fixture").map_err(TestError::Fs)?;
            let tmp_path = tmp.path().to_path_buf();

            // Render outputs into temp dir.
            for p in &plan.outputs {
                let rendered = render_planned_output(&repo_root, p)
                    .map_err(|e| TestError::Render(e.to_string()))?;
                let dest = tmp_path.join(p.path.as_str());
                fsutil::atomic_write(&dest, rendered.content_with_stamp.as_bytes())?;
            }

            let expect_dir = if use_case_subdir {
                expect_root.join(&agent_id).join(&case.name)
            } else {
                expect_root.join(&agent_id)
            };
            let mismatches = compare_dirs(&expect_dir, &tmp_path)?;

            if mismatches.is_empty() {
                out.passed += 1;
            } else {
                out.failed += 1;
                out.failures.push(FixtureFailure {
                    fixture: fixture_root
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("<fixture>")
                        .to_string(),
                    agent_id: agent_id.clone(),
                    case: case.name.clone(),
                    actual_dir: tmp_path,
                    mismatches,
                });

                // Keep tmp dir for inspection when failing.
                std::mem::forget(tmp);
            }
        }
    }

    Ok(out)
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureMatrix {
    #[serde(default)]
    cases: Vec<FixtureCase>,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureCase {
    name: String,
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    profile: Option<String>,
    #[serde(default)]
    backend: Option<agents_core::model::BackendKind>,
}

fn load_matrix(path: &Path) -> Result<(Vec<FixtureCase>, bool), TestError> {
    if !path.is_file() {
        return Ok((
            vec![FixtureCase {
                name: "default".to_string(),
                mode: None,
                profile: None,
                backend: None,
            }],
            false,
        ));
    }

    let s = fsutil::read_to_string(path)?;
    let m: FixtureMatrix = serde_yaml::from_str(&s)
        .map_err(|e| TestError::Load(format!("invalid matrix.yaml: {e}")))?;

    let mut cases = m.cases;
    if cases.is_empty() {
        cases.push(FixtureCase {
            name: "default".to_string(),
            mode: None,
            profile: None,
            backend: None,
        });
    }

    Ok((cases, true))
}

fn compare_dirs(expect_dir: &Path, actual_dir: &Path) -> Result<Vec<FileMismatch>, TestError> {
    let expected_files = collect_rel_files(expect_dir)?;
    let actual_files = collect_rel_files(actual_dir)?;

    let mut all: BTreeSet<String> = BTreeSet::new();
    for p in expected_files.iter().chain(actual_files.iter()) {
        all.insert(p.clone());
    }

    let mut mismatches = vec![];
    for rel in all {
        let exp_path = expect_dir.join(&rel);
        let act_path = actual_dir.join(&rel);

        let exp_exists = exp_path.is_file();
        let act_exists = act_path.is_file();

        match (exp_exists, act_exists) {
            (true, false) => mismatches.push(FileMismatch {
                path: rel,
                kind: "missing_actual".to_string(),
                diff: None,
            }),
            (false, true) => mismatches.push(FileMismatch {
                path: rel,
                kind: "unexpected_actual".to_string(),
                diff: None,
            }),
            (false, false) => {}
            (true, true) => {
                let exp = std::fs::read(&exp_path).map_err(|e| fsutil::FsError::Io {
                    path: exp_path.clone(),
                    source: e,
                })?;
                let act = std::fs::read(&act_path).map_err(|e| fsutil::FsError::Io {
                    path: act_path.clone(),
                    source: e,
                })?;

                if exp != act {
                    let diff = text_diff_if_applicable(&rel, &exp, &act);
                    mismatches.push(FileMismatch {
                        path: rel,
                        kind: "content_mismatch".to_string(),
                        diff,
                    });
                }
            }
        }
    }

    mismatches.sort_by(|a, b| a.path.cmp(&b.path).then_with(|| a.kind.cmp(&b.kind)));
    Ok(mismatches)
}

fn collect_rel_files(root: &Path) -> Result<Vec<String>, TestError> {
    if !root.is_dir() {
        return Ok(vec![]);
    }

    let mut files = vec![];
    for entry in walkdir::WalkDir::new(root).follow_links(false) {
        let entry = entry.map_err(|e| TestError::Fs(fsutil::FsError::Io {
            path: root.to_path_buf(),
            source: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
        }))?;

        if !entry.file_type().is_file() {
            continue;
        }

        let rel = entry
            .path()
            .strip_prefix(root)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .replace('\\', "/");
        files.push(rel);
    }

    files.sort();
    Ok(files)
}

fn text_diff_if_applicable(path: &str, exp: &[u8], act: &[u8]) -> Option<String> {
    let ext = Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let is_text = matches!(
        ext.as_str(),
        "md" | "txt" | "yaml" | "yml" | "json" | "jsonc"
    );
    if !is_text {
        return None;
    }

    let exp_s = std::str::from_utf8(exp).ok()?;
    let act_s = std::str::from_utf8(act).ok()?;
    Some(unified_diff_for(exp_s, act_s, "expected", path))
}
