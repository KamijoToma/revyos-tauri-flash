<template>
  <n-card title="Step 5: Flash Files to Device" class="inner-card">
    <div class="grid grid-cols-1 gap-4 mb-6">
      <file-uploader
        file-type="bin"
        button-text="Select uboot.bin"
        v-model:files="files.ubootBin"
        @error="$emit('error', $event)"
      />
      <file-uploader
        file-type="ext4"
        button-text="Select boot.ext4"
        v-model:files="files.bootExt4"
        @error="$emit('error', $event)"
      />
      <file-uploader
        file-type="ext4"
        button-text="Select root.ext4"
        v-model:files="files.rootExt4"
        @error="$emit('error', $event)"
      />
    </div>
    <n-button @click="flash"
             :disabled="!files.ubootBin.length || !files.bootExt4.length || !files.rootExt4.length || loading"
             :loading="loading" type="primary" class="w-full">
      {{ loading ? "Flashing..." : "Flash Files" }}
    </n-button>
  </n-card>
</template>

<script setup lang="ts">
import { computed, defineProps, defineEmits } from 'vue';
import { NCard, NButton, type UploadFileInfo } from 'naive-ui';
import FileUploader from '../FileUploader.vue';

interface FileCollection {
  ubootBin: UploadFileInfo[];
  bootExt4: UploadFileInfo[];
  rootExt4: UploadFileInfo[];
}

const props = defineProps<{
  fileCollection: FileCollection;
  loading: boolean;
}>();

const emit = defineEmits<{
  'update:fileCollection': [files: FileCollection];
  'flash': [];
  'error': [message: string];
}>();

const files = computed({
  get: () => props.fileCollection,
  set: (value) => emit('update:fileCollection', value)
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
