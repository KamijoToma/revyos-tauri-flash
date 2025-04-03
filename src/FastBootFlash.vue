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
      <Step2FlashUboot
        v-else-if="currentStep === 2"
        v-model:files="files.ubootBin"
        :loading="isProcessing"
        :selected-image-variant="selectedImageVariant"
        :image-flash-progress="step2FlashProgress"
        @update:selectedImageVariant="selectedImageVariant = $event"
        @flash="flashUbootToRam"
        @error="handleError"
      />
      
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
      <Step5FlashFiles
        v-else-if="currentStep === 5"
        v-model:fileCollection="files"
        :loading="isProcessing"
        :selected-image-variant="selectedImageVariant"
        :image-flash-progress="step5FlashProgress"
        @update:selectedImageVariant="selectedImageVariant = $event"
        @flash="flashFilesToDevice"
        @error="handleError"
      />
      
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
import Step1Connect from './components/fastboot/steps/Step1Connect.vue';
import Step2FlashUboot from './components/fastboot/steps/Step2FlashUboot.vue';
import Step5FlashFiles from './components/fastboot/steps/Step5FlashFiles.vue';
import { type ImageVariant } from './components/ImageSelector.vue';

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
import type { FileCollection, ImageFlashProgressInfo } from './components/fastboot/steps/Step5FlashFiles.vue';

const files = ref<FileCollection>({
  ubootBin: [],
  bootExt4: [],
  rootExt4: [],
});

// 添加进度跟踪状态
const step2FlashProgress = ref(0); // 简单的百分比进度
const step5FlashProgress = ref<ImageFlashProgressInfo>({
  currentStep: '',
  percentage: 0,
  currentFile: 0,
  totalFiles: 3 // uboot, boot, root 这三个文件
});

const usbDevices = ref<USBDevice[]>([]);
const selectedDevice = ref<USBDevice | null>(null);
const selectedImageVariant = ref<ImageVariant | null>(null);

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
  isProcessing.value = true;
  status.value = "Flashing uboot.bin to RAM...";
  
  const onProgressEvent = new Channel<UploadProgressEvent>();
  onProgressEvent.onmessage = (event) => {
    const { current, total } = event.data;
    if (files.value.ubootBin[0]) {
      files.value.ubootBin[0].percentage = parseFloat(((current / total) * 100).toFixed(1));
    }
    // 更新在线镜像刷入进度，限制为一位小数
    step2FlashProgress.value = parseFloat(((current / total) * 100).toFixed(1));
  };
  
  try {
    let filePath = '';
    
    // 检查是使用本地文件还是在线镜像
    if (files.value.ubootBin.length) {
      // 本地文件模式
      filePath = files.value.ubootBin[0].fullPath || '';
      files.value.ubootBin[0].status = "uploading";
    } else if (selectedImageVariant.value) {
      // 在线镜像模式
      const ubootBinary = selectedImageVariant.value.image_binarys.find(
        binary => binary.binary_type === 'UBoot' && binary.local_path
      );
      
      if (ubootBinary?.local_path) {
        filePath = ubootBinary.local_path;
        status.value = `Flashing ${ubootBinary.name} to RAM...`;
      } else {
        throw new Error("No uboot binary found in selected image variant");
      }
    } else {
      throw new Error("No file selected");
    }
    
    // 重置进度
    step2FlashProgress.value = 0;
    
    const result = await invoke<string>("flash_to_partition", {
      filePath,
      partition: "ram",
      device: selectedDevice.value,
      onEvent: onProgressEvent,
    });
    
    if (files.value.ubootBin.length) {
      files.value.ubootBin[0].status = "finished";
    }
    
    status.value = result;
    if (result.includes("success")) {
      nextStep();
    }
  } catch (error: any) {
    status.value = `Error: ${error}`;
    if (files.value.ubootBin.length) {
      files.value.ubootBin[0].status = "error";
    }
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
  isProcessing.value = true;
  status.value = "Flashing files to device...";
  
  // 重置进度信息
  step5FlashProgress.value = {
    currentStep: '',
    percentage: 0,
    currentFile: 0,
    totalFiles: 3 // uboot, boot, root
  };
  
  const onProgressEvent = new Channel<UploadProgressEvent>();
  onProgressEvent.onmessage = (event) => {
    const { current, total } = event.data;
    const percentage = parseFloat(((current / total) * 100).toFixed(1));

    // 更新本地文件进度
    if (files.value.ubootBin[0]?.status === "uploading") {
      files.value.ubootBin[0].percentage = percentage;
    } else if (files.value.bootExt4[0]?.status === "uploading") {
      files.value.bootExt4[0].percentage = percentage;
    } else if (files.value.rootExt4[0]?.status === "uploading") {
      files.value.rootExt4[0].percentage = percentage;
    }
    
    // 更新在线镜像进度，限制为一位小数
    step5FlashProgress.value.percentage = percentage;
  };

  try {
    // 决定使用本地文件还是在线镜像
    if (files.value.ubootBin.length && files.value.bootExt4.length && files.value.rootExt4.length) {
      // 使用本地文件
      // Flash uboot
      files.value.ubootBin[0].status = "uploading";
      step5FlashProgress.value.currentStep = "Flashing uboot.bin";
      step5FlashProgress.value.currentFile = 1;
      await invoke<string>("flash_to_partition", {
        filePath: files.value.ubootBin[0].fullPath,
        partition: "uboot",
        device: selectedDevice.value,
        onEvent: onProgressEvent,
      });
      files.value.ubootBin[0].status = "finished";

      // Flash boot
      files.value.bootExt4[0].status = "uploading";
      step5FlashProgress.value.currentStep = "Flashing boot.ext4";
      step5FlashProgress.value.currentFile = 2;
      step5FlashProgress.value.percentage = 0; // 重置进度
      await invoke<string>("flash_to_partition", {
        filePath: files.value.bootExt4[0].fullPath,
        partition: "boot",
        device: selectedDevice.value,
        onEvent: onProgressEvent,
      });
      files.value.bootExt4[0].status = "finished";

      // Flash root
      files.value.rootExt4[0].status = "uploading";
      step5FlashProgress.value.currentStep = "Flashing root.ext4";
      step5FlashProgress.value.currentFile = 3;
      step5FlashProgress.value.percentage = 0; // 重置进度
      await invoke<string>("flash_to_partition", {
        filePath: files.value.rootExt4[0].fullPath,
        partition: "root",
        device: selectedDevice.value,
        onEvent: onProgressEvent,
      });
      files.value.rootExt4[0].status = "finished";
    } else if (selectedImageVariant.value) {
      // 使用在线镜像
      const ubootBinary = selectedImageVariant.value.image_binarys.find(
        binary => binary.binary_type === 'UBoot' && binary.local_path
      );
      const bootBinary = selectedImageVariant.value.image_binarys.find(
        binary => binary.binary_type === 'Boot' && binary.local_path
      );
      const rootBinary = selectedImageVariant.value.image_binarys.find(
        binary => binary.binary_type === 'Root' && binary.local_path
      );

      if (!ubootBinary?.local_path || !bootBinary?.local_path || !rootBinary?.local_path) {
        throw new Error("Missing required binaries in selected image variant");
      }

      // Flash uboot
      step5FlashProgress.value.currentStep = `Flashing ${ubootBinary.name}`;
      step5FlashProgress.value.currentFile = 1;
      status.value = `Flashing ${ubootBinary.name} to uboot partition...`;
      await invoke<string>("flash_to_partition", {
        filePath: ubootBinary.local_path,
        partition: "uboot",
        device: selectedDevice.value,
        onEvent: onProgressEvent,
      });

      // Flash boot
      step5FlashProgress.value.currentStep = `Flashing ${bootBinary.name}`;
      step5FlashProgress.value.currentFile = 2;
      step5FlashProgress.value.percentage = 0; // 重置进度
      status.value = `Flashing ${bootBinary.name} to boot partition...`;
      await invoke<string>("flash_to_partition", {
        filePath: bootBinary.local_path,
        partition: "boot",
        device: selectedDevice.value,
        onEvent: onProgressEvent,
      });

      // Flash root
      step5FlashProgress.value.currentStep = `Flashing ${rootBinary.name}`;
      step5FlashProgress.value.currentFile = 3;
      step5FlashProgress.value.percentage = 0; // 重置进度
      status.value = `Flashing ${rootBinary.name} to root partition...`;
      await invoke<string>("flash_to_partition", {
        filePath: rootBinary.local_path,
        partition: "root",
        device: selectedDevice.value,
        onEvent: onProgressEvent,
      });
    } else {
      throw new Error("No files selected for flashing");
    }

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
    } else {
      status.value = `Error flashing files: ${error.message}`;
    }
  } finally {
    isProcessing.value = false;
    // 完成后清空进度信息
    step5FlashProgress.value.currentStep = '';
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
