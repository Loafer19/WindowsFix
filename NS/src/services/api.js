import { invoke } from '@tauri-apps/api/core'

async function getNetworkStats() {
    return await invoke('get_network_stats')
}

async function getProcesses() {
    return await invoke('get_processes')
}

async function setGlobalLimit(bytesPerSec) {
    return await invoke('set_global_limit', { bytesPerSec })
}

async function setProcessLimit(pid, bytesPerSec) {
    try {
        return await invoke('set_process_limit', { pid, bytesPerSec })
    } catch (e) {
        return { error: true, message: `Failed to set process limit: ${e}` }
    }
}

async function blockProcess(pid) {
    try {
        return await invoke('block_process', { pid })
    } catch (e) {
        return { error: true, message: `Failed to block process: ${e}` }
    }
}

async function unblockProcess(pid) {
    try {
        return await invoke('unblock_process', { pid })
    } catch (e) {
        return { error: true, message: `Failed to unblock process: ${e}` }
    }
}

async function killProcess(pid) {
    try {
        return await invoke('kill_process', { pid })
    } catch (e) {
        return { error: true, message: `Failed to kill process: ${e}` }
    }
}

async function startCapture() {
    return await invoke('start_capture')
}

async function stopCapture() {
    return await invoke('stop_capture')
}

async function getProcessHistory(exePath, period = '24h') {
    return await invoke('get_process_history', { exePath, period })
}

async function get24hTotals() {
    return await invoke('get_24h_totals')
}

async function getSettings() {
    return await invoke('get_settings')
}

async function setSettings(settings) {
    return await invoke('set_settings', { settings })
}

async function getNotificationConfig() {
    return await invoke('get_notification_config')
}

async function setNotificationConfig(config) {
    return await invoke('set_notification_config', { config })
}

async function showNativeNotification(title, message) {
    try {
        return await invoke('show_native_notification', { title, message })
    } catch (e) {
        console.error('Failed to show native notification:', e)
    }
}

async function checkWinDivertStatus() {
    return await invoke('check_windivert_status')
}

async function installWinDivert() {
    return await invoke('install_windivert')
}

async function startWinDivertService() {
    return await invoke('start_windivert_service')
}

async function exitApp() {
    return await invoke('exit_app')
}

export {
    getNetworkStats,
    getProcesses,
    setGlobalLimit,
    setProcessLimit,
    blockProcess,
    unblockProcess,
    killProcess,
    startCapture,
    stopCapture,
    getProcessHistory,
    get24hTotals,
    getSettings,
    setSettings,
    getNotificationConfig,
    setNotificationConfig,
    showNativeNotification,
    checkWinDivertStatus,
    installWinDivert,
    startWinDivertService,
    exitApp,
}
