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
        .context("registered the CoreText-discovered font file but could not resolve PingFang SC")?;

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
    let descriptors = unsafe { collection.matching_font_descriptors_for_family(&family_name, None) }?;
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
