<template>
  <div>
    <n-scrollbar style="max-height: 40vh" class="mb-4">
      <n-list v-if="devices.length" hoverable clickable bordered>
        <n-list-item v-for="(device, index) in devices" :key="index"
          :class="{ 'bg-blue-50 border-l-4 border-blue-500': selectedDevice?.product_string === device.product_string, 
                    'hover:bg-gray-50': selectedDevice?.product_string !== device.product_string }" 
          class="transition-colors duration-200"
          @click="selectDevice(device)">
          <n-thing :title="device.product_string" content-style="margin-top: 10px;">
            <template #description>
              <n-space size="small" style="margin-top: 4px">
                <n-tag :bordered="false" type="info" size="small">
                  Vendor ID: {{ "0x" + Number(device.vendor_id).toString(16).toUpperCase() }}
                </n-tag>
                <n-tag :bordered="false" type="info" size="small">
                  Product ID: {{ "0x" + Number(device.product_id).toString(16).toUpperCase() }}
                </n-tag>
                <n-tag :bordered="false" type="info" size="small">
                  Address: {{ device.device_address }}
                </n-tag>
                <n-tag v-if="isStage1Device(device)" :bordered="false" type="warning" size="small">
                  C920 stage 1 gadget
                </n-tag>
                <n-tag v-if="isStage2Device(device)" :bordered="false" type="success" size="small">
                  C920 stage 2 gadget
                </n-tag>
              </n-space>
            </template>
            <div class="text-sm text-gray-500 mt-2">Click to select this device</div>
          </n-thing>
        </n-list-item>
      </n-list>
      <n-alert v-else type="info">No USB devices found</n-alert>
    </n-scrollbar>
    <div class="flex gap-4 mt-4">
      <n-button @click="refreshDevices" :loading="loading" 
               class="bg-gray-100 hover:bg-gray-200 transition-colors">
        {{ loading ? "Refreshing..." : "Refresh Device List" }}
      </n-button>
      <n-button @click="connectToSelected" :disabled="!selectedDevice || loading" :loading="loading"
               type="primary" class="flex-grow">
        {{ loading ? "Connecting..." : "Connect" }}
      </n-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, defineProps, defineEmits } from 'vue';
import { NList, NListItem, NThing, NSpace, NTag, NAlert, NScrollbar, NButton } from 'naive-ui';

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
  'select': [device: USBDevice];
  'refresh': [];
  'connect': [];
}>();

function selectDevice(device: USBDevice) {
  emit('select', device);
}

function refreshDevices() {
  emit('refresh');
}

function connectToSelected() {
  emit('connect');
}

function isStage1Device(device: USBDevice): boolean {
  return '0x' + Number(device.vendor_id).toString(16) === '0x2345';
}

function isStage2Device(device: USBDevice): boolean {
  return '0x' + Number(device.vendor_id).toString(16) === '0x1234';
}
</script>
