<template>
  <div class="episode-list-container">
    <h3 class="section-title">章节列表</h3>

    <!-- 集数统计信息 -->
    <div v-if="!loading && !error && episodeStats" class="episode-stats">
      <span class="stats-text">
        共{{ episodeStats.totalCount }}章节，已有{{
          episodeStats.availableCount
        }}章节
        <template
          v-if="displayMode === 'grid' && needsPagination && currentPageRange"
        >
          (第{{ currentPageRange.start }}-{{ currentPageRange.end }}集，当前页{{
            episodeStats.currentPageAvailableCount
          }}章节)
        </template>
      </span>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="loading-state">
      <p>正在加载章节信息...</p>
    </div>

    <!-- 错误状态 -->
    <div v-else-if="error" class="error-state">
      <p>{{ error }}</p>
    </div>

    <!-- Carousel Display -->
    <div v-else-if="displayMode === 'carousel' && episodes.length > 0">
      <div class="carousel-wrapper">
        <div class="episode-carousel" ref="carouselContainer">
          <div
            v-for="episode in episodes"
            :key="episode.number"
            :class="[
              'episode-card',
              episode.available ? 'available' : 'unavailable',
            ]"
            @click="handleEpisodeClick(episode)"
          >
            <div class="episode-number">
              <span class="number">{{
                String(episode.number).padStart(2, "0")
              }}</span>
            </div>
            <div class="episode-info">
              <h4 class="episode-title">
                {{ episode.title || `第${episode.number}集` }}
              </h4>
              <p class="episode-subtitle" v-if="episode.subtitle">
                {{ episode.subtitle }}
              </p>
              <div class="episode-meta">
                <span v-if="episode.duration" class="duration"
                  >时长: {{ episode.duration }}</span
                >
                <span v-if="episode.airdate" class="airdate"
                  >首播: {{ formatDate(episode.airdate) }}</span
                >
              </div>
            </div>
            <div class="resource-status">
              <div v-if="episode.available" class="has-resources">
                <span class="resource-count"
                  >{{ episode.resourceCount }}个资源</span
                >
                <button class="download-btn">下载</button>
              </div>
              <div v-else class="no-resources">
                <span class="no-resource-text">暂无资源</span>
                <button class="refresh-btn">刷新</button>
              </div>
            </div>
          </div>
        </div>
        <div class="carousel-controls">
          <button
            @click="scrollLeft"
            :disabled="isAtStart"
            class="control-btn prev-btn"
          >
            ←
          </button>
          <button
            @click="scrollRight"
            :disabled="isAtEnd"
            class="control-btn next-btn"
          >
            →
          </button>
        </div>
      </div>
    </div>

    <!-- Grid Display -->
    <div v-else-if="displayMode === 'grid' && episodes.length > 0">
      <div class="episode-grid" :style="gridStyle">
        <div
          v-for="episode in currentPageEpisodes"
          :key="episode.number"
          :class="[
            'episode-item',
            episode.available ? 'available' : 'unavailable',
          ]"
          @click="handleEpisodeClick(episode)"
          :title="
            episode.available
              ? `${episode.title} (${episode.resourceCount}个资源)`
              : `${episode.title} (暂无资源)`
          "
        >
          {{ episode.number }}
        </div>
      </div>
      <div v-if="needsPagination" class="pagination-container">
        <div class="pagination-info">
          <span>第 {{ currentPage }} / {{ totalPages }} 页</span>
        </div>
        <div class="pagination-controls">
          <button
            @click="goToPrevPage"
            :disabled="currentPage === 1"
            class="pagination-btn prev-btn"
          >
            上一页
          </button>
          <div class="page-numbers">
            <button
              v-if="currentPage > 3"
              @click="goToPage(1)"
              class="pagination-btn page-btn"
            >
              1
            </button>
            <span v-if="currentPage > 4" class="pagination-ellipsis">...</span>
            <button
              v-for="page in getVisiblePages()"
              :key="page"
              @click="goToPage(page)"
              :class="[
                'pagination-btn',
                'page-btn',
                { active: page === currentPage },
              ]"
            >
              {{ page }}
            </button>
            <span
              v-if="currentPage < totalPages - 3"
              class="pagination-ellipsis"
              >...</span
            >
            <button
              v-if="currentPage < totalPages - 2"
              @click="goToPage(totalPages)"
              class="pagination-btn page-btn"
            >
              {{ totalPages }}
            </button>
          </div>
          <button
            @click="goToNextPage"
            :disabled="currentPage === totalPages"
            class="pagination-btn next-btn"
          >
            下一页
          </button>
        </div>
        <div class="pagination-jump">
          <span>跳转到</span>
          <input
            type="number"
            :min="1"
            :max="totalPages"
            v-model.number="jumpPage"
            @keyup.enter="handleJumpToPage"
            class="jump-input"
          />
          <button @click="handleJumpToPage" class="pagination-btn jump-btn">
            确定
          </button>
        </div>
      </div>
    </div>

    <!-- No Data State -->
    <div v-else class="no-data-state">
      <p>暂无集数信息</p>
    </div>

    <!-- Episode Detail Modal -->
    <EpisodeDetailModal
      :visible="modalVisible"
      :episode-data="selectedEpisode"
      :bangumi-id="bangumiId"
      :subject="props.subject"
      @close="closeModal"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, watch, defineAsyncComponent } from "vue";

const EpisodeDetailModal = defineAsyncComponent(
  () => import("./EpisodeDetailModal.vue"),
);

// Props definition
interface Props {
  displayMode: "carousel" | "grid";
  bangumiId: number;
  totalEpisodes: number;
  bangumiEpisodes?: any[];
  preloadedAvailability?: any;
  loading: boolean;
  error: string | null;
  subject: any; // 可细化类型
}

const props = defineProps<Props>();

// --- Common Data & Logic ---

const availabilityData = computed(() => props.preloadedAvailability);

interface EpisodeDetail {
  number: number;
  title: string;
  subtitle?: string;
  duration?: string;
  airdate?: string;
  desc?: string;
  comment?: number;
  available: boolean;
  resourceCount: number;
  bangumiData?: any;
}

const episodes = computed((): EpisodeDetail[] => {
  const episodeList: EpisodeDetail[] = [];
  const bangumiEps = props.bangumiEpisodes;

  if (bangumiEps && bangumiEps.length > 0) {
    bangumiEps.forEach((bangumiEp) => {
      const episodeKey = Math.floor(
        bangumiEp.sort || bangumiEp.ep || 0,
      ).toString();
      const episodeData = availabilityData.value?.episodes[episodeKey];
      episodeList.push({
        number: Math.floor(bangumiEp.sort || bangumiEp.ep || 0),
        title:
          bangumiEp.name_cn ||
          bangumiEp.name ||
          `第${Math.floor(bangumiEp.sort)}集`,
        subtitle: bangumiEp.name_cn ? bangumiEp.name : undefined,
        duration: bangumiEp.duration || undefined,
        airdate: bangumiEp.airdate || undefined,
        desc: bangumiEp.desc || undefined,
        comment: bangumiEp.comment || 0,
        available: episodeData?.available || false,
        resourceCount: episodeData?.resource_count || 0,
        bangumiData: bangumiEp,
      });
    });
    episodeList.sort((a, b) => a.number - b.number);
  } else if (props.totalEpisodes > 0 && !props.loading) {
    for (let i = 1; i <= props.totalEpisodes; i++) {
      const episodeKey = i.toString();
      const episodeData = availabilityData.value?.episodes[episodeKey];
      episodeList.push({
        number: i,
        title: `第${i}集`,
        available: episodeData?.available || false,
        resourceCount: episodeData?.resource_count || 0,
        subtitle: undefined,
        duration: "N/A",
        airdate: undefined,
        desc: undefined,
        comment: 0,
        bangumiData: null,
      });
    }
  }
  return episodeList;
});

const episodeStats = computed(() => {
  if (!availabilityData.value) return null;
  const availableCount = episodes.value.filter((ep) => ep.available).length;
  const currentPageAvailableCount =
    props.displayMode === "grid"
      ? currentPageEpisodes.value.filter((ep) => ep.available).length
      : 0;
  return {
    totalCount: props.totalEpisodes,
    availableCount,
    currentPageAvailableCount,
  };
});

// Modal Logic
const modalVisible = ref(false);
const selectedEpisode = ref<EpisodeDetail | null>(null);

const handleEpisodeClick = (episode: EpisodeDetail) => {
  selectedEpisode.value = episode;
  modalVisible.value = true;
};

const closeModal = () => {
  modalVisible.value = false;
  selectedEpisode.value = null;
};

// --- Carousel Specific Logic ---

const carouselContainer = ref<HTMLElement>();
const isAtStart = ref(true);
const isAtEnd = ref(false);

const scrollLeft = () => {
  if (carouselContainer.value) {
    carouselContainer.value.scrollBy({ left: -320, behavior: "smooth" });
  }
};

const scrollRight = () => {
  if (carouselContainer.value) {
    carouselContainer.value.scrollBy({ left: 320, behavior: "smooth" });
  }
};

const updateScrollButtons = () => {
  if (carouselContainer.value) {
    const container = carouselContainer.value;
    isAtStart.value = container.scrollLeft <= 0;
    isAtEnd.value =
      container.scrollLeft + container.clientWidth >= container.scrollWidth - 1;
  }
};

watch(carouselContainer, (newEl) => {
  if (newEl) {
    nextTick(() => {
      updateScrollButtons();
      newEl.addEventListener("scroll", updateScrollButtons);
    });
  }
});

// --- Grid Specific Logic ---

const currentPage = ref(1);
const episodesPerPage = 85; // 5 rows x 17 cols
const jumpPage = ref<number>(1);

const actualTotalEpisodes = computed(() => episodes.value.length);
const needsPagination = computed(
  () => actualTotalEpisodes.value > episodesPerPage,
);
const totalPages = computed(() =>
  Math.ceil(actualTotalEpisodes.value / episodesPerPage),
);

const currentPageEpisodes = computed(() => {
  if (!needsPagination.value) {
    return episodes.value;
  }
  const startIndex = (currentPage.value - 1) * episodesPerPage;
  const endIndex = Math.min(
    startIndex + episodesPerPage,
    actualTotalEpisodes.value,
  );
  return episodes.value.slice(startIndex, endIndex);
});

const currentPageRange = computed(() => {
  if (!needsPagination.value) return null;
  const startEpisode = (currentPage.value - 1) * episodesPerPage + 1;
  const endEpisode = Math.min(
    currentPage.value * episodesPerPage,
    actualTotalEpisodes.value,
  );
  return { start: startEpisode, end: endEpisode };
});

const columnsPerRow = computed(() => {
  const totalEps = currentPageEpisodes.value.length;
  if (!needsPagination.value) {
    return totalEps > 17 ? 17 : totalEps;
  }
  return 17;
});

const gridStyle = computed(() => {
  const columns = columnsPerRow.value;
  if (columns <= 0) return {};
  return {
    gridTemplateColumns: `repeat(${columns}, 1fr)`,
    gap: "6px",
  };
});

const goToPage = (page: number) => {
  if (page >= 1 && page <= totalPages.value) {
    currentPage.value = page;
  }
};
const goToPrevPage = () => goToPage(currentPage.value - 1);
const goToNextPage = () => goToPage(currentPage.value + 1);
const handleJumpToPage = () => goToPage(jumpPage.value);

const getVisiblePages = () => {
  const pages = [];
  const start = Math.max(1, currentPage.value - 2);
  const end = Math.min(totalPages.value, currentPage.value + 2);
  for (let i = start; i <= end; i++) {
    pages.push(i);
  }
  return pages;
};

// --- Helper Functions ---

const formatDate = (dateStr: string): string => {
  if (!dateStr) return "";
  try {
    return new Date(dateStr).toLocaleDateString("zh-CN", {
      month: "short",
      day: "numeric",
    });
  } catch {
    return dateStr;
  }
};
</script>

<style scoped>
/* Common Styles */
.episode-list-container {
  background: white;
  padding: 2rem;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.section-title {
  font-size: 1.3rem;
  font-weight: 600;
  color: #2c3e50;
  margin-bottom: 1rem;
}

.episode-stats {
  margin-bottom: 1.5rem;
}

.stats-text {
  color: #7f8c8d;
  font-size: 0.9rem;
}

.loading-state,
.error-state,
.no-data-state {
  text-align: center;
  padding: 2rem;
  color: #7f8c8d;
}

.error-state {
  color: #e74c3c;
}

/* Carousel Styles */
.carousel-wrapper {
  position: relative;
}

.episode-carousel {
  display: flex;
  gap: 1rem;
  overflow-x: auto;
  scroll-behavior: smooth;
  padding-bottom: 1rem;
  scrollbar-width: thin;
}

.episode-carousel::-webkit-scrollbar {
  height: 6px;
}
.episode-carousel::-webkit-scrollbar-track {
  background: #f1f1f1;
  border-radius: 3px;
}
.episode-carousel::-webkit-scrollbar-thumb {
  background: #888;
  border-radius: 3px;
}
.episode-carousel::-webkit-scrollbar-thumb:hover {
  background: #555;
}

.episode-card {
  flex: 0 0 300px;
  height: 200px;
  border-radius: 8px;
  padding: 1rem;
  cursor: pointer;
  transition: all 0.3s ease;
  display: flex;
  flex-direction: column;
  position: relative;
  overflow: hidden;
}

.episode-card.available {
  background: linear-gradient(135deg, #d34642 0%, #b73b3b 100%);
  color: white;
  box-shadow: 0 4px 12px rgba(183, 59, 59, 0.3);
}

.episode-card.unavailable {
  background: linear-gradient(135deg, #f5b5b3 0%, #e87572 100%);
  color: #2d3436;
  opacity: 0.8;
  box-shadow: 0 4px 12px rgba(232, 117, 114, 0.3);
}

.episode-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 25px rgba(0, 0, 0, 0.15);
}

.episode-number {
  position: absolute;
  top: 1rem;
  right: 1rem;
  z-index: 1;
  background: rgba(255, 255, 255, 0.2);
  padding: 0.25rem 0.5rem;
  border-radius: 12px;
  font-weight: bold;
  font-size: 0.9rem;
}

.episode-info {
  flex: 1;
  margin-bottom: 0.5rem;
  z-index: 1;
  margin-right: 60px;
}

.episode-title {
  font-size: 1.2rem;
  font-weight: 700;
  margin-bottom: 0.25rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.episode-subtitle {
  font-size: 0.8rem;
  opacity: 0.7;
  margin-bottom: 0.3rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.episode-meta {
  display: flex;
  gap: 1rem;
  font-size: 0.8rem;
  opacity: 0.9;
}

.resource-status {
  margin-top: auto;
  padding-top: 0.5rem;
  z-index: 1;
}

.has-resources,
.no-resources {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.resource-count,
.no-resource-text {
  font-size: 0.85rem;
  font-weight: 500;
}

.download-btn,
.refresh-btn {
  background: rgba(255, 255, 255, 0.2);
  border: none;
  padding: 0.4rem 0.8rem;
  border-radius: 6px;
  color: inherit;
  cursor: pointer;
  font-size: 0.8rem;
  font-weight: 500;
  transition: all 0.2s ease;
}

.download-btn:hover,
.refresh-btn:hover {
  background: rgba(255, 255, 255, 0.35);
}

.carousel-controls {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  width: 100%;
  display: flex;
  justify-content: space-between;
  pointer-events: none;
  padding: 0 1rem;
}

.control-btn {
  background: rgba(255, 255, 255, 0.9);
  border: none;
  width: 40px;
  height: 40px;
  border-radius: 50%;
  cursor: pointer;
  font-size: 1.2rem;
  color: #2c3e50;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  transition: all 0.2s ease;
  pointer-events: auto;
}

.control-btn:hover:not(:disabled) {
  background: white;
  transform: scale(1.1);
}

.control-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Grid Styles */
.episode-grid {
  display: grid;
  width: 100%;
}

.episode-item {
  aspect-ratio: 1;
  min-height: 28px;
  min-width: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 500;
  transition: all 0.2s ease;
  user-select: none;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.episode-item.available {
  background-color: #3498db;
  color: white;
  cursor: pointer;
}

.episode-item.available:hover {
  background-color: #2980b9;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(52, 152, 219, 0.4);
}

.episode-item.unavailable {
  background-color: #ecf0f1;
  color: #bdc3c7;
  cursor: not-allowed;
}

.pagination-container {
  margin-top: 2rem;
  padding-top: 1.5rem;
  border-top: 1px solid #e1e8ed;
}

.pagination-info {
  text-align: center;
  margin-bottom: 1rem;
  color: #7f8c8d;
  font-size: 0.9rem;
}

.pagination-controls {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 1rem;
  flex-wrap: wrap;
}

.page-numbers {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.pagination-btn {
  padding: 0.5rem 0.75rem;
  border: 1px solid #ddd;
  background: white;
  cursor: pointer;
  border-radius: 4px;
  font-size: 0.9rem;
  transition: all 0.2s ease;
}

.pagination-btn:hover:not(:disabled) {
  background-color: #f8f9fa;
  border-color: #3498db;
}

.pagination-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.pagination-btn.active {
  background-color: #3498db;
  color: white;
  border-color: #3498db;
}

.pagination-ellipsis {
  padding: 0.5rem;
  color: #7f8c8d;
}

.pagination-jump {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  color: #7f8c8d;
  font-size: 0.9rem;
}

.jump-input {
  width: 60px;
  padding: 0.25rem 0.5rem;
  border: 1px solid #ddd;
  border-radius: 4px;
  text-align: center;
}

.jump-input:focus {
  outline: none;
  border-color: #3498db;
}

/* Responsive Design */
@media (max-width: 768px) {
  .episode-list-container {
    padding: 1.5rem;
  }
  .episode-card {
    flex: 0 0 250px;
    height: 180px;
  }
  .carousel-controls {
    display: none;
  }
  .pagination-controls {
    gap: 0.25rem;
  }
}

@media (max-width: 480px) {
  .episode-card {
    flex: 0 0 220px;
    height: 160px;
  }
  .pagination-controls {
    flex-direction: column;
    gap: 0.75rem;
  }
}
</style>
