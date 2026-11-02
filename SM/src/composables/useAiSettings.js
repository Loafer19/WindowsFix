import { ref } from 'vue'
import { errorMessage, getAiStatus } from '../services/api.js'

// Shared (module-level) so the Settings tab and the service details modal
// always agree on whether a key is configured
const aiStatus = ref(null)
const aiStatusError = ref('')

async function loadAiStatus() {
    try {
        aiStatus.value = await getAiStatus()
        aiStatusError.value = ''
    } catch (err) {
        console.error('Failed to load AI status:', err)
        aiStatusError.value = errorMessage(err, 'Failed to load AI settings')
    }
    return aiStatus.value
}

function setAiStatus(status) {
    aiStatus.value = status
}

export function useAiSettings() {
    return { aiStatus, aiStatusError, loadAiStatus, setAiStatus }
}
