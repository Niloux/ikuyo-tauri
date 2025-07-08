/**
 * 订阅API服务
 * 封装所有订阅相关的后端API调用
 */
import { invoke } from '@tauri-apps/api/core'
import { UserManager } from '../../utils/userManager'
import type {
    UserSubscription,
    SubscriptionStatus,
    GetSubscriptionsParams,
    SubscriptionsResponse,
    SubscriptionResult,
    SubscriptionIdsResponse
} from './subscriptionTypes'

class SubscriptionApiService {
    private readonly baseURL = '/subscriptions'

    /**
     * 获取HTTP请求头，包含用户ID
     */
    private getHeaders(): HeadersInit {
        return {
            'Content-Type': 'application/json',
            'X-User-Id': UserManager.getUserId()
        }
    }

    /**
     * 处理API响应
     */
    private async handleResponse<T>(response: Response): Promise<T> {
        if (!response.ok) {
            const errorData = await response.json().catch(() => ({}))
            throw new Error(errorData.detail || `HTTP ${response.status}: ${response.statusText}`)
        }
        return response.json()
    }

    /**
     * 添加订阅
     */
    async subscribe(
        bangumi_id: number,
        anime_name: string,
        anime_name_cn: string,
        anime_rating?: number,
        anime_air_date?: string,
        anime_air_weekday?: number,
        // 新增参数
        url?: string,
        item_type?: number,
        summary?: string,
        rank?: number,
        images?: string, // 存储 BangumiImages 的 JSON 字符串
    ): Promise<UserSubscription> {
        return invoke('subscribe', {
            user_id: UserManager.getUserId(),
            bangumi_id,
            anime_name,
            anime_name_cn,
            anime_rating,
            anime_air_date,
            anime_air_weekday,
            // 新增参数传递
            url,
            item_type,
            summary,
            rank,
            images,
        })
    }

    /**
     * 取消订阅
     */
    async unsubscribe(bangumi_id: number): Promise<void> {
        await invoke('unsubscribe', {
            user_id: UserManager.getUserId(),
            bangumi_id,
        })
    }

    /**
     * 获取订阅列表
     */
    async getSubscriptions(params: GetSubscriptionsParams = {}): Promise<SubscriptionsResponse> {
        const response: SubscriptionsResponse = await invoke('get_subscriptions', {
            user_id: UserManager.getUserId(),
            sort: params.sort,
            order: params.order,
            search: params.search,
            page: params.page,
            limit: params.limit,
        });
        return response;
    }

    /**
     * 检查订阅状态
     */
    async checkSubscription(bangumi_id: number): Promise<SubscriptionStatus> {
        const response: SubscriptionStatus = await invoke('check_subscription', {
            user_id: UserManager.getUserId(),
            bangumi_id,
        });
        return response
    }

    /**
     * 安全的订阅操作（带错误处理）
     */
    async safeSubscribe(bangumi_id: number, anime_name: string, anime_name_cn: string, anime_rating?: number, anime_air_date?: string, anime_air_weekday?: number): Promise<SubscriptionResult> {
        try {
            const data = await this.subscribe(bangumi_id, anime_name, anime_name_cn, anime_rating, anime_air_date, anime_air_weekday)
            return { success: true, data }
        } catch (error) {
            return {
                success: false,
                error: {
                    message: error instanceof Error ? error.message : '订阅失败'
                }
            }
        }
    }

    /**
     * 安全的取消订阅操作（带错误处理）
     */
    async safeUnsubscribe(bangumi_id: number): Promise<SubscriptionResult> {
        try {
            await this.unsubscribe(bangumi_id)
            return { success: true }
        } catch (error) {
            return {
                success: false,
                error: {
                    message: error instanceof Error ? error.message : '取消订阅失败'
                }
            }
        }
    }

    /**
     * 获取所有已订阅bangumi_id（轻量接口）
     */
    async getAllSubscriptionIds(): Promise<SubscriptionIdsResponse> {
        const res: SubscriptionIdsResponse = await invoke('get_all_subscription_ids', {
            user_id: UserManager.getUserId()
        })
        return res
    }
}

// 导出单例实例
export default new SubscriptionApiService()
