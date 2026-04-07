<template>
    <div class="space-y-6">
        <div class="card bg-base-200 rounded-lg">
            <div class="card-body">
                <h2 class="card-title gap-2">
                    <Icon name="broadcast" class="w-6 h-6 text-warning" />
                    Global Speed Limit
                </h2>
                <p class="text-sm text-base-content/60 mb-4">
                    Set the overall network speed limit for all traffic.
                </p>
                <input type="range" class="range range-warning w-full" min="0" :max="LIMIT_PRESETS.length - 1"
                    :value="limitIndex" @input="onLimitChange" />
                <div class="flex justify-between text-xs text-base-content/50 mt-1">
                    <span v-for="preset in LIMIT_PRESETS" :key="preset.value">{{ preset.label }}</span>
                </div>
            </div>
        </div>

        <div class="card bg-base-200 rounded-lg">
            <div class="card-body">
                <h2 class="card-title gap-2">
                    <Icon name="alarmWarning" class="w-6 h-6 text-info" />
                    Notifications
                </h2>
                <p class="text-sm text-base-content/60 mb-4">
                    Configure alert triggers and where they should appear.
                </p>

                <div class="form-control">
                    <label class="label cursor-pointer justify-start gap-4">
                        <input v-model="notif.newProcessAlert" type="checkbox" class="toggle toggle-info"
                            @change="saveNotif" />
                        <span class="label-text">
                            <span class="font-medium">New process alert</span>
                            <br />
                            <span class="text-xs text-base-content/50">
                                Show an alert when a new process starts generating network traffic.
                            </span>
                        </span>
                    </label>
                </div>

                <div class="divider my-2"></div>

                <div class="flex items-center gap-2 flex-wrap">
                    <Icon name="arrowDownCircle" class="w-5 h-5 text-success shrink-0" />
                    <span class="text-sm font-medium w-40">Download threshold</span>
                    <div class="flex items-center gap-2">
                        <input v-model.number="notif.downloadThresholdGb" type="number"
                            class="input input-bordered input-sm w-24 font-mono" min="0" step="0.5"
                            @change="saveNotif" />
                        <span class="text-xs text-base-content/50">GB / 24h (0 = disabled)</span>
                    </div>
                </div>

                <div class="flex items-center gap-2 flex-wrap mt-2">
                    <Icon name="arrowUpCircle" class="w-5 h-5 text-info shrink-0" />
                    <span class="text-sm font-medium w-40">Upload threshold</span>
                    <div class="flex items-center gap-2">
                        <input v-model.number="notif.uploadThresholdGb" type="number"
                            class="input input-bordered input-sm w-24 font-mono" min="0" step="0.5"
                            @change="saveNotif" />
                        <span class="text-xs text-base-content/50">GB / 24h (0 = disabled)</span>
                    </div>
                </div>

                <div class="divider my-2"></div>

                <div class="flex items-center gap-4 flex-wrap">
                    <Icon name="alarmWarning" class="w-5 h-5 text-info shrink-0" />
                    <span class="text-sm font-medium">Display mode</span>
                    <div class="flex gap-4">
                        <label v-for="mode in NOTIF_MODES" :key="mode.value"
                            class="flex items-center gap-1 cursor-pointer">
                            <input v-model="notif.displayMode" type="radio" :value="mode.value"
                                class="radio radio-info radio-sm" @change="saveNotif" />
                            <span class="text-sm">{{ mode.label }}</span>
                        </label>
                    </div>
                </div>
            </div>
        </div>

        <div class="card bg-base-200 rounded-lg">
            <div class="card-body">
                <h2 class="card-title gap-2">
                    <Icon name="settings" class="w-6 h-6 text-primary" />
                    Settings
                </h2>
                <p class="text-sm text-base-content/60 mb-4">
                    Application behaviour and system integration.
                </p>

                <div class="form-control">
                    <label class="label cursor-pointer justify-start gap-4">
                        <input v-model="appSettings.startWithWindows" type="checkbox" class="toggle toggle-primary"
                            :disabled="savingSettings" @change="saveAppSettings" />
                        <span class="label-text">
                            <span class="font-medium flex items-center gap-2">
                                <Icon name="windows" class="w-4 h-4" />
                                Start with Windows
                            </span>
                            <br />
                            <span class="text-xs text-base-content/50">
                                Launch NetSentry automatically when Windows starts
                                (writes to HKCU Run registry key).
                            </span>
                        </span>
                    </label>
                </div>

                <div class="divider my-2"></div>

                <div class="form-control">
                    <label class="label cursor-pointer justify-start gap-4">
                        <input v-model="appSettings.minimizeToTray" type="checkbox" class="toggle toggle-primary"
                            :disabled="savingSettings" @change="saveAppSettings" />
                        <span class="label-text">
                            <span class="font-medium flex items-center gap-2">
                                <Icon name="hardDrive2" class="w-4 h-4" />
                                Minimize to system tray on close
                            </span>
                            <br />
                            <span class="text-xs text-base-content/50">
                                Pressing ✕ hides the window to the tray instead of quitting.
                                Right-click the tray icon to quit.
                            </span>
                        </span>
                    </label>
                </div>

                <div class="divider my-2"></div>

                <div class="form-control">
                    <label class="label cursor-pointer justify-start gap-4">
                        <input v-model="appSettings.startMinimized" type="checkbox" class="toggle toggle-primary"
                            :disabled="savingSettings" @change="saveAppSettings" />
                        <span class="label-text">
                            <span class="font-medium flex items-center gap-2">
                                <Icon name="hardDrive2" class="w-4 h-4" />
                                Start minimized
                            </span>
                            <br />
                            <span class="text-xs text-base-content/50">
                                Hide the main window immediately on launch. Access via tray icon.
                            </span>
                        </span>
                    </label>
                </div>

                <div class="divider my-2"></div>

                <div class="flex items-center gap-3 flex-wrap">
                    <Icon name="settings" class="w-5 h-5 text-primary" />
                    <span class="text-sm font-medium">Version</span>
                    <span class="badge badge-dark font-mono text-xs">{{ appVersion || '—' }}</span>
                    <button class="btn btn-xs btn-outline" :disabled="checkingUpdates" @click="checkForUpdates">
                        <span v-if="checkingUpdates" class="loading loading-spinner loading-xs mr-1"></span>
                        Check for updates
                    </button>
                    <a v-if="updateChecked && updateAvailable" :href="releaseUrl" target="_blank"
                        class="link link-primary text-sm font-medium">
                        v{{ latestVersion }} available →
                    </a>
                    <span v-if="updateChecked && !updateAvailable" class="text-xs text-success">✓ Up to date</span>
                </div>

                <div class="divider my-2"></div>

                <div class="flex items-center gap-3">
                    <Button class="btn btn-error btn-sm" @clicked="clearAllData">
                        Clear All Data
                    </Button>
                </div>
            </div>
        </div>

        <div class="card bg-base-200 rounded-lg">
            <div class="card-body">
                <h2 class="card-title gap-2">
                    <Icon name="puzzle" class="w-6 h-6 text-warning" />
                    WinDivert
                </h2>
                <p class="text-sm text-base-content/60 mb-4">
                    Packet capture driver required for network monitoring.
                </p>

                <div class="flex items-center gap-4 flex-wrap">
                    <div class="flex items-center gap-2">
                        <Icon name="settings" class="w-4 h-4 text-neutral" />
                        <span class="text-sm">Library:</span>
                        <span :class="windivertStatus.libraryExists ? 'text-success' : 'text-error'">
                            {{ windivertStatus.libraryExists ? 'Installed' : 'Missing' }}
                        </span>
                    </div>
                    <div class="flex items-center gap-2">
                        <Icon name="settings" class="w-4 h-4 text-neutral" />
                        <span class="text-sm">Service:</span>
                        <span
                            :class="windivertStatus.serviceExists ? (windivertStatus.serviceRunning ? 'text-success' : 'text-warning') : 'text-error'">
                            {{ windivertStatus.serviceExists ? (windivertStatus.serviceRunning ? 'Running' : 'Stopped')
                                : 'Not Created' }}
                        </span>
                    </div>
                </div>

                <div class="mt-4 flex gap-2 flex-wrap">
                    <button v-if="!windivertStatus.libraryExists || !windivertStatus.serviceExists"
                        class="btn btn-primary btn-sm" :disabled="installingWindivert" @click="installWindivertHandler">
                        <span v-if="installingWindivert" class="loading loading-spinner loading-sm mr-2"></span>
                        Install WinDivert
                    </button>
                    <button v-if="windivertStatus.libraryExists" class="btn btn-secondary btn-sm"
                        :disabled="installingWindivert" @click="installWindivertHandler">
                        <span v-if="installingWindivert" class="loading loading-spinner loading-sm mr-2"></span>
                        Reinstall Library
                    </button>
                    <button
                        v-if="windivertStatus.libraryExists && windivertStatus.serviceExists && !windivertStatus.serviceRunning"
                        class="btn btn-warning btn-sm" :disabled="installingWindivert"
                        @click="startWindivertServiceHandler">
                        <span v-if="installingWindivert" class="loading loading-spinner loading-sm mr-2"></span>
                        Start Service
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup>
import { getVersion } from '@tauri-apps/api/app'
import { onMounted, reactive, ref } from 'vue'
import { useToast } from '../../composables/useToast.js'
import { rustService } from '../../services/rust.js'
import Button from '../Button.vue'
import Icon from '../Icon.vue'

const { success: showSuccess, error: showError } = useToast()

const LIMIT_PRESETS = Object.freeze([
    { value: 0, label: 'Unlimited' },
    { value: 131_072, label: '128 KB/s' },
    { value: 524_288, label: '512 KB/s' },
    { value: 1_048_576, label: '1 MB/s' },
    { value: 5_242_880, label: '5 MB/s' },
    { value: 10_485_760, label: '10 MB/s' },
])

const NOTIF_MODES = Object.freeze([
    { value: 'app', label: 'App only' },
    { value: 'native', label: 'Native Windows' },
])

const notif = reactive({
    newProcessAlert: false,
    downloadThresholdGb: 5,
    uploadThresholdGb: 5,
    displayMode: 'app',
})

const appSettings = reactive({
    startWithWindows: false,
    minimizeToTray: false,
    startMinimized: false,
    globalLimitBps: 0,
})

const windivertStatus = reactive({
    libraryExists: false,
    serviceExists: false,
    serviceRunning: false,
})

const savingSettings = ref(false)
const installingWindivert = ref(false)
const limitIndex = ref(0)

const appVersion = ref('')
const checkingUpdates = ref(false)
const updateAvailable = ref(false)
const updateChecked = ref(false)
const latestVersion = ref('')
const releaseUrl = ref('https://github.com/Loafer19/WindowsFix/releases/latest')

onMounted(async () => {
    try {
        const [s, n, w, ver] = await Promise.all([
            rustService.getSettings(),
            rustService.getNotificationConfig(),
            rustService.checkWinDivertStatus(),
            getVersion().catch(() => ''),
        ])
        if (s) Object.assign(appSettings, s)
        if (n) Object.assign(notif, n)
        if (w) Object.assign(windivertStatus, w)
        appVersion.value = ver
        const index = LIMIT_PRESETS.findIndex(
            (p) => p.value === appSettings.globalLimitBps,
        )
        if (index >= 0) {
            limitIndex.value = index
        }
    } catch {}
})

async function checkForUpdates() {
    checkingUpdates.value = true
    updateChecked.value = false
    updateAvailable.value = false
    try {
        const res = await fetch(
            'https://api.github.com/repos/Loafer19/WindowsFix/releases/latest',
        )
        const data = await res.json()
        const latest = (data.tag_name ?? '').replace(/^v/, '')
        latestVersion.value = latest
        releaseUrl.value =
            data.html_url ?? 'https://github.com/Loafer19/WindowsFix/releases'
        updateAvailable.value =
            latest !== '' && compareVersions(latest, appVersion.value) > 0
        updateChecked.value = true
    } catch {
        showError('Failed to check for updates')
    } finally {
        checkingUpdates.value = false
    }
}

/** Compare two semver strings. Returns >0 if a is newer, 0 if equal, <0 if older. */
function compareVersions(a, b) {
    const pa = a.split('.').map(Number)
    const pb = b.split('.').map(Number)
    for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
        const diff = (pa[i] ?? 0) - (pb[i] ?? 0)
        if (diff !== 0) return diff
    }
    return 0
}

async function saveNotif() {
    try {
        await rustService.setNotificationConfig({ ...notif })
        showSuccess('Notification settings updated')
    } catch {
        // Error toast already shown by rustService
    }
}

async function saveGlobalLimit(bytesPerSec) {
    try {
        await rustService.setGlobalLimit(bytesPerSec)
        showSuccess('Global speed limit updated')
    } catch {
        // Error toast already shown by rustService
    }
}

function onLimitChange(e) {
    limitIndex.value = Number(e.target.value)
    const newLimit = LIMIT_PRESETS[limitIndex.value].value
    appSettings.globalLimitBps = newLimit
    saveGlobalLimit(newLimit)
}

async function saveAppSettings() {
    savingSettings.value = true
    try {
        await rustService.setSettings({ ...appSettings })
        showSuccess('Application settings updated')
    } catch {
        // Error toast already shown by rustService
    } finally {
        savingSettings.value = false
    }
}

async function installWindivertHandler() {
    installingWindivert.value = true
    try {
        await rustService.installWinDivert()
        showSuccess('WinDivert installed successfully')
        const status = await rustService.checkWinDivertStatus()
        if (status) Object.assign(windivertStatus, status)
    } catch {
        // Error toast already shown by rustService
    } finally {
        installingWindivert.value = false
    }
}

async function startWindivertServiceHandler() {
    installingWindivert.value = true
    try {
        await rustService.startWinDivertService()
        showSuccess('WinDivert service started successfully')
        const status = await rustService.checkWinDivertStatus()
        if (status) Object.assign(windivertStatus, status)
    } catch {
        // Error toast already shown by rustService
    } finally {
        installingWindivert.value = false
    }
}

async function clearAllData() {
    if (confirm('Are you sure you want to clear all saved data?')) {
        try {
            await rustService.clearAllData()
            showSuccess('All data cleared successfully')
        } catch {
            // Error toast already shown by rustService
        }
    }
}
</script>
