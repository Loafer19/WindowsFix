<template>
    <div class="space-y-6">

        <!-- Notifications card -->
        <div class="card bg-base-200 border border-base-300">
            <div class="card-body">
                <h2 class="card-title gap-2">
                    <Icon name="bell" class="w-5 h-5 text-warning" />
                    Notifications
                </h2>
                <p class="text-sm text-base-content/60 mb-4">
                    Configure alert triggers. Alerts appear as toasts in the app.
                </p>

                <!-- New process alert -->
                <div class="form-control">
                    <label class="label cursor-pointer justify-start gap-4">
                        <input
                            v-model="notif.newProcessAlert"
                            type="checkbox"
                            class="toggle toggle-warning"
                            @change="saveNotif"
                        />
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
                <div class="flex items-center gap-4 flex-wrap">
                    <Icon name="arrowDown" class="w-4 h-4 text-info shrink-0" />
                    <span class="text-sm font-medium w-40">Download threshold</span>
                    <div class="flex items-center gap-2">
                        <input
                            v-model.number="notif.downloadThresholdGb"
                            type="number"
                            class="input input-bordered input-sm w-24 font-mono"
                            min="0"
                            step="0.5"
                            @change="saveNotif"
                        />
                        <span class="text-xs text-base-content/50">GB / 24h (0 = disabled)</span>
                    </div>
                </div>

                <!-- Upload threshold -->
                <div class="flex items-center gap-4 flex-wrap mt-2">
                    <Icon name="arrowUp" class="w-4 h-4 text-success shrink-0" />
                    <span class="text-sm font-medium w-40">Upload threshold</span>
                    <div class="flex items-center gap-2">
                        <input
                            v-model.number="notif.uploadThresholdGb"
                            type="number"
                            class="input input-bordered input-sm w-24 font-mono"
                            min="0"
                            step="0.5"
                            @change="saveNotif"
                        />
                        <span class="text-xs text-base-content/50">GB / 24h (0 = disabled)</span>
                    </div>
                </div>
            </div>
        </div>

        <!-- Settings card -->
        <div class="card bg-base-200 border border-base-300">
            <div class="card-body">
                <h2 class="card-title gap-2">
                    <Icon name="settings" class="w-5 h-5 text-primary" />
                    Settings
                </h2>
                <p class="text-sm text-base-content/60 mb-4">
                    Application behaviour and system integration.
                </p>

                <!-- Start with Windows -->
                <div class="form-control">
                    <label class="label cursor-pointer justify-start gap-4">
                        <input
                            v-model="appSettings.startWithWindows"
                            type="checkbox"
                            class="toggle toggle-primary"
                            :disabled="savingSettings"
                            @change="saveAppSettings"
                        />
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
                        <input
                            v-model="appSettings.minimizeToTray"
                            type="checkbox"
                            class="toggle toggle-primary"
                            :disabled="savingSettings"
                            @change="saveAppSettings"
                        />
                        <span class="label-text">
                            <span class="font-medium flex items-center gap-2">
                                <Icon name="tray" class="w-4 h-4" />
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

                <div v-if="settingsError" class="alert alert-error mt-3 py-2 text-sm">
                    {{ settingsError }}
                </div>
                <div v-if="settingsSaved" class="alert alert-success mt-3 py-2 text-sm">
                    Settings saved.
                </div>
            </div>
        </div>
    </div>
</template>

<script setup>
import { onMounted, reactive, ref } from 'vue'
import {
    getNotificationConfig,
    getSettings,
    setNotificationConfig,
    setSettings,
} from '../../services/api.js'
import Icon from '../Icon.vue'

const notif = reactive({
    newProcessAlert: false,
    downloadThresholdGb: 5,
    uploadThresholdGb: 5,
})

const appSettings = reactive({
    startWithWindows: false,
    minimizeToTray: false,
})

const savingSettings = ref(false)
const settingsError = ref('')
const settingsSaved = ref(false)

onMounted(async () => {
    try {
        const [s, n] = await Promise.all([
            getSettings(),
            getNotificationConfig(),
        ])
        Object.assign(appSettings, s)
        Object.assign(notif, n)
    } catch {
        /* ignore — backend may not be fully started */
    }
})

async function saveNotif() {
    try {
        await setNotificationConfig({ ...notif })
    } catch {
        /* ignore */
    }
}

async function saveAppSettings() {
    savingSettings.value = true
    settingsError.value = ''
    settingsSaved.value = false
    try {
        await setSettings({ ...appSettings })
        settingsSaved.value = true
        setTimeout(() => {
            settingsSaved.value = false
        }, 2500)
    } catch (e) {
        settingsError.value = String(e)
    } finally {
        savingSettings.value = false
    }
}
</script>
