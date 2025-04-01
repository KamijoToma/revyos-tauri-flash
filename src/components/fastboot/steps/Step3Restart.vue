<template>
  <n-card title="Step 3: Restart to Stage 2" class="inner-card">
    <div class="text-gray-700 mb-4">
      The device will restart to Stage 2 mode. Please make sure your device is ready.
    </div>
    <n-button @click="restart" :loading="loading" type="primary" class="w-full">
      {{ loading ? "Restarting..." : "Restart" }}
    </n-button>
  </n-card>
</template>

<script setup lang="ts">
import { defineProps, defineEmits } from 'vue';
import { NCard, NButton } from 'naive-ui';

defineProps<{
  loading: boolean;
}>();

const emit = defineEmits<{
  'restart': [];
}>();

function restart() {
  emit('restart');
}
</script>

<style scoped>
.inner-card {
  margin-bottom: 1rem;
  border-radius: 0.5rem;
  background-color: white;
}
</style>
