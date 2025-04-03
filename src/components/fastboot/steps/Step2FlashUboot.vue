<template>
  <n-card title="Step 2: Flash uboot.bin to RAM" class="inner-card">
    <file-uploader
      :file-type="['bin']"
      button-text="Select uboot.bin file"
      v-model:files="fileList"
      @error="$emit('error', $event)"
    />
    <n-button @click="flash" :disabled="!fileList.length || loading"
             :loading="loading" type="primary" class="w-full mt-4">
      {{ loading ? "Flashing..." : "Flash to RAM" }}
    </n-button>
  </n-card>
</template>

<script setup lang="ts">
import { computed, defineProps, defineEmits } from 'vue';
import { NCard, NButton, type UploadFileInfo } from 'naive-ui';
import FileUploader from '../FileUploader.vue';

const props = defineProps<{
  files: UploadFileInfo[];
  loading: boolean;
}>();

const emit = defineEmits<{
  'update:files': [files: UploadFileInfo[]];
  'flash': [];
  'error': [message: string];
}>();

const fileList = computed({
  get: () => props.files,
  set: (value) => emit('update:files', value)
});

function flash() {
  emit('flash');
}
</script>

<style scoped>
.inner-card {
  margin-bottom: 1rem;
  border-radius: 0.5rem;
  background-color: white;
}
</style>
