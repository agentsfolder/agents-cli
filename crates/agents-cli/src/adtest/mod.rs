use std::path::{Path, PathBuf};

use agents_testutil::run_fixture;

use crate::{AppError, ErrorCategory};

pub fn cmd_test_adapters(root: &Path, agent: Option<String>, update: bool) -> Result<(), AppError> {
    let fixtures_dir = root.join("fixtures");
    if !fixtures_dir.is_dir() {
        return Err(AppError {
            category: ErrorCategory::Io,
            message: "fixtures directory not found".to_string(),
            context: vec![format!("path: {}", fixtures_dir.display())],
        });
    }

    let mut fixtures: Vec<PathBuf> = std::fs::read_dir(&fixtures_dir)
        .map_err(|e| AppError {
            category: ErrorCategory::Io,
            message: e.to_string(),
            context: vec![format!("path: {}", fixtures_dir.display())],
        })?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    fixtures.sort();

    if update {
        let ok = std::env::var("AGENTS_UPDATE_GOLDENS")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        if !ok {
            return Err(AppError {
                category: ErrorCategory::InvalidArgs,
                message: "refusing to update goldens without AGENTS_UPDATE_GOLDENS=1".to_string(),
                context: vec![
                    "hint: rerun with `AGENTS_UPDATE_GOLDENS=1 agents test adapters --update`"
                        .to_string(),
                ],
            });
        }
    }

    let agent_filter = agent.map(|s| s.to_string());

    if update {
        return update_goldens(&fixtures_dir, &agent_filter);
    }

    // Run fixtures in parallel to keep runtime reasonable.
    let mut handles = vec![];
    for fixture in fixtures {
        let fixture_clone = fixture.clone();
        let agent_filter = agent_filter.clone();

        let h = std::thread::spawn(move || {
            run_fixture(&fixture_clone, agent_filter.as_deref())
                .map(|r| (fixture_clone, r))
        });
        handles.push(h);
    }

    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut failures: Vec<String> = vec![];

    for h in handles {
        let (fixture, report) = h
            .join()
            .map_err(|_| AppError {
                category: ErrorCategory::Io,
                message: "fixture thread panicked".to_string(),
                context: vec![],
            })?
            .map_err(|e| AppError {
                category: ErrorCategory::Io,
                message: e.to_string(),
                context: vec![],
            })?;

        passed += report.passed;
        failed += report.failed;
        for f in report.failures {
            failures.push(f.render_human());
        }

        let _ = fixture;
    }

    if failures.is_empty() {
        println!("ok: adapters fixtures passed (passed={passed})");
        return Ok(());
    }

    for f in failures {
        print!("{f}");
    }

    Err(AppError {
        category: ErrorCategory::Io,
        message: format!("adapter fixtures failed (passed={passed} failed={failed})"),
        context: vec!["hint: inspect actual outputs paths above".to_string()],
    })
}

fn update_goldens(fixtures_dir: &Path, agent_filter: &Option<String>) -> Result<(), AppError> {
    let mut fixtures: Vec<PathBuf> = std::fs::read_dir(fixtures_dir)
        .map_err(|e| AppError {
            category: ErrorCategory::Io,
            message: e.to_string(),
            context: vec![format!("path: {}", fixtures_dir.display())],
        })?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    fixtures.sort();

    let mut updated = 0usize;
    for fixture in fixtures {
        let report = run_fixture(&fixture, agent_filter.as_deref()).map_err(|e| AppError {
            category: ErrorCategory::Io,
            message: e.to_string(),
            context: vec![format!("fixture: {}", fixture.display())],
        })?;

        let has_matrix = fixture.join("matrix.yaml").is_file();
        for failure in report.failures {
            let expect_dir = if has_matrix {
                fixture.join("expect").join(&failure.agent_id).join(&failure.case)
            } else {
                fixture.join("expect").join(&failure.agent_id)
            };

            if expect_dir.exists() {
                std::fs::remove_dir_all(&expect_dir).map_err(|e| AppError {
                    category: ErrorCategory::Io,
                    message: e.to_string(),
                    context: vec![format!("path: {}", expect_dir.display())],
                })?;
            }

            copy_dir_recursive(&failure.actual_dir, &expect_dir)?;
            let _ = std::fs::remove_dir_all(&failure.actual_dir);
            updated += 1;
            println!("update: {}", expect_dir.display());
        }
    }

    println!("ok: updated goldens (cases={updated})");
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), AppError> {
    for entry in walkdir::WalkDir::new(src).follow_links(false) {
        let entry = entry.map_err(|e| AppError {
            category: ErrorCategory::Io,
            message: e.to_string(),
            context: vec![format!("src: {}", src.display())],
        })?;

        let rel = entry.path().strip_prefix(src).unwrap_or(entry.path());
        let dest = dst.join(rel);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&dest).map_err(|e| AppError {
                category: ErrorCategory::Io,
                message: e.to_string(),
                context: vec![format!("path: {}", dest.display())],
            })?;
            continue;
        }

        if entry.file_type().is_file() {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).map_err(|e| AppError {
                    category: ErrorCategory::Io,
                    message: e.to_string(),
                    context: vec![format!("path: {}", parent.display())],
                })?;
            }
            std::fs::copy(entry.path(), &dest).map_err(|e| AppError {
                category: ErrorCategory::Io,
                message: e.to_string(),
                context: vec![format!("path: {}", dest.display())],
            })?;
        }
    }

    Ok(())
}
