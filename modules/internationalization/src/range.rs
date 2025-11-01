use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::ops::Range;

pub const BASIC_LATIN: Range<u32> = 0x20..0x7F;
pub const LATIN_1_SUPPLEMENT: Range<u32> = 0xA0..0xFF;
pub const LATIN_EXTENDED_A: Range<u32> = 0x100..0x17F;
pub const LATIN_EXTENDED_B: Range<u32> = 0x180..0x24F;
pub const LATIN_EXTENDED_ADDITIONAL: Range<u32> = 0x1E00..0x1EFF;
pub const CYRILLIC: Range<u32> = 0x400..0x4FF;
pub const CYRILLIC_SUPPLEMENT: Range<u32> = 0x500..0x52F;
pub const GREEK: Range<u32> = 0x370..0x3FF;
pub const GREEK_EXTENDED: Range<u32> = 0x1F00..0x1FFF;
pub const ARABIC: Range<u32> = 0x600..0x6FF;
pub const ARABIC_SUPPLEMENT: Range<u32> = 0x750..0x77F;
pub const HEBREW: Range<u32> = 0x590..0x5FF;
pub const CJK_UNIFIED_IDEOGRAPHS: Range<u32> = 0x4E00..0x9FFF;
pub const HIRAGANA: Range<u32> = 0x3040..0x309F;
pub const KATAKANA: Range<u32> = 0x30A0..0x30FF;
pub const HANGUL_SYLLABLES: Range<u32> = 0xAC00..0xD7AF;
pub const DEVANAGARI: Range<u32> = 0x900..0x97F;
pub const THAI: Range<u32> = 0xE00..0xE7F;
pub const VIETNAMESE_EXTENSIONS: Range<u32> = 0x1EA0..0x1EFF;

pub fn get_locale_ranges(locale: &str) -> Option<&'static [Range<u32>]> {
    let ranges: &[Range<u32>] = match locale {
        // English
        "en" => &[BASIC_LATIN],

        // French
        "fr" => &[
            BASIC_LATIN,
            LATIN_1_SUPPLEMENT,
            LATIN_EXTENDED_A,
            LATIN_EXTENDED_B,
        ],

        // German
        "de" => &[BASIC_LATIN, LATIN_1_SUPPLEMENT, LATIN_EXTENDED_A],

        // Spanish
        "es" => &[BASIC_LATIN, LATIN_1_SUPPLEMENT],

        // Italian
        "it" => &[BASIC_LATIN, LATIN_1_SUPPLEMENT],

        // Portuguese
        "pt" => &[BASIC_LATIN, LATIN_1_SUPPLEMENT, LATIN_EXTENDED_A],

        // Dutch
        "nl" => &[BASIC_LATIN, LATIN_1_SUPPLEMENT],

        // Swedish, Norwegian, Danish
        "sv" | "no" | "da" => &[BASIC_LATIN, LATIN_1_SUPPLEMENT],

        // Polish
        "pl" => &[BASIC_LATIN, LATIN_1_SUPPLEMENT, LATIN_EXTENDED_A],

        // Czech, Slovak
        "cs" | "sk" => &[BASIC_LATIN, LATIN_1_SUPPLEMENT, LATIN_EXTENDED_A],

        // Hungarian
        "hu" => &[BASIC_LATIN, LATIN_1_SUPPLEMENT, LATIN_EXTENDED_A],

        // Romanian
        "ro" => &[BASIC_LATIN, LATIN_1_SUPPLEMENT, LATIN_EXTENDED_A],

        // Turkish
        "tr" => &[BASIC_LATIN, LATIN_1_SUPPLEMENT, LATIN_EXTENDED_A],

        // Russian, Ukrainian, Belarusian
        "ru" | "uk" | "be" => &[BASIC_LATIN, CYRILLIC, CYRILLIC_SUPPLEMENT],

        // Greek
        "el" => &[BASIC_LATIN, GREEK, GREEK_EXTENDED],

        // Arabic
        "ar" => &[BASIC_LATIN, ARABIC, ARABIC_SUPPLEMENT],

        // Hebrew
        "he" => &[BASIC_LATIN, HEBREW],

        // Japanese
        "ja" => &[BASIC_LATIN, HIRAGANA, KATAKANA, CJK_UNIFIED_IDEOGRAPHS],

        // Chinese (Simplified and Traditional)
        "zh" | "zh-CN" | "zh-TW" => &[BASIC_LATIN, CJK_UNIFIED_IDEOGRAPHS],

        // Korean
        "ko" => &[BASIC_LATIN, HANGUL_SYLLABLES, CJK_UNIFIED_IDEOGRAPHS],

        // Hindi
        "hi" => &[BASIC_LATIN, DEVANAGARI],

        // Thai
        "th" => &[BASIC_LATIN, THAI],

        // Vietnamese
        "vi" => &[
            BASIC_LATIN,
            LATIN_1_SUPPLEMENT,
            LATIN_EXTENDED_A,
            LATIN_EXTENDED_ADDITIONAL,
            VIETNAMESE_EXTENSIONS,
        ],
        _ => return None,
    };

    Some(ranges)
}

pub fn merge_contiguous_ranges(ranges: Vec<Range<u32>>) -> Vec<Range<u32>> {
    if ranges.is_empty() {
        return ranges;
    }

    let mut sorted_ranges = ranges;
    sorted_ranges.sort_by_key(|r| r.start);

    let mut merged_ranges = Vec::new();
    let mut current_range = sorted_ranges[0].clone();

    for range in sorted_ranges.into_iter().skip(1) {
        if range.start <= current_range.end {
            current_range.end = current_range.end.max(range.end);
        } else {
            merged_ranges.push(current_range);
            current_range = range;
        }
    }

    merged_ranges.push(current_range);

    merged_ranges
}

pub fn format_range(range: &Range<u32>) -> String {
    if range.start + 1 == range.end {
        format!("{}", range.start)
    } else {
        format!("{}-{}", range.start, range.end - 1)
    }
}

pub fn format_ranges<'a>(ranges: impl IntoIterator<Item = &'a Range<u32>>) -> String {
    ranges
        .into_iter()
        .map(format_range)
        .collect::<Vec<String>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::vec;

    #[test]
    fn test_get_locale_ranges_english() {
        let ranges = get_locale_ranges("en").unwrap();
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0], BASIC_LATIN);
    }

    #[test]
    fn test_get_locale_ranges_french() {
        let ranges = get_locale_ranges("fr").unwrap();
        assert_eq!(ranges.len(), 4);
        assert!(ranges.contains(&BASIC_LATIN));
        assert!(ranges.contains(&LATIN_1_SUPPLEMENT));
    }

    #[test]
    fn test_get_locale_ranges_russian() {
        let ranges = get_locale_ranges("ru").unwrap();
        assert_eq!(ranges.len(), 3);
        assert!(ranges.contains(&CYRILLIC));
    }

    #[test]
    fn test_get_locale_ranges_japanese() {
        let ranges = get_locale_ranges("ja").unwrap();
        assert_eq!(ranges.len(), 4);
        assert!(ranges.contains(&HIRAGANA));
        assert!(ranges.contains(&KATAKANA));
        assert!(ranges.contains(&CJK_UNIFIED_IDEOGRAPHS));
    }

    #[test]
    fn test_get_locale_ranges_invalid() {
        assert!(get_locale_ranges("invalid").is_none());
        assert!(get_locale_ranges("xx").is_none());
    }

    #[test]
    fn test_merge_contiguous_ranges_empty() {
        let ranges = vec![];
        let merged = merge_contiguous_ranges(ranges);
        assert_eq!(merged.len(), 0);
    }

    #[test]
    fn test_merge_contiguous_ranges_single() {
        let ranges = vec![0x20..0x7F];
        let merged = merge_contiguous_ranges(ranges);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0], 0x20..0x7F);
    }

    #[test]
    fn test_merge_contiguous_ranges_adjacent() {
        let ranges = vec![0x20..0x7F, 0x7F..0xFF];
        let merged = merge_contiguous_ranges(ranges);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0], 0x20..0xFF);
    }

    #[test]
    fn test_merge_contiguous_ranges_overlapping() {
        let ranges = vec![0x20..0x80, 0x50..0xFF];
        let merged = merge_contiguous_ranges(ranges);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0], 0x20..0xFF);
    }

    #[test]
    fn test_merge_contiguous_ranges_non_overlapping() {
        let ranges = vec![0x20..0x7F, 0x100..0x17F];
        let merged = merge_contiguous_ranges(ranges);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0], 0x20..0x7F);
        assert_eq!(merged[1], 0x100..0x17F);
    }

    #[test]
    fn test_merge_contiguous_ranges_unsorted() {
        let ranges = vec![0x100..0x17F, 0x20..0x7F, 0x7F..0xFF];
        let merged = merge_contiguous_ranges(ranges);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0], 0x20..0xFF);
        assert_eq!(merged[1], 0x100..0x17F);
    }

    #[test]
    fn test_merge_contiguous_ranges_multiple_groups() {
        let ranges = vec![0x20..0x7F, 0x7F..0xFF, 0x200..0x2FF, 0x2FF..0x3FF];
        let merged = merge_contiguous_ranges(ranges);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0], 0x20..0xFF);
        assert_eq!(merged[1], 0x200..0x3FF);
    }

    #[test]
    fn test_unicode_range_boundaries() {
        assert_eq!(BASIC_LATIN.start, 0x20);
        assert_eq!(BASIC_LATIN.end, 0x7F);
        assert_eq!(CJK_UNIFIED_IDEOGRAPHS.start, 0x4E00);
        assert_eq!(CJK_UNIFIED_IDEOGRAPHS.end, 0x9FFF);
    }

    #[test]
    fn test_get_locale_ranges_aliases() {
        // Test Chinese variants
        assert!(get_locale_ranges("zh").is_some());
        assert!(get_locale_ranges("zh-CN").is_some());
        assert!(get_locale_ranges("zh-TW").is_some());

        // Test Scandinavian languages
        assert!(get_locale_ranges("sv").is_some());
        assert!(get_locale_ranges("no").is_some());
        assert!(get_locale_ranges("da").is_some());
    }
}
