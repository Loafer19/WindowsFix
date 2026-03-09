<template>
    <div>
        <div class="flex items-center gap-2 mb-4">
            <input v-model="searchQuery" type="text" class="input input-bordered flex-1" placeholder="Search by process name or PID..." />
            <Button class="btn btn-ghost btn-square" @clicked="searchQuery = ''"><Icon name="refresh" /></Button>
        </div>
        <div v-if="filtered.length === 0" class="text-center py-12">
            <h3 class="text-lg font-bold text-base-content">No processes with network activity</h3>
            <p class="mt-1 text-base-content/70">Waiting for network activity...</p>
        </div>
        <div v-else class="overflow-x-auto">
            <table class="table">
                <thead><tr>
                    <th>Process</th><th>PID</th><th>Download</th><th>Upload</th><th>Total Download</th><th>Total Upload</th>
                    <th class="min-w-36">Throttle (KB/s)</th><th>Actions</th>
                </tr></thead>
                <tbody>
                    <tr v-for="proc in filtered" :key="proc.pid" :class="{ 'opacity-50': proc.blocked }">
                        <td class="font-medium"><span :class="proc.blocked ? 'line-through text-error' : ''">{{ proc.name }}</span></td>
                        <td class="text-base-content/70 font-mono text-sm">{{ proc.pid }}</td>
                        <td><span class="badge badge-info font-mono">{{ formatSpeed(proc.downloadBps) }}</span></td>
                        <td><span class="badge badge-success font-mono">{{ formatSpeed(proc.uploadBps) }}</span></td>
                        <td><span class="badge badge-info font-mono">{{ formatBytes(proc.totalDownloadBytes) }}</span></td>
                        <td><span class="badge badge-success font-mono">{{ formatBytes(proc.totalUploadBytes) }}</span></td>
                        <td>
                            <div class="flex items-center gap-1">
                                <input type="number" class="input input-bordered input-sm w-24 font-mono" min="0" placeholder="no limit"
                                    :value="proc.limitBps ? Math.round(proc.limitBps / 1024) : ''"
                                    @change="onThrottleChange(proc, $event)" @keydown.enter="$event.target.blur()" />
                                <span class="text-xs text-base-content/50">KB/s</span>
                            </div>
                        </td>
                        <td>
                            <div class="flex items-center gap-1">
                                <div class="tooltip" :data-tip="proc.blocked ? 'Unblock' : 'Block traffic'">
                                    <Button :class="proc.blocked ? 'btn btn-error btn-sm btn-square' : 'btn btn-neutral btn-sm btn-square'"
                                        :is-loading="proc.isPending" :disabled="proc.isPending" @clicked="emit('block-toggle', proc)">
                                        <Icon :name="proc.blocked ? 'unblock' : 'block'" />
                                    </Button>
                                </div>
                                <div class="tooltip" data-tip="Free ports (close TCP connections)">
                                    <Button class="btn btn-warning btn-sm btn-square" :is-loading="proc.isFreeing"
                                        :disabled="proc.isFreeing || proc.isPending" @clicked="emit('free-ports', proc)">
                                        <Icon name="unplug" />
                                    </Button>
                                </div>
                                <div class="tooltip" data-tip="Terminate process">
                                    <Button class="btn btn-error btn-sm btn-square" :is-loading="proc.isTerminating"
                                        :disabled="proc.isTerminating || proc.isPending" @clicked="emit('terminate', proc)">
                                        <Icon name="kill" />
                                    </Button>
                                </div>
                            </div>
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>
    </div>
</template>

<script setup>
import { computed, ref } from 'vue'
import Button from '../Button.vue'
import Icon from '../Icon.vue'
import { formatSpeed, formatBytes } from '../../composables/useNetwork.js'

const props = defineProps({
    processes: { type: Array, default: () => [] },
})

const emit = defineEmits(['block-toggle', 'terminate', 'free-ports', 'throttle'])

const searchQuery = ref('')

const filtered = computed(() => {
    const q = searchQuery.value.toLowerCase()
    if (!q) return props.processes
    return props.processes.filter(
        (p) => p.name.toLowerCase().includes(q) || String(p.pid).includes(q),
    )
})

function onThrottleChange(proc, event) {
    const kb = Number(event.target.value)
    const bps = kb > 0 ? kb * 1024 : 0
    emit('throttle', { proc, bps })
}
</script>
