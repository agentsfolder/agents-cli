use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::loadag::RepoConfig;
use crate::resolv::types::ScopeMatch;

#[derive(Debug, thiserror::Error)]
pub enum ScopeMatchError {
    #[error("invalid glob in scope {scope_id}: {glob}: {message}")]
    InvalidGlob {
        scope_id: String,
        glob: String,
        message: String,
    },
}

pub fn match_scopes(
    cfg: &RepoConfig,
    target_path: &str,
) -> Result<Vec<ScopeMatch>, ScopeMatchError> {
    let mut matches: Vec<ScopeMatch> = vec![];

    for (id, scope) in &cfg.scopes {
        let set = compile_apply_to(id, &scope.apply_to)?;
        if set.is_match(target_path) {
            let score = specificity_score(&scope.apply_to);
            matches.push(ScopeMatch {
                id: id.clone(),
                score,
                priority: scope.priority,
            });
        }
    }

    matches.sort_by(|a, b| {
        // Higher score is more specific.
        b.score
            .cmp(&a.score)
            // Higher priority wins when specificity ties.
            .then_with(|| b.priority.cmp(&a.priority))
            // Deterministic tie-breaker.
            .then_with(|| a.id.cmp(&b.id))
    });

    Ok(matches)
}

fn compile_apply_to(scope_id: &str, apply_to: &[String]) -> Result<GlobSet, ScopeMatchError> {
    let mut builder = GlobSetBuilder::new();
    for g in apply_to {
        let glob = Glob::new(g).map_err(|e| ScopeMatchError::InvalidGlob {
            scope_id: scope_id.to_string(),
            glob: g.clone(),
            message: e.to_string(),
        })?;
        builder.add(glob);
    }

    builder.build().map_err(|e| ScopeMatchError::InvalidGlob {
        scope_id: scope_id.to_string(),
        glob: "<set>".to_string(),
        message: e.to_string(),
    })
}

fn specificity_score(patterns: &[String]) -> i64 {
    // Take the best (most specific) applyTo glob.
    patterns.iter().map(|p| score_one(p)).max().unwrap_or(0)
}

fn score_one(pat: &str) -> i64 {
    // Heuristic scoring:
    // - Prefer more segments
    // - Penalize wildcards
    // - Prefer literal characters
    let mut segs = 0i64;
    let mut wild = 0i64;
    let mut literals = 0i64;

    for seg in pat.split('/') {
        if seg.is_empty() {
            continue;
        }
        segs += 1;
        if seg.contains("**") {
            wild += 5;
        }
        if seg.contains('*') {
            wild += 2;
        }
        if seg.contains('?') {
            wild += 1;
        }

        literals += seg.chars().filter(|c| *c != '*' && *c != '?').count() as i64;
    }

    // Higher is better.
    (segs * 100) + literals - (wild * 50)
}

#[cfg(test)]
mod tests {
    use super::specificity_score;

    #[test]
    fn more_specific_glob_scores_higher() {
        let a = specificity_score(&["apps/**".to_string()]);
        let b = specificity_score(&["apps/web/**".to_string()]);
        assert!(b > a);
    }
}
