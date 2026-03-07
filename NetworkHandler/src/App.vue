<template>
    <div class="min-h-screen bg-base-200">
        <div class="container mx-auto p-6">

            <div role="tablist" class="tabs tabs-boxed gap-2 p-0 mb-6">
                <label v-for="tab in tabs" :key="tab.id" class="tab gap-1 text-lg font-medium hover:text-info">
                    <input v-model="activeTab" type="radio" name="tabs_main" class="tab" :value="tab.component" />
                    <Icon :name="tab.icon" />
                    {{ tab.name }}
                </label>
            </div>

            <div v-if="error" class="alert alert-error mb-6">
                <Icon name="alarmWarning" />
                <h3 class="font-bold">Failed to start packet capture</h3>
                <div class="text-xs">Ensure the application is running as Administrator.</div>
            </div>

            <div class="card bg-base-100 card-border border-base-300">
                <div class="card-body">
                    <component
                        :is="activeTab"
                        :download-history="downloadHistory"
                        :upload-history="uploadHistory"
                        :labels="labels"
                        :format-speed="formatSpeed"
                        :processes="processes"
                        @limit-change="onLimitChange"
                        @block="onBlock"
                        @unblock="onUnblock"
                    />
                </div>
            </div>

        </div>
    </div>
</template>

<script setup>
import { markRaw, onMounted, onUnmounted, ref } from 'vue'
import Icon from './components/Icon.vue'
import DashboardTab from './components/Tabs/DashboardTab.vue'
import ProcessesTab from './components/Tabs/ProcessesTab.vue'
import { useNetwork } from './composables/useNetwork.js'
import {
    blockProcess,
    getNetworkStats,
    getProcesses,
    setGlobalLimit,
    startCapture,
    stopCapture,
    unblockProcess,
} from './services/api.js'

const tabs = ref([
    { id: 'dashboard', name: 'Dashboard', component: markRaw(DashboardTab), icon: 'dashboard' },
    { id: 'processes', name: 'Processes', component: markRaw(ProcessesTab), icon: 'processes' },
])

const activeTab = ref(markRaw(DashboardTab))
const error = ref(false)
const processes = ref([])

const { downloadHistory, uploadHistory, labels, pushStats, formatSpeed } = useNetwork()

let pollInterval = null

onMounted(async () => {
    try {
        await startCapture()
    } catch {
        error.value = true
    }
    pollInterval = setInterval(poll, 1000)
})

onUnmounted(async () => {
    clearInterval(pollInterval)
    try {
        await stopCapture()
    } catch {
        // ignore
    }
})

async function poll() {
    try {
        const [stats, procs] = await Promise.all([getNetworkStats(), getProcesses()])
        pushStats(stats.downloadBps, stats.uploadBps)
        processes.value = procs
    } catch {
        // capture may not be running yet
    }
}

async function onLimitChange(bytesPerSec) {
    try {
        await setGlobalLimit(bytesPerSec)
    } catch {
        // ignore
    }
}

async function onBlock(proc) {
    proc.isPending = true
    try {
        await blockProcess(proc.pid)
        proc.blocked = true
    } finally {
        proc.isPending = false
    }
}

async function onUnblock(proc) {
    proc.isPending = true
    try {
        await unblockProcess(proc.pid)
        proc.blocked = false
    } finally {
        proc.isPending = false
    }
}
</script>

<style scoped>
.tab {
    border: var(--border) solid var(--color-base-300) !important;
    border-radius: var(--radius-field) !important;
    box-shadow: none;
    transition: all 0.2s ease;
}

.tab:hover {
    border-color: var(--color-primary) !important;
    background-color: var(--color-primary) !important;
    color: var(--color-primary-content) !important;
}

.tab:has(input:checked) {
    border-color: var(--color-primary) !important;
    background-color: var(--color-primary) !important;
    color: var(--color-primary-content) !important;
}
</style>
