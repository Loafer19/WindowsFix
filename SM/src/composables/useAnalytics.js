import { computed } from 'vue'

export function useAnalytics(allServices) {
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

    return {
        totalServices,
        servicesByStatus,
        servicesByStartupType,
    }
}
