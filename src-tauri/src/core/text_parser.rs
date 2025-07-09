use chrono::{DateTime, NaiveDateTime, Utc};
use regex::Regex;
use std::collections::HashMap;

// =============================================================================
// Episode Number Parsing
// =============================================================================

pub fn parse_episode_number(title: &str) -> Option<i32> {
    let patterns = [
        r"第(\d{1,4})[话集]",
        r"EP(\d{1,3})",
        r"E(\d{1,3})",
        r"Episode\s*(\d{1,3})",
        r"- (\d{1,3})v\d+",
        r"- (\d{1,3})\s",
        r"\[(\d{1,3})v\d+\]",
        r"\[(\d{1,3})\]",
        r"【(\d{1,3})】",
    ];

    for pattern in patterns.iter() {
        let re = Regex::new(pattern).unwrap();
        for caps in re.captures_iter(title) {
            if let Some(matched) = caps.get(1) {
                if let Ok(num) = matched.as_str().parse::<i32>() {
                    if is_valid_episode_number(title, matched.as_str(), num) {
                        return Some(num);
                    }
                }
            }
        }
    }
    None
}

fn is_valid_episode_number(_title: &str, _matched_str: &str, episode_num: i32) -> bool {
    !(2000..=2030).contains(&episode_num)
}

// =============================================================================
// Resolution Parsing
// =============================================================================

pub fn parse_resolution(title: &str) -> Option<String> {
    if let Some(res) = from_resolution_str(title) {
        return Some(res);
    }
    if let Some(res) = from_dimensions(title) {
        return Some(res);
    }
    if let Some(res) = infer_from_source(title) {
        return Some(res);
    }
    None
}

fn from_resolution_str(title: &str) -> Option<String> {
    let re = Regex::new(r"(\d{3,4}[pP])").unwrap();
    re.captures(title).map(|caps| caps[1].to_lowercase())
}

fn from_dimensions(title: &str) -> Option<String> {
    let re = Regex::new(r"(\d{3,4})[xX](\d{3,4})").unwrap();
    re.captures(title).and_then(|caps| {
        let h = caps[2].parse::<i32>().ok()?;
        Some(match h {
            h if h >= 2100 => "2160p".to_string(),
            h if h >= 1070 => "1080p".to_string(),
            h if h >= 700 => "720p".to_string(),
            _ => "480p".to_string(),
        })
    })
}

fn infer_from_source(title: &str) -> Option<String> {
    let title_upper = title.to_uppercase();
    let source_map = [
        ("BDRIP", "1080p"), ("BLURAY", "1080p"), ("BD", "1080p"),
        ("WEBRIP", "1080p"), ("WEB-DL", "1080p"), ("WEBDL", "1080p"),
        ("HDTV", "720p"), ("HDTVRIP", "720p"),
        ("DVDRIP", "480p"), ("DVD", "480p"),
    ];
    for (source, resolution) in source_map.iter() {
        if title_upper.contains(source) {
            return Some(resolution.to_string());
        }
    }
    None
}

// =============================================================================
// Subtitle Type Parsing & Normalization
// =============================================================================

pub fn parse_and_normalize_subtitle_type(title: &str) -> Option<String> {
    let keywords = [
        "简繁日内封", "简繁日内嵌", "简繁日多语", "简繁英", "简体日语双语",
        "繁体日语双语", "简日双语", "简日双字", "繁日双语", "繁日双字",
        "中日双语", "中日双字", "简繁双语", "简繁双字", "双语字幕",
        "简日", "繁日", "简英", "繁英", "简繁", "简日内封", "繁日内封",
        "简日内嵌", "繁日内嵌", "简繁内封", "简繁内挂", "简繁内嵌",
        "简体内封", "简体内挂", "简体内嵌", "繁体内封", "繁体内挂",
        "繁体内嵌", "简体外挂", "繁体外挂", "简繁外挂", "外挂字幕",
        "CHT", "CHS", "GB", "BIG5", "简体", "繁体", "简中", "繁中",
        "中字", "英语", "内嵌字幕", "内挂字幕", "中文字幕", "日语原声",
        "无字幕", "RAW",
    ];

    for keyword in keywords.iter() {
        if title.contains(keyword) {
            return Some(normalize_subtitle_type(keyword));
        }
    }
    Some("其他".to_string())
}

fn normalize_subtitle_type(raw_type: &str) -> String {
    let mut map = HashMap::new();
    // 中日双语
    map.insert("简日双语", "中日双语");
    map.insert("繁日双语", "中日双语");
    map.insert("中日双语", "中日双语");
    map.insert("简日", "中日双语");
    // 简繁双语
    map.insert("简繁", "简繁双语");
    map.insert("简繁双语", "简繁双语");
    // 简体中文
    map.insert("CHS", "简体中文");
    map.insert("简体", "简体中文");
    map.insert("简中", "简体中文");
    map.insert("GB", "简体中文");
    // 繁体中文
    map.insert("CHT", "繁体中文");
    map.insert("繁体", "繁体中文");
    map.insert("繁中", "繁体中文");
    map.insert("BIG5", "繁体中文");
    // 无字幕
    map.insert("无字幕", "无字幕");
    map.insert("RAW", "无字幕");

    map.get(raw_type).map_or_else(|| raw_type.to_string(), |s| s.to_string())
}

// =============================================================================
// Datetime Parsing
// =============================================================================

pub fn parse_datetime_to_timestamp(date_str: &str) -> Option<i64> {
    let formats = [
        "%Y/%m/%d %H:%M",
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d",
    ];
    let date_str = date_str.trim();

    for fmt in formats.iter() {
        if let Ok(dt) = NaiveDateTime::parse_from_str(date_str, fmt) {
            return Some(dt.and_utc().timestamp_millis());
        }
        if let Ok(d) = chrono::NaiveDate::parse_from_str(date_str, fmt) {
            return Some(d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp_millis());
        }
    }
    None
}
