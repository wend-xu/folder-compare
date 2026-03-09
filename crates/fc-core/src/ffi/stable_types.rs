//! Stable FFI-oriented wrappers.

/// Minimal FFI-safe summary projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct FfiCompareSummary {
    /// Total entry count.
    pub total_entries: u64,
    /// Different entry count.
    pub different: u64,
}
