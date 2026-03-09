//! Large directory protection policy service.

use crate::domain::error::CompareError;
use crate::domain::options::{CompareOptions, LargeDirPolicy};

/// Lightweight workload statistics for directory compare guardrails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LargeDirStats {
    /// Aligned entry count across both trees.
    pub aligned_entries: usize,
    /// Estimated total bytes across both trees.
    pub total_bytes: u64,
}

/// Protection plan resolved from workload stats and compare options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LargeDirPlan {
    /// Whether large-directory protection mode is active.
    pub large_mode: bool,
    /// Whether summary-first mode is active.
    pub summary_first_mode: bool,
    /// Optional entry processing cap for truncation.
    pub truncate_entries_to: Option<usize>,
    /// Whether report-level truncation flag must be set.
    pub force_truncated: bool,
    /// Non-fatal guard warnings.
    pub warnings: Vec<String>,
}

impl LargeDirPlan {
    /// Returns plan for normal-size workloads.
    pub(crate) fn normal() -> Self {
        Self {
            large_mode: false,
            summary_first_mode: false,
            truncate_entries_to: None,
            force_truncated: false,
            warnings: Vec::new(),
        }
    }
}

/// Resolves large-directory guard behavior for one compare request.
pub(crate) fn build_plan(
    stats: LargeDirStats,
    options: &CompareOptions,
) -> Result<LargeDirPlan, CompareError> {
    let soft_entries_hit = stats.aligned_entries > options.max_entries_soft_limit;
    let soft_bytes_hit = stats.total_bytes > options.max_total_bytes_soft_limit;
    let hard_entries_hit = stats.aligned_entries > options.max_entries_hard_limit;
    let hard_bytes_hit = stats.total_bytes > options.max_total_bytes_hard_limit;

    let large_mode = soft_entries_hit || soft_bytes_hit || hard_entries_hit || hard_bytes_hit;
    if !large_mode {
        return Ok(LargeDirPlan::normal());
    }

    if matches!(
        options.large_dir_policy,
        LargeDirPolicy::RefuseAboveHardLimit
    ) && (hard_entries_hit || hard_bytes_hit)
    {
        return Err(CompareError::DirectoryTooLarge {
            entries: stats.aligned_entries,
            total_bytes: stats.total_bytes,
            max_entries_hard_limit: options.max_entries_hard_limit,
            max_total_bytes_hard_limit: options.max_total_bytes_hard_limit,
        });
    }

    let mut warnings = Vec::new();
    warnings.push(format!(
        "large directory protection enabled (entries={}, total_bytes={})",
        stats.aligned_entries, stats.total_bytes
    ));

    if soft_entries_hit || soft_bytes_hit {
        warnings.push(format!(
            "soft limits reached (max_entries_soft_limit={}, max_total_bytes_soft_limit={})",
            options.max_entries_soft_limit, options.max_total_bytes_soft_limit
        ));
    }
    if hard_entries_hit || hard_bytes_hit {
        warnings.push(format!(
            "hard limits reached (max_entries_hard_limit={}, max_total_bytes_hard_limit={})",
            options.max_entries_hard_limit, options.max_total_bytes_hard_limit
        ));
    }

    let summary_first_mode = matches!(options.large_dir_policy, LargeDirPolicy::SummaryFirst);
    let truncate_entries_to = if summary_first_mode && hard_entries_hit {
        Some(options.max_entries_hard_limit)
    } else {
        None
    };
    let force_truncated = summary_first_mode && (hard_entries_hit || hard_bytes_hit);

    Ok(LargeDirPlan {
        large_mode,
        summary_first_mode,
        truncate_entries_to,
        force_truncated,
        warnings,
    })
}
