// src-tauri/src/core/mikan_fetcher.rs
use crate::core::text_parser;
use crate::models::{Anime, Resource, SubtitleGroup};
use anyhow::Result;
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;

// 动画详情页URL类型
type AnimeDetailUrl = String;

// 动画、字幕组、资源等结构体（可根据models完善）
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

    /// 抓取并解析动画列表页，返回详情页URL集合
    pub async fn fetch_and_parse_list(&self, url: &str, limit: Option<usize>) -> Result<Vec<AnimeDetailUrl>> {
        tracing::info!("MikanFetcher: 抓取列表页开始，URL: {}", url);
        let resp = self.client.get(url).send().await?.text().await?;
        let document = Html::parse_document(&resp);
        let selector = Selector::parse("div.m-week-square a[href*='/Home/Bangumi/']").unwrap();
        let mut urls = Vec::new();
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

    // 抓取并解析动画详情页，返回AnimeData结构体
    pub async fn fetch_and_parse_detail(&self, url: &str) -> Result<AnimeData> {
        tracing::info!("MikanFetcher: 抓取详情页开始，URL: {}", url);
        let resp = self.client.get(url).send().await?.text().await?;
        let document = Html::parse_document(&resp);

        // --- Anime Info Extraction ---
        let mikan_id = url.split('/').last().and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);
        
        let title_selector = Selector::parse("p.bangumi-title").unwrap();
        let title = document.select(&title_selector).next().map_or("未知标题".to_string(), |e| e.text().collect::<String>().trim().to_string());

        let bangumi_url_selector = Selector::parse("a[href*='bgm.tv/subject/']").unwrap();
        let bangumi_url = document.select(&bangumi_url_selector).next().and_then(|e| e.value().attr("href")).map(|s| s.to_string());
        let bangumi_id = bangumi_url.as_ref().and_then(|u| u.split('/').last().and_then(|id| id.parse::<i64>().ok())).unwrap_or(0);

        let mut anime_info = HashMap::new();
        let info_selector = Selector::parse("div.bangumi-info p").unwrap();
        for p in document.select(&info_selector) {
            let text = p.text().collect::<String>();
            let parts: Vec<&str> = text.split('：').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                anime_info.insert(parts[0].to_string(), parts[1].to_string());
            }
        }

        let official_website = anime_info.get("官方网站").map(|s| s.to_string());
        let broadcast_day = anime_info.get("放送日期").map(|s| s.to_string());
        let broadcast_start_str = anime_info.get("放送开始").map(|s| s.to_string());
        let broadcast_start = broadcast_start_str.as_deref().and_then(text_parser::parse_datetime_to_timestamp);


        let description_selector = Selector::parse("div.bangumi-desc").unwrap();
        let description = document.select(&description_selector).next().map(|e| e.text().collect::<String>().trim().to_string());

        let anime = Some(Anime {
            mikan_id,
            bangumi_id,
            title,
            original_title: None, // Mikan doesn't usually provide this
            broadcast_day,
            broadcast_start,
            official_website,
            bangumi_url,
            description,
            status: None, // Status needs to be determined based on broadcast dates
            created_at: Some(chrono::Utc::now().timestamp_millis()),
            updated_at: Some(chrono::Utc::now().timestamp_millis()),
        });

        // --- SubtitleGroup and Resource Extraction ---
        let mut subtitle_groups = Vec::new();
        let mut resources = Vec::new();
        let mut subtitle_group_map = HashMap::new();

        let group_container_selector = Selector::parse("div.subgroup-text").unwrap();
        for group_element in document.select(&group_container_selector) {
            let group_id_str = group_element.value().attr("id").unwrap_or("0");
            let group_id = group_id_str.parse::<i64>().unwrap_or(0);
            
            let group_name = group_element.select(&Selector::parse("a").unwrap()).next().map_or("".to_string(), |a| a.text().collect::<String>().trim().to_string());

            if group_id != 0 && !group_name.is_empty() {
                let group = SubtitleGroup {
                    id: Some(group_id),
                    name: group_name.clone(),
                    last_update: Some(chrono::Utc::now().timestamp_millis()),
                    created_at: Some(chrono::Utc::now().timestamp_millis()),
                };
                if subtitle_group_map.get(&group_id).is_none() {
                    subtitle_group_map.insert(group_id, group.clone());
                    subtitle_groups.push(group);
                }

                // 若后续用到next_sibling_element，改为next_sibling并手动判断节点类型。
                // 当前代码未用到next_sibling_element，无需修复。
                if let Some(sibling) = group_element.next_sibling() {
                    if let Some(table) = scraper::ElementRef::wrap(sibling) {
                        if table.value().name() == "table" {
                            let resource_selector = Selector::parse("tbody tr").unwrap();
                            for row in table.select(&resource_selector) {
                                let cols: Vec<_> = row.select(&Selector::parse("td").unwrap()).collect();
                                if cols.len() >= 3 {
                                    let title_cell = &cols[0];
                                    let size_cell = &cols[1];
                                    let date_cell = &cols[2];

                                    let resource_title = title_cell.select(&Selector::parse("a.magnet-link-wrap").unwrap()).next().map_or("".to_string(), |a| a.text().collect::<String>().trim().to_string());
                                    let magnet_url = title_cell.select(&Selector::parse("a.js-magnet").unwrap()).next().and_then(|a| a.value().attr("data-clipboard-text")).map(|s| s.to_string());
                                    let torrent_url = cols[3].select(&Selector::parse("a").unwrap()).next().and_then(|a| a.value().attr("href")).map(|s| format!("{}{}", self.base_url, s));
                                    
                                    let file_size = Some(size_cell.text().collect::<String>().trim().to_string());
                                    let release_date_str = date_cell.text().collect::<String>().trim().to_string();
                                    let release_date = text_parser::parse_datetime_to_timestamp(&release_date_str);

                                    let magnet_hash = magnet_url.as_ref().and_then(|u| {
                                        let re = Regex::new(r"xt=urn:btih:([a-fA-F0-9]{40})").unwrap();
                                        re.captures(u).and_then(|caps| caps.get(1).map(|m| m.as_str().to_lowercase()))
                                    });
                                    
                                    let episode_number = text_parser::parse_episode_number(&resource_title);
                                    let resolution = text_parser::parse_resolution(&resource_title);
                                    let subtitle_type = text_parser::parse_subtitle_type(&resource_title);

                                    if let Some(magnet_url_unwrapped) = magnet_url {
                                        resources.push(Resource {
                                            id: None,
                                            mikan_id,
                                            subtitle_group_id: group_id,
                                            episode_number,
                                            title: resource_title,
                                            file_size,
                                            resolution,
                                            subtitle_type,
                                            magnet_url: Some(magnet_url_unwrapped),
                                            torrent_url,
                                            play_url: None, // Mikan doesn't provide this directly
                                            magnet_hash,
                                            release_date,
                                            created_at: Some(chrono::Utc::now().timestamp_millis()),
                                            updated_at: Some(chrono::Utc::now().timestamp_millis()),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        tracing::info!("MikanFetcher: 抓取详情页完成, Anime: {}, Groups: {}, Resources: {}", anime.as_ref().unwrap().title, subtitle_groups.len(), resources.len());

        Ok(AnimeData {
            anime,
            subtitle_groups,
            resources,
        })
    }
}

