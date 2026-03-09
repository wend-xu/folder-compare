//! Directory compare API.

use crate::domain::error::CompareError;
use crate::domain::options::CompareRequest;
use crate::domain::report::CompareReport;
use crate::services::comparer;

/// Compares two directories and returns a structured report.
///
/// This entry validates input, scans both directory trees, aligns entries by
/// normalized relative paths, and applies large-input protection policies.
pub fn compare_dirs(req: CompareRequest) -> Result<CompareReport, CompareError> {
    comparer::run_compare(req)
}
