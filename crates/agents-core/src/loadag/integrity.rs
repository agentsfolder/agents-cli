use crate::loadag::LoadError;
use crate::loadag::RepoConfig;

pub fn check_referential_integrity(cfg: &RepoConfig) -> Result<(), LoadError> {
    // enabled.* sets
    for id in &cfg.manifest.enabled.modes {
        if !cfg.modes.contains_key(id) {
            return Err(LoadError::MissingId {
                kind: "modes",
                id: id.clone(),
            });
        }
    }

    for id in &cfg.manifest.enabled.policies {
        if !cfg.policies.contains_key(id) {
            return Err(LoadError::MissingId {
                kind: "policies",
                id: id.clone(),
            });
        }
    }

    for id in &cfg.manifest.enabled.skills {
        if !cfg.skills.contains_key(id) {
            return Err(LoadError::MissingId {
                kind: "skills",
                id: id.clone(),
            });
        }
    }

    for id in &cfg.manifest.enabled.adapters {
        if !cfg.adapters.contains_key(id) {
            return Err(LoadError::MissingId {
                kind: "adapters",
                id: id.clone(),
            });
        }
    }

    // defaults
    if !cfg.modes.contains_key(&cfg.manifest.defaults.mode) {
        return Err(LoadError::MissingId {
            kind: "defaults.mode",
            id: cfg.manifest.defaults.mode.clone(),
        });
    }

    if !cfg.policies.contains_key(&cfg.manifest.defaults.policy) {
        return Err(LoadError::MissingId {
            kind: "defaults.policy",
            id: cfg.manifest.defaults.policy.clone(),
        });
    }

    if let Some(profile) = &cfg.manifest.defaults.profile {
        if !cfg.profiles.contains_key(profile) {
            return Err(LoadError::MissingId {
                kind: "defaults.profile",
                id: profile.clone(),
            });
        }
    }

    Ok(())
}
