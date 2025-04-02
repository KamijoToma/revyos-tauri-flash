<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { NCascader, NCard, NSpace, NSpin, NButton } from 'naive-ui';

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

const loading = ref(true);
const cascaderOptions = ref<any[]>([]);
const selectedPath = ref<string[] | null>(null);
const selectedVariant = ref<ImageVariant | null>(null);

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

// 组件加载时获取镜像列表
onMounted(() => {
  loadImageVersions();
});
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
                            </li>
                        </ul>
                    </div>
                </div>
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
                    @click="loadImageVersions"
                >
                    Refresh Image List
                </NButton>
            </NSpace>
        </NSpin>
    </NCard>
</template>
