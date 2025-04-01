<template>
  <n-alert v-if="status" type="info" class="status my-4 bg-blue-50">{{ status }}</n-alert>
</template>

<script setup lang="ts">
import { defineProps } from 'vue';
import { NAlert } from 'naive-ui';

defineProps<{
  status: string;
}>();
</script>

<style scoped>
.status {
  border-left: 4px solid #2080F0;
}
</style>
