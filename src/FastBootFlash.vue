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
                <n-button @click="connectToDevice" :loading="isProcessing" type="primary">
                    {{ isProcessing ? "Connecting..." : "Connect" }}
                </n-button>
            </n-card>
        </div>
        <div v-else-if="currentStep === 2">
            <n-card title="Step 2: Flash uboot.bin to RAM">
                <n-upload v-model:file-list="files.ubootBin" :max="1" accept=".bin" @change="onFileSelected('ubootBin', $event)">
                    <n-button type="default">Select uboot.bin</n-button>
                </n-upload>
                <n-button @click="flashUbootToRam" :disabled="!files.ubootBin || isProcessing" :loading="isProcessing" type="primary">
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
                <n-button @click="connectToStage2" :loading="isProcessing" type="primary">
                    {{ isProcessing ? "Waiting..." : "Connect" }}
                </n-button>
            </n-card>
        </div>
        <div v-else-if="currentStep === 5">
            <n-card title="Step 5: Flash Files to Device">
                <n-upload v-model:file-list="files.ubootBin" :max="1" accept=".bin" @change="onFileSelected('ubootBin', $event)">
                    <n-button type="default">Select uboot.bin</n-button>
                </n-upload>
                <n-upload v-model:file-list="files.bootExt4" :max="1" accept=".ext4" @change="onFileSelected('bootExt4', $event)">
                    <n-button type="default">Select boot.ext4</n-button>
                </n-upload>
                <n-upload v-model:file-list="files.rootExt4" :max="1" accept=".ext4" @change="onFileSelected('rootExt4', $event)">
                    <n-button type="default">Select root.ext4</n-button>
                </n-upload>
                <n-button @click="flashFilesToDevice" :disabled="!files.ubootBin || !files.bootExt4 || !files.rootExt4 || isProcessing" :loading="isProcessing" type="primary">
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
import { ref } from "vue";
import * as fastboot from "android-fastboot-ts";
import { NCard, NSteps, NStep, NButton, NUpload, NAlert, NScrollbar, type UploadFileInfo } from "naive-ui";

const currentStep = ref(1);
const isProcessing = ref(false);
const status = ref("");
const files = ref<{ [key: string]: UploadFileInfo[] }>({
    ubootBin: [],
    bootExt4: [],
    rootExt4: [],
});
const device = new fastboot.FastbootDevice();

// Enable verbose debug logging
fastboot.setDebugLevel(2);

function onFileSelected(key: string, event: { file: UploadFileInfo }) {
    const { file } = event;
    files.value[key] = [
        {
            id: Date.now().toString(),
            name: file.name,
            status: 'finished',
            file: file.file as File,
        },
    ];
}

async function connectToDevice() {
    isProcessing.value = true;
    status.value = "Connecting to stage 1 USB device...";
    try {
        await device.connect();
        const product = await device.getVariable("product");
        const serial = await device.getVariable("serialno");
        status.value = `Connected to ${product} (serial: ${serial})`;
    } catch (error: any) {
        status.value = `Error: ${error.message}`;
    } finally {
        isProcessing.value = false;
    }
}

async function flashUbootToRam() {
    if (!files.value.ubootBin) return;
    isProcessing.value = true;
    status.value = "Flashing uboot.bin to RAM...";
    try {
        await device.flashBlob("ram", files.value.ubootBin[0].file as Blob);
        status.value = "Flashed uboot.bin to RAM successfully.";
    } catch (error: any) {
        status.value = `Error: ${error.message}`;
    } finally {
        isProcessing.value = false;
    }
}

async function rebootToStage2() {
    isProcessing.value = true;
    status.value = "Rebooting to stage 2...";
    try {
        await device.runCommand("reboot");
        status.value = "Rebooted to stage 2.";
    } catch (error: any) {
        status.value = `Error: ${error.message}`;
    } finally {
        isProcessing.value = false;
    }
}

async function connectToStage2() {
    isProcessing.value = true;
    status.value = "Waiting for stage 2 USB device...";
    try {
        let connected = false;
        while (!connected) {
            try {
                await device.connect();
                connected = true;
            } catch {
                await new Promise((resolve) => setTimeout(resolve, 1000));
            }
        }
        status.value = "Connected to stage 2 USB device.";
    } catch (error: any) {
        status.value = `Error: ${error.message}`;
    } finally {
        isProcessing.value = false;
    }
}

async function flashFilesToDevice() {
    if (!files.value.ubootBin || !files.value.bootExt4 || !files.value.rootExt4) return;
    isProcessing.value = true;
    status.value = "Flashing files to device...";
    try {
        await device.flashBlob("uboot", files.value.ubootBin[0].file as Blob);
        await device.flashBlob("boot", files.value.bootExt4[0].file as Blob);
        await device.flashBlob("root", files.value.rootExt4[0].file as Blob);
        status.value = "Flashed all files successfully.";
    } catch (error: any) {
        status.value = `Error: ${error.message}`;
    } finally {
        isProcessing.value = false;
    }
}

async function rebootDevice() {
    isProcessing.value = true;
    status.value = "Rebooting the device...";
    try {
        await device.runCommand("reboot");
        status.value = "Device rebooted successfully.";
    } catch (error: any) {
        status.value = `Error: ${error.message}`;
    } finally {
        isProcessing.value = false;
    }
}

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
</style>
