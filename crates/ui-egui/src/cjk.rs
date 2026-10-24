//! Chinese fallback fonts for document text in the interface, independent of UI language.
//! Optional craft-fonts are preferred; native builds without them reuse one installed face.

use std::sync::{Arc, OnceLock};

use egui::{FontData, FontDefinitions, FontFamily};

const SAMPLE: &str = "中文轨道配乐音效简体汉语繁體漢語车";
const PREFERRED: &[&str] = &[
    "Microsoft YaHei UI",
    "Microsoft YaHei",
    "PingFang SC",
    "Noto Sans CJK SC",
    "Noto Sans SC",
    "Source Han Sans SC",
    "Source Han Sans CN",
    "WenQuanYi Micro Hei",
    "Microsoft JhengHei UI",
    "Microsoft JhengHei",
];

pub(crate) fn craft_fonts() -> impl Iterator<Item = &'static filmcraft_text::fonts::CraftFont> {
    filmcraft_text::fonts::CRAFT_FONTS.iter().filter(|f| f.scripts.iter().any(|s| matches!(*s, "Hans" | "Hant")))
}

// Hangul: neither the UI fonts nor the Chinese/Japanese fallbacks have it, so typed, pasted and
// IME-composed Korean drew as missing-glyph boxes (artcraft#2093).
const KOREAN_SAMPLE: &str = "한국어입력테스트";
const KOREAN_PREFERRED: &[&str] = &["Malgun Gothic", "Apple SD Gothic Neo", "Noto Sans CJK KR", "Noto Sans KR", "Source Han Sans K", "NanumGothic"];

// Every Lao-script codepoint used in the Lao catalog, so a partial fallback cannot silently
// render one of its combining marks or letters as a missing-glyph box.
const LAO_SAMPLE: &str = "\u{0e81}\u{0e82}\u{0e84}\u{0e87}\u{0e88}\u{0e8a}\u{0e8d}\u{0e94}\u{0e95}\u{0e96}\u{0e97}\u{0e99}\u{0e9a}\u{0e9b}\u{0e9c}\u{0e9d}\u{0e9e}\u{0e9f}\u{0ea1}\u{0ea2}\u{0ea3}\u{0ea5}\u{0ea7}\u{0eaa}\u{0eab}\u{0ead}\u{0eae}\u{0eb0}\u{0eb1}\u{0eb2}\u{0eb3}\u{0eb4}\u{0eb5}\u{0eb6}\u{0eb7}\u{0eb8}\u{0eb9}\u{0ebb}\u{0ebc}\u{0ebd}\u{0ec0}\u{0ec1}\u{0ec2}\u{0ec3}\u{0ec4}\u{0ec6}\u{0ec8}\u{0ec9}\u{0ecc}\u{0ecd}\u{0edc}\u{0edd}";
const LAO_PREFERRED: &[&str] = &[
    "Noto Sans Lao",
    "Noto Sans Lao UI",
    "Lao UI",
    "Phetsarath OT",
    "Saysettha OT",
    "Lao Sangam MN",
];

fn system_font() -> Option<Arc<FontData>> {
    static FONT: OnceLock<Option<Arc<FontData>>> = OnceLock::new();
    FONT.get_or_init(|| find_system_font(SAMPLE, PREFERRED)).clone()
}

fn korean_font() -> Option<Arc<FontData>> {
    static FONT: OnceLock<Option<Arc<FontData>>> = OnceLock::new();
    FONT.get_or_init(|| find_system_font(KOREAN_SAMPLE, KOREAN_PREFERRED)).clone()
}

fn lao_font() -> Option<Arc<FontData>> {
    static FONT: OnceLock<Option<Arc<FontData>>> = OnceLock::new();
    FONT.get_or_init(|| find_system_font(LAO_SAMPLE, LAO_PREFERRED)).clone()
}

/// A font collection's face index matters: Windows CJK UI fonts commonly live in TTC files.
/// Keep the owned bytes behind an Arc so theme reinstalls do not reread or leak a font file.
fn find_system_font(sample: &str, preferred: &[&str]) -> Option<Arc<FontData>> {
    if cfg!(target_arch = "wasm32") {
        return None;
    }
    filmcraft_text::fonts::scan_system();
    let faces: Vec<_> = filmcraft_text::fonts::all_faces().into_iter().filter(|f| f.info.origin == "system" && !f.info.italic).collect();
    let covers = |f: &filmcraft_text::fonts::Face| f.covers_text(sample);
    let by_weight = |f: &&Arc<filmcraft_text::fonts::Face>| f.info.weight.abs_diff(400);
    let found = preferred.iter().find_map(|name| faces.iter().filter(|f| f.info.family.eq_ignore_ascii_case(name) && covers(f)).min_by_key(by_weight));
    let face = found.or_else(|| faces.iter().filter(|f| covers(f)).min_by_key(by_weight))?;
    Some(Arc::new(FontData { font: std::borrow::Cow::Owned(face.data()?), index: face.info.index, tweak: Default::default() }))
}

/// Whether [`install`] has a Hangul face to add (an installed system face; none is bundled).
/// The Korean interface needs one.
pub(crate) fn korean_available() -> bool {
    korean_font().is_some()
}

/// Whether an installed system font covers every Lao character used by the interface catalog.
/// There is no bundled Lao face, so the Lao interface is unavailable without this fallback.
pub(crate) fn lao_available() -> bool {
    lao_font().is_some()
}

/// Whether [`install`] has a Chinese face to add: craft-fonts Hans/Hant faces, or an installed
/// system face (scanned once per process). The Simplified Chinese interface needs one.
pub(crate) fn available() -> bool {
    craft_fonts().next().is_some() || system_font().is_some()
}

/// Append Chinese faces after the Latin UI fonts and the Japanese fallbacks. This keeps the
/// interface's Latin typography and Japanese glyph forms, and covers the simplified and
/// traditional hanzi in media and track names that Japanese fonts lack.
pub(crate) fn install(fonts: &mut FontDefinitions) {
    install_chinese(fonts);
    if let Some(font) = korean_font() {
        fonts.font_data.insert("system-korean".into(), font);
        for stack in fonts.families.values_mut() {
            stack.push("system-korean".into());
        }
    }
    if let Some(font) = lao_font() {
        fonts.font_data.insert("system-lao".into(), font);
        for stack in fonts.families.values_mut() {
            stack.push("system-lao".into());
        }
    }
}

/// Optional installed CJK fallbacks, separate from the Japanese fallback under test.
#[cfg(test)]
pub(crate) fn is_system_face(name: &str) -> bool {
    matches!(name, "system-chinese" | "system-korean" | "system-lao")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lao_sample_covers_every_lao_character_in_the_catalog() {
        for line in include_str!("i18n/lo.tsv").lines().filter(|line| !line.is_empty() && !line.starts_with('#')) {
            let translation = line.split('\t').nth(2).unwrap_or_default();
            for ch in translation.chars().filter(|ch| ('\u{0e80}'..='\u{0eff}').contains(ch)) {
                assert!(LAO_SAMPLE.contains(ch), "Lao catalog character U+{:04X} is absent from the font coverage sample", ch as u32);
            }
        }
        assert!(LAO_PREFERRED.contains(&"Noto Sans Lao"));
    }
}

fn install_chinese(fonts: &mut FontDefinitions) {
    let chinese: Vec<_> = craft_fonts().collect();
    if chinese.is_empty() {
        if let Some(font) = system_font() {
            fonts.font_data.insert("system-chinese".into(), font);
            for stack in fonts.families.values_mut() {
                stack.push("system-chinese".into());
            }
        }
        return;
    }
    let name = |f: &filmcraft_text::fonts::CraftFont| format!("craft:{} {}", f.family, f.style);
    for f in &chinese {
        fonts.font_data.insert(name(f), Arc::new(FontData::from_static(f.bytes)));
    }
    for (family, stack) in &mut fonts.families {
        let heavy = matches!(family, FontFamily::Name(n) if matches!(n.as_ref(), "semibold" | "medium"));
        let mut order = chinese.clone();
        order.sort_by_key(|f| (f.family.contains("Serif") && !f.family.contains("Sans"), (f.style == "Bold") != heavy));
        stack.extend(order.iter().map(|f| name(f)));
    }
}
