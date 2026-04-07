<template>
    <div class="card bg-base-100 card-border border-base-300 mb-6">
        <div class="card-body">
            <div class="collapse collapse-arrow bg-base-200">
                <input type="checkbox" checked />
                <div class="collapse-title text-lg font-semibold text-base-content">
                    Filters
                </div>
                <div class="collapse-content">
                    <div class="mb-4">
                        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                            <div class="bg-base-100 rounded-lg p-4">
                                <div class="capitalize font-medium">Showing</div>
                                <div class="text-2xl text-primary">{{ history.length }}</div>
                                <div class="text-base-content/70">
                                    {{ calcPercentage(history.length, allHistory.length) }}% of total
                                </div>
                                <progress class="progress progress-primary mt-2"
                                    :value="calcPercentage(history.length, allHistory.length)"
                                    max="100"></progress>
                            </div>
                        </div>
                    </div>

                    <div class="flex gap-2">
                        <button :class="`btn btn-sm ${filterType === null ? 'btn-primary' : 'btn-ghost'}`"
                            @click="setFilterType(null)">
                            All Changes
                        </button>
                        <button :class="`btn btn-sm ${filterType === 'service' ? 'btn-info' : 'btn-ghost'}`"
                            @click="setFilterType('service')">
                            Services
                        </button>
                        <button :class="`btn btn-sm ${filterType === 'startup_app' ? 'btn-info' : 'btn-ghost'}`"
                            @click="setFilterType('startup_app')">
                            Startup Apps
                        </button>
                    </div>
                </div>
            </div>

            <div class="flex justify-end mt-4">
                <Button v-if="allHistory.length > 0" class="btn btn-error btn-sm" @clicked="confirmClear">
                    <Icon name="deleteBin" class="w-5 h-5" />
                    Clear History
                </Button>
            </div>
        </div>
    </div>

    <div v-if="loading" class="alert alert-info">
        <div class="flex items-center justify-center">
            <span class="loading loading-spinner loading-lg"></span>
        </div>
    </div>

    <div v-else-if="allHistory.length === 0" class="alert alert-warning">
        <Icon name="history" />
        <h3 class="font-bold">No changes recorded</h3>
        <div class="text-xs">Service and startup app changes will appear here</div>
    </div>

    <div v-else class="card bg-base-100 card-border border-base-300">
        <div class="overflow-x-auto">
            <table class="table">
                <thead>
                    <tr>
                        <th>Name</th>
                        <th>Type</th>
                        <th>Action</th>
                        <th>Details</th>
                        <th>Time</th>
                    </tr>
                </thead>
                <tbody>
                    <tr v-for="entry in history" :key="entry.timestamp">
                        <td class="font-medium">
                            {{ entry.serviceName || entry.appName || entry.name || 'Unknown' }}
                        </td>

                        <td>
                            <div class="badge badge-neutral text-nowrap">
                                {{ entry.type === 'service' ? 'Service' :
                                    entry.type === 'startupApp' ? 'Startup App' : entry.type || 'Unknown' }}
                            </div>
                        </td>

                        <td>
                            <div :class="`badge badge-${getActionColor(entry.action)}`">
                                {{ entry.action || 'Unknown' }}
                            </div>
                        </td>

                        <td class="text-sm">
                            <div v-if="entry.type === 'service'">
                                <div v-if="entry.newValue && entry.action !== 'set_startup_type'"
                                    class="flex items-center gap-1">
                                    <span v-if="entry.oldValue"
                                        :class="`badge badge-xs badge-${getStatusColor(entry.oldValue)}`">
                                        {{ entry.oldValue }}
                                    </span>
                                    <span v-if="entry.oldValue" class="text-base-content/50">→</span>
                                    <span :class="`badge badge-xs badge-${getStatusColor(entry.newValue)}`">
                                        {{ entry.newValue }}
                                    </span>
                                </div>

                                <div v-if="entry.newValue && entry.action === 'set_startup_type'"
                                    class="flex items-center gap-1 mt-1">
                                    <span v-if="entry.oldValue"
                                        :class="`badge badge-xs badge-${getStartupTypeColor(entry.oldValue)}`">
                                        {{ entry.oldValue }}
                                    </span>
                                    <span v-if="entry.oldValue" class="text-base-content/50">→</span>
                                    <span :class="`badge badge-xs badge-${getStartupTypeColor(entry.newValue)}`">
                                        {{ entry.newValue }}
                                    </span>
                                </div>
                            </div>

                            <div v-else-if="entry.type === 'startupApp'" class="space-y-1">
                                <div class="flex items-center gap-2">
                                    <span class="badge badge-xs">{{ entry.location }}</span>
                                </div>
                                <div v-if="entry.command" class="text-base-content/70 text-xs mt-1">
                                    <code class="bg-base-200 p-1 rounded break-all">{{ entry.command }}</code>
                                </div>
                            </div>
                        </td>

                        <td class="text-sm text-base-content/50 tooltip"
                            :data-tip="new Date(entry.timestamp * 1000).toLocaleString()">
                            {{ formatTime(entry.timestamp * 1000) }}
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>
    </div>

    <ConfirmModal :show="showConfirmClear" title="Clear History?"
        message="This will delete all recorded changes. This action cannot be undone." confirm-text="Clear"
        @close="showConfirmClear = false" @confirm="handleClearHistory" />
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useHistory } from '../../composables/useHistory.js'
import Button from '../Button.vue'
import Icon from '../Icon.vue'
import ConfirmModal from '../Modals/ConfirmModal.vue'
import { getStatusColor, getStartupTypeColor, formatTime, getActionColor, calcPercentage } from '../../services/helpers.js'

const {
    history,
    allHistory,
    loading,
    filterType,
    setFilterType,
    loadHistory,
    clearHistory,
} = useHistory()

onMounted(() => {
    loadHistory()
})

const showConfirmClear = ref(false)

const confirmClear = () => {
    showConfirmClear.value = true
}

const handleClearHistory = async () => {
    try {
        await clearHistory()
        showConfirmClear.value = false
    } catch (err) {
        console.error('Failed to clear history:', err)
    }
}
</script>
