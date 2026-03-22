import { invoke } from '@tauri-apps/api/core'

async function loadServices() {
    return await invoke('get_services')
}

async function refreshServices() {
    return await invoke('refresh_services')
}

async function reloadServiceInfo(serviceName) {
    try {
        return await invoke('reload_service_info', { serviceName })
    } catch {
        return { error: true, message: 'Failed to reload service information' }
    }
}

async function disableService(serviceName) {
    try {
        return await invoke('disable_service', { serviceName })
    } catch {
        return { error: true, message: 'Failed to disable service' }
    }
}

async function startService(serviceName) {
    return await invoke('start_service', { serviceName })
}

async function stopService(serviceName) {
    return await invoke('stop_service', { serviceName })
}

async function restartService(serviceName) {
    return await invoke('restart_service', { serviceName })
}

async function setStartupType(serviceName, startupType) {
    return await invoke('set_startup_type', { serviceName, startupType })
}

async function getStartupApps() {
    return await invoke('get_startup_apps')
}

async function addStartupApp(app) {
    return await invoke('add_startup_app', { app })
}

async function removeStartupApp(app) {
    return await invoke('remove_startup_app', { app })
}

async function getHistory() {
    return await invoke('get_history')
}

async function getHistoryByType(entryType) {
    return await invoke('get_history_by_type', { entryType })
}

async function clearHistoryAction() {
    return await invoke('clear_history')
}

export {
    loadServices,
    refreshServices,
    reloadServiceInfo,
    disableService,
    startService,
    stopService,
    restartService,
    setStartupType,
    getStartupApps,
    addStartupApp,
    removeStartupApp,
    getHistory,
    getHistoryByType,
    clearHistoryAction,
}
