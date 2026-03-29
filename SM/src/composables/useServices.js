import { ref, computed, watch } from 'vue'
import {
    loadServices,
    refreshServices,
    reloadServiceInfo,
    restartService,
    setStartupType,
    startService,
    stopService,
    disableService,
} from '../services/api.js'

export function useServices() {
    const allServices = ref([])
    const error = ref(false)
    const loading = ref(true)

    const loadServicesData = async () => {
        try {
            loading.value = true
            error.value = false
            const data = await loadServices()
            // Raw data is stored unsorted
            allServices.value = data
        } catch (err) {
            console.error('Failed to load services:', err)
            error.value = true
        } finally {
            loading.value = false
        }
    }

    const refresh = async () => {
        try {
            loading.value = true
            await refreshServices()
            await loadServicesData()
        } catch (err) {
            console.error('Failed to refresh services:', err)
            loading.value = false
        }
    }

    const reloadInfo = async (service) => {
        service.isReloading = true

        try {
            const data = await reloadServiceInfo(service.name)
            service.info = data

            const originalService = allServices.value.find(
                (s) => s.name === service.name,
            )
            if (originalService) {
                originalService.info = data
            }

            console.log(
                `Successfully reloaded information for service: ${service.name}`,
            )
        } catch (error) {
            console.error('Failed to reload service info:', error)
            service.info = { error: true, message: 'Failed to reload information' }

            const originalService = allServices.value.find(
                (s) => s.name === service.name,
            )
            if (originalService) {
                originalService.info = service.info
            }
        } finally {
            service.isReloading = false
        }
    }

    const handleServiceAction = async (service, payload) => {
        service.isActioning = true
        service.actionError = ''

        try {
            let updated = null
            let actionLabel = ''

            if (payload.action === 'start') {
                updated = await startService(service.name)
                actionLabel = 'Started'
            } else if (payload.action === 'stop') {
                updated = await stopService(service.name)
                actionLabel = 'Stopped'
            } else if (payload.action === 'restart') {
                updated = await restartService(service.name)
                actionLabel = 'Restarted'
            } else if (payload.action === 'startup') {
                updated = await setStartupType(service.name, payload.startupType)
                actionLabel = 'Startup Type Changed'
            }

            if (updated) {
                service.status = updated.status
                service.startupType = updated.startupType

                const original = allServices.value.find(
                    (s) => s.name === service.name,
                )
                if (original) {
                    original.status = updated.status
                    original.startupType = updated.startupType
                }


            }
        } catch (err) {
            console.error('Service action failed:', err)
            service.actionError = err?.message || String(err)
        } finally {
            service.isActioning = false
        }
    }

    const disable = async (service) => {
        service.isDisabling = true

        try {
            const data = await disableService(service.name)
            Object.assign(service, data)
        } catch (error) {
            console.error('Failed to disable service:', error)
            service.info = { error: true, message: 'Failed to disable service' }
        } finally {
            service.isDisabling = false
        }
    }

    const totalServices = computed(() => allServices.value.length)

    const servicesByStatus = computed(() => {
        const counts = {}
        allServices.value.forEach((service) => {
            counts[service.status] = (counts[service.status] || 0) + 1
        })
        return counts
    })

    const servicesByStartupType = computed(() => {
        const counts = {}
        allServices.value.forEach((service) => {
            counts[service.startupType] = (counts[service.startupType] || 0) + 1
        })
        return counts
    })

    const filteredServices = ref([])
    const searchQuery = ref('')
    const selectedStatus = ref('')
    const selectedStartupType = ref('')

    const loadFiltersFromStorage = () => {
        const stored = localStorage.getItem('serviceFilters')
        if (stored) {
            try {
                const filters = JSON.parse(stored)
                searchQuery.value = filters.searchQuery || ''
                selectedStatus.value = filters.selectedStatus || ''
                selectedStartupType.value = filters.selectedStartupType || ''
            } catch (e) {
                console.warn('Failed to parse stored filters:', e)
            }
        }
    }

    const saveFiltersToStorage = () => {
        const filters = {
            searchQuery: searchQuery.value,
            selectedStatus: selectedStatus.value,
            selectedStartupType: selectedStartupType.value,
        }
        localStorage.setItem('serviceFilters', JSON.stringify(filters))
    }

    watch(
        [searchQuery, selectedStatus, selectedStartupType],
        () => {
            saveFiltersToStorage()
            filterServices()
        },
    )

    const filterServices = () => {
        let filtered = [...allServices.value]

        if (searchQuery.value) {
            const query = searchQuery.value.toLowerCase()
            filtered = filtered.filter(
                (service) =>
                    service.name.toLowerCase().includes(query) ||
                    service.displayName.toLowerCase().includes(query),
            )
        }

        if (selectedStatus.value) {
            filtered = filtered.filter(
                (service) => service.status === selectedStatus.value,
            )
        }

        if (selectedStartupType.value) {
            filtered = filtered.filter(
                (service) => service.startupType === selectedStartupType.value,
            )
        }

        filteredServices.value = filtered
    }

    watch(allServices, filterServices)

    const handleFilter = (filterData) => {
        searchQuery.value = filterData.searchQuery ?? searchQuery.value
        selectedStatus.value = filterData.selectedStatus ?? selectedStatus.value
        selectedStartupType.value =
            filterData.selectedStartupType ?? selectedStartupType.value
        filterServices()
    }

    const clearFilters = () => {
        searchQuery.value = ''
        selectedStatus.value = ''
        selectedStartupType.value = ''
        filterServices()
    }

    loadFiltersFromStorage()
    filterServices()

    return {
        allServices,
        error,
        loading,
        loadServicesData,
        refresh,
        reloadInfo,
        handleServiceAction,
        disable,
        totalServices,
        servicesByStatus,
        servicesByStartupType,
        filteredServices,
        searchQuery,
        selectedStatus,
        selectedStartupType,
        filterServices,
        handleFilter,
        clearFilters,
    }
}
