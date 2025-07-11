pub mod text_parser;
pub mod http_fetcher;
pub mod mikan_parser;
pub mod anime_parser;

use crate::models::{Anime, Resource, SubtitleGroup};
// 动画、字幕组、资源等结构体（可根据models完善）
#[derive(Debug)]
pub struct AnimeData {
    pub anime: Option<Anime>,
    pub subtitle_groups: Vec<SubtitleGroup>,
    pub resources: Vec<Resource>,
}