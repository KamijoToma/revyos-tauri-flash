<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { NCascader, NCard, NSpace, NSpin, NButton, NProgress, NText, NList, NListItem } from 'naive-ui';

export interface ImageBinary {
    name: string;
    web_path: string | null;
    local_path: string | null;
    binary_type: string;
    hash_type: string | null;
    hash_value: string | null;
}

export interface ImageVariant {
    name: string;
    image_binarys: ImageBinary[];
}

export interface ImageVersion {
    version: string;
    image_variants: ImageVariant[];
}

interface DownloadProgress {
    filename: string;
    current: number;
    total: number;
    lastUpdate: number;   // 上次更新的时间戳（用于计算瞬时速度）
    prevBytes: number;    // 上次更新的字节数（用于计算瞬时速度）
    speed: number;        // 当前速度，字节/秒
    startTime: number;    // 开始下载的时间戳（用于计算平均速度）
    progressType: string; // 'download' 或 'extract'
}

const loading = ref(true);
const cascaderOptions = ref<any[]>([]);
const selectedPath = ref<string[] | null>(null);
const selectedVariant = ref<ImageVariant | null>(null);
const isDownloading = ref(false);
const downloadProgress = ref<Record<string, DownloadProgress>>({});
const errorMessage = ref<string | null>(null);

// 将后端返回的数据转换为Cascader需要的格式
const transformToCascaderOptions = (imageVersions: ImageVersion[]) => {
  return imageVersions.map(version => ({
    label: `${version.version}`,
    value: version.version,
    children: version.image_variants.map(variant => ({
      label: variant.name,
      value: variant.name,
      variant: variant // 存储完整的variant数据
    }))
  }));
};

// 加载镜像数据
const loadImageVersions = async () => {
  try {
    loading.value = true;
    const result = await invoke<ImageVersion[]>('fetch_lpi4a_image_versions');
    cascaderOptions.value = transformToCascaderOptions(result);
    console.log('镜像版本数据:', result);
  } catch (error) {
    console.error('加载镜像版本失败:', error);
  } finally {
    loading.value = false;
  }
};

// 处理选择变化
const handleCascaderChange = (value: string[], option: any) => {
    selectedVariant.value = option.variant;
};

// 处理下载图像
const handleDownloadImage = async () => {
    if (!selectedVariant.value) return;
    
    try {
        isDownloading.value = true;
        errorMessage.value = null;
        downloadProgress.value = {};
        
        // 为每个二进制文件初始化进度
        const currentTime = Date.now();
        selectedVariant.value.image_binarys.forEach(binary => {
            if (binary.name) {
                downloadProgress.value[binary.name] = {
                    filename: binary.name,
                    current: 0,
                    total: 100, // 默认值，将在进度更新时替换
                    lastUpdate: currentTime,
                    prevBytes: 0,
                    speed: 0,
                    startTime: currentTime,
                    progressType: 'download'
                };
            }
        });
        
        // 调用后端下载方法
        const result = await invoke('download_image_variant', {
            variant: selectedVariant.value
        });
        
        // 成功下载后更新本地变体数据
        await loadImageVersions();
        console.log('下载结果:', result);
    } catch (error) {
        console.error('下载图像失败:', error);
        errorMessage.value = `下载失败: ${error}`;
    } finally {
        isDownloading.value = false;
    }
};

// 监听下载进度事件
let unlisten: any;
onMounted(async () => {
    loadImageVersions();
    
    unlisten = await listen('image-download-progress', (event: any) => {
        const { filename, current, total, progressType } = event.payload;
        const now = Date.now();
        
        // 如果是首次更新这个文件的进度
        if (!downloadProgress.value[filename]) {
            downloadProgress.value[filename] = {
                filename,
                current,
                total,
                lastUpdate: now,
                prevBytes: 0,
                speed: 0,
                startTime: now,
                progressType: progressType
            };
            return;
        }
        
        // 更新现有进度对象
        const progress = downloadProgress.value[filename];
        const timeDiff = now - progress.lastUpdate;
        const bytesDiff = current - progress.prevBytes;
        
        // 计算速度 (字节/秒)，仅当有意义的时间间隔时
        if (timeDiff > 200) { // 至少200毫秒，避免除以很小的数导致不稳定
            progress.speed = (bytesDiff / timeDiff) * 1000;
            progress.lastUpdate = now;
            progress.prevBytes = current;
        }
        
        progress.current = current;
        progress.total = total;
        progress.progressType = progressType;
    });
});

onUnmounted(() => {
    if (unlisten) unlisten();
});

// 计算下载百分比
const getProgressPercentage = (progress: DownloadProgress) => {
    if (progress.total === 0) return 0;
    return Math.round((progress.current / progress.total) * 100);
};

// 格式化文件大小
const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

// 格式化速度
const formatSpeed = (bytesPerSecond: number) => {
    if (bytesPerSecond === 0) return '0 B/s';
    
    const k = 1024;
    
    if (bytesPerSecond < k) {
        return `${bytesPerSecond.toFixed(1)} B/s`;
    } else if (bytesPerSecond < k * k) {
        return `${(bytesPerSecond / k).toFixed(1)} KB/s`;
    } else if (bytesPerSecond < k * k * k) {
        return `${(bytesPerSecond / (k * k)).toFixed(1)} MB/s`;
    } else {
        return `${(bytesPerSecond / (k * k * k)).toFixed(1)} GB/s`;
    }
};

// 计算平均速度
const getAverageSpeed = (progress: DownloadProgress) => {
    const elapsedTime = (Date.now() - progress.startTime) / 1000; // 秒
    if (elapsedTime <= 0) return 0;
    return progress.current / elapsedTime; // 字节/秒
};

// 估计剩余时间
const getEstimatedTimeRemaining = (progress: DownloadProgress) => {
    if (progress.speed <= 0) return '计算中...';
    
    const remainingBytes = progress.total - progress.current;
    const remainingSeconds = remainingBytes / progress.speed;
    
    if (remainingSeconds < 60) {
        return `${Math.ceil(remainingSeconds)}秒`;
    } else if (remainingSeconds < 3600) {
        return `${Math.ceil(remainingSeconds / 60)}分钟`;
    } else {
        return `${Math.floor(remainingSeconds / 3600)}小时${Math.ceil((remainingSeconds % 3600) / 60)}分钟`;
    }
};
</script>

<template>
    <NCard title="Select RevyOS Image Version" class="mb-4">
        <NSpin :show="loading">
            <NSpace vertical>
                <div class="mb-2">
                    <p class="text-gray-600">Select the RevyOS image version and variant you want to flash</p>
                </div>
                <NCascader 
                    v-model:value="selectedPath"
                    :options="cascaderOptions"
                    placeholder="Select RevyOS version and variant"
                    @update:value="handleCascaderChange"
                    check-strategy="child"
                    class="w-full"
                />
                <div v-if="selectedVariant" class="mt-4">
                    <h3 class="font-bold text-gray-800">Selected Variant Details:</h3>
                    <div class="pl-4 mt-2">
                        <p><span class="font-semibold">Name:</span> {{ selectedVariant.name }}</p>
                        <p class="font-semibold mt-2">Included Files:</p>
                        <ul class="list-disc pl-6">
                            <li v-for="binary in selectedVariant.image_binarys" :key="binary.name">
                                {{ binary.name }} ({{ binary.binary_type }})
                                <NText v-if="binary.local_path" type="success" class="ml-2">
                                    [已下载]
                                </NText>
                            </li>
                        </ul>
                    </div>
                </div>
                
                <!-- 下载进度显示 -->
                <div v-if="isDownloading && Object.keys(downloadProgress).length > 0" class="mt-4">
                    <h3 class="font-bold text-gray-800 mb-2">下载进度:</h3>
                    <NList bordered>
                        <NListItem v-for="progress in downloadProgress" :key="progress.filename">
                            <div class="w-full">
                                <div class="flex justify-between mb-1">
                                    <span>{{ progress.filename }}</span>
                                    <span>
                                        {{ progress.progressType === 'extract' ? '解压中: ' : '' }}
                                        {{ getProgressPercentage(progress) }}%
                                    </span>
                                </div>
                                <NProgress
                                    :percentage="getProgressPercentage(progress)"
                                    :indicator-placement="'inside'"
                                    :height="12"
                                    :status="progress.progressType === 'extract' ? 'warning' : 'info'"
                                />
                                <div class="flex justify-between text-xs mt-1 text-gray-500">
                                    <span>{{ formatFileSize(progress.current) }} / {{ formatFileSize(progress.total) }}</span>
                                    <span v-if="progress.progressType === 'download'">{{ formatSpeed(progress.speed) }}</span>
                                    <span v-else>解压中...</span>
                                </div>
                                <div class="text-right text-xs mt-1 text-gray-500" v-if="progress.progressType === 'download'">
                                    <span>剩余时间: {{ getEstimatedTimeRemaining(progress) }} | 平均: {{ formatSpeed(getAverageSpeed(progress)) }}</span>
                                </div>
                            </div>
                        </NListItem>
                    </NList>
                </div>
                
                <NText v-if="errorMessage" type="error" class="mt-2">{{ errorMessage }}</NText>
                
                <NButton 
                    class="mt-3 w-full" 
                    type="primary" 
                    :disabled="!selectedVariant"
                    @click="$emit('select-variant', selectedVariant)"
                >
                    Use Selected Image
                </NButton>
                
                <NButton 
                    class="mt-2 w-full" 
                    type="info"
                    :disabled="!selectedVariant || isDownloading"
                    :loading="isDownloading"
                    @click="handleDownloadImage"
                >
                    {{ isDownloading ? 'Downloading...' : 'Download Selected Image' }}
                </NButton>
                
                <NButton 
                    class="mt-2 w-full" 
                    @click="loadImageVersions"
                    :disabled="isDownloading"
                >
                    Refresh Image List
                </NButton>
            </NSpace>
        </NSpin>
    </NCard>
</template>
