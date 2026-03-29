import { ref } from 'vue'
import { disableService } from '../services/api.js'

export function usePresets(allServices) {
    const showPresetModal = ref(false)
    const selectedPreset = ref(null)
    const applyingPreset = ref(false)

    const openPresetModal = (preset) => {
        selectedPreset.value = preset
        showPresetModal.value = true
    }

    const applyPreset = async (preset) => {
        applyingPreset.value = true
        const results = []

        for (const presetSvc of preset.services) {
            const service = allServices.value.find(
                (s) =>
                    s.name === presetSvc.name ||
                    s.name.toLowerCase() === presetSvc.name.toLowerCase(),
            )
            if (!service || service.startupType === 'Disabled') continue

            try {
                const data = await disableService(service.name)
                Object.assign(service, data)
                results.push({ name: service.name, success: true })
            } catch (err) {
                console.error(`Failed to disable ${service.name}:`, err)
                results.push({ name: service.name, success: false })
            }
        }

        applyingPreset.value = false
        showPresetModal.value = false
    }

    return {
        showPresetModal,
        selectedPreset,
        applyingPreset,
        openPresetModal,
        applyPreset,
    }
}
