use crate::core::AnimeData;
use crate::error::Result;

pub trait AnimeParser {
    /// 解析番剧列表页，返回详情页URL列表
    fn parse_list(&self, html: &str) -> Result<Vec<String>>;
    /// 解析番剧详情页，返回结构化数据，需传入mikan_id
    fn parse_detail(&self, html: &str, mikan_id: i64) -> Result<AnimeData>;
} 