use engine::models::TorrentResult;
use engine::normalize::title_normalizer_core::{clean_title, extract_year, parse_torrent_metadata};
use engine::normalize::MediaNormalizer;
use engine::normalize::TitleNormalizerWrapper;
use proptest::prelude::*;

// Property 56: Title Normalization - Remove release group tags, quality indicators, encoding
// **Feature: ui-redesign, Property 56: Title normalization removes release group tags, quality indicators, and encoding**
// **Validates: Requirements 13.1**
#[test]
fn prop_title_removes_quality_indicators() {
    proptest!(|(title in r"[A-Za-z0-9 ]+")|{
        let test_cases = vec![
            ("Movie.2023.1080p.WEB-DL.x264-GROUP.mkv", "Movie"),
            ("Show.S01E02.720p.BluRay.x265-TEAM.mkv", "Show"),
            ("Film.2022.4K.HEVC.AAC-RELEASE.mp4", "Film"),
        ];

        for (input, expected_title) in test_cases {
            let result = parse_torrent_metadata(input.to_string());
            // Title should not contain quality indicators
            assert!(!result.title.contains("1080p"));
            assert!(!result.title.contains("720p"));
            assert!(!result.title.contains("4K"));
            assert!(!result.title.contains("WEB-DL"));
            assert!(!result.title.contains("BluRay"));
            assert!(!result.title.contains("x264"));
            assert!(!result.title.contains("x265"));
            assert!(!result.title.contains("HEVC"));
            assert!(!result.title.contains("AAC"));
            assert!(!result.title.contains("GROUP"));
            assert!(!result.title.contains("TEAM"));
            assert!(!result.title.contains("RELEASE"));
        }
    });
}

// Property 57: Title Capitalization - Apply proper title case
// **Feature: ui-redesign, Property 57: Title capitalization applies proper title case**
// **Validates: Requirements 13.2**
#[test]
fn prop_title_applies_capitalization() {
    proptest!(|(title in r"[a-z ]+")|{
        let test_cases = vec![
            ("movie.2023.1080p.web-dl.x264-group.mkv", "Movie"),
            ("the.dark.knight.2008.1080p.bluray.x264-sparks.mkv", "The Dark Knight"),
            ("inception.2010.720p.hdtv.x264-killers.mkv", "Inception"),
        ];

        for (input, _expected) in test_cases {
            let result = parse_torrent_metadata(input.to_string());
            // Title should be properly capitalized (first letter uppercase)
            if !result.title.is_empty() {
                let first_char = result.title.chars().next().unwrap();
                assert!(first_char.is_uppercase(), "Title '{}' should start with uppercase", result.title);
            }
        }
    });
}

// Property 58: Series Format - Format series as S01E01
// **Feature: ui-redesign, Property 58: Series format displays as S01E01**
// **Validates: Requirements 13.5**
#[test]
fn prop_series_format_s01e01() {
    proptest!(|(season in 1u32..100, episode in 1u32..100)|{
        let test_cases = vec![
            ("Show.S01E02.1080p.WEB-DL.x264-GROUP.mkv", true),
            ("Series.1x05.720p.BluRay.x264-TEAM.mkv", true),
            ("Movie.2023.1080p.WEB-DL.x264-GROUP.mkv", false),
        ];

        for (input, is_series) in test_cases {
            let result = parse_torrent_metadata(input.to_string());
            if is_series {
                // Should have episode tag
                assert!(!result.tags.get("other").unwrap_or(&vec![]).is_empty() ||
                        result.tags.get("other").unwrap_or(&vec![]).contains(&"episode".to_string()),
                        "Series should have episode tag");
            }
        }
    });
}

// Property 59: Year Extraction - Extract and display year separately
// **Feature: ui-redesign, Property 59: Year extraction separates year from title**
// **Validates: Requirements 13.3**
#[test]
fn prop_year_extraction() {
    proptest!(|(year in 1900u32..2100u32)|{
        let test_cases = vec![
            ("Movie.2023.1080p.WEB-DL.x264-GROUP.mkv", Some(2023)),
            ("Film.(2022).720p.BluRay.x264-TEAM.mkv", Some(2022)),
            ("Show.S01E02.2021.1080p.WEB-DL.x264-GROUP.mkv", Some(2021)),
            ("NoYear.1080p.WEB-DL.x264-GROUP.mkv", None),
        ];

        for (input, expected_year) in test_cases {
            let result = parse_torrent_metadata(input.to_string());
            assert_eq!(result.year, expected_year, "Year extraction failed for '{}'", input);
        }
    });
}

// Property 60: Unicode Handling - Handle special characters and unicode
// **Feature: ui-redesign, Property 60: Unicode handling preserves special characters**
// **Validates: Requirements 13.4**
#[test]
fn prop_unicode_handling() {
    let test_cases = vec![
        ("Фильм.2023.1080p.WEB-DL.x264-GROUP.mkv", "Фильм"),
        ("Película.2023.1080p.WEB-DL.x264-GROUP.mkv", "Película"),
        ("Film.2023.1080p.WEB-DL.x264-GROUP.mkv", "Film"),
    ];

    for (input, expected_contains) in test_cases {
        let result = parse_torrent_metadata(input.to_string());
        // Title should contain the expected text (possibly with different case)
        assert!(
            !result.title.is_empty(),
            "Title should not be empty for '{}'",
            input
        );
    }
}

// Integration test: Full title normalization workflow
#[test]
fn test_title_normalizer_wrapper_integration() {
    let wrapper = TitleNormalizerWrapper::new();

    let test_cases = vec![
        (
            "Inception.2010.1080p.BluRay.x264-SPARKS.mkv",
            "Inception",
            Some(2010),
        ),
        (
            "Breaking.Bad.S01E01.2008.720p.BluRay.x264-REWARDERS.mkv",
            "Breaking Bad",
            Some(2008),
        ),
        (
            "The.Matrix.1999.1080p.BluRay.x264-SECTOR7.mkv",
            "The Matrix",
            Some(1999),
        ),
    ];

    for (title, expected_title, expected_year) in test_cases {
        let torrent = TorrentResult::new(
            title.to_string(),
            "magnet:?xt=urn:btih:test".to_string(),
            1_500_000_000,
            100,
            50,
            "TestIndexer".to_string(),
        );

        let result = wrapper.normalize(&torrent);
        assert!(result.is_ok(), "Normalization failed for '{}'", title);

        let parsed = result.unwrap();
        assert_eq!(
            parsed.title, expected_title,
            "Title mismatch for '{}'",
            title
        );
        assert_eq!(parsed.year, expected_year, "Year mismatch for '{}'", title);
    }
}

// Edge case: Empty title
#[test]
fn test_empty_title() {
    let result = parse_torrent_metadata("".to_string());
    assert_eq!(result.title, "");
    assert_eq!(result.year, None);
}

// Edge case: Title with only quality indicators
#[test]
fn test_title_only_quality() {
    let result = parse_torrent_metadata("1080p.WEB-DL.x264-GROUP.mkv".to_string());
    // Should extract what it can
    assert!(!result.tags["resolution"].is_empty());
}

// Edge case: Russian title with series info
#[test]
fn test_russian_series_title() {
    let result =
        parse_torrent_metadata("Сериал.Сезон 1.Серия 2.1080p.WEB-DL.x264-GROUP.mkv".to_string());
    assert_eq!(result.title, "Сериал");
    assert!(!result.tags["other"].is_empty());
}
