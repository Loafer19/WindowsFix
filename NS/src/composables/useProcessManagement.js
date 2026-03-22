import { computed, ref } from 'vue'
import { rustService } from '../services/rust.js'

/**
 * Composable that encapsulates all process-related state and operations.
 *
 * @returns {{
 *   processes: import('vue').Ref<import('../services/types.js').NetworkProcess[]>,
 *   loading: import('vue').Ref<boolean>,
 *   sorted: import('vue').ComputedRef<import('../services/types.js').NetworkProcess[]>,
 *   update: (newProcs: import('../services/types.js').NetworkProcess[]) => void,
 *   setLimit: (pid: number, exePath: string, bps: number) => Promise<void>,
 *   toggleBlock: (proc: import('../services/types.js').NetworkProcess) => Promise<void>,
 *   terminate: (proc: import('../services/types.js').NetworkProcess) => Promise<void>,
 * }}
 */
export function useProcessManagement() {
    /** @type {import('vue').Ref<import('../services/types.js').NetworkProcess[]>} */
    const processes = ref([])
    const loading = ref(false)

    /** Processes sorted by total bandwidth (highest first). */
    const sorted = computed(() =>
        [...processes.value].sort(
            (a, b) =>
                b.totalDownloadBytes +
                b.totalUploadBytes -
                (a.totalDownloadBytes + a.totalUploadBytes),
        ),
    )

    /**
     * Merge a fresh list from the backend into the local ref, preserving any
     * optimistic UI flags that may have been set.
     *
     * @param {import('../services/types.js').NetworkProcess[]} newProcs
     */
    function update(newProcs) {
        processes.value = newProcs.map((p) => {
            const existing =
                processes.value.find((e) => e.exePath === p.exePath) ?? {}
            return {
                ...p,
                isPending: existing.isPending ?? false,
                isTerminating: existing.isTerminating ?? false,
            }
        })
    }

    /**
     * Apply a bandwidth limit to a process.
     * On success the local cache is updated optimistically.
     *
     * @param {number} pid
     * @param {string} exePath
     * @param {number} bps - 0 removes the limit
     */
    async function setLimit(pid, exePath, bps) {
        await rustService.setProcessLimit(pid, bps)
        const found = processes.value.find((p) => p.exePath === exePath)
        if (found) found.limitBps = bps
    }

    /**
     * Toggle the block state of a process.
     * Uses `isPending` to prevent duplicate actions.
     *
     * @param {import('../services/types.js').NetworkProcess} proc
     */
    async function toggleBlock(proc) {
        proc.isPending = true
        try {
            if (proc.blocked) {
                await rustService.unblockProcess(proc.pid)
                proc.blocked = false
            } else {
                await rustService.blockProcess(proc.pid)
                proc.blocked = true
            }
        } catch {
            // Error toast already shown by rustService; rollback flag
        } finally {
            proc.isPending = false
        }
    }

    /**
     * Terminate a process and remove it from the local list on success.
     *
     * @param {import('../services/types.js').NetworkProcess} proc
     */
    async function terminate(proc) {
        proc.isTerminating = true
        try {
            await rustService.killProcess(proc.pid)
            processes.value = processes.value.filter(
                (p) => p.exePath !== proc.exePath,
            )
        } catch {
            // Error toast already shown by rustService
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
