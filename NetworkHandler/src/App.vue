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

            <div class="fixed top-4 right-4 z-50 flex flex-col gap-2 max-w-sm">
                <div v-for="n in notifications" :key="n.id"
                    :class="`alert alert-${n.type} shadow-lg py-2 px-4 text-sm flex items-center gap-2`">
                    <Icon name="alarmWarning" class="w-4 h-4 shrink-0" />
                    <span>{{ n.message }}</span>
                    <Button class="btn btn-ghost btn-xs btn-square ml-auto" @clicked="dismissNotification(n.id)">
                        <Icon name="close" class="w-3 h-3" />
                    </Button>
                </div>
            </div>

            <div class="card bg-base-100 card-border border-base-300">
                <div class="card-body">
                    <component :is="activeTab" :download-history="downloadHistory" :upload-history="uploadHistory"
                        :labels="labels" :processes="processes" :totals="totals24h" @block-toggle="onBlockToggle"
                        @throttle="onThrottle" @terminate="onTerminate"
                        @notification="onNotification" />
                </div>
            </div>

        </div>
    </div>
</template>

<script setup>
import { getCurrentWindow } from '@tauri-apps/api/window'
import { markRaw, onMounted, onUnmounted, ref } from 'vue'
import Button from './components/Button.vue'
import Icon from './components/Icon.vue'
import ConfigsTab from './components/Tabs/ConfigsTab.vue'
import DashboardTab from './components/Tabs/DashboardTab.vue'
import ProcessesTab from './components/Tabs/ProcessesTab.vue'
import { useNetwork } from './composables/useNetwork.js'
import {
    blockProcess,
    checkWinDivertStatus,
    exitApp,
    get24hTotals,
    getNetworkStats,
    getNotificationConfig,
    getProcesses,
    getSettings,
    killProcess,
    setProcessLimit,
    showNativeNotification,
    startCapture,
    stopCapture,
    unblockProcess,
} from './services/api.js'

const tabs = ref([
    {
        id: 'dashboard',
        name: 'Dashboard',
        component: markRaw(DashboardTab),
        icon: 'dashboard2',
    },
    {
        id: 'processes',
        name: 'Processes',
        component: markRaw(ProcessesTab),
        icon: 'listView',
    },
    {
        id: 'configs',
        name: 'Configs',
        component: markRaw(ConfigsTab),
        icon: 'settings4',
    },
])

const activeTab = ref(markRaw(DashboardTab))
const error = ref(false)
const processes = ref([])
const totals24h = ref({ downloadBytes: 0, uploadBytes: 0 })

const notifications = ref([])
let nextNotifId = 0
// Track which threshold notification was already fired this session
const notifFiredDl = ref(false)
const notifFiredUl = ref(false)
// Track seen exe paths for new-process alerts
const seenExes = new Set()
let firstPoll = true

const { downloadHistory, uploadHistory, labels, pushStats } = useNetwork()

let pollInterval = null

onMounted(async () => {
    try {
        await startCapture()
    } catch {
        error.value = true
    }
    pollInterval = setInterval(poll, 1000)

    // Check WinDivert status on startup and notify if library is not working
    try {
        const wdStatus = await checkWinDivertStatus()
        if (!wdStatus.libraryExists) {
            pushNotification({
                type: 'warning',
                message: 'WinDivert library is missing. Network monitoring is unavailable. Go to Configs → WinDivert to install.',
            })
        } else if (!wdStatus.serviceRunning) {
            pushNotification({
                type: 'info',
                message: 'WinDivert service is not running. Network monitoring may be limited. Go to Configs → WinDivert to start it.',
            })
        }
    } catch {
        /* ignore — backend may not be fully started */
    }

    // Intercept window close: hide to tray if configured, otherwise exit
    try {
        const appWindow = getCurrentWindow()
        await appWindow.onCloseRequested(async (event) => {
            event.preventDefault()
            try {
                const s = await getSettings()
                if (s.minimizeToTray) {
                    await appWindow.hide()
                } else {
                    await exitApp()
                }
            } catch {
                await exitApp()
            }
        })
    } catch {
        /* not in Tauri context (e.g. browser preview) */
    }
})

onUnmounted(async () => {
    clearInterval(pollInterval)
    try {
        await stopCapture()
    } catch {
        /* ignore */
    }
})

async function poll() {
    try {
        const [stats, procs, totals] = await Promise.all([
            getNetworkStats(),
            getProcesses(),
            get24hTotals(),
        ])
        pushStats(stats.downloadBps, stats.uploadBps)
        totals24h.value = totals

        // Merge server data with local UI state flags (isPending, isTerminating)
        processes.value = procs.map((p) => {
            const existing =
                processes.value.find((e) => e.exePath === p.exePath) ?? {}
            return {
                ...p,
                isPending: existing.isPending ?? false,
                isTerminating: existing.isTerminating ?? false,
            }
        })

        await checkNotifications(procs, totals)
    } catch {
        // capture may not be running yet
    }
}

async function checkNotifications(procs, totals) {
    try {
        const notifConfig = await getNotificationConfig()

        // New process alert — seed on first poll, fire on subsequent polls
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
                            pushNotification({
                                type: 'warning',
                                message,
                            })
                        } else if (notifConfig.displayMode === 'native') {
                            showNativeNotification('NetSentry Alert', message)
                        }
                    }
                }
            }
        } else if (firstPoll) {
            firstPoll = false
        }

        // Download threshold
        const dlGb = totals.downloadBytes / 1_073_741_824
        if (
            notifConfig.downloadThresholdGb > 0 &&
            dlGb >= notifConfig.downloadThresholdGb &&
            !notifFiredDl.value
        ) {
            notifFiredDl.value = true
            const message = `24h download reached ${dlGb.toFixed(2)} GB (threshold: ${notifConfig.downloadThresholdGb} GB)`
            if (notifConfig.displayMode === 'app') {
                pushNotification({
                    type: 'warning',
                    message,
                })
            } else if (notifConfig.displayMode === 'native') {
                showNativeNotification('NetSentry Alert', message)
            }
        }

        // Upload threshold
        const ulGb = totals.uploadBytes / 1_073_741_824
        if (
            notifConfig.uploadThresholdGb > 0 &&
            ulGb >= notifConfig.uploadThresholdGb &&
            !notifFiredUl.value
        ) {
            notifFiredUl.value = true
            const message = `24h upload reached ${ulGb.toFixed(2)} GB (threshold: ${notifConfig.uploadThresholdGb} GB)`
            if (notifConfig.displayMode === 'app') {
                pushNotification({
                    type: 'warning',
                    message,
                })
            } else if (notifConfig.displayMode === 'native') {
                showNativeNotification('NetSentry Alert', message)
            }
        }
    } catch {
        /* ignore */
    }
}

function pushNotification(notification) {
    const id = nextNotifId++
    notifications.value.push({ id, ...notification })
    setTimeout(() => dismissNotification(id), 8000)
}

function dismissNotification(id) {
    notifications.value = notifications.value.filter((n) => n.id !== id)
}

function onNotification(notification) {
    pushNotification(notification)
}

async function onThrottle({ proc, bps }) {
    try {
        await setProcessLimit(proc.pid, bps)
        const found = processes.value.find((p) => p.exePath === proc.exePath)
        if (found) found.limitBps = bps
    } catch {
        /* ignore */
    }
}

async function onBlockToggle(proc) {
    proc.isPending = true
    try {
        if (proc.blocked) {
            await unblockProcess(proc.pid)
            proc.blocked = false
        } else {
            await blockProcess(proc.pid)
            proc.blocked = true
        }
    } finally {
        proc.isPending = false
    }
}

async function onTerminate(proc) {
    proc.isTerminating = true
    try {
        await killProcess(proc.pid)
        processes.value = processes.value.filter(
            (p) => p.exePath !== proc.exePath,
        )
    } catch {
        /* ignore — process may already be gone */
    } finally {
        proc.isTerminating = false
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
