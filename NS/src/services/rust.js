import { invoke } from '@tauri-apps/api/core'
import { useToast } from '../composables/useToast.js'

const { error: showError } = useToast()

class RustService {
    async execute(command, args = {}, suppressError = false) {
        try {
            return await invoke(command, args)
        } catch (e) {
            const msg = String(e)
            if (!suppressError) {
                showError(msg)
            }
            throw e
        }
    }

    async startCapture() {
        return this.execute('start_capture')
    }

    async stopCapture() {
        return this.execute('stop_capture')
    }

    async getNetworkStats() {
        return this.execute('get_network_stats')
    }

    async get24hTotals() {
        return this.execute('get_24h_totals')
    }

    async getProcessHistory(exePath, period = '24h') {
        return this.execute('get_process_history', { exePath, period })
    }

    async getProcesses() {
        return this.execute('get_processes')
    }

    /**
     * @param {number} bytesPerSec - 0 to remove the limit
     * @returns {Promise<void>}
     */
    async setGlobalLimit(bytesPerSec) {
        return this.execute('set_global_limit', { bytesPerSec })
    }

    /**
     * @param {number} pid
     * @param {number} bytesPerSec - 0 to remove the limit
     * @returns {Promise<void>}
     */
    async setProcessLimit(pid, bytesPerSec) {
        return this.execute('set_process_limit', { pid, bytesPerSec })
    }

    async blockProcess(pid) {
        return this.execute('block_process', { pid })
    }

    async unblockProcess(pid) {
        return this.execute('unblock_process', { pid })
    }

    async killProcess(pid) {
        return this.execute('kill_process', { pid })
    }

    async getSettings() {
        return this.execute('get_settings')
    }

    async setSettings(settings) {
        return this.execute('set_settings', { settings })
    }

    async getNotificationConfig() {
        return this.execute('get_notification_config')
    }

    async setNotificationConfig(config) {
        return this.execute('set_notification_config', { config })
    }

    async clearAllData() {
        return this.execute('clear_all_data')
    }

    async checkWinDivertStatus() {
        return this.execute('check_windivert_status')
    }

    async installWinDivert() {
        return this.execute('install_windivert')
    }

    async startWinDivertService() {
        return this.execute('start_windivert_service')
    }

    async showNativeNotification(title, message) {
        return this.execute('show_native_notification', { title, message })
    }

    async getMetrics() {
        return this.execute('get_metrics')
    }

    async exitApp() {
        return this.execute('exit_app')
    }
}

export const rustService = new RustService()
