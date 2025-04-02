<template>
  <div class="file-upload-item">
    <n-upload v-model:file-list="fileList" :max="1" abstract>
      <n-button @click="selectFile" type="default" 
               class="bg-gray-100 hover:bg-gray-200 transition-colors mb-2">
        {{ buttonText }}
      </n-button>
      <n-upload-file-list />
    </n-upload>
  </div>
</template>

<script setup lang="ts">
import { computed, defineProps, defineEmits} from 'vue';
import { open } from "@tauri-apps/plugin-dialog";
import { NUpload, NButton, NUploadFileList, type UploadFileInfo } from 'naive-ui';

const props = defineProps<{
  fileType: string[];
  buttonText: string;
  files?: UploadFileInfo[];
}>();

const emit = defineEmits<{
  'update:files': [files: UploadFileInfo[]];
  'error': [message: string];
}>();

const fileList = computed({
  get: () => props.files || [],
  set: (value) => {
    emit('update:files', value);
  }
});

async function selectFile() {
  try {
    // 根据文件类型设置过滤器
    const fileFilters = [{
      name: props.fileType.join(', '),
      extensions: props.fileType
    }];
    
    // 使用Tauri的文件对话框API
    const filePath = await open({
      multiple: false,
      filters: fileFilters
    });
    
    if (filePath && typeof filePath === 'string') {
      const newFile = {
        id: Date.now().toString(),
        name: filePath.split(/[/\\]/).pop() || "",
        fullPath: filePath,
        status: "finished" as const
      };
      emit('update:files', [newFile]);
    }
  } catch (error: any) {
    emit('error', `Error selecting file: ${error.message}`);
  }
}
</script>

<style scoped>
.file-upload-item {
  border: 1px dashed #d9e1ec;
  padding: 1rem;
  border-radius: 0.5rem;
  transition: all 0.3s;
}

.file-upload-item:hover {
  border-color: #2080F0;
  background-color: rgba(32, 128, 240, 0.05);
}
</style>
