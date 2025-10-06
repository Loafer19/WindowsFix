import { ref } from 'vue'

export function useModals() {
    const showModal = ref(false)
    const selectedService = ref(null)
    const showDetailsModal = ref(false)
    const selectedServiceForDetails = ref(null)

    const openModal = (service) => {
        selectedService.value = service
        showModal.value = true
    }

    const confirmDisable = (disableCallback) => {
        if (selectedService.value) {
            disableCallback(selectedService.value)
            showModal.value = false
            selectedService.value = null
        }
    }

    const openModalForDetails = (service) => {
        selectedServiceForDetails.value = service
        showDetailsModal.value = true
    }

    return {
        showModal,
        selectedService,
        showDetailsModal,
        selectedServiceForDetails,
        openModal,
        confirmDisable,
        openModalForDetails,
    }
}
