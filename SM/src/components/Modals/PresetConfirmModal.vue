<template>
    <div v-if="showModal" class="modal modal-open">
        <div class="modal-box max-w-2xl">
            <div class="flex items-center justify-between mb-2">
                <h3 class="font-bold text-lg flex items-center gap-2">
                    <Icon :name="preset?.icon" class="w-5 h-5" />
                    Apply "{{ preset?.name }}" Preset
                </h3>
                <button class="btn btn-ghost btn-square btn-sm" @click="$emit('close')">
                    <Icon name="close" />
                </button>
            </div>

            <p class="text-base-content/70 mb-4">{{ preset?.description }}</p>

            <div class="alert alert-warning mb-4 py-2">
                <Icon name="alarmWarning" class="w-4 h-4 shrink-0" />
                <span class="text-sm">The following services will be <strong>disabled</strong>. Review carefully before applying.</span>
            </div>

            <div class="overflow-x-auto max-h-72 overflow-y-auto mb-4">
                <table class="table table-sm">
                    <thead class="sticky top-0 bg-base-100">
                        <tr>
                            <th>Service</th>
                            <th>Reason</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr v-for="svc in preset?.services" :key="svc.name">
                            <td>
                                <div class="font-medium text-base-content">{{ svc.name }}</div>
                                <div class="text-xs text-base-content/50">{{ svc.displayName }}</div>
                            </td>
                            <td class="text-base-content/70 text-sm">{{ svc.reason }}</td>
                        </tr>
                    </tbody>
                </table>
            </div>

            <div class="modal-action">
                <button class="btn" @click="$emit('close')" :disabled="applying">Cancel</button>
                <Button
                    :class="`btn btn-${preset?.color || 'primary'}`"
                    :is-loading="applying"
                    :disabled="applying"
                    @clicked="$emit('confirm', preset)">
                    <Icon :name="preset?.icon" />
                    Apply Preset
                </Button>
            </div>
        </div>
    </div>
</template>

<script setup>
import Button from '../Button.vue'
import Icon from '../Icon.vue'

defineProps({
    showModal: {
        type: Boolean,
        default: false,
    },
    preset: {
        type: Object,
        default: null,
    },
    applying: {
        type: Boolean,
        default: false,
    },
})

defineEmits(['close', 'confirm'])
</script>
