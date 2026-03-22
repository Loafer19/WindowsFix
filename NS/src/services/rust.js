import { invoke } from '@tauri-apps/api/core'
import { useToast } from '../composables/useToast.js'

const { error: showError } = useToast()

/**
 * Unified service layer for all Rust (Tauri) invocations.
 *
 * All errors are automatically caught, displayed as in-app toasts, and then
 * re-thrown so callers can still react to failures (e.g. roll back UI state).
 * Native browser error dialogs are never shown.
 */
class RustService {
    /**
     * Execute a Tauri command with centralised error handling.
     *
     * @template T
     * @param {string} command - Tauri command name
     * @param {Record<string, unknown>} [args]
     * @param {boolean} [suppressError] - When true the error is not toasted
     * @returns {Promise<T>}
     */
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

    // ── Capture ────────────────────────────────────────────────────────────

    /** @returns {Promise<void>} */
    async startCapture() {
        return this.execute('start_capture')
    }

    /** @returns {Promise<void>} */
    async stopCapture() {
        return this.execute('stop_capture')
    }

    // ── Network Stats ──────────────────────────────────────────────────────

    /**
     * @returns {Promise<{ downloadBps: number, uploadBps: number }>}
     */
    async getNetworkStats() {
        return this.execute('get_network_stats')
    }

    /**
     * @returns {Promise<{ downloadBytes: number, uploadBytes: number }>}
     */
    async get24hTotals() {
        return this.execute('get_24h_totals')
    }

    /**
     * @param {string} exePath
     * @param {'24h'|'7d'|'30d'} [period]
     * @returns {Promise<Array<{ downloadBytes: number, uploadBytes: number }>>}
     */
    async getProcessHistory(exePath, period = '24h') {
        return this.execute('get_process_history', { exePath, period })
    }

    // ── Processes ──────────────────────────────────────────────────────────

    /**
     * @returns {Promise<import('./types.js').NetworkProcess[]>}
     */
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

    /**
     * @param {number} pid
     * @returns {Promise<void>}
     */
    async blockProcess(pid) {
        return this.execute('block_process', { pid })
    }

    /**
     * @param {number} pid
     * @returns {Promise<void>}
     */
    async unblockProcess(pid) {
        return this.execute('unblock_process', { pid })
    }

    /**
     * @param {number} pid
     * @returns {Promise<void>}
     */
    async killProcess(pid) {
        return this.execute('kill_process', { pid })
    }

    // ── Settings ───────────────────────────────────────────────────────────

    /**
     * @returns {Promise<import('./types.js').Settings>}
     */
    async getSettings() {
        return this.execute('get_settings')
    }

    /**
     * @param {import('./types.js').Settings} settings
     * @returns {Promise<void>}
     */
    async setSettings(settings) {
        return this.execute('set_settings', { settings })
    }

    /**
     * @returns {Promise<import('./types.js').NotificationConfig>}
     */
    async getNotificationConfig() {
        return this.execute('get_notification_config')
    }

    /**
     * @param {import('./types.js').NotificationConfig} config
     * @returns {Promise<void>}
     */
    async setNotificationConfig(config) {
        return this.execute('set_notification_config', { config })
    }

    /** @returns {Promise<void>} */
    async clearAllData() {
        return this.execute('clear_all_data')
    }

    // ── WinDivert ──────────────────────────────────────────────────────────

    /**
     * @returns {Promise<import('./types.js').WinDivertStatus>}
     */
    async checkWinDivertStatus() {
        return this.execute('check_windivert_status')
    }

    /** @returns {Promise<void>} */
    async installWinDivert() {
        return this.execute('install_windivert')
    }

    /** @returns {Promise<void>} */
    async startWinDivertService() {
        return this.execute('start_windivert_service')
    }

    // ── Notifications ──────────────────────────────────────────────────────

    /**
     * Show a native Windows toast notification.
     * @param {string} title
     * @param {string} message
     * @returns {Promise<void>}
     */
    async showNativeNotification(title, message) {
        return this.execute('show_native_notification', { title, message })
    }

    // ── Metrics ────────────────────────────────────────────────────────────

    /**
     * @returns {Promise<{ packetsProcessed: number, packetsDropped: number, captureErrors: number, bytesSeen: number }>}
     */
    async getMetrics() {
        return this.execute('get_metrics')
    }

    // ── App ────────────────────────────────────────────────────────────────

    /** @returns {Promise<void>} */
    async exitApp() {
        return this.execute('exit_app')
    }
}

export const rustService = new RustService()
