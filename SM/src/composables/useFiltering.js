import { ref, watch } from 'vue'

export function useFiltering(allServices) {
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
        saveFiltersToStorage,
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

    const handleFilter = (filterData) => {
        searchQuery.value = filterData.searchQuery || ''
        selectedStatus.value = filterData.selectedStatus || ''
        selectedStartupType.value = filterData.selectedStartupType || ''
        filterServices()
    }

    const clearFilters = () => {
        searchQuery.value = ''
        selectedStatus.value = ''
        selectedStartupType.value = ''
        filterServices()
    }

    loadFiltersFromStorage()

    return {
        filteredServices,
        searchQuery,
        selectedStatus,
        selectedStartupType,
        filterServices,
        handleFilter,
        clearFilters,
    }
}
