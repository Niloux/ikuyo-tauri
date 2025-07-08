use crate::types::bangumi::{BangumiSubject, BangumiWeekday};

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
}
