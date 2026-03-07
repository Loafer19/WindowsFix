<template>
    <div>
        <div class="flex items-center gap-2 mb-4">
            <input
                v-model="searchQuery"
                type="text"
                class="input input-bordered flex-1"
                placeholder="Search by process name or PID…"
            />
            <Button class="btn btn-ghost btn-square" @clicked="searchQuery = ''">
                <Icon name="refresh" />
            </Button>
        </div>

        <div v-if="filtered.length === 0" class="text-center py-12">
            <h3 class="text-lg font-bold text-base-content">No active network processes</h3>
            <p class="mt-1 text-base-content/70">Waiting for network activity…</p>
        </div>

        <div v-else class="overflow-x-auto">
            <table class="table">
                <thead>
                    <tr>
                        <th>Process</th>
                        <th>PID</th>
                        <th>Download</th>
                        <th>Upload</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>
                    <tr v-for="proc in filtered" :key="proc.pid">
                        <td class="font-medium text-base-content">{{ proc.name }}</td>
                        <td class="text-base-content/70">{{ proc.pid }}</td>
                        <td>
                            <span class="badge badge-info font-mono">{{ formatSpeed(proc.downloadBps) }}</span>
                        </td>
                        <td>
                            <span class="badge badge-success font-mono">{{ formatSpeed(proc.uploadBps) }}</span>
                        </td>
                        <td>
                            <div class="flex items-center gap-2">
                                <Button
                                    :class="proc.blocked ? 'btn btn-error btn-sm btn-square' : 'btn btn-neutral btn-sm btn-square'"
                                    :is-loading="proc.isPending"
                                    :disabled="proc.isPending"
                                    @clicked="toggleBlock(proc)"
                                >
                                    <Icon :name="proc.blocked ? 'unblock' : 'block'" />
                                </Button>
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

const props = defineProps({
    processes: { type: Array, default: () => [] },
    formatSpeed: { type: Function, required: true },
})

const emit = defineEmits(['block', 'unblock'])

const searchQuery = ref('')

const filtered = computed(() => {
    const q = searchQuery.value.toLowerCase()
    if (!q) return props.processes
    return props.processes.filter(
        (p) => p.name.toLowerCase().includes(q) || String(p.pid).includes(q),
    )
})

function toggleBlock(proc) {
    emit(proc.blocked ? 'unblock' : 'block', proc)
}
</script>
