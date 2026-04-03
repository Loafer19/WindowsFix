import { ref, computed } from 'vue'
import {
    getStartupApps,
    addStartupApp,
    removeStartupApp,
    toggleStartupApp,
} from '../services/api.js'

export function useStartupApps() {
    const allStartupApps = ref([])
    const loading = ref(false)
    const error = ref(null)

    const searchQuery = ref('')
    const selectedLocation = ref('')
    const selectedStatus = ref('')

    const locationOptions = [
        { value: 'hkeyLocalMachine', label: 'HKLM Registry (System)' },
        { value: 'hkeyCurrentUser', label: 'HKCU Registry (User)' },
        { value: 'startupFolder', label: 'Startup Folder' },
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

            const matchesStatus = !selectedStatus.value ||
                app.enabled === (selectedStatus.value === 'true')

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

    const toggleApp = async (app) => {
        try {
            loading.value = true
            await toggleStartupApp(app)
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
        selectedLocation.value = ''
        selectedStatus.value = ''
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
        toggleApp,
        clearFilters,
    }
}
