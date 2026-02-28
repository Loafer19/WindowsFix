<template>
    <div class="mb-4">
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
            <div class="bg-base-200 rounded-lg p-4">
                <div class="capitalize font-medium">Disabled</div>
                <div class="text-2xl text-neutral">{{ history.length }}</div>
                <div class="text-base-content/70">{{ ((history.length / totalServices) * 100).toFixed(1) }}% of total</div>
                <progress class="progress progress-neutral h-2 mt-2" :value="((history.length / totalServices) * 100)" max="100"></progress>
            </div>
        </div>
    </div>

    <div class="flex items-center justify-between mb-4">
        <h4 class="text-lg font-semibold text-base-content">Disabled Services</h4>
        <Button v-if="history.length > 0" class="btn btn-neutral btn-sm" @clicked="clearHistory">
            <Icon name="filterOff" />
            Clear History
        </Button>
    </div>

    <div v-if="history.length === 0" class="text-center py-8">
        <h3 class="mt-2 text-lg font-bold text-base-content">No history yet</h3>
        <p class="mt-1 text-base-content/70">Services you disable will appear here :)</p>
    </div>

    <div v-else class="overflow-x-auto">
        <table class="table">
            <thead>
                <tr>
                    <th>Name</th>
                    <th>Display Name</th>
                    <th>Disabled At</th>
                </tr>
            </thead>
            <tbody>
                <tr v-for="(entry, index) in history" :key="index">
                    <td class="font-medium text-base-content">{{ entry.name }}</td>
                    <td class="text-base-content/70">{{ entry.displayName }}</td>
                    <td class="text-base-content/70">{{ formatDate(entry.disabledAt) }}</td>
                </tr>
            </tbody>
        </table>
    </div>
</template>

<script setup>
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

const formatDate = (isoString) => {
    return new Date(isoString).toLocaleString('en-GB', { hour12: false })
}
</script>
