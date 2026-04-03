<template>
    <div v-if="show" class="modal modal-open">
        <div class="modal-box max-w-lg">
            <div class="flex items-center justify-between mb-4">
                <h3 class="font-bold text-lg flex items-center gap-2">
                    <Icon :name="success ? 'check' : 'alarmWarning'" class="w-5 h-5" />
                    Preset Applied
                </h3>
                <button class="btn btn-ghost btn-square btn-sm" @click="$emit('close')">
                    <Icon name="close" />
                </button>
            </div>

            <div v-if="applying" class="py-8 text-center">
                <div class="flex justify-center mb-4">
                    <span class="loading loading-spinner loading-lg text-primary"></span>
                </div>
                <p class="text-base-content/70">{{ currentService || 'Processing...' }}</p>
                <progress class="progress progress-primary w-full mt-4" :value="progress" :max="total"></progress>
                <p class="text-sm text-base-content/50 mt-2">{{ progress }} of {{ total }} services</p>
            </div>

            <div v-else-if="results.length > 0">
                <div class="grid grid-cols-3 gap-2 mb-4">
                    <div class="bg-base-200 rounded-lg p-3 text-center">
                        <div class="text-2xl font-bold text-primary">{{ totalCount }}</div>
                        <div class="text-xs text-base-content/60">In Preset</div>
                    </div>
                    <div class="bg-base-200 rounded-lg p-3 text-center">
                        <div class="text-2xl font-bold text-info">{{ foundCount }}</div>
                        <div class="text-xs text-base-content/60">Found</div>
                    </div>
                    <div class="bg-base-200 rounded-lg p-3 text-center">
                        <div class="text-2xl font-bold" :class="success ? 'text-success' : 'text-warning'">{{ successCount }}</div>
                        <div class="text-xs text-base-content/60">Disabled</div>
                    </div>
                </div>

                <div :class="`alert ${success ? 'alert-success' : 'alert-warning'} mb-4`">
                    <Icon :name="success ? 'checkBoxCircle' : 'alarmWarning'" class="w-5 h-5" />
                    <span>
                        {{ successCount }} of {{ foundCount }} found services disabled successfully
                    </span>
                </div>

                <div class="overflow-x-auto max-h-60 overflow-y-auto">
                    <table class="table table-sm">
                        <thead class="sticky top-0 bg-base-100">
                            <tr>
                                <th>Service</th>
                                <th>Status</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr v-for="result in results" :key="result.name">
                                <td class="font-medium">{{ result.name }}</td>
                                <td>
                                    <span :class="`badge badge-sm ${result.success ? 'badge-success' : 'badge-error'}`">
                                        {{ result.success ? 'Disabled' : 'Failed' }}
                                    </span>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div v-else class="alert alert-info">
                <Icon name="info" class="w-5 h-5" />
                <span>No services from this preset were found on the system.</span>
            </div>

            <div class="modal-action" v-if="!applying">
                <Button class="btn btn-primary" @clicked="$emit('close')">Done</Button>
            </div>
        </div>
    </div>
</template>

<script setup>
import { computed } from 'vue'
import Button from '../Button.vue'
import Icon from '../Icon.vue'

const props = defineProps({
    show: {
        type: Boolean,
        default: false,
    },
    results: {
        type: Array,
        default: () => [],
    },
    applying: {
        type: Boolean,
        default: false,
    },
    progress: {
        type: Number,
        default: 0,
    },
    total: {
        type: Number,
        default: 0,
    },
    totalInPreset: {
        type: Number,
        default: 0,
    },
    currentService: {
        type: String,
        default: '',
    },
})

defineEmits(['close'])

const successCount = computed(() => props.results.filter(r => r.success).length)
const foundCount = computed(() => props.results.length)
const totalCount = computed(() => props.totalInPreset)
const success = computed(() => successCount.value === props.results.length && props.results.length > 0)
</script>
