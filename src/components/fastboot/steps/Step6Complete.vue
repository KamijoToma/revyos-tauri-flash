<template>
  <n-card title="Step 6: Restart Device" class="inner-card">
    <div class="text-green-600 mb-6 text-center">
      <div class="flex justify-center mb-4">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-16 w-16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
      </div>
      <h3 class="text-xl font-bold">Flashing process completed</h3>
      <p class="text-gray-600 mt-2">You can now safely restart your device</p>
    </div>
    <n-button @click="restart" :loading="loading" type="primary" class="w-full">
      {{ loading ? "Restarting..." : "Restart Device" }}
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
