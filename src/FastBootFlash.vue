<template>
    <n-card class="card-shadow rounded-lg border border-gray-200" :bordered="false">
        <n-scrollbar class="steps-container mb-6" x-scrollable>
            <n-steps :current="currentStep" size="small" class="steps">
                <n-step title="Connect Stage 1" />
                <n-step title="Flash uboot" />
                <n-step title="Restart to Stage 2" />
                <n-step title="Connect Stage 2" />
                <n-step title="Flash Images" />
                <n-step title="Restart Device" />
            </n-steps>
        </n-scrollbar>
        
        <!-- Step 1 -->
        <div v-if="currentStep === 1" class="step-content">
            <n-card title="Step 1: Connect to Stage 1 USB Device" class="inner-card">
                <n-scrollbar style="max-height: 40vh" class="mb-4">
                    <n-list v-if="usbDevices.length" hoverable clickable bordered>
                        <n-list-item v-for="(device, index) in usbDevices" :key="index"
                            :class="{ 'bg-blue-50 border-l-4 border-blue-500': selectedDevice?.product_string === device.product_string, 
                                      'hover:bg-gray-50': selectedDevice?.product_string !== device.product_string }" 
                            class="transition-colors duration-200"
                            @click="selectUsbDevice(device)">
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
                                        <n-tag v-if="('0x' + Number(device.vendor_id).toString(16)) === '0x2345'" :bordered="false" 
                                              type="warning" size="small">
                                            C920 stage 1 gadget
                                        </n-tag>
                                    </n-space>
                                </template>
                            </n-thing>
                        </n-list-item>
                    </n-list>
                    <n-alert v-else type="info">No USB devices found</n-alert>
                </n-scrollbar>
                <div class="flex gap-4 mt-4">
                    <n-button @click="refreshUsbDevices" :loading="isProcessing" 
                             class="bg-gray-100 hover:bg-gray-200 transition-colors">
                        {{ isProcessing ? "Refreshing..." : "Refresh Device List" }}
                    </n-button>
                    <n-button @click="connectToDevice" :disabled="!selectedDevice || isProcessing" :loading="isProcessing"
                             type="primary" class="flex-grow">
                        {{ isProcessing ? "Connecting..." : "Connect" }}
                    </n-button>
                </div>
            </n-card>
        </div>
        
        <!-- Step 2 -->
        <div v-else-if="currentStep === 2" class="step-content">
            <n-card title="Step 2: Flash uboot.bin to RAM" class="inner-card">
                <n-upload v-model:file-list="files.ubootBin" :max="1" accept=".bin" abstract>
                    <n-button type="default" @click="selectFile('ubootBin')" 
                             class="bg-gray-100 hover:bg-gray-200 mb-4 transition-colors">
                        Select uboot.bin file
                    </n-button>
                    <n-upload-file-list />
                </n-upload>
                <n-button @click="flashUbootToRam" :disabled="!files.ubootBin.length || isProcessing"
                         :loading="isProcessing" type="primary" class="w-full mt-4">
                    {{ isProcessing ? "Flashing..." : "Flash to RAM" }}
                </n-button>
            </n-card>
        </div>
        
        <!-- Step 3 -->
        <div v-else-if="currentStep === 3" class="step-content">
            <n-card title="Step 3: Restart to Stage 2" class="inner-card">
                <div class="text-gray-700 mb-4">
                    The device will restart to Stage 2 mode. Please make sure your device is ready.
                </div>
                <n-button @click="rebootToStage2" :loading="isProcessing" type="primary" class="w-full">
                    {{ isProcessing ? "Restarting..." : "Restart" }}
                </n-button>
            </n-card>
        </div>
        
        <!-- Step 4 -->
        <div v-else-if="currentStep === 4" class="step-content">
            <n-card title="Step 4: Connect to Stage 2 USB Device" class="inner-card">
                <n-scrollbar style="max-height: 40vh" class="mb-4">
                    <n-list v-if="usbDevices.length" hoverable clickable bordered>
                        <n-list-item v-for="(device, index) in usbDevices" :key="index"
                            :class="{ 'bg-blue-50 border-l-4 border-blue-500': selectedDevice?.product_string === device.product_string, 
                                     'hover:bg-gray-50': selectedDevice?.product_string !== device.product_string }"
                            class="transition-colors duration-200" 
                            @click="selectUsbDevice(device)">
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
                                        <n-tag v-if="('0x' + Number(device.vendor_id).toString(16)) === '0x1234'" :bordered="false" 
                                              type="success" size="small">
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
                    <n-button @click="refreshUsbDevices" :loading="isProcessing" 
                             class="bg-gray-100 hover:bg-gray-200 transition-colors">
                        {{ isProcessing ? "Refreshing..." : "Refresh Device List" }}
                    </n-button>
                    <n-button @click="connectToStage2" :disabled="!selectedDevice || isProcessing" :loading="isProcessing"
                             type="primary" class="flex-grow">
                        {{ isProcessing ? "Connecting..." : "Connect" }}
                    </n-button>
                </div>
            </n-card>
        </div>
        
        <!-- Step 5 -->
        <div v-else-if="currentStep === 5" class="step-content">
            <n-card title="Step 5: Flash Files to Device" class="inner-card">
                <div class="grid grid-cols-1 gap-4 mb-6">
                    <div class="file-upload-item">
                        <n-upload v-model:file-list="files.ubootBin" :max="1" abstract>
                            <n-button @click="selectFile('ubootBin')" type="default" 
                                     class="bg-gray-100 hover:bg-gray-200 transition-colors mb-2">
                                Select uboot.bin
                            </n-button>
                            <n-upload-file-list />
                        </n-upload>
                    </div>
                    <div class="file-upload-item">
                        <n-upload v-model:file-list="files.bootExt4" :max="1" abstract>
                            <n-button @click="selectFile('bootExt4')" type="default" 
                                     class="bg-gray-100 hover:bg-gray-200 transition-colors mb-2">
                                Select boot.ext4
                            </n-button>
                            <n-upload-file-list />
                        </n-upload>
                    </div>
                    <div class="file-upload-item">
                        <n-upload v-model:file-list="files.rootExt4" :max="1" abstract>
                            <n-button @click="selectFile('rootExt4')" type="default" 
                                     class="bg-gray-100 hover:bg-gray-200 transition-colors mb-2">
                                Select root.ext4
                            </n-button>
                            <n-upload-file-list />
                        </n-upload>
                    </div>
                </div>
                <n-button @click="flashFilesToDevice"
                         :disabled="!files.ubootBin.length || !files.bootExt4.length || !files.rootExt4.length || isProcessing"
                         :loading="isProcessing" type="primary" class="w-full">
                    {{ isProcessing ? "Flashing..." : "Flash Files" }}
                </n-button>
            </n-card>
        </div>
        
        <!-- Step 6 -->
        <div v-else-if="currentStep === 6" class="step-content">
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
                <n-button @click="rebootDevice" :loading="isProcessing" type="primary" class="w-full">
                    {{ isProcessing ? "Restarting..." : "Restart Device" }}
                </n-button>
            </n-card>
        </div>
        
        <n-alert v-if="status" type="info" class="status my-4 bg-blue-50">{{ status }}</n-alert>
        
        <div class="navigation flex justify-between mt-6">
            <n-button @click="prevStep" :disabled="currentStep === 1 || isProcessing" 
                     class="min-w-[100px]">
                Previous
            </n-button>
            <div class="step-indicators flex-1 mx-4 flex items-center justify-center">
                <div v-for="step in 6" :key="step" 
                     :class="['step-dot', currentStep === step ? 'bg-primary' : 'bg-gray-300']">
                </div>
            </div>
            <n-button @click="nextStep" :disabled="currentStep === 6 || isProcessing" 
                     type="primary" class="min-w-[100px]">
                Next
            </n-button>
        </div>
    </n-card>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke, Channel } from "@tauri-apps/api/core";
import { NCard, NSteps, NStep, NButton, NUpload, NUploadFileList, NAlert, NScrollbar, NList, NListItem, NThing, NSpace, NTag, type UploadFileInfo } from "naive-ui";

interface USBDevice {
    vendor_id: string;
    product_id: string;
    product_string: string;
    device_address: string;
}

const currentStep = ref(1);
const isProcessing = ref(false);
const status = ref("");
const files = ref<{ [key: string]: UploadFileInfo[] }>({
    ubootBin: [],
    bootExt4: [],
    rootExt4: [],
});

const usbDevices = ref<USBDevice[]>([]);
const selectedDevice = ref<USBDevice | null>(null); // Store the entire USBDevice object

type UploadProgressEvent = { 
    event: "progress",
    data: { current: number, total: number }
 };

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
    selectedDevice.value = device; // Store the selected device object
}

async function connectToDevice() {
    if (!selectedDevice.value) return;
    isProcessing.value = true;
    status.value = `Connecting to ${selectedDevice.value.product_string}...`;
    try {
        const result = await invoke<string>("connect_to_device", { device: selectedDevice.value });
        status.value = result;
    } catch (error: any) {
        status.value = `Error: ${error.message}`;
    } finally {
        isProcessing.value = false;
    }
}

async function flashUbootToRam() {
    if (!files.value.ubootBin.length) return;
    isProcessing.value = true;
    status.value = "Flashing uboot.bin to RAM...";
    // Register upload event listener
    const onProgressEvent = new Channel<UploadProgressEvent>();
    onProgressEvent.onmessage = (event) => {
        const { current, total } = event.data;
        files.value.ubootBin[0].percentage = current / total * 100;
    };
    try {
        files.value.ubootBin[0].status = "uploading";
        const result = await invoke<string>("flash_to_partition", {
            filePath: files.value.ubootBin[0].fullPath, // Use the full file path
            partition: "ram", // Flash to RAM
            device: selectedDevice.value,
            onEvent: onProgressEvent,
        });
        files.value.ubootBin[0].status = "finished";
        status.value = result;
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
        files.value.ubootBin[0].status = "uploading";
        await invoke<string>("flash_to_partition", {
            filePath: files.value.ubootBin[0].fullPath,
            partition: "uboot",
            device: selectedDevice.value,
            onEvent: onProgressEvent,
        });
        files.value.ubootBin[0].status = "finished";

        files.value.bootExt4[0].status = "uploading";
        await invoke<string>("flash_to_partition", {
            filePath: files.value.bootExt4[0].fullPath,
            partition: "boot",
            device: selectedDevice.value,
            onEvent: onProgressEvent,
        });
        files.value.bootExt4[0].status = "finished";

        files.value.rootExt4[0].status = "uploading";
        await invoke<string>("flash_to_partition", {
            filePath: files.value.rootExt4[0].fullPath,
            partition: "root",
            device: selectedDevice.value,
            onEvent: onProgressEvent,
        });
        files.value.rootExt4[0].status = "finished";

        status.value = "All files flashed successfully.";
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

async function selectFile(key: string) {
    try {
        const filePath = await invoke<string>("select_file");
        if (filePath) {
            files.value[key] = [{
                id: Date.now().toString(), // Generate a unique ID
                name: filePath.split(/[/\\]/).pop() || "",
                fullPath: filePath,
                status: "finished" // Set a default status
            }];
        }
    } catch (error: any) {
        status.value = `Error: ${error.message}`;
    }
}

// Automically refresh USB devices when meeting certain steps
watch(currentStep, async (newStep) => {
    if (newStep === 1 || newStep === 4) {
        await refreshUsbDevices();
    }
});
// Also refresh USB devices when the component is mounted
refreshUsbDevices();

function nextStep() {
    if (currentStep.value < 6) currentStep.value++;
}

function prevStep() {
    if (currentStep.value > 1) currentStep.value--;
}
</script>
<style scoped>
.fastboot-flash {
    width: 100%;
    max-width: 100%;
    margin: 0 auto;
    box-sizing: border-box;
}

.steps-container {
    overflow-x: auto;
    padding: 0 10px;
}

.steps {
    margin: 10px 0;
}

.inner-card {
    margin-bottom: 1rem;
    border-radius: 0.5rem;
    background-color: white;
}

.status {
    border-left: 4px solid #2080F0;
}

.step-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    margin: 0 8px;
    transition: background-color 0.3s;
}

.step-content {
    animation: fadeIn 0.5s ease-in-out;
}

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

@keyframes fadeIn {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
}

.selected {
    background-color: #e6f7ff;
    border-left: 4px solid #2080F0;
    cursor: pointer;
}
</style>
