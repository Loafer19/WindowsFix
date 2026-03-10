import { ref, watch } from 'vue'

const STATUS_ORDER = {
    Running: 0,
    Paused: 1,
    'Start Pending': 2,
    'Stop Pending': 2,
    Stopped: 3,
    Unknown: 4,
}
const STARTUP_ORDER = {
    Automatic: 0,
    Boot: 1,
    System: 2,
    Manual: 3,
    Disabled: 4,
    Unknown: 5,
}

export function useFiltering(allServices) {
    const filteredServices = ref([])
    const searchQuery = ref('')
    const selectedStatus = ref('')
    const selectedStartupType = ref('')
    const sortBy = ref('status')
    const sortDir = ref('asc')

    const loadFiltersFromStorage = () => {
        const stored = localStorage.getItem('serviceFilters')
        if (stored) {
            try {
                const filters = JSON.parse(stored)
                searchQuery.value = filters.searchQuery || ''
                selectedStatus.value = filters.selectedStatus || ''
                selectedStartupType.value = filters.selectedStartupType || ''
                sortBy.value = filters.sortBy || 'status'
                sortDir.value = filters.sortDir || 'asc'
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
            sortBy: sortBy.value,
            sortDir: sortDir.value,
        }
        localStorage.setItem('serviceFilters', JSON.stringify(filters))
    }

    watch(
        [searchQuery, selectedStatus, selectedStartupType, sortBy, sortDir],
        saveFiltersToStorage,
    )

    const compareServices = (a, b) => {
        const dir = sortDir.value === 'asc' ? 1 : -1
        if (sortBy.value === 'name') {
            return dir * a.name.localeCompare(b.name)
        }
        if (sortBy.value === 'status') {
            const diff =
                (STATUS_ORDER[a.status] ?? 99) - (STATUS_ORDER[b.status] ?? 99)
            return diff !== 0 ? dir * diff : a.name.localeCompare(b.name)
        }
        if (sortBy.value === 'startupType') {
            const diff =
                (STARTUP_ORDER[a.startupType] ?? 99) -
                (STARTUP_ORDER[b.startupType] ?? 99)
            return diff !== 0 ? dir * diff : a.name.localeCompare(b.name)
        }
        return 0
    }

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

        filtered.sort(compareServices)
        filteredServices.value = filtered
    }

    const handleFilter = (filterData) => {
        searchQuery.value = filterData.searchQuery ?? searchQuery.value
        selectedStatus.value = filterData.selectedStatus ?? selectedStatus.value
        selectedStartupType.value =
            filterData.selectedStartupType ?? selectedStartupType.value
        sortBy.value = filterData.sortBy ?? sortBy.value
        sortDir.value = filterData.sortDir ?? sortDir.value
        filterServices()
    }

    const clearFilters = () => {
        searchQuery.value = ''
        selectedStatus.value = ''
        selectedStartupType.value = ''
        sortBy.value = 'status'
        sortDir.value = 'asc'
        filterServices()
    }

    loadFiltersFromStorage()

    return {
        filteredServices,
        searchQuery,
        selectedStatus,
        selectedStartupType,
        sortBy,
        sortDir,
        filterServices,
        handleFilter,
        clearFilters,
    }
}
