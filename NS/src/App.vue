<template>
    <div class="min-h-screen bg-base-200">
        <div class="container mx-auto p-6">

            <div role="tablist" class="tabs tabs-boxed gap-2 p-0 mb-6">
                <label v-for="tab in tabs" :key="tab.id" class="tab gap-1 text-lg font-medium hover:text-info">
                    <input v-model="activeTab" type="radio" name="tabs_main" class="tab" :value="tab.id" />
                    <Icon :name="tab.icon" />
                    {{ tab.name }}
                </label>
            </div>

            <div v-if="captureError" class="alert alert-error mb-6">
                <Icon name="alarmWarning" />
                <h3 class="font-bold">Failed to start packet capture</h3>
                <div class="text-xs">Ensure the application is running as Administrator.</div>
            </div>

            <div class="fixed top-4 right-4 z-50 flex flex-col gap-2 max-w-sm">
                <div v-for="n in notifications" :key="n.id"
                    :class="`alert alert-${n.type} shadow-lg py-2 px-4 text-sm flex items-center gap-2`">
                    <Icon name="alarmWarning" class="w-4 h-4 shrink-0" />
                    <span>{{ n.message }}</span>
                    <Button class="btn btn-ghost btn-xs btn-square ml-auto" @clicked="dismiss(n.id)">
                        <Icon name="close" class="w-3 h-3" />
                    </Button>
                </div>
            </div>

            <div class="card bg-base-100 card-border border-base-300">
                <div class="card-body">
                    <KeepAlive>
                        <component
                            :is="currentComponent"
                            :download-history="downloadHistory"
                            :upload-history="uploadHistory"
                            :labels="labels"
                            :processes="processes"
                            :totals="totals24h"
                            @block-toggle="onBlockToggle"
                            @throttle="onThrottle"
                            @terminate="onTerminate"
                        />
                    </KeepAlive>
                </div>
            </div>

        </div>
    </div>
</template>

<script setup>
import { getCurrentWindow } from '@tauri-apps/api/window'
import { markRaw, onMounted, onUnmounted, ref, computed } from 'vue'
import Button from './components/Button.vue'
import Icon from './components/Icon.vue'
import ConfigsTab from './components/Tabs/ConfigsTab.vue'
import DashboardTab from './components/Tabs/DashboardTab.vue'
import ProcessesTab from './components/Tabs/ProcessesTab.vue'
import { useNetwork } from './composables/useNetwork.js'
import { useProcessManagement } from './composables/useProcessManagement.js'
import { useToast } from './composables/useToast.js'
import { rustService } from './services/rust.js'

const tabs = [
    { id: 'dashboard', name: 'Dashboard', component: markRaw(DashboardTab), icon: 'dashboard2' },
    { id: 'processes', name: 'Processes', component: markRaw(ProcessesTab), icon: 'listView' },
    { id: 'configs', name: 'Configs', component: markRaw(ConfigsTab), icon: 'settings4' },
]

const activeTab = ref('dashboard')

const currentComponent = computed(() => {
    const tab = tabs.find(t => t.id === activeTab.value)
    return tab?.component
})

const captureError = ref(false)
const totals24h = ref({ downloadBytes: 0, uploadBytes: 0 })

const { notifications, warning: warnToast, dismiss } = useToast()
const { processes, update: updateProcesses, setLimit, toggleBlock, terminate } = useProcessManagement()
const { downloadHistory, uploadHistory, labels, pushStats } = useNetwork()

const notifFiredDl = ref(false)
const notifFiredUl = ref(false)
const seenExes = new Set()
let firstPoll = true

let pollInterval = null

onMounted(async () => {
    try {
        await rustService.startCapture()
    } catch {
        captureError.value = true
    }
    pollInterval = setInterval(poll, 1000)

    try {
        const wdStatus = await rustService.checkWinDivertStatus()
        if (!wdStatus.libraryExists) {
            warnToast(
                'WinDivert library is missing. Network monitoring is unavailable. Go to Configs → WinDivert to install.',
            )
        } else if (!wdStatus.serviceRunning) {
            warnToast(
                'WinDivert service is not running. Network monitoring may be limited. Go to Configs → WinDivert to start it.',
            )
        }
    } catch {}

    try {
        const appWindow = getCurrentWindow()
        await appWindow.onCloseRequested(async (event) => {
            event.preventDefault()
            try {
                const s = await rustService.getSettings()
                if (s?.minimizeToTray) {
                    await appWindow.hide()
                } else {
                    await rustService.exitApp()
                }
            } catch {
                await rustService.exitApp()
            }
        })
    } catch {}
})

onUnmounted(async () => {
    clearInterval(pollInterval)
    try {
        await rustService.stopCapture()
    } catch {}
})

async function poll() {
    try {
        const [stats, procs, totals] = await Promise.all([
            rustService.getNetworkStats(),
            rustService.getProcesses(),
            rustService.get24hTotals(),
        ])
        if (stats) pushStats(stats.downloadBps, stats.uploadBps)
        if (totals) totals24h.value = totals
        if (procs) {
            updateProcesses(procs)
            await checkNotifications(procs, totals)
        }
    } catch {}
}

async function checkNotifications(procs, totals) {
    try {
        const notifConfig = await rustService.getNotificationConfig()
        if (!notifConfig) return

        if (notifConfig.newProcessAlert) {
            if (firstPoll) {
                for (const p of procs) seenExes.add(p.exePath)
                firstPoll = false
            } else {
                for (const p of procs) {
                    if (!seenExes.has(p.exePath)) {
                        seenExes.add(p.exePath)
                        const message = `New process: ${p.name}`
                        if (notifConfig.displayMode === 'app') {
                            warnToast(message)
                        } else if (notifConfig.displayMode === 'native') {
                            rustService
                                .showNativeNotification(
                                    'NetSentry Alert',
                                    message,
                                )
                                .catch(() => {})
                        }
                    }
                }
            }
        } else if (firstPoll) {
            firstPoll = false
        }

        if (!totals) return

        const dlGb = totals.downloadBytes / 1_073_741_824
        if (
            notifConfig.downloadThresholdGb > 0 &&
            dlGb >= notifConfig.downloadThresholdGb &&
            !notifFiredDl.value
        ) {
            notifFiredDl.value = true
            const message = `24h download reached ${dlGb.toFixed(2)} GB (threshold: ${notifConfig.downloadThresholdGb} GB)`
            if (notifConfig.displayMode === 'app') {
                warnToast(message)
            } else if (notifConfig.displayMode === 'native') {
                rustService
                    .showNativeNotification('NetSentry Alert', message)
                    .catch(() => {})
            }
        }

        const ulGb = totals.uploadBytes / 1_073_741_824
        if (
            notifConfig.uploadThresholdGb > 0 &&
            ulGb >= notifConfig.uploadThresholdGb &&
            !notifFiredUl.value
        ) {
            notifFiredUl.value = true
            const message = `24h upload reached ${ulGb.toFixed(2)} GB (threshold: ${notifConfig.uploadThresholdGb} GB)`
            if (notifConfig.displayMode === 'app') {
                warnToast(message)
            } else if (notifConfig.displayMode === 'native') {
                rustService
                    .showNativeNotification('NetSentry Alert', message)
                    .catch(() => {})
            }
        }
    } catch {}
}

async function onThrottle({ proc, bps }) {
    await setLimit(proc.pid, proc.exePath, bps)
}

async function onBlockToggle(proc) {
    await toggleBlock(proc)
}

async function onTerminate(proc) {
    await terminate(proc)
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
