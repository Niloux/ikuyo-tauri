// =============================================================================
// Bangumi API Service
// =============================================================================

import { invoke } from "@tauri-apps/api/core";
import type {
  BangumiCalendarItem,
  BangumiSubject,
  BangumiWeekday,
  EpisodeAvailabilityData,
  BangumiEpisodesData,
  EpisodeResourcesData,
  SearchLibraryResponse,
} from "./bangumiTypes";
import { debounceAsync, throttleAsync } from "../common/common";

/**
 * 数据转换工具：BangumiSubject 转换为 BangumiCalendarItem
 */
export function convertSubjectToCalendarItem(
  subject: BangumiSubject,
): BangumiCalendarItem {
  return {
    id: subject.id,
    url: `https://bgm.tv/subject/${subject.id}`,
    type: 2, // 动画类型
    name: subject.name,
    name_cn: subject.name_cn,
    summary: subject.summary,
    air_date: subject.date,
    air_weekday: subject.air_weekday,
    rating: subject.rating,
    rank: subject.rank,
    images: subject.images,
  };
}

// API服务类
export class BangumiApiService {
  /**
   * 获取每日放送
   */
  static async getCalendar(): Promise<BangumiWeekday[]> {
    const response: BangumiWeekday[] = await invoke("get_calendar");
    return response;
  }

  /**
   * 获取番剧详情
   */
  static async getSubject(bangumiId: number): Promise<BangumiSubject> {
    const response: BangumiSubject = await invoke("get_subject", {
      id: bangumiId,
    });
    return response;
  }

  /**
   * 获取集数可用性状态
   */
  static async getEpisodeAvailability(
    bangumi_id: number,
  ): Promise<EpisodeAvailabilityData | null> {
    const response: EpisodeAvailabilityData | null = await invoke(
      "get_episode_availability",
      { bangumi_id },
    );
    return response;
  }

  /**
   * 获取Bangumi章节信息
   */
  static async getBangumiEpisodes(
    subject_id: number,
    episode_type?: number,
    limit: number = 100,
    offset: number = 0,
  ): Promise<BangumiEpisodesData> {
    const response: BangumiEpisodesData = await invoke("get_episodes", {
      subject_id,
      episode_type: episode_type !== undefined ? episode_type : null,
      limit,
      offset,
    });
    return response;
  }

  /**
   * 获取特定集数的资源列表
   */
  static async getEpisodeResources(
    bangumi_id: number,
    episode: number,
  ): Promise<EpisodeResourcesData | null> {
    const response: EpisodeResourcesData | null = await invoke(
      "get_episode_resources",
      { bangumi_id, episode },
    );
    return response;
  }

  /**
   * 获取番剧的所有资源列表
   */
  static async getAnimeResources(
    bangumi_id: number,
    options?: {
      resolution?: string;
      subtitle_type?: string;
      limit?: number;
      offset?: number;
    },
  ): Promise<EpisodeResourcesData> {
    const response: EpisodeResourcesData | null = await invoke(
      "get_anime_resources",
      {
        bangumi_id,
        resolution: options?.resolution,
        subtitle_type: options?.subtitle_type,
        limit: options?.limit,
        offset: options?.offset,
      },
    );
    return response!;
  }

  /**
   * 资源库搜索
   */
  static searchLibrary = (
    query: string,
    page: number = 1,
    limit: number = 12,
    options?: { debounce?: boolean; throttle?: boolean; delay?: number },
  ): Promise<SearchLibraryResponse> => {
    const fn = async (q: string, p: number, l: number) => {
      const response: SearchLibraryResponse = await invoke("search_library", {
        query: q,
        page: p,
        limit: l,
      });
      return response;
    };
    if (options?.debounce) {
      return debounceAsync(fn, options.delay)(query, page, limit);
    } else if (options?.throttle) {
      return throttleAsync(fn, options.delay)(query, page, limit);
    } else {
      return fn(query, page, limit);
    }
  };

  /**
   * 批量获取番剧详情
   */
  static async batchGetSubjects(bangumiIds: number[]): Promise<{
    success: BangumiSubject[];
    failed: { id: number; reason: any }[];
  }> {
    const promises = bangumiIds.map((id) =>
      this.getSubject(id).then(
        (value) => ({ status: "fulfilled", value, id }) as const,
        (reason) => ({ status: "rejected", reason, id }) as const,
      ),
    );
    const results = await Promise.all(promises);
    const success: BangumiSubject[] = [];
    const failed: { id: number; reason: any }[] = [];
    for (const r of results) {
      if (r.status === "fulfilled") success.push(r.value);
      else if (r.status === "rejected")
        failed.push({ id: r.id, reason: r.reason });
    }
    return { success, failed };
  }
}

export default BangumiApiService;
