import { ref } from 'vue'

const HISTORY_POINTS = 60

export function useNetwork() {
    const downloadHistory = ref(Array(HISTORY_POINTS).fill(0))
    const uploadHistory = ref(Array(HISTORY_POINTS).fill(0))
    const labels = ref(Array.from({ length: HISTORY_POINTS }, (_, i) => `${HISTORY_POINTS - i}s`))

    function pushStats(downloadBps, uploadBps) {
        downloadHistory.value = [...downloadHistory.value.slice(1), downloadBps]
        uploadHistory.value = [...uploadHistory.value.slice(1), uploadBps]
    }

    function formatSpeed(bps) {
        if (bps >= 1_048_576) return `${(bps / 1_048_576).toFixed(1)} MB/s`
        if (bps >= 1024) return `${(bps / 1024).toFixed(1)} KB/s`
        return `${bps} B/s`
    }

    return {
        downloadHistory,
        uploadHistory,
        labels,
        pushStats,
        formatSpeed,
    }
}
