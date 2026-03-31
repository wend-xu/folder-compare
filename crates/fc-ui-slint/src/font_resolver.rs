#[cfg(target_os = "macos")]
use fontique::{Collection, CollectionOptions, QueryFamily, QueryStatus, SourceCache};

#[cfg(target_os = "macos")]
const MACOS_CJK_FAMILY_CANDIDATES: [&str; 3] = ["PingFang SC", "Hiragino Sans GB", "Heiti SC"];

#[cfg(target_os = "macos")]
#[derive(Clone, Copy, Debug)]
struct CandidateProbe {
    family: &'static str,
    hit: bool,
}

#[cfg(target_os = "macos")]
pub(crate) fn resolve_runtime_text_font_family() -> Option<&'static str> {
    let mut collection = Collection::new(CollectionOptions {
        shared: false,
        system_fonts: true,
    });
    let mut source_cache = SourceCache::default();

    let probes = MACOS_CJK_FAMILY_CANDIDATES.map(|family| CandidateProbe {
        family,
        hit: probe_family(&mut collection, &mut source_cache, family),
    });
    let resolved_family = probes
        .iter()
        .find(|probe| probe.hit)
        .map(|probe| probe.family);

    tracing::info!(
        target: "fc_ui_slint::font_resolver",
        requested_families = ?MACOS_CJK_FAMILY_CANDIDATES,
        resolved_family = resolved_family.unwrap_or(""),
        candidate_hits = ?probes,
        "Resolved runtime macOS CJK font family"
    );

    resolved_family
}

#[cfg(target_os = "macos")]
fn probe_family(
    collection: &mut Collection,
    source_cache: &mut SourceCache,
    family: &'static str,
) -> bool {
    let mut query = collection.query(source_cache);
    query.set_families(core::iter::once(QueryFamily::Named(family)));

    let mut hit = false;
    query.matches_with(|_| {
        hit = true;
        QueryStatus::Stop
    });

    hit
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn resolve_runtime_text_font_family() -> Option<&'static str> {
    None
}
