import { ref } from 'vue'

export function useFiltering(allServices) {
    const filteredServices = ref([])

    const filterServices = () => {
        filteredServices.value = allServices.value
    }

    const handleFilter = (filterData) => {
        let filtered = [...allServices.value]

        if (filterData.searchQuery) {
            const query = filterData.searchQuery.toLowerCase()
            filtered = filtered.filter(
                (service) =>
                    service.name.toLowerCase().includes(query) ||
                    service.displayName.toLowerCase().includes(query),
            )
        }

        if (filterData.selectedStatus) {
            filtered = filtered.filter(
                (service) => service.status === filterData.selectedStatus,
            )
        }

        if (filterData.selectedStartupType) {
            filtered = filtered.filter(
                (service) =>
                    service.startupType === filterData.selectedStartupType,
            )
        }

        filteredServices.value = filtered
    }

    return {
        filteredServices,
        filterServices,
        handleFilter,
    }
}
