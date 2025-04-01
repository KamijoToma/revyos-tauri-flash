<script setup lang="ts">
import FastBootFlash from "./FastBootFlash.vue"; // Import FastBootFlash component
// import "./index.css"; // Import Tailwind CSS
</script>

<template>
  <main class="min-h-screen py-8 px-4">
    <div class="container mx-auto max-w-4xl">
      <div class="text-center mb-6">
        <h1 class="text-3xl font-bold text-gray-800">RevyOS Fastboot Flash Tool</h1>
        <p class="text-gray-600 mt-2">Easily flash RevyOS to your device</p>
      </div>
      <!-- Integrate FastBootFlash component -->
      <FastBootFlash />
    </div>
  </main>
</template>

<style>
/* 全局样式覆盖 */
html, body {
  margin: 0;
  padding: 0;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
}

@import "tailwindcss";
</style>