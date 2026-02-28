import { ref } from 'vue'

const STORAGE_KEY = 'disabledServicesHistory'

export function useHistory() {
    const history = ref(loadFromStorage())

    function loadFromStorage() {
        try {
            return JSON.parse(localStorage.getItem(STORAGE_KEY) || '[]')
        } catch {
            return []
        }
    }

    function addToHistory(service) {
        const entry = {
            name: service.name,
            displayName: service.displayName,
            disabledAt: new Date().toISOString(),
        }
        history.value = [entry, ...history.value]
        localStorage.setItem(STORAGE_KEY, JSON.stringify(history.value))
    }

    function clearHistory() {
        history.value = []
        localStorage.removeItem(STORAGE_KEY)
    }

    return { history, addToHistory, clearHistory }
}
