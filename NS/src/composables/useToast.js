import { ref } from 'vue'

const notifications = ref([])
let nextId = 0

export function useToast() {
    const show = (message, type = 'info', duration = 8000) => {
        const id = nextId++
        notifications.value.push({ id, message, type })
        if (duration > 0) {
            setTimeout(() => dismiss(id), duration)
        }
        return id
    }

    const error = (message) => show(message, 'error', 8000)
    const warning = (message) => show(message, 'warning', 8000)
    const success = (message) => show(message, 'success', 5000)

    const dismiss = (id) => {
        notifications.value = notifications.value.filter((n) => n.id !== id)
    }

    return { notifications, show, error, warning, success, dismiss }
}
