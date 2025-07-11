use reqwest::Client;
use crate::error::Result;

pub struct HttpFetcher {
    client: Client,
}

impl HttpFetcher {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn fetch(&self, url: &str) -> Result<String> {
        let resp = self.client.get(url).send().await?.text().await?;
        Ok(resp)
    }
} 