<template>
  <div>
    <n-scrollbar class="steps-container mb-6" x-scrollable>
      <n-steps :current="currentStep" size="small" class="steps">
        <n-step v-for="(title, index) in stepTitles" :key="index" :title="title" />
      </n-steps>
    </n-scrollbar>
    
    <div class="navigation flex justify-between mt-6">
      <n-button @click="prev" :disabled="currentStep === 1 || loading" 
               class="min-w-[100px]">
        Previous
      </n-button>
      <n-button @click="next" :disabled="currentStep === stepTitles.length || loading" 
               type="primary" class="min-w-[100px]">
        Next
      </n-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { defineProps, defineEmits } from 'vue';
import { NSteps, NStep, NScrollbar, NButton } from 'naive-ui';

const props = defineProps<{
  currentStep: number;
  loading: boolean;
  stepTitles: string[];
}>();

const emit = defineEmits<{
  'prev': [];
  'next': [];
}>();

function prev() {
  emit('prev');
}

function next() {
  emit('next');
}
</script>

<style scoped>
.steps-container {
  overflow-x: auto;
  padding: 0 10px;
}

.steps {
  margin: 10px 0;
}

.step-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  margin: 0 8px;
  transition: background-color 0.3s;
}
</style>
