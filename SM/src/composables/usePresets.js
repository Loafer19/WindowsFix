import { ref } from 'vue'
import { disableService } from '../services/api.js'

export function usePresets(allServices) {
    const showPresetModal = ref(false)
    const showResultModal = ref(false)
    const selectedPreset = ref(null)
    const applyingPreset = ref(false)
    const presetResults = ref([])
    const progress = ref(0)
    const totalInPreset = ref(0)
    const totalServices = ref(0)
    const currentServiceName = ref('')

    const openPresetModal = (preset) => {
        selectedPreset.value = preset
        showPresetModal.value = true
    }

    const applyPreset = async (preset) => {
        applyingPreset.value = true
        showPresetModal.value = false
        showResultModal.value = true
        presetResults.value = []
        progress.value = 0
        currentServiceName.value = ''
        totalInPreset.value = preset.services.length

        const servicesToProcess = preset.services.filter(presetSvc => {
            const service = allServices.value.find(
                (s) =>
                    s.name === presetSvc.name ||
                    s.name.toLowerCase() === presetSvc.name.toLowerCase(),
            )
            return service && service.startupType !== 'Disabled'
        })

        totalServices.value = servicesToProcess.length

        for (let i = 0; i < servicesToProcess.length; i++) {
            const presetSvc = servicesToProcess[i]
            const service = allServices.value.find(
                (s) =>
                    s.name === presetSvc.name ||
                    s.name.toLowerCase() === presetSvc.name.toLowerCase(),
            )

            currentServiceName.value = service.name
            progress.value = i

            try {
                const data = await disableService(service.name)
                Object.assign(service, data)
                presetResults.value.push({ name: service.name, success: true })
            } catch (err) {
                console.error(`Failed to disable ${service.name}:`, err)
                presetResults.value.push({ name: service.name, success: false })
            }
        }

        progress.value = totalServices.value
        applyingPreset.value = false
        currentServiceName.value = ''
    }

    const closeResultModal = () => {
        showResultModal.value = false
        presetResults.value = []
    }

    return {
        showPresetModal,
        showResultModal,
        selectedPreset,
        applyingPreset,
        presetResults,
        progress,
        totalInPreset,
        totalServices,
        currentServiceName,
        openPresetModal,
        applyPreset,
        closeResultModal,
    }
}