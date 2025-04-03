<template>
  <n-card title="Step 5: Flash Files to Device" class="inner-card">
    <!-- 添加选项卡以选择本地文件或在线镜像 -->
    <n-tabs v-model:value="selectedMode" type="segment" class="mb-4">
      <n-tab-pane name="localFiles" tab="本地文件">
        <div class="grid grid-cols-1 gap-4 mb-6">
          <file-uploader
            :file-type="['bin']"
            button-text="Select uboot.bin"
            v-model:files="files.ubootBin"
            @error="$emit('error', $event)"
          />
          <file-uploader
            :file-type="['ext4']"
            button-text="Select boot.ext4"
            v-model:files="files.bootExt4"
            @error="$emit('error', $event)"
          />
          <file-uploader
            :file-type="['ext4']"
            button-text="Select root.ext4"
            v-model:files="files.rootExt4"
            @error="$emit('error', $event)"
          />
        </div>
      </n-tab-pane>
      <n-tab-pane name="imageVariant" tab="在线镜像">
        <ImageSelector
          :image-variant="selectedImageVariant"
          @select-variant="handleImageVariantSelected"/>
          
        <!-- 添加在线镜像刷入进度显示 -->
        <div v-if="selectedImageVariant && imageFlashProgress.currentStep !== ''" class="mt-4">
          <div class="font-medium mb-2">当前刷入: {{ imageFlashProgress.currentStep }}</div>
          <div class="flex justify-between mb-1">
            <span>刷入进度</span>
            <span>{{ Math.floor(imageFlashProgress.percentage) }}%</span>
          </div>
          <n-progress
            :percentage="imageFlashProgress.percentage"
            :indicator-placement="'inside'"
            :processing="loading"
            :height="12"
          />
          <div class="text-xs text-gray-500 mt-1 text-right">
            {{ Math.floor(imageFlashProgress.currentFile) }} / {{ Math.floor(imageFlashProgress.totalFiles) }}
          </div>
        </div>
      </n-tab-pane>
    </n-tabs>

    <n-button @click="flash"
             :disabled="!isReadyToFlash || loading"
             :loading="loading" type="primary" class="w-full">
      {{ loading ? "Flashing..." : "Flash Files" }}
    </n-button>
  </n-card>
</template>

<script setup lang="ts">
import { computed, defineProps, defineEmits, ref } from 'vue';
import { NCard, NButton, NTabs, NTabPane, NProgress, type UploadFileInfo } from 'naive-ui';
import FileUploader from '../FileUploader.vue';
import ImageSelector from '../../ImageSelector.vue';
import { type ImageVariant } from '../../ImageSelector.vue';

export interface FileCollection {
  ubootBin: UploadFileInfo[];
  bootExt4: UploadFileInfo[];
  rootExt4: UploadFileInfo[];
}

export interface ImageFlashProgressInfo {
  currentStep: string;
  percentage: number;
  currentFile: number;
  totalFiles: number;
}

const props = defineProps<{
  fileCollection: FileCollection;
  loading: boolean;
  selectedImageVariant: ImageVariant | null;
  imageFlashProgress: ImageFlashProgressInfo; // 添加新的属性接收进度值
}>();

const emit = defineEmits<{
  'update:fileCollection': [files: FileCollection];
  'update:selectedImageVariant': [imageVariant: ImageVariant | null];
  'flash': [];
  'error': [message: string];
}>();

const selectedMode = ref('localFiles'); // 默认使用本地文件模式

const files = computed({
  get: () => props.fileCollection,
  set: (value) => emit('update:fileCollection', value)
});

const isReadyToFlash = computed(() => {
  if (selectedMode.value === 'localFiles') {
    return files.value.ubootBin.length && files.value.bootExt4.length && files.value.rootExt4.length;
  } else {
    // 检查选择的镜像是否有所需的二进制文件
    return props.selectedImageVariant !== null && 
           hasRequiredBinaries(props.selectedImageVariant);
  }
});

function hasRequiredBinaries(variant: ImageVariant) {
  if (!variant) return false;
  
  const hasUboot = variant.image_binarys.some(binary => 
    binary.binary_type === 'UBoot' && binary.local_path !== null);
  const hasBoot = variant.image_binarys.some(binary => 
    binary.binary_type === 'Boot' && binary.local_path !== null);
  const hasRoot = variant.image_binarys.some(binary => 
    binary.binary_type === 'Root' && binary.local_path !== null);
  
  return hasUboot && hasBoot && hasRoot;
}

function flash() {
  // 传递选中的模式到父组件
  emit('flash');
}

function handleImageVariantSelected(variant: ImageVariant) {
  emit('update:selectedImageVariant', variant);
}
</script>

<style scoped>
.inner-card {
  margin-bottom: 1rem;
  border-radius: 0.5rem;
  background-color: white;
}
</style>
