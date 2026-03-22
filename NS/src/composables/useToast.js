import { ref } from 'vue'

// Module-level reactive state — singleton across the entire app.
// All components that call useToast() share the same notifications array.
const notifications = ref([])
let nextId = 0

/**
 * Composable for centralised toast notifications.
 * Errors from Rust commands are displayed here instead of native dialogs.
 *
 * @returns {{ notifications, show, error, warning, success, dismiss }}
 */
export function useToast() {
    /**
     * @param {string} message
     * @param {'info'|'success'|'warning'|'error'} type
     * @param {number} duration - ms before auto-dismiss (0 = manual only)
     * @returns {number} notification id
     */
    function show(message, type = 'info', duration = 8000) {
        const id = nextId++
        notifications.value.push({ id, message, type })
        if (duration > 0) {
            setTimeout(() => dismiss(id), duration)
        }
        return id
    }

    /** Show an error toast (8 s auto-dismiss). */
    function error(message) {
        return show(message, 'error', 8000)
    }

    /** Show a warning toast (8 s auto-dismiss). */
    function warning(message) {
        return show(message, 'warning', 8000)
    }

    /** Show a success toast (5 s auto-dismiss). */
    function success(message) {
        return show(message, 'success', 5000)
    }

    /** Manually dismiss a notification by id. */
    function dismiss(id) {
        notifications.value = notifications.value.filter((n) => n.id !== id)
    }

    return { notifications, show, error, warning, success, dismiss }
}
