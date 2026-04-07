<template>
    <div>
        <div class="flex items-center gap-2 mb-4">
            <input v-model="searchQuery" type="text" class="input input-bordered flex-1" placeholder="Search by process name, path or PID..." />
            <Button class="btn btn-ghost btn-square" @clicked="searchQuery = ''"><Icon name="filterOff" /></Button>
        </div>
        <div v-if="filtered.length === 0" class="text-center py-12">
            <h3 class="text-lg font-bold text-base-content">No processes with network activity</h3>
            <p class="mt-1 text-base-content/70">Waiting for network activity...</p>
        </div>
        <div v-else class="overflow-x-auto">
            <table class="table">
                <thead>
                    <tr>
                        <th>
                            <button class="flex items-center gap-1 hover:text-primary transition-colors" @click="setSort('name')">
                                Process
                                <Icon :name="sortIcon('name')" class="w-3 h-3" />
                            </button>
                        </th>
                        <th>
                            <button class="flex items-center gap-1 hover:text-primary transition-colors" @click="setSort('downloadBps')">
                                Download
                                <Icon :name="sortIcon('downloadBps')" class="w-3 h-3" />
                            </button>
                        </th>
                        <th>
                            <button class="flex items-center gap-1 hover:text-primary transition-colors" @click="setSort('uploadBps')">
                                Upload
                                <Icon :name="sortIcon('uploadBps')" class="w-3 h-3" />
                            </button>
                        </th>
                        <th>
                            <button class="flex items-center gap-1 hover:text-primary transition-colors" @click="setSort('totalDownloadBytes')">
                                Total DL
                                <Icon :name="sortIcon('totalDownloadBytes')" class="w-3 h-3" />
                            </button>
                        </th>
                        <th>
                            <button class="flex items-center gap-1 hover:text-primary transition-colors" @click="setSort('totalUploadBytes')">
                                Total UL
                                <Icon :name="sortIcon('totalUploadBytes')" class="w-3 h-3" />
                            </button>
                        </th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>
                    <tr
                        v-for="{ proc, procInfo } in sortedFiltered.map(proc => ({ proc, procInfo: getProcInfo(proc) }))"
                        :key="proc.exePath"
                        class="cursor-pointer hover"
                        :class="procInfo.rowClass"
                        @click.stop="openModal(proc)"
                    >
                        <td>
                            <div class="flex flex-col">
                                <span class="font-medium" :class="procInfo.nameClass">{{ proc.name }}</span>
                            </div>
                        </td>
                        <td>
                            <span v-if="procInfo.isThrottled" class="tooltip inline-block w-2 h-2 bg-error rounded-full me-1" :data-tip="procInfo.throttleTip"></span>
                            <span class="badge badge-primary font-mono whitespace-nowrap">{{ calcSpeed(proc.downloadBps) }}</span>
                        </td>
                        <td>
                            <span v-if="procInfo.isThrottled" class="tooltip inline-block w-2 h-2 bg-error rounded-full me-1" :data-tip="procInfo.throttleTip"></span>
                            <span class="badge badge-info font-mono whitespace-nowrap">{{ calcSpeed(proc.uploadBps) }}</span>
                        </td>
                        <td><span class="badge badge-primary font-mono whitespace-nowrap">{{ calcBytes(proc.totalDownloadBytes) }}</span></td>
                        <td><span class="badge badge-info font-mono whitespace-nowrap">{{ calcBytes(proc.totalUploadBytes) }}</span></td>
                        <td @click.stop>
                            <div class="flex items-center gap-1">
                                <div class="tooltip" :data-tip="procInfo.btnTip">
                                    <Button :class="procInfo.btnClass"
                                        :is-loading="proc.isPending" :disabled="proc.isPending" @clicked="emit('block-toggle', proc)">
                                        <Icon :name="procInfo.icon" />
                                    </Button>
                                </div>
                                <div v-if="proc.pid" class="tooltip tooltip-left" data-tip="Terminate process">
                                    <Button class="btn btn-error btn-sm btn-square" :is-loading="proc.isTerminating"
                                        :disabled="proc.isTerminating || proc.isPending" @clicked="emit('terminate', proc)">
                                        <Icon name="deleteBin2" />
                                    </Button>
                                </div>
                            </div>
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>

        <ProcessModal
            v-if="modalProc"
            :proc="modalProc"
            @close="modalProc = null"
            @throttle="(e) => { emit('throttle', e); modalProc = null }"
        />
    </div>
</template>

<script setup>
import { computed, ref } from 'vue'
import { calcSpeed, calcBytes, getBlockStatusInfo, sortIcon as getSortIcon } from '../../services/helpers.js'
import Button from '../Button.vue'
import Icon from '../Icon.vue'
import ProcessModal from '../Modals/ProcessModal.vue'

const props = defineProps({
    processes: { type: Array, default: () => [] },
})

const emit = defineEmits(['block-toggle', 'terminate', 'throttle'])

const searchQuery = ref('')
const sortField = ref('totalDownloadBytes')
const sortDir = ref('desc')
const modalProc = ref(null)

const filtered = computed(() => {
    const q = searchQuery.value.toLowerCase()
    if (!q) return props.processes
    return props.processes.filter(
        (p) =>
            p.name.toLowerCase().includes(q) ||
            p.exePath.toLowerCase().includes(q) ||
            String(p.pid).includes(q),
    )
})

const sortedFiltered = computed(() => {
    const list = [...filtered.value]
    const field = sortField.value
    const dir = sortDir.value === 'asc' ? 1 : -1
    list.sort((a, b) => {
        const av = field === 'name' ? a[field] : (a[field] ?? 0)
        const bv = field === 'name' ? b[field] : (b[field] ?? 0)
        if (field === 'name') return dir * av.localeCompare(bv)
        return dir * (Number(av) - Number(bv))
    })
    return list
})

const setSort = (field) => {
    if (sortField.value === field) {
        sortDir.value = sortDir.value === 'asc' ? 'desc' : 'asc'
    } else {
        sortField.value = field
        sortDir.value = 'desc'
    }
}

const sortIcon = (field) => getSortIcon(field, sortField.value, sortDir.value)

const getProcInfo = (proc) => {
    const info = getBlockStatusInfo(proc)
    const isThrottled = !!proc.limitBps
    return {
        ...info,
        isThrottled,
        throttleTip: isThrottled ? `Throttled to ${calcSpeed(proc.limitBps)}` : '',
    }
}

const openModal = (proc) => {
    modalProc.value = proc
}
</script>
