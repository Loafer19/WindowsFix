import { getVersion } from '@tauri-apps/api/app'
import { relaunch } from '@tauri-apps/plugin-process'
import { check } from '@tauri-apps/plugin-updater'
import { ref, shallowRef } from 'vue'

// Shared, module-level state so the startup check and the manual button
// drive the same confirmation modal
const currentVersion = ref('')
// shallowRef: the Update object holds a private resource id that breaks under a deep proxy
const update = shallowRef(null)
const checking = ref(false)
const checked = ref(false)
const installing = ref(false)
const progress = ref(0)
const error = ref('')
const showPrompt = ref(false)

let startupCheckDone = false

async function loadVersion() {
    if (currentVersion.value) return
    try {
        currentVersion.value = await getVersion()
    } catch {
        currentVersion.value = ''
    }
}

/**
 * Ask the updater endpoint whether a newer signed build exists
 * @param {{ silent?: boolean }} options silent checks never surface errors
 */
async function checkForUpdates({ silent = false } = {}) {
    if (checking.value || installing.value) return null
    checking.value = true
    error.value = ''
    try {
        await loadVersion()
        const result = await check()
        update.value = result ?? null
        checked.value = true
        if (result) showPrompt.value = true
        return result ?? null
    } catch (e) {
        if (!silent) error.value = `Update check failed: ${e?.message ?? e}`
        return null
    } finally {
        checking.value = false
    }
}

/** Run one silent check per app session, used on startup */
async function checkOnStartup() {
    if (startupCheckDone) return
    startupCheckDone = true
    await loadVersion()
    await checkForUpdates({ silent: true })
}

async function installUpdate() {
    if (!update.value || installing.value) return
    installing.value = true
    progress.value = 0
    error.value = ''
    let total = 0
    let downloaded = 0
    try {
        await update.value.downloadAndInstall((event) => {
            if (event.event === 'Started') {
                total = event.data.contentLength ?? 0
            } else if (event.event === 'Progress') {
                downloaded += event.data.chunkLength
                progress.value = total
                    ? Math.min(100, Math.round((downloaded / total) * 100))
                    : 0
            } else if (event.event === 'Finished') {
                progress.value = 100
            }
        })
        // On Windows the installer usually closes the app itself, relaunch covers the rest
        await relaunch()
    } catch (e) {
        error.value = `Update failed: ${e?.message ?? e}`
        installing.value = false
    }
}

function openPrompt() {
    if (update.value) showPrompt.value = true
}

function dismissPrompt() {
    if (installing.value) return
    showPrompt.value = false
}

export function useUpdater() {
    return {
        currentVersion,
        update,
        checking,
        checked,
        installing,
        progress,
        error,
        showPrompt,
        loadVersion,
        checkForUpdates,
        checkOnStartup,
        installUpdate,
        openPrompt,
        dismissPrompt,
    }
}
