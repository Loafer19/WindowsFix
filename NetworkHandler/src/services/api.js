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

async function blockProcess(pid) {
    try {
        return await invoke('block_process', { pid })
    } catch {
        return { error: true, message: 'Failed to block process' }
    }
}

async function unblockProcess(pid) {
    try {
        return await invoke('unblock_process', { pid })
    } catch {
        return { error: true, message: 'Failed to unblock process' }
    }
}

async function startCapture() {
    return await invoke('start_capture')
}

async function stopCapture() {
    return await invoke('stop_capture')
}

export {
    getNetworkStats,
    getProcesses,
    setGlobalLimit,
    blockProcess,
    unblockProcess,
    startCapture,
    stopCapture,
}
