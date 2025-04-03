<template>
  <n-card title="Step 2: Flash uboot.bin to RAM" class="inner-card">
    <!-- 添加选项卡以选择本地文件或在线镜像 -->
    <n-tabs v-model:value="selectedMode" type="segment" class="mb-4">
      <n-tab-pane name="localFiles" tab="本地文件">
        <file-uploader
          :file-type="['bin']"
          button-text="Select uboot.bin file"
          v-model:files="fileList"
          @error="$emit('error', $event)"
        />
      </n-tab-pane>
      <n-tab-pane name="imageVariant" tab="在线镜像">
        <ImageSelector
          :image-variant="selectedImageVariant"
          @select-variant="handleImageVariantSelected"/>
        
        <!-- 添加在线镜像刷入进度显示 -->
        <div v-if="selectedImageVariant && imageFlashProgress > 0" class="mt-4">
          <div class="flex justify-between mb-1">
            <span>刷入进度</span>
            <span>{{ Math.floor(imageFlashProgress) }}%</span>
          </div>
          <n-progress
            :percentage="imageFlashProgress"
            :indicator-placement="'inside'"
            :processing="loading"
            :height="12"
          />
        </div>
      </n-tab-pane>
    </n-tabs>

    <n-button @click="flash" 
              :disabled="!isReadyToFlash || loading"
              :loading="loading" type="primary" class="w-full mt-4">
      {{ loading ? "Flashing..." : "Flash to RAM" }}
    </n-button>
  </n-card>
</template>

<script setup lang="ts">
import { computed, defineProps, defineEmits, ref } from 'vue';
import { NCard, NButton, NTabs, NTabPane, NProgress, type UploadFileInfo } from 'naive-ui';
import FileUploader from '../FileUploader.vue';
import ImageSelector from '../../ImageSelector.vue';
import { type ImageVariant } from '../../ImageSelector.vue';

const props = defineProps<{
  files: UploadFileInfo[];
  loading: boolean;
  selectedImageVariant: ImageVariant | null;
  imageFlashProgress: number; // 添加新的属性接收进度值
}>();

const emit = defineEmits<{
  'update:files': [files: UploadFileInfo[]];
  'update:selectedImageVariant': [imageVariant: ImageVariant | null];
  'flash': [];
  'error': [message: string];
}>();

const selectedMode = ref('localFiles'); // 默认使用本地文件模式

const fileList = computed({
  get: () => props.files,
  set: (value) => emit('update:files', value)
});

const isReadyToFlash = computed(() => {
  if (selectedMode.value === 'localFiles') {
    return fileList.value.length > 0;
  } else {
    // 检查选择的镜像是否有 uboot 可用
    return props.selectedImageVariant !== null && 
           props.selectedImageVariant.image_binarys.some(binary => 
              binary.binary_type === 'UBoot' && binary.local_path !== null);
  }
});

function flash() {
  // 传递选中模式到父组件
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
