use crate::types::bangumi::{BangumiSubject, BangumiWeekday, BangumiEpisodesData};

pub struct BangumiService {
    base_url: String,
    client: reqwest::Client,
}

impl BangumiService {
    pub fn new() -> Self {
        let proxy = reqwest::Proxy::all("http://127.0.0.1:7890")
            .expect("代理配置失败");
        let client = reqwest::Client::builder()
            .proxy(proxy)
            .user_agent("Ikuyo-App/1.0 (https://github.com/your-repo-link)") // 添加User-Agent
            .build()
            .expect("reqwest client 构建失败");
        Self {
            base_url: "https://api.bgm.tv".to_string(),
            client,
        }
    }

    pub async fn get_calendar(&self) -> Result<Vec<BangumiWeekday>, String> {
        let url = format!("{}/calendar", self.base_url);
        let response = self.client.get(&url).send().await.map_err(|e| e.to_string())?;

        if response.status().is_success() {
            let data: Vec<BangumiWeekday> = response.json().await.map_err(|e| e.to_string())?;
            Ok(data)
        } else {
            Err(format!("请求失败: {}", response.status()))
        }
    }

    pub async fn get_subject(&self, id: i64) -> Result<BangumiSubject, String> {
        let url = format!("{}/subject/{}", self.base_url, id);
        let response = self.client.get(&url).send().await.map_err(|e| e.to_string())?;
        let data: BangumiSubject = response.json().await.map_err(|e| e.to_string())?;
        Ok(data)
    }

    pub async fn get_episodes(
        &self,
        subject_id: i64,
        episode_type: Option<i64>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<BangumiEpisodesData, String> {
        let mut url = format!("{}/v0/episodes", self.base_url);
        let mut params = Vec::new();

        params.push(format!("subject_id={}", subject_id));

        if let Some(ep_type) = episode_type {
            params.push(format!("type={}", ep_type));
        }
        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }
        if let Some(o) = offset {
            params.push(format!("offset={}", o));
        }

        if !params.is_empty() {
            url.push_str(&format!("?{}", params.join("&")));
        }

        tracing::info!("请求URL: {}", url);
        let response = self.client.get(&url).send().await.map_err(|e| e.to_string())?;

        if response.status().is_success() {
            let data: BangumiEpisodesData = response.json().await.map_err(|e| e.to_string())?;
            Ok(data)
        } else {
            Err(format!("请求失败: {}", response.status()))
        }
    }
}
