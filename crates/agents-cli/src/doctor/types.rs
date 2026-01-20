use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DoctorLevel {
    Info,
    Warning,
    Error,
}

impl fmt::Display for DoctorLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DoctorLevel::Info => write!(f, "INFO"),
            DoctorLevel::Warning => write!(f, "WARN"),
            DoctorLevel::Error => write!(f, "ERROR"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DoctorItem {
    pub level: DoctorLevel,
    pub check: String,
    pub message: String,
    pub context: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DoctorReport {
    pub items: Vec<DoctorItem>,
}

impl DoctorReport {
    pub fn add(&mut self, item: DoctorItem) {
        self.items.push(item);
    }

    pub fn has_errors(&self) -> bool {
        self.items.iter().any(|i| i.level == DoctorLevel::Error)
    }

    pub fn has_warnings(&self) -> bool {
        self.items.iter().any(|i| i.level == DoctorLevel::Warning)
    }

    pub fn normalize_order(&mut self) {
        self.items.sort_by(|a, b| {
            a.level
                .cmp(&b.level)
                .then_with(|| a.check.cmp(&b.check))
                .then_with(|| a.message.cmp(&b.message))
        });
    }
}

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub items: Vec<DoctorItem>,
}

#[derive(Debug, Clone)]
pub struct FixResult {
    pub items: Vec<DoctorItem>,
}

pub trait DoctorCheck {
    fn name(&self) -> &'static str;

    fn run(&self, ctx: &DoctorContext) -> CheckResult;

    fn fix(&self, _ctx: &DoctorContext) -> Option<FixResult> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct DoctorContext {
    pub repo_root: PathBuf,
    pub repo: Option<RepoConfig>,
    pub effective: Option<EffectiveConfig>,
    pub ci: bool,
    pub fix: bool,
}
use std::path::PathBuf;

use agents_core::loadag::RepoConfig;
use agents_core::resolv::EffectiveConfig;
