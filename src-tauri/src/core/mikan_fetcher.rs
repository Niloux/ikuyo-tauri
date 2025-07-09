// src-tauri/src/core/mikan_fetcher.rs
use reqwest::Client;
use scraper::{Html, Selector};
use anyhow::Result;
use crate::models::{Anime, SubtitleGroup, Resource};

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

    /// 抓取并解析动画详情页，返回AnimeData结构体
    pub async fn fetch_and_parse_detail(&self, url: &str) -> Result<AnimeData> {
        tracing::info!("MikanFetcher: 抓取详情页开始，URL: {}", url);
        let resp = self.client.get(url).send().await?.text().await?;
        let document = Html::parse_document(&resp);
        // mikan_id 提取自URL
        let mikan_id = url.split('/').last().and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);
        // title
        let title_selector = Selector::parse("div.an-info > h1").unwrap();
        let title = document
            .select(&title_selector)
            .next()
            .map(|e| e.text().collect::<String>())
            .unwrap_or_else(|| "未知标题".to_string());
        // 字幕组
        let mut subtitle_groups = Vec::new();
        let group_selector = Selector::parse("div.an-subgroup > a").unwrap();
        for group in document.select(&group_selector) {
            let name = group.text().collect::<String>().trim().to_string();
            if !name.is_empty() {
                subtitle_groups.push(SubtitleGroup {
                    id: None,
                    name,
                    last_update: None,
                    created_at: None,
                });
            }
        }
        // 资源（只提取磁链和标题）
        let mut resources = Vec::new();
        let res_selector = Selector::parse("table.table tbody tr").unwrap();
        for row in document.select(&res_selector) {
            let cols: Vec<_> = row.select(&Selector::parse("td").unwrap()).collect();
            if cols.len() >= 4 {
                let title = cols[1].text().collect::<String>().trim().to_string();
                let magnet = cols[3].select(&Selector::parse("a[href^='magnet']").unwrap())
                    .next()
                    .and_then(|a| a.value().attr("href"))
                    .map(|s| s.to_string());
                if let Some(magnet_url) = magnet {
                    resources.push(Resource {
                        id: None,
                        mikan_id,
                        subtitle_group_id: 0, // 暂不解析分组ID
                        episode_number: None,
                        title,
                        file_size: None,
                        resolution: None,
                        subtitle_type: None,
                        magnet_url: Some(magnet_url),
                        torrent_url: None,
                        play_url: None,
                        magnet_hash: None,
                        release_date: None,
                        created_at: None,
                        updated_at: None,
                    });
                }
            }
        }
        // Anime
        let anime = Some(Anime {
            mikan_id,
            bangumi_id: 0,
            title,
            original_title: None,
            broadcast_day: None,
            broadcast_start: None,
            official_website: None,
            bangumi_url: None,
            description: None,
            status: None,
            created_at: None,
            updated_at: None,
        });
        Ok(AnimeData {
            anime,
            subtitle_groups,
            resources,
        })
    }
}
 