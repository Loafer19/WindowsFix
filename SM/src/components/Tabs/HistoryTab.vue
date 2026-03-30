<template>
    <div class="card bg-base-100 card-border border-base-300 mb-6">
        <div class="card-body">
            <div class="collapse collapse-arrow bg-base-200 mb-4">
                <input type="checkbox" />
                <div class="collapse-title text-lg font-semibold text-base-content">
                    Analytics
                </div>
                <div class="collapse-content">
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                        <div class="bg-base-100 rounded-lg p-4">
                            <div class="text-sm font-medium text-base-content/70">Total Changes</div>
                            <div class="text-3xl font-bold text-primary">{{ stats.total }}</div>
                            <progress class="progress progress-primary h-2 mt-2" :value="stats.total"
                                max="100"></progress>
                        </div>
                        <div class="bg-base-200 rounded-lg p-4">
                            <div class="text-sm font-medium text-base-content/70">Service Changes</div>
                            <div class="text-3xl font-bold text-info">{{ stats.services }}</div>
                            <div class="text-xs text-base-content/50 mt-1">{{ ((stats.services / (stats.total || 1)) *
                                100).toFixed(1) }}%</div>
                        </div>
                        <div class="bg-base-200 rounded-lg p-4">
                            <div class="text-sm font-medium text-base-content/70">Startup App Changes</div>
                            <div class="text-3xl font-bold text-success">{{ stats.startupApps }}</div>
                            <div class="text-xs text-base-content/50 mt-1">{{ ((stats.startupApps / (stats.total || 1))
                                * 100).toFixed(1) }}%</div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="collapse collapse-arrow bg-base-200">
                <input type="checkbox" checked />
                <div class="collapse-title text-lg font-semibold text-base-content">
                    Filters
                </div>
                <div class="collapse-content">
                    <div class="flex gap-2">
                        <button :class="`btn btn-sm ${filterType === null ? 'btn-primary' : 'btn-ghost'}`"
                            @click="setFilterType(null)">
                            All Changes
                        </button>
                        <button :class="`btn btn-sm ${filterType === 'service' ? 'btn-info' : 'btn-ghost'}`"
                            @click="setFilterType('service')">
                            Services
                        </button>
                        <button :class="`btn btn-sm ${filterType === 'startup_app' ? 'btn-success' : 'btn-ghost'}`"
                            @click="setFilterType('startup_app')">
                            Startup Apps
                        </button>
                    </div>
                </div>
            </div>

            <div class="flex justify-end mt-4">
                <Button v-if="allHistory.length > 0" class="btn btn-error btn-sm" @clicked="confirmClear">
                    <Icon name="trash" class="w-4 h-4" />
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
                        <!-- Name -->
                        <td class="font-medium">
                            {{ entry.serviceName || entry.appName || entry.name || 'Unknown' }}
                        </td>

                        <!-- Type Badge -->
                        <td>
                            <div
                                :class="`badge ${entry.type === 'Service' ? 'badge-info' : entry.type === 'StartupApp' ? 'badge-success' : 'badge-neutral'}`">
                                {{ entry.type === 'Service' ? 'Service' : entry.type === 'StartupApp' ? 'Startup App' :
                                    entry.type || 'Unknown' }}
                            </div>
                        </td>

                        <!-- Action Badge -->
                        <td>
                            <div :class="`badge badge-${getActionColor(entry.action)}`">
                                {{ entry.action || 'Unknown' }}
                            </div>
                        </td>

                        <!-- Details -->
                        <td class="text-sm">
                            <div v-if="entry.type === 'Service'">
                                <!-- Service Details -->
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

                            <div v-else-if="entry.type === 'StartupApp'" class="space-y-1">
                                <!-- Startup App Details -->
                                <div class="flex items-center gap-2">
                                    <span class="badge badge-xs">{{ entry.location }}</span>
                                </div>
                            </div>
                        </td>

                        <!-- Timestamp -->
                        <td class="text-sm text-base-content/50"
                            :title="new Date(entry.timestamp * 1000).toLocaleString()">
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
import { getStatusColor, getStartupTypeColor } from '../../services/helpers.js'

const {
    history,
    allHistory,
    loading,
    stats,
    filterType,
    setFilterType,
    loadHistory,
    clearHistory,
} = useHistory()

// Load history on mount
onMounted(() => {
    // Delay loading to allow UI to render first
    setTimeout(async () => {
        await loadHistory()
    }, 100)
})

const showConfirmClear = ref(false)

const getActionColor = (action) => {
    const map = {
        'Started': 'success',
        'Stopped': 'warning',
        'Restarted': 'info',
        'Disabled': 'error',
        'Added': 'success',
        'Removed': 'error',
    }
    return map[action] || 'neutral'
}

const truncatePath = (path, maxLen = 50) => {
    return path.length > maxLen ? `${path.substring(0, maxLen)}...` : path
}

const formatTime = (timestamp) => {
    const date = new Date(timestamp)
    const now = new Date()
    const diff = now - date

    // Less than a minute
    if (diff < 60000) return 'just now'
    // Less than an hour
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`
    // Less than a day
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`
    // Otherwise show date
    return date.toLocaleDateString()
}

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
