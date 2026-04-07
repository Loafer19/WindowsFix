import { ref } from 'vue'
import { calcSpeed, calcBytes } from '../services/helpers.js'

const HISTORY_POINTS = 60

function makeLabels() {
    return Array.from({ length: HISTORY_POINTS }, (_, i) => {
        const offset = HISTORY_POINTS - 1 - i
        return offset === 0 ? 'now' : `-${offset}s`
    })
}

export function useNetwork() {
    const downloadHistory = ref(Array(HISTORY_POINTS).fill(0))
    const uploadHistory = ref(Array(HISTORY_POINTS).fill(0))
    const labels = ref(makeLabels())

    const pushStats = (downloadBps, uploadBps) => {
        downloadHistory.value = [...downloadHistory.value.slice(1), downloadBps]
        uploadHistory.value = [...uploadHistory.value.slice(1), uploadBps]
    }

    return {
        downloadHistory,
        uploadHistory,
        labels,
        pushStats,
    }
}

export { calcSpeed, calcBytes }
