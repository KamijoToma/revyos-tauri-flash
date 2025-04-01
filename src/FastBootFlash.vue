<template>
  <n-card class="card-shadow rounded-lg border border-gray-200" :bordered="false">
    <!-- Step Navigation -->
    <step-navigation
      :current-step="currentStep"
      :loading="isProcessing"
      :step-titles="[
        'Connect Stage 1',
        'Flash uboot',
        'Restart to Stage 2',
        'Connect Stage 2',
        'Flash Images',
        'Restart Device'
      ]"
      @prev="prevStep"
      @next="nextStep"
    />
    
    <!-- Step Content -->
    <div class="step-content">
      <!-- Step 1: Connect to Stage 1 -->
      <Step1Connect 
        v-if="currentStep === 1"
        :devices="usbDevices"
        :selectedDevice="selectedDevice"
        :loading="isProcessing"
        @select-device="selectUsbDevice"
        @refresh-devices="refreshUsbDevices"
        @connect="connectToDevice"
      />
      
      <!-- Step 2: Flash uboot -->
      <n-card v-else-if="currentStep === 2" title="Step 2: Flash uboot.bin to RAM" class="inner-card">
        <file-uploader
          file-type="bin"
          button-text="Select uboot.bin file"
          v-model:files="files.ubootBin"
          @error="handleError"
        />
        <n-button @click="flashUbootToRam" :disabled="!files.ubootBin.length || isProcessing"
                 :loading="isProcessing" type="primary" class="w-full mt-4">
          {{ isProcessing ? "Flashing..." : "Flash to RAM" }}
        </n-button>
      </n-card>
      
      <!-- Step 3: Restart to Stage 2 -->
      <n-card v-else-if="currentStep === 3" title="Step 3: Restart to Stage 2" class="inner-card">
        <div class="text-gray-700 mb-4">
          The device will restart to Stage 2 mode. Please make sure your device is ready.
        </div>
        <n-button @click="rebootToStage2" :loading="isProcessing" type="primary" class="w-full">
          {{ isProcessing ? "Restarting..." : "Restart" }}
        </n-button>
      </n-card>
      
      <!-- Step 4: Connect to Stage 2 -->
      <Step1Connect
        v-else-if="currentStep === 4"
        :devices="usbDevices"
        :selectedDevice="selectedDevice"
        :loading="isProcessing"
        @select-device="selectUsbDevice"
        @refresh-devices="refreshUsbDevices"
        @connect="connectToStage2"
      />
      
      <!-- Step 5: Flash Files -->
      <n-card v-else-if="currentStep === 5" title="Step 5: Flash Files to Device" class="inner-card">
        <div class="grid grid-cols-1 gap-4 mb-6">
          <file-uploader
            file-type="bin"
            button-text="Select uboot.bin"
            v-model:files="files.ubootBin"
            @error="handleError"
          />
          <file-uploader
            file-type="ext4"
            button-text="Select boot.ext4"
            v-model:files="files.bootExt4"
            @error="handleError"
          />
          <file-uploader
            file-type="ext4"
            button-text="Select root.ext4"
            v-model:files="files.rootExt4"
            @error="handleError"
          />
        </div>
        <n-button @click="flashFilesToDevice"
                 :disabled="!files.ubootBin.length || !files.bootExt4.length || !files.rootExt4.length || isProcessing"
                 :loading="isProcessing" type="primary" class="w-full">
          {{ isProcessing ? "Flashing..." : "Flash Files" }}
        </n-button>
      </n-card>
      
      <!-- Step 6: Complete -->
      <n-card v-else-if="currentStep === 6" title="Step 6: Restart Device" class="inner-card">
        <div class="text-green-600 mb-6 text-center">
          <div class="flex justify-center mb-4">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-16 w-16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </div>
          <h3 class="text-xl font-bold">Flashing process completed</h3>
          <p class="text-gray-600 mt-2">You can now safely restart your device</p>
        </div>
        <n-button @click="rebootDevice" :loading="isProcessing" type="primary" class="w-full">
          {{ isProcessing ? "Restarting..." : "Restart Device" }}
        </n-button>
      </n-card>
    </div>
    
    <!-- Status Display -->
    <status-display :status="status" />
  </n-card>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke, Channel } from "@tauri-apps/api/core";
import { NCard, NButton, type UploadFileInfo } from "naive-ui";

// 导入自定义组件
import StepNavigation from './components/fastboot/StepNavigation.vue';
import StatusDisplay from './components/fastboot/StatusDisplay.vue';
import FileUploader from './components/fastboot/FileUploader.vue';
import Step1Connect from './components/fastboot/steps/Step1Connect.vue';

interface USBDevice {
  vendor_id: string;
  product_id: string;
  product_string: string;
  device_address: string;
}

// 状态管理
const currentStep = ref(1);
const isProcessing = ref(false);
const status = ref("");
const files = ref<{ [key: string]: UploadFileInfo[] }>({
  ubootBin: [],
  bootExt4: [],
  rootExt4: [],
});

const usbDevices = ref<USBDevice[]>([]);
const selectedDevice = ref<USBDevice | null>(null);

type UploadProgressEvent = { 
  event: "progress",
  data: { current: number, total: number }
};

// 处理错误消息
function handleError(message: string) {
  status.value = message;
}

// USB设备处理方法
async function refreshUsbDevices() {
  isProcessing.value = true;
  status.value = "Refreshing USB device list...";
  try {
    const devices = await invoke<USBDevice[]>("list_usb_devices");
    usbDevices.value = devices;
    status.value = "USB device list refreshed.";
  } catch (error: any) {
    status.value = `Error: ${error.message}`;
  } finally {
    isProcessing.value = false;
  }
}

function selectUsbDevice(device: USBDevice) {
  selectedDevice.value = device;
}

async function connectToDevice() {
  if (!selectedDevice.value) return;
  isProcessing.value = true;
  status.value = `Connecting to ${selectedDevice.value.product_string}...`;
  try {
    const result = await invoke<string>("connect_to_device", { device: selectedDevice.value });
    status.value = result;
    if (result.includes("success")) {
      nextStep();
    }
  } catch (error: any) {
    status.value = `Error: ${error.message}`;
  } finally {
    isProcessing.value = false;
  }
}

// Flash操作方法
async function flashUbootToRam() {
  if (!files.value.ubootBin.length) return;
  isProcessing.value = true;
  status.value = "Flashing uboot.bin to RAM...";
  
  const onProgressEvent = new Channel<UploadProgressEvent>();
  onProgressEvent.onmessage = (event) => {
    const { current, total } = event.data;
    files.value.ubootBin[0].percentage = current / total * 100;
  };
  
  try {
    files.value.ubootBin[0].status = "uploading";
    const result = await invoke<string>("flash_to_partition", {
      filePath: files.value.ubootBin[0].fullPath,
      partition: "ram",
      device: selectedDevice.value,
      onEvent: onProgressEvent,
    });
    files.value.ubootBin[0].status = "finished";
    status.value = result;
    if (result.includes("success")) {
      nextStep();
    }
  } catch (error: any) {
    status.value = `Error: ${error}`;
    files.value.ubootBin[0].status = "error";
  } finally {
    isProcessing.value = false;
  }
}

async function rebootToStage2() {
  isProcessing.value = true;
  status.value = "Rebooting to stage 2...";
  try {
    const result = await invoke<string>("reboot_device", { device: selectedDevice.value });
    status.value = result;
    if (result.includes("success")) {
      nextStep();
    }
  } catch (error: any) {
    status.value = `Error: ${error.message}`;
  } finally {
    isProcessing.value = false;
  }
}

async function connectToStage2() {
  if (!selectedDevice.value) return;
  isProcessing.value = true;
  status.value = `Connecting to ${selectedDevice.value.product_string}...`;
  try {
    const result = await invoke<string>("connect_to_device", { device: selectedDevice.value });
    status.value = result;
    if (result.includes("success")) {
      nextStep();
    }
  } catch (error: any) {
    status.value = `Error: ${error.message}`;
  } finally {
    isProcessing.value = false;
  }
}

async function flashFilesToDevice() {
  if (!files.value.ubootBin.length || !files.value.bootExt4.length || !files.value.rootExt4.length) return;
  isProcessing.value = true;
  status.value = "Flashing files to device...";
  
  const onProgressEvent = new Channel<UploadProgressEvent>();
  onProgressEvent.onmessage = (event) => {
    const { current, total } = event.data;
    const percentage = current / total * 100;

    if (files.value.ubootBin[0]?.status === "uploading") {
      files.value.ubootBin[0].percentage = percentage;
    } else if (files.value.bootExt4[0]?.status === "uploading") {
      files.value.bootExt4[0].percentage = percentage;
    } else if (files.value.rootExt4[0]?.status === "uploading") {
      files.value.rootExt4[0].percentage = percentage;
    }
  };

  try {
    // Flash uboot
    files.value.ubootBin[0].status = "uploading";
    await invoke<string>("flash_to_partition", {
      filePath: files.value.ubootBin[0].fullPath,
      partition: "uboot",
      device: selectedDevice.value,
      onEvent: onProgressEvent,
    });
    files.value.ubootBin[0].status = "finished";

    // Flash boot
    files.value.bootExt4[0].status = "uploading";
    await invoke<string>("flash_to_partition", {
      filePath: files.value.bootExt4[0].fullPath,
      partition: "boot",
      device: selectedDevice.value,
      onEvent: onProgressEvent,
    });
    files.value.bootExt4[0].status = "finished";

    // Flash root
    files.value.rootExt4[0].status = "uploading";
    await invoke<string>("flash_to_partition", {
      filePath: files.value.rootExt4[0].fullPath,
      partition: "root",
      device: selectedDevice.value,
      onEvent: onProgressEvent,
    });
    files.value.rootExt4[0].status = "finished";

    status.value = "All files flashed successfully.";
    nextStep();
  } catch (error: any) {
    if (files.value.ubootBin[0]?.status === "uploading") {
      files.value.ubootBin[0].status = "error";
      status.value = `Error flashing uboot.bin: ${error.message}`;
    } else if (files.value.bootExt4[0]?.status === "uploading") {
      files.value.bootExt4[0].status = "error";
      status.value = `Error flashing boot.ext4: ${error.message}`;
    } else if (files.value.rootExt4[0]?.status === "uploading") {
      files.value.rootExt4[0].status = "error";
      status.value = `Error flashing root.ext4: ${error.message}`;
    }
  } finally {
    isProcessing.value = false;
  }
}

async function rebootDevice() {
  isProcessing.value = true;
  status.value = "Rebooting the device...";
  try {
    const result = await invoke<string>("reboot_device", { device: selectedDevice.value });
    status.value = result;
  } catch (error: any) {
    status.value = `Error: ${error.message}`;
  } finally {
    isProcessing.value = false;
  }
}

// 步骤导航
function nextStep() {
  if (currentStep.value < 6) currentStep.value++;
}

function prevStep() {
  if (currentStep.value > 1) currentStep.value--;
}

// 监听步骤变化，自动刷新USB设备
watch(currentStep, async (newStep) => {
  if (newStep === 1 || newStep === 4) {
    await refreshUsbDevices();
  }
});

// 组件加载时刷新USB设备
refreshUsbDevices();
</script>

<style scoped>
.inner-card {
  margin-bottom: 1rem;
  border-radius: 0.5rem;
  background-color: white;
}

.step-content {
  animation: fadeIn 0.5s ease-in-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
