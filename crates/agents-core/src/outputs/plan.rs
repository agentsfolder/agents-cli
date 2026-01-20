use crate::outputs::{OutputPlan, SourceMapSkeleton};

#[derive(Debug)]
pub struct PlanResult {
    pub plan: OutputPlan,
    pub sources: Vec<SourceMapSkeleton>,
}

// Placeholder; implemented in later steps.
