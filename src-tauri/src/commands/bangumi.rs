use crate::types::bangumi::{BangumiCalendarItem, BangumiSubject, BangumiWeekday, BangumiRating, BangumiImages, BangumiCollection, BangumiTag, WeekdayInfo};
use std::collections::HashMap;

#[tauri::command]
pub fn get_bangumi_calendar() -> Result<Vec<BangumiWeekday>, String> {
    tracing::info!("Fetching bangumi calendar");
    // 返回示例数据
    Ok(vec![
        BangumiWeekday {
            weekday: WeekdayInfo { en: "Mon".to_string(), cn: "星期一".to_string(), ja: "月".to_string(), id: 1 },
            items: vec![
                BangumiCalendarItem {
                    id: 1,
                    url: "https://example.com/bangumi/1".to_string(),
                    item_type: 2,
                    name: "示例番剧A".to_string(),
                    name_cn: "示例番剧A中文".to_string(),
                    summary: "这是一个示例番剧A的简介".to_string(),
                    air_date: "2023-01-01".to_string(),
                    air_weekday: 1,
                    rating: BangumiRating {
                        total: 100,
                        count: HashMap::from([("5".to_string(), 50), ("4".to_string(), 30), ("3".to_string(), 20)]),
                        score: 8.5,
                    },
                    rank: 10,
                    images: BangumiImages {
                        large: "large_a.jpg".to_string(),
                        common: "common_a.jpg".to_string(),
                        medium: "medium_a.jpg".to_string(),
                        small: "small_a.jpg".to_string(),
                        grid: "grid_a.jpg".to_string(),
                    },
                },
            ],
        },
    ])
}

#[tauri::command]
pub fn get_bangumi_subject(id: u32) -> Result<BangumiSubject, String> {
    tracing::info!("Fetching bangumi subject with ID: {}", id);
    // 返回示例数据
    Ok(BangumiSubject {
        id,
        name: "示例番剧B".to_string(),
        name_cn: "示例番剧B中文".to_string(),
        summary: "这是一个示例番剧B的简介".to_string(),
        date: "2023-04-01".to_string(),
        air_weekday: 4,
        eps: 12,
        total_episodes: 12,
        rating: BangumiRating {
            total: 200,
            count: HashMap::from([("5".to_string(), 100), ("4".to_string(), 60), ("3".to_string(), 40)]),
            score: 9.0,
        },
        rank: 5,
        images: BangumiImages {
            large: "large_b.jpg".to_string(),
            common: "common_b.jpg".to_string(),
            medium: "medium_b.jpg".to_string(),
            small: "small_b.jpg".to_string(),
            grid: "grid_b.jpg".to_string(),
        },
        collection: BangumiCollection {
            wish: 50,
            collect: 150,
            doing: 20,
            on_hold: 5,
            dropped: 2,
        },
        tags: vec![
            BangumiTag { name: "奇幻".to_string(), count: 100, total_cont: 1000 },
            BangumiTag { name: "冒险".to_string(), count: 80, total_cont: 800 },
        ],
    })
}