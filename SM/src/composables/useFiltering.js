import { ref, watch } from 'vue'

export function useFiltering(allServices) {
    const searchQuery = ref('')
    const selectedStatus = ref('')
    const selectedStartupType = ref('')
    const filteredServices = ref([])

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
        searchQuery.value = filterData.searchQuery
        selectedStatus.value = filterData.selectedStatus
        selectedStartupType.value = filterData.selectedStartupType
        filterServices()
    }

    // Watch for filter changes
    watch([searchQuery, selectedStatus, selectedStartupType], () => {
        filterServices()
    })

    return {
        searchQuery,
        selectedStatus,
        selectedStartupType,
        filteredServices,
        filterServices,
        handleFilter,
    }
}
