use regex::Regex;
use chrono::{DateTime, NaiveDateTime, Utc};

// Extracts episode number from a title string.
pub fn parse_episode_number(title: &str) -> Option<i32> {
    let re = Regex::new(r"(?i)\[(\d{2,3})\]|\[E(\d{2,3})\]").unwrap();
    re.captures(title)
        .and_then(|caps| caps.get(1).or_else(|| caps.get(2)))
        .and_then(|m| m.as_str().parse::<i32>().ok())
}

// Extracts video resolution from a title string.
pub fn parse_resolution(title: &str) -> Option<String> {
    let re = Regex::new(r"(?i)(\d{3,4}p)").unwrap();
    re.captures(title)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_lowercase())
}

// Extracts subtitle language/type from a title string.
pub fn parse_subtitle_type(title: &str) -> Option<String> {
    let re = Regex::new(r"(?i)(简繁|简日|繁日|简体|繁体|BIG5|GB)").unwrap();
    re.captures(title)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

// Parses a datetime string into a Unix timestamp (milliseconds).
pub fn parse_datetime_to_timestamp(date_str: &str) -> Option<i64> {
    let format = "%Y/%m/%d %H:%M";
    NaiveDateTime::parse_from_str(date_str, format)
        .ok()
        .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc).timestamp_millis())
}
