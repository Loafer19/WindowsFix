import { ref } from 'vue'

const HISTORY_POINTS = 60

export function formatSpeed(bps) {
    if (bps >= 1_048_576) return `${(bps / 1_048_576).toFixed(1)} MB/s`
    if (bps >= 1024) return `${(bps / 1024).toFixed(1)} KB/s`
    return `${bps} B/s`
}

export function formatBytes(bytes) {
    if (bytes >= 1_073_741_824)
        return `${(bytes / 1_073_741_824).toFixed(2)} GB`
    if (bytes >= 1_048_576) return `${(bytes / 1_048_576).toFixed(1)} MB`
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(1)} KB`
    return `${bytes} B`
}

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

    function pushStats(downloadBps, uploadBps) {
        downloadHistory.value = [...downloadHistory.value.slice(1), downloadBps]
        uploadHistory.value = [...uploadHistory.value.slice(1), uploadBps]
        // Labels represent relative time offsets and stay constant;
        // the data window shifts left on every push.
    }

    return {
        downloadHistory,
        uploadHistory,
        labels,
        pushStats,
    }
}
