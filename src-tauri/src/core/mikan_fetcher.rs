// src-tauri/src/core/mikan_fetcher.rs
use crate::core::text_parser;
use crate::models::{Anime, Resource, SubtitleGroup};
use anyhow::Result;
use regex::Regex;
use reqwest::Client;
use scraper::{ElementRef, Html, Selector};
use std::collections::{HashMap, HashSet};

// 动画详情页URL类型
type AnimeDetailUrl = String;

// 动画、字幕组、资源等结构体（可根据models完善）
#[derive(Debug)]
pub struct AnimeData {
    pub anime: Option<Anime>,
    pub subtitle_groups: Vec<SubtitleGroup>,
    pub resources: Vec<Resource>,
}

pub struct MikanFetcher {
    client: Client,
    base_url: String,
}

impl MikanFetcher {
    pub fn new(base_url: &str, proxy: Option<&str>) -> Self {
        let client = if let Some(proxy_url) = proxy {
            reqwest::Client::builder()
                .proxy(reqwest::Proxy::all(proxy_url).expect("无效的代理地址"))
                .build()
                .expect("创建带代理的Client失败")
        } else {
            reqwest::Client::new()
        };
        Self {
            client,
            base_url: base_url.to_string(),
        }
    }

    pub async fn fetch_and_parse_list(
        &self,
        url: &str,
        limit: Option<i64>,
    ) -> Result<Vec<AnimeDetailUrl>> {
        tracing::info!("MikanFetcher: 抓取列表页开始，URL: {}", url);
        let resp = self.client.get(url).send().await?.text().await?;
        let document = Html::parse_document(&resp);
        let selector = Selector::parse("div.m-week-square a[href*='/Home/Bangumi/']").unwrap();
        let mut urls = Vec::new();
        let limit = limit.map(|l| l as usize);

        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                let full_url = if href.starts_with("http") {
                    href.to_string()
                } else {
                    format!("{}{}", self.base_url, href)
                };
                urls.push(full_url);
                if let Some(lim) = limit {
                    if urls.len() >= lim {
                        break;
                    }
                }
            }
        }
        tracing::info!("MikanFetcher: 抓取列表页完成，共抓取{}个URL", urls.len());
        Ok(urls)
    }

    pub async fn fetch_and_parse_detail(&self, url: &str) -> Result<AnimeData> {
        tracing::info!("MikanFetcher: 抓取详情页开始，URL: {}", url);
        let resp = self.client.get(url).send().await?.text().await?;
        let document = Html::parse_document(&resp);
        let mikan_id = url
            .split('/')
            .last()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);

        let anime = self.parse_anime_info(&document, mikan_id);
        let (subtitle_groups, resources) = self.parse_groups_and_resources(&document, mikan_id);

        tracing::info!(
            "MikanFetcher: 抓取详情页完成, Anime: {}, Groups: {}, Resources: {}",
            anime.title,
            subtitle_groups.len(),
            resources.len()
        );

        Ok(AnimeData {
            anime: Some(anime),
            subtitle_groups,
            resources,
        })
    }

    fn parse_anime_info(&self, document: &Html, mikan_id: i64) -> Anime {
        let title = self.extract_text(document, &["p.bangumi-title", "title"])
            .unwrap_or_else(|| "未知标题".to_string())
            .replace("Mikan Project - ", "");

        let bangumi_url = self.extract_href(document, &["a[href*='bgm.tv/subject/']"]);
        let bangumi_id = bangumi_url
            .as_ref()
            .and_then(|u| u.split('/').last().and_then(|id| id.parse::<i64>().ok()))
            .unwrap_or(0);

        let (broadcast_day, broadcast_start_str) = self.extract_broadcast_info(document);
        let broadcast_start = broadcast_start_str.as_deref().and_then(text_parser::parse_datetime_to_timestamp);

        let official_website = self.extract_official_website(document);
        let description = self.extract_text(document, &["div.bangumi-desc", ".header2-desc"]);

        Anime {
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
        let selector = Selector::parse("div.bangumi-info p, .central-container p").unwrap();
        for element in document.select(&selector) {
            let text = element.text().collect::<String>();
            if text.contains("放送日期") {
                day = Some(text.replace("放送日期：", "").trim().to_string());
            }
            if text.contains("放送开始") {
                start = Some(text.replace("放送开始：", "").trim().to_string());
            }
            if day.is_some() && start.is_some() {
                break;
            }
        }
        (day, start)
    }

    fn extract_official_website(&self, document: &Html) -> Option<String> {
        // Strategy 1: Find <a> tag whose text contains "官方网站"
        let selector1 = Selector::parse("a").unwrap();
        for element in document.select(&selector1) {
            let text = element.text().collect::<String>();
            if text.contains("官方网站") {
                if let Some(href) = element.value().attr("href") {
                    return Some(href.to_string());
                }
            }
        }

        // Strategy 2: Find link in a <p> that contains "官方网站"
        let selector2 = Selector::parse("div.bangumi-info p, .central-container p").unwrap();
        for p_element in document.select(&selector2) {
            let text = p_element.text().collect::<String>();
            if text.contains("官方网站") {
                 if let Some(a_element) = p_element.select(&Selector::parse("a").unwrap()).next() {
                     if let Some(href) = a_element.value().attr("href") {
                        return Some(href.to_string());
                    }
                 }
            }
        }
        None
    }

    // Helper to extract text from the first matching selector
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

    // Helper to extract href from the first matching selector
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
    ) -> (Vec<SubtitleGroup>, Vec<Resource>) {
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
                subtitle_groups.push(SubtitleGroup {
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
                            if let Some(resource) = self.parse_resource_row(&row, mikan_id, group_id) {
                                resources.push(resource);
                            }
                        }
                        break; // Found the table for this group, move to the next group
                    }
                }
                next_node = node.next_sibling();
            }
        }
        (subtitle_groups, resources)
    }

    fn parse_resource_row(&self, row: &ElementRef, mikan_id: i64, group_id: i64) -> Option<Resource> {
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
        let release_date = text_parser::parse_datetime_to_timestamp(&release_date_str);

        let magnet_hash = {
            let re = Regex::new(r"xt=urn:btih:([a-fA-F0-9]{40})").unwrap();
            re.captures(&magnet_url)
                .and_then(|caps| caps.get(1).map(|m| m.as_str().to_lowercase()))
        };

        let episode_number = text_parser::parse_episode_number(&resource_title);
        let resolution = text_parser::parse_resolution(&resource_title);
        let subtitle_type = text_parser::parse_and_normalize_subtitle_type(&resource_title);

        Some(Resource {
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


