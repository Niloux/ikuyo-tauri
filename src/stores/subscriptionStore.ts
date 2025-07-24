/**
 * 订阅状态管理Store
 * 管理用户订阅数据，提供乐观更新和缓存功能
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import subscriptionApiService from '../services/subscription/subscriptionApiService'
import type {
    SubscriptionWithAnime,
    GetSubscriptionsParams,
    PaginationInfo,
} from '../services/subscription/subscriptionTypes'
import type { BangumiCalendarItem } from '../services/bangumi/bangumiTypes'
import { toast } from 'vue-sonner'

export const useSubscriptionStore = defineStore('subscription', () => {
    // 状态
    const subscriptions = ref<SubscriptionWithAnime[]>([])
    const loading = ref(false)
    const error = ref<string | null>(null)
    const pagination = ref<PaginationInfo>({
        page: 1,
        limit: 8,
        total: 0,
        pages: 0
    })

    // 当前查询参数
    const currentParams = ref<GetSubscriptionsParams>({
        sort: 'subscribed_at',
        order: 'desc',
        page: 1,
        limit: 8
    })

    // 全量订阅ID集合（用于全局判断）
    const allSubscribedBangumiIds = ref<Set<number> | null>(null)

    // 新增：全局搜索内容状态
    const searchQuery = ref('')
    const setSearchQuery = (val: string) => {
        searchQuery.value = val
    }

    /**
     * 计算属性：订阅的番剧ID集合，用于快速查询订阅状态
     */
    const subscribedBangumiIds = computed(() => {
        return new Set(subscriptions.value.map(sub => sub.bangumi_id))
    })

    /**
     * 获取所有订阅ID（轻量接口）
     */
    const fetchAllSubscriptionIds = async () => {
        try {
            const response = await subscriptionApiService.getAllSubscriptionIds()
            allSubscribedBangumiIds.value = new Set(response.ids)
        } catch (err) {
            toast.error('获取全部订阅ID失败')
        }
    }

    /**
     * 判断是否已订阅指定番剧
     */
    const isSubscribed = (bangumiId: number): boolean => {
        if (allSubscribedBangumiIds.value) {
            return allSubscribedBangumiIds.value.has(bangumiId)
        }
        return subscribedBangumiIds.value.has(bangumiId)
    }

    /**
     * 获取订阅列表（包含番剧详情）
     */
    const fetchSubscriptions = async (params: GetSubscriptionsParams = {}) => {
        try {
            loading.value = true
            error.value = null

            // 合并参数
            const finalParams = { ...currentParams.value, ...params }
            currentParams.value = finalParams

            // 获取订阅列表
            const response = await subscriptionApiService.getSubscriptions(finalParams)

            // 直接使用缓存的番剧数据构建 subscriptionsWithAnime
            const subscriptionsWithAnime: SubscriptionWithAnime[] = response.subscriptions
                .map(sub => {
                    // 将 UserSubscription 转换为 SubscriptionWithAnime
                    // 注意：这里假设 UserSubscription 已经包含了所有 BangumiCalendarItem 的必要字段
                    // 如果有缺失，需要在这里进行处理或补充默认值
                    return {
                        ...sub,
                        anime: {
                            id: sub.bangumi_id,
                            url: sub.url || '', // 提供默认值
                            type: sub.item_type || 0, // 提供默认值
                            name: sub.anime_name || '', // 提供默认值
                            name_cn: sub.anime_name_cn || '', // 提供默认值
                            summary: sub.summary || '', // 提供默认值
                            air_date: sub.anime_air_date || '', // 提供默认值
                            air_weekday: sub.anime_air_weekday || 0, // 提供默认值
                            rating: sub.anime_rating ? { total: 0, count: {}, score: sub.anime_rating } : { total: 0, count: {}, score: 0 }, // 提供默认值
                            rank: sub.rank || 0, // 提供默认值
                            images: sub.images ? JSON.parse(sub.images) : { large: '', common: '', medium: '', small: '', grid: '' }, // 解析 JSON 字符串
                        } as BangumiCalendarItem
                    }
                })

            subscriptions.value = subscriptionsWithAnime
            pagination.value = response.pagination

        } catch (err) {
            error.value = err instanceof Error ? err.message : '获取订阅列表失败'
            toast.error(error.value)
        } finally {
            loading.value = false
        }
    }

    /**
     * 乐观订阅：立即更新UI，然后调用API
     * 同步本地allSubscribedBangumiIds集合，提升首页订阅按钮响应速度
     */
    const optimisticSubscribe = async (anime: BangumiCalendarItem) => {
        // 立即添加到列表（乐观更新）
        const optimisticSubscription: SubscriptionWithAnime = {
            user_id: '',
            bangumi_id: anime.id,
            subscribed_at: Date.now() / 1000,
            anime,
            anime_name: anime.name,
            anime_name_cn: anime.name_cn,
            anime_rating: anime.rating?.score,
            anime_air_date: anime.air_date,
            anime_air_weekday: anime.air_weekday,
            // 新增字段
            url: anime.url,
            item_type: anime.type,
            summary: anime.summary,
            rank: anime.rank,
            images: JSON.stringify(anime.images), // 将 BangumiImages 转换为 JSON 字符串
        }

        subscriptions.value.unshift(optimisticSubscription)

        // 本地同步allSubscribedBangumiIds集合（如已初始化）
        let addedToAllIds = false
        if (allSubscribedBangumiIds.value) {
            allSubscribedBangumiIds.value.add(anime.id)
            addedToAllIds = true
        }

        try {
            // 调用API确认订阅
            await subscriptionApiService.subscribe(
                anime.id,
                anime.name,
                anime.name_cn,
                anime.rating?.score,
                anime.air_date,
                anime.air_weekday,
                // 新增参数传递
                anime.url,
                anime.type,
                anime.summary,
                anime.rank,
                JSON.stringify(anime.images),
            )
            toast.success('订阅成功')
        } catch (err) {
            // 如果失败，移除乐观添加的项目
            subscriptions.value = subscriptions.value.filter(sub => sub.bangumi_id !== anime.id)
            // 回滚allSubscribedBangumiIds集合
            if (allSubscribedBangumiIds.value && addedToAllIds) {
                allSubscribedBangumiIds.value.delete(anime.id)
            }
            const errorMsg = err instanceof Error ? err.message : '订阅失败'
            toast.error(errorMsg)
            throw err
        }
    }

    /**
     * 乐观取消订阅：立即更新UI，然后调用API
     * 同步本地allSubscribedBangumiIds集合，提升首页订阅按钮响应速度
     */
    const optimisticUnsubscribe = async (bangumiId: number) => {
        // 保存原始数据以便回滚
        const originalSubscriptions = [...subscriptions.value]

        // 立即从列表中移除（乐观更新）
        subscriptions.value = subscriptions.value.filter(sub => sub.bangumi_id !== bangumiId)

        // 本地同步allSubscribedBangumiIds集合（如已初始化）
        let removedFromAllIds = false
        if (allSubscribedBangumiIds.value) {
            removedFromAllIds = allSubscribedBangumiIds.value.delete(bangumiId)
        }

        try {
            // 调用API确认取消订阅
            await subscriptionApiService.unsubscribe(bangumiId)
            toast.success('取消订阅成功')
        } catch (err) {
            // 如果失败，恢复原始数据
            subscriptions.value = originalSubscriptions
            // 回滚allSubscribedBangumiIds集合
            if (allSubscribedBangumiIds.value && removedFromAllIds) {
                allSubscribedBangumiIds.value.add(bangumiId)
            }
            const errorMsg = err instanceof Error ? err.message : '取消订阅失败'
            toast.error(errorMsg)
            throw err
        }
    }

    /**
     * 切换订阅状态
     */
    const toggleSubscription = async (anime: BangumiCalendarItem) => {
        if (isSubscribed(anime.id)) {
            await optimisticUnsubscribe(anime.id)
        } else {
            await optimisticSubscribe(anime)
        }
    }

    /**
     * 搜索订阅
     */
    const searchSubscriptions = async (query: string) => {
        await fetchSubscriptions({ ...currentParams.value, search: query, page: 1 })
    }

    /**
     * 切换排序
     */
    const sortSubscriptions = async (sort: GetSubscriptionsParams['sort'], order: GetSubscriptionsParams['order'] = 'desc') => {
        await fetchSubscriptions({ ...currentParams.value, sort, order, page: 1 })
    }

    /**
     * 翻页
     */
    const goToPage = async (page: number) => {
        await fetchSubscriptions({ ...currentParams.value, page })
    }

    /**
     * 清空状态
     */
    const clear = () => {
        subscriptions.value = []
        error.value = null
        loading.value = false
        pagination.value = {
            page: 1,
            limit: 20,
            total: 0,
            pages: 0
        }
        currentParams.value = {
            sort: 'subscribed_at',
            order: 'desc',
            page: 1,
            limit: 20
        }
    }

    return {
        // 状态
        subscriptions,
        loading,
        error,
        pagination,
        currentParams,
        allSubscribedBangumiIds,
        // 新增：导出 searchQuery 和 setSearchQuery
        searchQuery,
        setSearchQuery,
        // 计算属性
        subscribedBangumiIds,
        // 方法
        isSubscribed,
        fetchSubscriptions,
        fetchAllSubscriptionIds,
        optimisticSubscribe,
        optimisticUnsubscribe,
        toggleSubscription,
        searchSubscriptions,
        sortSubscriptions,
        goToPage,
        clear
    }
})
