<template>
    <n-card title="RevyOS Fastboot Flash Utility" class="fastboot-flash">
        <n-scrollbar class="steps-container" x-scrollable>
            <n-steps :current="currentStep" size="small" class="steps">
                <n-step title="Connect Stage 1" />
                <n-step title="Flash uboot" />
                <n-step title="Reboot to Stage 2" />
                <n-step title="Connect Stage 2" />
                <n-step title="Flash images" />
                <n-step title="Reboot" />
            </n-steps>
        </n-scrollbar>
        <div v-if="currentStep === 1">
            <n-card title="Step 1: Connect to Stage 1 USB Device">
                <n-scrollbar style="max-height: 40vh">
                    <n-list v-if="usbDevices.length" hoverable clickable bordered>
                        <n-list-item v-for="(device, index) in usbDevices" :key="index"
                            :class="{ selected: selectedDevice?.product_string === device.product_string }" @click="selectUsbDevice(device)">
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
                                        <n-tag v-if="('0x' + Number(device.vendor_id).toString(16)) === '0x2345'" :bordered="false" type="warning" size="small">
                                            C920 stage 1 gadget
                                        </n-tag>
                                    </n-space>
                                </template>
                            </n-thing>
                        </n-list-item>
                    </n-list>
                    <n-alert v-else type="info">No USB devices found.</n-alert>
                </n-scrollbar>
                <n-button @click="refreshUsbDevices" :loading="isProcessing" type="default">
                    {{ isProcessing ? "Refreshing..." : "Refresh Device List" }}
                </n-button>
                <n-button @click="connectToDevice" :disabled="!selectedDevice || isProcessing" :loading="isProcessing"
                    type="primary">
                    {{ isProcessing ? "Connecting..." : "Connect" }}
                </n-button>
            </n-card>
        </div>
        <div v-else-if="currentStep === 2">
            <n-card title="Step 2: Flash uboot.bin to RAM">
                <n-upload v-model:file-list="files.ubootBin" :max="1" accept=".bin" abstract>
                    <n-button type="default" @click="selectFile('ubootBin')">Select uboot.bin</n-button>
                    <n-upload-file-list />
                </n-upload>
                <n-button @click="flashUbootToRam" :disabled="!files.ubootBin.length || isProcessing"
                    :loading="isProcessing" type="primary">
                    {{ isProcessing ? "Flashing..." : "Flash to RAM" }}
                </n-button>
            </n-card>
        </div>
        <div v-else-if="currentStep === 3">
            <n-card title="Step 3: Reboot to Stage 2">
                <n-button @click="rebootToStage2" :loading="isProcessing" type="primary">
                    {{ isProcessing ? "Rebooting..." : "Reboot" }}
                </n-button>
            </n-card>
        </div>
        <div v-else-if="currentStep === 4">
            <n-card title="Step 4: Connect to Stage 2 USB Device">
                <n-scrollbar style="max-height: 40vh">
                    <n-list v-if="usbDevices.length" hoverable clickable bordered>
                        <n-list-item v-for="(device, index) in usbDevices" :key="index"
                            :class="{ selected: selectedDevice?.product_string === device.product_string }" @click="selectUsbDevice(device)">
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
                                        <n-tag v-if="('0x' + Number(device.vendor_id).toString(16)) === '0x1234'" :bordered="false" type="warning" size="small">
                                            C920 stage 2 gadget
                                        </n-tag>
                                    </n-space>
                                </template>
                                Click to select this device.
                            </n-thing>
                        </n-list-item>
                    </n-list>
                    <n-alert v-else type="info">No USB devices found.</n-alert>
                </n-scrollbar>
                <n-button @click="refreshUsbDevices" :loading="isProcessing" type="default">
                    {{ isProcessing ? "Refreshing..." : "Refresh Device List" }}
                </n-button>
                <n-button @click="connectToStage2" :disabled="!selectedDevice || isProcessing" :loading="isProcessing"
                    type="primary">
                    {{ isProcessing ? "Connecting..." : "Connect" }}
                </n-button>
            </n-card>
        </div>
        <div v-else-if="currentStep === 5">
            <n-card title="Step 5: Flash Files to Device">
                <n-upload v-model:file-list="files.ubootBin" :max="1" abstract>
                    <n-button @click="selectFile('ubootBin')" type="default">Select uboot.bin</n-button>
                    <n-upload-file-list />
                </n-upload>
                <n-upload v-model:file-list="files.bootExt4" :max="1" abstract>
                    <n-button @click="selectFile('bootExt4')" type="default">Select boot.ext4</n-button>
                    <n-upload-file-list />
                </n-upload>
                <n-upload v-model:file-list="files.rootExt4" :max="1" abstract>
                    <n-button @click="selectFile('rootExt4')" type="default">Select root.ext4</n-button>
                    <n-upload-file-list />
                </n-upload>
                <n-button @click="flashFilesToDevice"
                    :disabled="!files.ubootBin || !files.bootExt4 || !files.rootExt4 || isProcessing"
                    :loading="isProcessing" type="primary">
                    {{ isProcessing ? "Flashing..." : "Flash Files" }}
                </n-button>
            </n-card>
        </div>
        <div v-else-if="currentStep === 6">
            <n-card title="Step 6: Reboot Device">
                <n-button @click="rebootDevice" :loading="isProcessing" type="primary">
                    {{ isProcessing ? "Rebooting..." : "Reboot" }}
                </n-button>
            </n-card>
        </div>
        <n-alert v-if="status" type="info" class="status">{{ status }}</n-alert>
        <div class="navigation">
            <n-button @click="prevStep" :disabled="currentStep === 1 || isProcessing">Previous</n-button>
            <n-button @click="nextStep" :disabled="currentStep === 6 || isProcessing">Next</n-button>
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
    width: 100%; /* Take full width of the container */
    max-width: 100vw; /* Limit the maximum width */
    margin: 20px auto;
    box-sizing: border-box; /* Include padding and border in size */
}

.steps-container {
    overflow-x: auto;
    padding: 0 20px;
}

.steps {
    margin: 10px 5px;
    /* min-width: 600px; Adjust as needed */
}

.status {
    margin-top: 20px;
}

.navigation {
    margin-top: 20px;
    display: flex;
    justify-content: space-between;
}

.selected {
    background-color: #e6f7ff;
    cursor: pointer;
}
</style>
