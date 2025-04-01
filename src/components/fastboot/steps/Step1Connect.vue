<template>
  <n-card title="Step 1: Connect to Stage 1 USB Device" class="inner-card">
    <USBDeviceList
      :devices="devices"
      :selectedDevice="selectedDevice"
      :loading="loading"
      @select="selectDevice"
      @refresh="refreshDevices"
      @connect="connectToDevice"
    />
  </n-card>
</template>

<script setup lang="ts">
import { defineProps, defineEmits } from 'vue';
import { NCard } from 'naive-ui';
import USBDeviceList from '../USBDeviceList.vue';

interface USBDevice {
  vendor_id: string;
  product_id: string;
  product_string: string;
  device_address: string;
}

const props = defineProps<{
  devices: USBDevice[];
  selectedDevice: USBDevice | null;
  loading: boolean;
}>();

const emit = defineEmits<{
  'select-device': [device: USBDevice];
  'refresh-devices': [];
  'connect': [];
}>();

function selectDevice(device: USBDevice) {
  emit('select-device', device);
}

function refreshDevices() {
  emit('refresh-devices');
}

function connectToDevice() {
  emit('connect');
}
</script>

<style scoped>
.inner-card {
  margin-bottom: 1rem;
  border-radius: 0.5rem;
  background-color: white;
}
</style>
