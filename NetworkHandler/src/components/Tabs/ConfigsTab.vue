<template>
    <div class="space-y-6">

        <!-- Global speed limit card -->
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

        <!-- Notifications card -->
        <div class="card bg-base-200 rounded-lg">
            <div class="card-body">
                <h2 class="card-title gap-2">
                    <Icon name="alarmWarning" class="w-6 h-6 text-info" />
                    Notifications
                </h2>
                <p class="text-sm text-base-content/60 mb-4">
                    Configure alert triggers and where they should appear.
                </p>

                <!-- New process alert -->
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

                <!-- Download threshold -->
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

                <!-- Upload threshold -->
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

                <!-- Notification display mode -->
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

        <!-- Settings card -->
        <div class="card bg-base-200 rounded-lg">
            <div class="card-body">
                <h2 class="card-title gap-2">
                    <Icon name="settings" class="w-6 h-6 text-primary" />
                    Settings
                </h2>
                <p class="text-sm text-base-content/60 mb-4">
                    Application behaviour and system integration.
                </p>

                <!-- Start with Windows -->
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

                <!-- Minimize to tray -->
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

                <!-- Start minimized -->
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

                <div v-if="settingsError" class="alert alert-error mt-3 py-2 text-sm">
                    {{ settingsError }}
                </div>
            </div>
        </div>

        <!-- WinDivert card -->
        <div class="card bg-base-200 rounded-lg">
            <div class="card-body">
                <h2 class="card-title gap-2">
                    <Icon name="puzzle" class="w-6 h-6 text-warning" />
                    WinDivert
                </h2>
                <p class="text-sm text-base-content/60 mb-4">
                    Packet capture driver required for network monitoring.
                </p>

                <!-- Status indicators -->
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

                <!-- Action buttons -->
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
import { computed, onMounted, reactive, ref } from 'vue'
import {
    checkWinDivertStatus,
    getNotificationConfig,
    getSettings,
    installWinDivert,
    setGlobalLimit,
    setNotificationConfig,
    setSettings,
    startWinDivertService,
} from '../../services/api.js'
import Icon from '../Icon.vue'

const emit = defineEmits(['notification'])

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
const settingsError = ref('')
const installingWindivert = ref(false)

const limitIndex = ref(0)
const limitLabel = computed(() => LIMIT_PRESETS[limitIndex.value].label)

onMounted(async () => {
    try {
        const [s, n, w] = await Promise.all([
            getSettings(),
            getNotificationConfig(),
            checkWinDivertStatus(),
        ])
        Object.assign(appSettings, s)
        Object.assign(notif, n)
        Object.assign(windivertStatus, w)
        const index = LIMIT_PRESETS.findIndex(
            (p) => p.value === appSettings.globalLimitBps,
        )
        if (index >= 0) {
            limitIndex.value = index
        }
    } catch {
        /* ignore — backend may not be fully started */
    }
})

async function saveNotif() {
    try {
        await setNotificationConfig({ ...notif })
        emit('notification', {
            type: 'success',
            message: 'Notification settings updated',
        })
    } catch {
        emit('notification', {
            type: 'error',
            message: 'Failed to update notification settings',
        })
    }
}

async function saveGlobalLimit(bytesPerSec) {
    try {
        await setGlobalLimit(bytesPerSec)
        emit('notification', {
            type: 'success',
            message: 'Global speed limit updated',
        })
    } catch {
        emit('notification', {
            type: 'error',
            message: 'Failed to update global speed limit',
        })
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
    settingsError.value = ''
    try {
        await setSettings({ ...appSettings })
        emit('notification', {
            type: 'success',
            message: 'Application settings updated',
        })
    } catch (e) {
        settingsError.value = String(e)
        emit('notification', {
            type: 'error',
            message: 'Failed to update application settings',
        })
    } finally {
        savingSettings.value = false
    }
}

async function installWindivertHandler() {
    installingWindivert.value = true
    try {
        await installWinDivert()
        emit('notification', {
            type: 'success',
            message: 'WinDivert installed successfully',
        })
        const status = await checkWinDivertStatus()
        Object.assign(windivertStatus, status)
    } catch (e) {
        emit('notification', {
            type: 'error',
            message: `Failed to install WinDivert: ${e}`,
        })
    } finally {
        installingWindivert.value = false
    }
}

async function startWindivertServiceHandler() {
    installingWindivert.value = true
    try {
        await startWinDivertService()
        emit('notification', {
            type: 'success',
            message: 'WinDivert service started successfully',
        })
        const status = await checkWinDivertStatus()
        Object.assign(windivertStatus, status)
    } catch (e) {
        emit('notification', {
            type: 'error',
            message: `Failed to start WinDivert service: ${e}`,
        })
    } finally {
        installingWindivert.value = false
    }
}
</script>
