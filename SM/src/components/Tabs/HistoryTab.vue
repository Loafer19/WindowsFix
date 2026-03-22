<template>
    <div>
        <!-- Stats Section -->
        <div class="mb-6">
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div class="bg-base-200 rounded-lg p-4">
                    <div class="text-sm font-medium text-base-content/70">Total Changes</div>
                    <div class="text-3xl font-bold text-primary">{{ stats.total }}</div>
                    <progress
                        class="progress progress-primary h-2 mt-2"
                        :value="stats.total" max="100"
                    ></progress>
                </div>
                <div class="bg-base-200 rounded-lg p-4">
                    <div class="text-sm font-medium text-base-content/70">Service Changes</div>
                    <div class="text-3xl font-bold text-info">{{ stats.services }}</div>
                    <div class="text-xs text-base-content/50 mt-1">{{ ((stats.services / (stats.total || 1)) * 100).toFixed(1) }}%</div>
                </div>
                <div class="bg-base-200 rounded-lg p-4">
                    <div class="text-sm font-medium text-base-content/70">Startup App Changes</div>
                    <div class="text-3xl font-bold text-success">{{ stats.startupApps }}</div>
                    <div class="text-xs text-base-content/50 mt-1">{{ ((stats.startupApps / (stats.total || 1)) * 100).toFixed(1) }}%</div>
                </div>
            </div>
        </div>

        <!-- Filter Buttons -->
        <div class="flex items-center justify-between mb-4 gap-2 flex-wrap">
            <div class="flex gap-2">
                <button
                    :class="`btn btn-sm ${filterType === null ? 'btn-primary' : 'btn-ghost'}`"
                    @click="setFilterType(null)"
                >
                    All Changes
                </button>
                <button
                    :class="`btn btn-sm ${filterType === 'service' ? 'btn-info' : 'btn-ghost'}`"
                    @click="setFilterType('service')"
                >
                    Services
                </button>
                <button
                    :class="`btn btn-sm ${filterType === 'startup_app' ? 'btn-success' : 'btn-ghost'}`"
                    @click="setFilterType('startup_app')"
                >
                    Startup Apps
                </button>
            </div>

            <Button
                v-if="allHistory.length > 0"
                class="btn btn-error btn-sm"
                @clicked="confirmClear"
            >
                <Icon name="trash" class="w-4 h-4" />
                Clear History
            </Button>
        </div>

        <!-- Loading State -->
        <div v-if="loading" class="text-center py-12">
            <span class="loading loading-spinner loading-lg"></span>
        </div>

        <!-- Empty State -->
        <div v-else-if="allHistory.length === 0" class="text-center py-12">
            <Icon name="history" class="w-12 h-12 mx-auto text-base-content/30 mb-4" />
            <h3 class="text-lg font-bold text-base-content">No changes recorded</h3>
            <p class="text-base-content/70">Service and startup app changes will appear here</p>
        </div>

        <!-- History Table -->
        <div v-else class="overflow-x-auto">
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
                    <tr v-for="entry in history" :key="entry.id">
                        <!-- Name -->
                        <td class="font-medium">{{ entry.name }}</td>

                        <!-- Type Badge -->
                        <td>
                            <div v-if="entry.entry_type.type === 'service'" class="badge badge-info">
                                Service
                            </div>
                            <div v-else-if="entry.entry_type.type === 'startup_app'" class="badge badge-success">
                                Startup App
                            </div>
                        </td>

                        <!-- Action Badge -->
                        <td>
                            <div :class="`badge badge-${getActionColor(entry.action)}`">
                                {{ entry.action }}
                            </div>
                        </td>

                        <!-- Details -->
                        <td class="text-sm">
                            <div v-if="entry.entry_type.type === 'service'">
                                <!-- Service Details -->
                                <div v-if="entry.entry_type.newStatus" class="flex items-center gap-1">
                                    <span v-if="entry.entry_type.previousStatus"
                                        :class="`badge badge-xs badge-${getStatusColor(entry.entry_type.previousStatus)}`">
                                        {{ entry.entry_type.previousStatus }}
                                    </span>
                                    <span v-if="entry.entry_type.previousStatus" class="text-base-content/50">→</span>
                                    <span :class="`badge badge-xs badge-${getStatusColor(entry.entry_type.newStatus)}`">
                                        {{ entry.entry_type.newStatus }}
                                    </span>
                                </div>

                                <div v-if="entry.entry_type.newStartupType" class="flex items-center gap-1 mt-1">
                                    <span v-if="entry.entry_type.previousStartupType"
                                        :class="`badge badge-xs badge-${getStartupTypeColor(entry.entry_type.previousStartupType)}`">
                                        {{ entry.entry_type.previousStartupType }}
                                    </span>
                                    <span v-if="entry.entry_type.previousStartupType" class="text-base-content/50">→</span>
                                    <span :class="`badge badge-xs badge-${getStartupTypeColor(entry.entry_type.newStartupType)}`">
                                        {{ entry.entry_type.newStartupType }}
                                    </span>
                                </div>
                            </div>

                            <div v-else-if="entry.entry_type.type === 'startup_app'" class="space-y-1">
                                <!-- Startup App Details -->
                                <div class="flex items-center gap-2">
                                    <span class="badge badge-xs">{{ entry.entry_type.location }}</span>
                                    <span class="text-base-content/50">•</span>
                                    <span class="tooltip tooltip-right" :data-tip="entry.entry_type.path">
                                        {{ truncatePath(entry.entry_type.path, 30) }}
                                    </span>
                                </div>
                                <div v-if="entry.entry_type.arguments" class="text-xs text-base-content/50">
                                    <code>{{ entry.entry_type.arguments }}</code>
                                </div>
                            </div>
                        </td>

                        <!-- Timestamp -->
                        <td class="text-sm text-base-content/50" :title="new Date(entry.timestamp).toLocaleString()">
                            {{ formatTime(entry.timestamp) }}
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>

        <!-- Confirm Clear Modal -->
        <ConfirmModal
            :show="showConfirmClear"
            title="Clear History?"
            message="This will delete all recorded changes. This action cannot be undone."
            confirm-text="Clear"
            @close="showConfirmClear = false"
            @confirm="handleClearHistory"
        />
    </div>
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
onMounted(async () => {
    await loadHistory()
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
