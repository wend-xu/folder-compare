//! Temporary macOS font bootstrap workaround.
//!
//! Why this exists:
//! - On macOS 15.x, the Slint 1.15.1 text stack used in this project still depends on
//!   `fontique 0.7.0` for system font discovery.
//! - We verified that `fontique 0.7.0` fails to discover `PingFang SC` correctly on
//!   newer macOS releases, which causes Chinese and full-width characters to render as
//!   TOFU unless we manually register a CJK-capable system font into Slint's shared
//!   font collection.
//! - Our investigation also showed that this is a dependency-layer issue rather than an
//!   application UI issue.
//!
//! Timeline:
//! - On macOS 13.5, the first user-visible problem was already a font
//!   fallback/selection issue in the Slint 1.15.1 text stack. At that stage, explicitly
//!   selecting `PingFang SC` was enough to keep Chinese and full-width text readable.
//! - After upgrading to macOS 15.7, the system font layout/exposure changed enough that
//!   `fontique 0.7.0` could no longer discover `PingFang SC` reliably, so the earlier
//!   workaround stopped being sufficient on its own.
//! - As a result, the current application workaround is compensating for two dependency
//!   issues at once: an older fallback/selection problem, and a newer macOS font
//!   discovery problem.
//!
//! What is already confirmed:
//! - Confirmed: `fontique 0.7.0` has a macOS system-font discovery problem. This file
//!   exists primarily to compensate for that specific issue in the current Slint stack.
//! - Confirmed: the current `Slint + Parley + fontique (+ renderer)` stack also shows a
//!   mixed-text font-selection/fallback problem for samples such as `中Ａ（`.
//! - Confirmed: in fallback-only experiments, `中` can hit CJK fallback, but `Ａ` and `（`
//!   still stay on the Latin generic path, and `zh-Hans` locale does not change that.
//! - Confirmed: this means the mixed-text behavior is not fully explained by the
//!   `fontique 0.7.0` discovery bug alone.
//! - Not yet isolated to a single crate: the remaining fallback/selection issue should be
//!   treated as a stack-level behavior until proven otherwise, not as a proven standalone
//!   `fontique 0.7.0` fallback bug.
//!
//! Responsibility boundary:
//! - Neither the fallback/selection issue nor the macOS discovery issue should ideally be
//!   owned long-term by application code.
//! - This file should therefore stay a narrowly scoped compatibility shim for the current
//!   dependency stack, not become the application's permanent font policy layer.
//!
//! Removal guidance:
//! - Treat this file as a temporary compatibility shim, not as long-term font policy.
//! - We have already validated that the relevant discovery issue is fixed in
//!   `fontique 0.8.0`, but Slint may not pick up that version immediately, so the real
//!   removal timing depends on when the Slint version used by this project includes the
//!   effective fix.
//! - We also validated a more system-like direction: stop explicitly pinning `PingFang SC`
//!   and instead let the text stack follow the system-resolved macOS UI font selection and
//!   fallback behavior. In practice, that direction likely requires a dependency-layer fix
//!   or upgrade (`fontique`/Parley/Slint), rather than further application-layer tweaks in
//!   this file.
//! - We are intentionally not patching third-party crates locally for now, because the
//!   maintenance and upgrade cost of carrying a private dependency fork is currently judged
//!   too high relative to this application's temporary compatibility need.
//! - During future Slint upgrades, explicitly test macOS text rendering with and without
//!   this bootstrap. If Chinese and full-width characters render correctly without this
//!   file, prefer deleting this workaround instead of evolving it further.
//! - Regression samples to verify before removal:
//!   `中`, `Ａ`, `（`, `中Ａ（`, navigator tree file rows, navigator tree directory rows,
//!   and diff body text.
//!
#[cfg(target_os = "macos")]
use anyhow::Context;
#[cfg(target_os = "macos")]
use objc2_core_foundation::{CFArray, CFRetained, CFString, CFURL};
#[cfg(target_os = "macos")]
use objc2_core_text::{CTFontCollection, CTFontDescriptor, kCTFontURLAttribute};
#[cfg(target_os = "macos")]
use slint::fontique_07::fontique::{self, FallbackKey, GenericFamily, Script};
#[cfg(target_os = "macos")]
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use std::sync::Once;

#[cfg(target_os = "macos")]
const PINGFANG_SC_FAMILY: &str = "PingFang SC";

#[cfg(target_os = "macos")]
static MACOS_FONT_BOOTSTRAP: Once = Once::new();

#[cfg(target_os = "macos")]
pub(crate) fn configure_slint_font_collection() {
    MACOS_FONT_BOOTSTRAP.call_once(|| {
        if std::env::var_os("SLINT_DEFAULT_FONT").is_some() {
            tracing::info!(
                target: "fc_ui_slint::macos_font_bootstrap",
                "Skipping PingFang SC bootstrap because SLINT_DEFAULT_FONT is already set"
            );
            return;
        }

        match bootstrap_pingfang_sc() {
            Ok(Some(path)) => {
                tracing::info!(
                    target: "fc_ui_slint::macos_font_bootstrap",
                    font_path = %path.display(),
                    "Registered PingFang SC into Slint's shared font collection"
                );
            }
            Ok(None) => {
                tracing::warn!(
                    target: "fc_ui_slint::macos_font_bootstrap",
                    family = PINGFANG_SC_FAMILY,
                    "CoreText did not return a font file for the requested family"
                );
            }
            Err(err) => {
                tracing::warn!(
                    target: "fc_ui_slint::macos_font_bootstrap",
                    family = PINGFANG_SC_FAMILY,
                    error = %err,
                    "Failed to bootstrap PingFang SC for Slint"
                );
            }
        }
    });
}

#[cfg(target_os = "macos")]
fn bootstrap_pingfang_sc() -> anyhow::Result<Option<PathBuf>> {
    let Some(font_path) = find_family_font_path(PINGFANG_SC_FAMILY) else {
        return Ok(None);
    };

    let bytes = std::fs::read(&font_path)
        .with_context(|| format!("failed reading {}", font_path.display()))?;
    let mut collection = slint::fontique_07::shared_collection();
    let registered = collection.register_fonts(bytes.into(), None);
    let family_id = registered
        .iter()
        .find_map(|(family_id, _)| {
            collection
                .family_name(*family_id)
                .filter(|name| *name == PINGFANG_SC_FAMILY)
                .map(|_| *family_id)
        })
        .or_else(|| collection.family_id(PINGFANG_SC_FAMILY))
        .context(
            "registered the CoreText-discovered font file but could not resolve PingFang SC",
        )?;

    for generic in [
        GenericFamily::SansSerif,
        GenericFamily::SystemUi,
        GenericFamily::UiSansSerif,
    ] {
        prepend_generic_family(&mut collection, generic, family_id);
    }

    for key in [
        FallbackKey::from(Script(*b"Hani")),
        FallbackKey::from((Script(*b"Hani"), "zh-Hans")),
    ] {
        prepend_fallback_family(&mut collection, key, family_id);
    }

    Ok(Some(font_path))
}

#[cfg(target_os = "macos")]
fn prepend_generic_family(
    collection: &mut fontique::Collection,
    generic: GenericFamily,
    family_id: fontique::FamilyId,
) {
    let existing: Vec<_> = collection
        .generic_families(generic)
        .filter(|existing_id| *existing_id != family_id)
        .collect();
    collection.set_generic_families(generic, std::iter::once(family_id).chain(existing));
}

#[cfg(target_os = "macos")]
fn prepend_fallback_family(
    collection: &mut fontique::Collection,
    key: FallbackKey,
    family_id: fontique::FamilyId,
) {
    let existing: Vec<_> = collection
        .fallback_families(key)
        .filter(|existing_id| *existing_id != family_id)
        .collect();
    let _ = collection.set_fallbacks(key, std::iter::once(family_id).chain(existing));
}

#[cfg(target_os = "macos")]
fn find_family_font_path(family_name: &str) -> Option<PathBuf> {
    let family_name = CFString::from_str(family_name);
    let collection = unsafe { CTFontCollection::from_available_fonts(None) };
    let descriptors =
        unsafe { collection.matching_font_descriptors_for_family(&family_name, None) }?;
    let descriptors: CFRetained<CFArray<CTFontDescriptor>> =
        unsafe { CFRetained::cast_unchecked(descriptors) };

    descriptors.into_iter().find_map(|descriptor| {
        let url = unsafe { descriptor.attribute(kCTFontURLAttribute) }?;
        let url = url.downcast::<CFURL>().ok()?;
        url.to_file_path()
    })
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn configure_slint_font_collection() {}
