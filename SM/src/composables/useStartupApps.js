import { ref, computed } from 'vue'
import {
    getStartupApps,
    addStartupApp,
    removeStartupApp,
} from '../services/api.js'

export function useStartupApps() {
    const allStartupApps = ref([])
    const loading = ref(false)
    const error = ref(null)

    const searchQuery = ref('')
    const selectedLocation = ref(null)
    const selectedStatus = ref(null)

    const locationOptions = [
        { value: 'HkeyLocalMachine', label: 'HKLM Registry (System)' },
        { value: 'HkeyCurrentUser', label: 'HKCU Registry (User)' },
        { value: 'StartupFolder', label: 'Startup Folder' },
    ]

    const statusOptions = [
        { value: true, label: 'Enabled' },
        { value: false, label: 'Disabled' },
    ]

    const filteredApps = computed(() => {
        return allStartupApps.value.filter(app => {
            const matchesSearch = app.name
                .toLowerCase()
                .includes(searchQuery.value.toLowerCase()) ||
                app.path
                    .toLowerCase()
                    .includes(searchQuery.value.toLowerCase())

            const matchesLocation = !selectedLocation.value ||
                app.location === selectedLocation.value

            const matchesStatus = selectedStatus.value === null ||
                app.enabled === selectedStatus.value

            return matchesSearch && matchesLocation && matchesStatus
        })
    })

    const appsByLocation = computed(() => {
        const grouped = {}
        allStartupApps.value.forEach(app => {
            if (!grouped[app.location]) {
                grouped[app.location] = []
            }
            grouped[app.location].push(app)
        })
        return grouped
    })

    const stats = computed(() => ({
        total: allStartupApps.value.length,
        enabled: allStartupApps.value.filter(a => a.enabled).length,
        disabled: allStartupApps.value.filter(a => !a.enabled).length,
        fromRegistry: allStartupApps.value.filter(a =>
            a.location === 'HkeyLocalMachine' || a.location === 'HkeyCurrentUser'
        ).length,
        fromFolder: allStartupApps.value.filter(a =>
            a.location === 'StartupFolder'
        ).length,
    }))

    const loadStartupApps = async () => {
        try {
            loading.value = true
            error.value = null
            const apps = await getStartupApps()
            allStartupApps.value = apps
        } catch (err) {
            error.value = err.message
            console.error('Failed to load startup apps:', err)
        } finally {
            loading.value = false
        }
    }

    const addApp = async (app) => {
        try {
            loading.value = true
            await addStartupApp(app)
            await loadStartupApps()
        } catch (err) {
            error.value = err.message
            throw err
        } finally {
            loading.value = false
        }
    }

    const removeApp = async (app) => {
        try {
            loading.value = true
            await removeStartupApp(app)
            await loadStartupApps()
        } catch (err) {
            error.value = err.message
            throw err
        } finally {
            loading.value = false
        }
    }

    const clearFilters = () => {
        searchQuery.value = ''
        selectedLocation.value = null
        selectedStatus.value = null
    }

    return {
        allStartupApps,
        filteredApps,
        appsByLocation,
        loading,
        error,
        searchQuery,
        selectedLocation,
        selectedStatus,
        stats,
        locationOptions,
        statusOptions,
        loadStartupApps,
        addApp,
        removeApp,
        clearFilters,
    }
}
