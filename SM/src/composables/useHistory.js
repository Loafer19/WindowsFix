import { ref, computed } from 'vue'
import {
    getHistory,
    getHistoryByType,
    clearHistoryAction
} from '../services/api.js'

export function useHistory() {
    const history = ref([])
    const loading = ref(false)
    const error = ref(null)

    const filterType = ref(null)  // 'service', 'startup_app', or null for all

    // ✨ UPDATED: Support both service and startup app entries
    const filteredHistory = computed(() => {
        if (!filterType.value) return history.value

        return history.value.filter(entry => {
            if (filterType.value === 'service') {
                return entry.entry_type?.type === 'service'
            } else if (filterType.value === 'startup_app') {
                return entry.entry_type?.type === 'startup_app'
            }
            return true
        })
    })

    const stats = computed(() => ({
        total: history.value.length,
        services: history.value.filter(h => h.entry_type?.type === 'service').length,
        startupApps: history.value.filter(h => h.entry_type?.type === 'startup_app').length,
    }))

    const loadHistory = async () => {
        try {
            loading.value = true
            error.value = null
            history.value = await getHistory()
        } catch (err) {
            error.value = err.message
            console.error('Failed to load history:', err)
        } finally {
            loading.value = false
        }
    }

    const loadHistoryByType = async (type) => {
        try {
            loading.value = true
            error.value = null
            history.value = await getHistoryByType(type)
        } catch (err) {
            error.value = err.message
        } finally {
            loading.value = false
        }
    }

    const clearHistory = async () => {
        try {
            await clearHistoryAction()
            history.value = []
        } catch (err) {
            error.value = err.message
            console.error('Failed to clear history:', err)
        }
    }

    const setFilterType = (type) => {
        filterType.value = type
    }

    return {
        history: filteredHistory,
        allHistory: history,
        loading,
        error,
        stats,
        filterType,
        loadHistory,
        loadHistoryByType,
        clearHistory,
        setFilterType,
    }
}
