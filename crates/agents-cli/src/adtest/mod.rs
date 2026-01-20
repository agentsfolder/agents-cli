use std::path::{Path, PathBuf};

use agents_testutil::run_fixture;

use crate::{AppError, ErrorCategory};

pub fn cmd_test_adapters(root: &Path, agent: Option<String>) -> Result<(), AppError> {
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

    let agent_filter = agent.map(|s| s.to_string());

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
