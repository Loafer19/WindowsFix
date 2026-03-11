<template>
    <div class="mb-4">
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
            <div class="bg-base-200 rounded-lg p-4">
                <div class="capitalize font-medium">Changes</div>
                <div class="text-2xl text-neutral">{{ history.length }}</div>
                <div class="text-base-content/70">{{ ((uniqueChangedServices / totalServices) * 100).toFixed(1) }}% of total</div>
                <progress class="progress progress-neutral h-2 mt-2" :value="((uniqueChangedServices / totalServices) * 100)" max="100"></progress>
            </div>
        </div>
    </div>

    <div class="flex items-center justify-between mb-2">
        <h4 class="text-lg font-semibold text-base-content">Change History</h4>
        <Button v-if="history.length > 0" class="btn btn-neutral btn-sm" @clicked="clearHistory">
            <Icon name="filterOff" />
            Clear History
        </Button>
    </div>

    <div v-if="history.length === 0" class="text-center py-8">
        <h3 class="mt-2 text-lg font-bold text-base-content">No history yet</h3>
        <p class="mt-1 text-base-content/70">Service changes will appear here :)</p>
    </div>

    <div v-else class="overflow-x-auto">
        <table class="table">
            <thead>
                <tr>
                    <th>Name</th>
                    <th>Action</th>
                    <th>State</th>
                    <th>Changed At</th>
                </tr>
            </thead>
            <tbody>
                <tr v-for="(entry, index) in history" :key="index">
                    <td>
                        <div class="font-medium text-base-content">{{ entry.name }}</div>
                        <div class="text-xs text-base-content/50">{{ entry.displayName }}</div>
                    </td>
                    <td class="whitespace-nowrap">
                        <div :class="`badge badge-${getActionColor(entry.action)}`">
                            {{ entry.action }}
                        </div>
                    </td>
                    <td class="whitespace-nowrap">
                        <div class="flex flex-col gap-1">
                            <!-- Status -->
                            <div v-if="entry.previousStatus && entry.newStatus && entry.previousStatus !== entry.newStatus"
                                class="flex items-center gap-1 text-sm">
                                <span :class="`badge badge-xs badge-${getStatusColor(entry.previousStatus)}`">
                                    {{ entry.previousStatus }}
                                </span>
                                <span class="text-base-content/50">→</span>
                                <span :class="`badge badge-xs badge-${getStatusColor(entry.newStatus)}`">
                                    {{ entry.newStatus }}
                                </span>
                            </div>
                            <div v-else-if="entry.newStatus" class="text-sm">
                                <span :class="`badge badge-xs badge-${getStatusColor(entry.newStatus)}`">
                                    {{ entry.newStatus }}
                                </span>
                            </div>

                            <!-- Startup Type -->
                            <div v-if="entry.previousStartupType && entry.newStartupType && entry.previousStartupType !== entry.newStartupType"
                                class="flex items-center gap-1 text-sm">
                                <span :class="`badge badge-xs badge-${getStartupTypeColor(entry.previousStartupType)}`">
                                    {{ entry.previousStartupType }}
                                </span>
                                <span class="text-base-content/50">→</span>
                                <span :class="`badge badge-xs badge-${getStartupTypeColor(entry.newStartupType)}`">
                                    {{ entry.newStartupType }}
                                </span>
                            </div>
                            <div v-else-if="entry.newStartupType" class="text-sm">
                                <span :class="`badge badge-xs badge-${getStartupTypeColor(entry.newStartupType)}`">
                                    {{ entry.newStartupType }}
                                </span>
                            </div>

                            <!-- No changes indicator -->
                            <span v-if="!entry.newStatus && !entry.newStartupType" class="text-base-content/40 text-xs">-</span>
                        </div>
                    </td>
                    <td class="text-base-content/70 text-sm">{{ formatDate(entry.changedAt) }}</td>
                </tr>
            </tbody>
        </table>
    </div>
</template>

<script setup>
import { computed } from 'vue'
import { getStartupTypeColor, getStatusColor } from '../../services/helpers.js'
import Button from '../Button.vue'
import Icon from '../Icon.vue'

const props = defineProps({
    history: {
        type: Array,
        default: () => [],
    },
    totalServices: {
        type: Number,
        default: 0,
    },
})

const emit = defineEmits(['clear-history'])

const clearHistory = () => {
    emit('clear-history')
}

const uniqueChangedServices = computed(() => {
    const uniqueNames = new Set(props.history.map(entry => entry.name))
    return uniqueNames.size
})

const getActionColor = (action) => {
    const colors = {
        Disabled: 'neutral',
        Started: 'success',
        Stopped: 'warning',
        Restarted: 'info',
        'Startup Type Changed': 'secondary',
        'Preset Applied': 'primary',
    }
    return colors[action] || 'neutral'
}

const formatDate = (isoString) => {
    if (!isoString) return '-'
    return new Date(isoString).toLocaleString('en-GB', { hour12: false })
}
</script>
