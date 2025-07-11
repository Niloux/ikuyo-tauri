use crate::core::anime_parser::AnimeParser;
use crate::core::AnimeData;
use crate::error::{AppError, DomainError, Result};
use scraper::{ElementRef, Html, Selector};
use regex::Regex;
use std::collections::HashSet;

pub struct MikanParser {
    pub base_url: String,
}

impl MikanParser {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
        }
    }
}

impl AnimeParser for MikanParser {
    fn parse_list(&self, html: &str) -> Result<Vec<String>> {
        let document = Html::parse_document(html);
        let mut urls = Vec::new();
        // 选择器抓取详情页链接
        let selector = Selector::parse("a[href*='/Home/Bangumi/']")
            .map_err(|e| AppError::Domain(DomainError::Serialization(e.to_string())))?;
        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                let full_url = if href.starts_with("http") {
                    href.to_string()
                } else {
                    format!("{}{}", self.base_url, href)
                };
                urls.push(full_url);
            }
        }
        // 去重
        let mut seen = HashSet::new();
        urls.retain(|url| seen.insert(url.clone()));
        // 若抓不到，用正则兜底
        if urls.is_empty() {
            let re = Regex::new(r#"/Home/Bangumi/(\\d+)"#)
                .map_err(|e| AppError::Domain(DomainError::Other(e.to_string())))?;
            for cap in re.captures_iter(html) {
                let mikan_id = &cap[1];
                let full_url = format!("{}/Home/Bangumi/{}", self.base_url, mikan_id);
                if !urls.contains(&full_url) {
                    urls.push(full_url);
                }
            }
        }
        Ok(urls)
    }

    fn parse_detail(&self, html: &str, mikan_id: i64) -> Result<AnimeData> {
        let document = Html::parse_document(html);
        let anime = self.parse_anime_info(&document, mikan_id);
        let (subtitle_groups, resources) = self.parse_groups_and_resources(&document, mikan_id);
        Ok(AnimeData {
            anime: Some(anime),
            subtitle_groups,
            resources,
        })
    }
}

impl MikanParser {
    fn parse_anime_info(&self, document: &Html, mikan_id: i64) -> crate::models::Anime {
        let title = self
            .extract_text(document, &["p.bangumi-title", "title"])
            .unwrap_or_else(|| "未知标题".to_string())
            .replace("Mikan Project - ", "");
        let bangumi_url = self.extract_href(document, &["a[href*='bgm.tv/subject/']"]);
        let bangumi_id = bangumi_url
            .as_ref()
            .and_then(|u| u.split('/').last().and_then(|id| id.parse::<i64>().ok()))
            .unwrap_or(0);
        let (broadcast_day, broadcast_start) = self.extract_broadcast_info(document);
        let official_website = self.extract_official_website(document);
        let description = self.extract_text(document, &["div.bangumi-desc", ".header2-desc"]);
        crate::models::Anime {
            mikan_id,
            bangumi_id,
            title,
            original_title: None,
            broadcast_day,
            broadcast_start,
            official_website,
            bangumi_url,
            description,
            status: None,
            created_at: Some(chrono::Utc::now().timestamp_millis()),
            updated_at: Some(chrono::Utc::now().timestamp_millis()),
        }
    }

    fn extract_broadcast_info(&self, document: &Html) -> (Option<String>, Option<String>) {
        let mut day = None;
        let mut start = None;
        let selector = Selector::parse("p.bangumi-info").unwrap();
        for element in document.select(&selector) {
            let text = element
                .text()
                .collect::<String>()
                .replace("\n", "")
                .trim()
                .to_string();
            if text.starts_with("放送日期：") {
                day = Some(text.replace("放送日期：", "").trim().to_string());
            }
            if text.starts_with("放送开始：") {
                start = Some(text.replace("放送开始：", "").trim().to_string());
            }
            if day.is_some() && start.is_some() {
                break;
            }
        }
        (day, start)
    }

    fn extract_official_website(&self, document: &Html) -> Option<String> {
        let selector = Selector::parse("p.bangumi-info").unwrap();
        for element in document.select(&selector) {
            let text = element
                .text()
                .collect::<String>()
                .replace("\n", "")
                .trim()
                .to_string();
            if text.contains("官方网站：") {
                let parts: Vec<&str> = text.split("官方网站：").collect();
                if parts.len() > 1 {
                    return Some(parts[1].trim().to_string());
                }
            }
        }
        None
    }

    fn extract_text(&self, document: &Html, selectors: &[&str]) -> Option<String> {
        for selector_str in selectors {
            let selector = Selector::parse(selector_str).ok()?;
            if let Some(element) = document.select(&selector).next() {
                let text = element.text().collect::<String>().trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
        None
    }

    fn extract_href(&self, document: &Html, selectors: &[&str]) -> Option<String> {
        for selector_str in selectors {
            let selector = Selector::parse(selector_str).ok()?;
            if let Some(element) = document.select(&selector).next() {
                if let Some(href) = element.value().attr("href") {
                    return Some(href.to_string());
                }
            }
        }
        None
    }

    fn parse_groups_and_resources(
        &self,
        document: &Html,
        mikan_id: i64,
    ) -> (Vec<crate::models::SubtitleGroup>, Vec<crate::models::Resource>) {
        let mut subtitle_groups = Vec::new();
        let mut resources = Vec::new();
        let mut seen_groups = HashSet::new();
        let group_selector = Selector::parse("div.subgroup-text").unwrap();
        for group_element in document.select(&group_selector) {
            let group_id = group_element
                .value()
                .attr("id")
                .unwrap_or("0")
                .parse::<i64>()
                .unwrap_or(0);
            let group_name = group_element
                .select(&Selector::parse("a").unwrap())
                .next()
                .map_or("".to_string(), |a| {
                    a.text().collect::<String>().trim().to_string()
                });
            if group_id == 0 || group_name.is_empty() {
                continue;
            }
            if seen_groups.insert(group_id) {
                subtitle_groups.push(crate::models::SubtitleGroup {
                    id: Some(group_id),
                    name: group_name,
                    last_update: Some(chrono::Utc::now().timestamp_millis()),
                    created_at: Some(chrono::Utc::now().timestamp_millis()),
                });
            }
            // Find the following sibling table
            let mut next_node = group_element.next_sibling();
            while let Some(node) = next_node {
                if let Some(table) = ElementRef::wrap(node) {
                    if table.value().name() == "table" {
                        let resource_selector = Selector::parse("tbody tr").unwrap();
                        for row in table.select(&resource_selector) {
                            if let Some(resource) =
                                self.parse_resource_row(&row, mikan_id, group_id)
                            {
                                resources.push(resource);
                            }
                        }
                        break;
                    }
                }
                next_node = node.next_sibling();
            }
        }
        (subtitle_groups, resources)
    }

    fn parse_resource_row(
        &self,
        row: &ElementRef,
        mikan_id: i64,
        group_id: i64,
    ) -> Option<crate::models::Resource> {
        let cols: Vec<_> = row.select(&Selector::parse("td").unwrap()).collect();
        if cols.len() < 4 {
            return None;
        }
        let title_cell = &cols[0];
        let size_cell = &cols[1];
        let date_cell = &cols[2];
        let torrent_cell = &cols[3];
        let resource_title = title_cell
            .select(&Selector::parse("a.magnet-link-wrap").unwrap())
            .next()?
            .text()
            .collect::<String>()
            .trim()
            .to_string();
        let magnet_url = title_cell
            .select(&Selector::parse("a.js-magnet").unwrap())
            .next()?
            .value()
            .attr("data-clipboard-text")?
            .to_string();
        let torrent_url = torrent_cell
            .select(&Selector::parse("a").unwrap())
            .next()
            .and_then(|a| a.value().attr("href"))
            .map(|s| format!("{}{}", self.base_url, s));
        let file_size = Some(size_cell.text().collect::<String>().trim().to_string());
        let release_date_str = date_cell.text().collect::<String>().trim().to_string();
        let release_date = crate::core::text_parser::parse_datetime_to_timestamp(&release_date_str);
        let magnet_hash = {
            let re = Regex::new(r"xt=urn:btih:([a-fA-F0-9]{40})").unwrap();
            re.captures(&magnet_url)
                .and_then(|caps| caps.get(1).map(|m| m.as_str().to_lowercase()))
        };
        let episode_number = crate::core::text_parser::parse_episode_number(&resource_title);
        let resolution = crate::core::text_parser::parse_resolution(&resource_title);
        let subtitle_type = crate::core::text_parser::parse_and_normalize_subtitle_type(&resource_title);
        Some(crate::models::Resource {
            id: None,
            mikan_id,
            subtitle_group_id: group_id,
            episode_number,
            title: resource_title,
            file_size,
            resolution,
            subtitle_type,
            magnet_url: Some(magnet_url),
            torrent_url,
            play_url: None,
            magnet_hash,
            release_date,
            created_at: Some(chrono::Utc::now().timestamp_millis()),
            updated_at: Some(chrono::Utc::now().timestamp_millis()),
        })
    }
} 