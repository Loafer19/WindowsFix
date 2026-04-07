import { computed, ref } from 'vue'
import { rustService } from '../services/rust.js'

export function useProcessManagement() {
    const processes = ref([])
    const loading = ref(false)

    const sorted = computed(() =>
        [...processes.value].sort(
            (a, b) =>
                b.totalDownloadBytes + b.totalUploadBytes -
                (a.totalDownloadBytes + a.totalUploadBytes),
        ),
    )

    const update = (newProcs) => {
        processes.value = newProcs.map((p) => {
            const existing = processes.value.find((e) => e.exePath === p.exePath) ?? {}
            return {
                ...p,
                isPending: existing.isPending ?? false,
                isTerminating: existing.isTerminating ?? false,
            }
        })
    }

    const setLimit = async (pid, exePath, bps) => {
        await rustService.setProcessLimit(pid, bps)
        const found = processes.value.find((p) => p.exePath === exePath)
        if (found) found.limitBps = bps
    }

    const toggleBlock = async (proc) => {
        proc.isPending = true
        try {
            if (proc.blocked) {
                await rustService.unblockProcess(proc.pid)
                proc.blocked = false
            } else {
                await rustService.blockProcess(proc.pid)
                proc.blocked = true
            }
        } finally {
            proc.isPending = false
        }
    }

    const terminate = async (proc) => {
        proc.isTerminating = true
        try {
            await rustService.killProcess(proc.pid)
            processes.value = processes.value.filter((p) => p.exePath !== proc.exePath)
        } finally {
            proc.isTerminating = false
        }
    }

    return {
        processes,
        loading,
        sorted,
        update,
        setLimit,
        toggleBlock,
        terminate,
    }
}
