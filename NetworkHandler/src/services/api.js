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

async function freeProcessPorts(pid) {
    try {
        return await invoke('free_process_ports', { pid })
    } catch (e) {
        return { error: true, message: `Failed to free process ports: ${e}` }
    }
}

async function startCapture() {
    return await invoke('start_capture')
}

async function stopCapture() {
    return await invoke('stop_capture')
}

async function getProcessHistory(pid) {
    return await invoke('get_process_history', { pid })
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

export {
    getNetworkStats,
    getProcesses,
    setGlobalLimit,
    setProcessLimit,
    blockProcess,
    unblockProcess,
    killProcess,
    freeProcessPorts,
    startCapture,
    stopCapture,
    getProcessHistory,
    get24hTotals,
    getSettings,
    setSettings,
    getNotificationConfig,
    setNotificationConfig,
}
