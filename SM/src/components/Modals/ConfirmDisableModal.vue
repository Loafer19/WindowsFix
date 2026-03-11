<template>
    <div v-if="showModal" class="modal modal-open">
        <div class="modal-box">
            <div class="flex items-center justify-between mb-2">
                <h3 class="font-bold text-lg">Confirm Disable Service</h3>
                <button class="btn btn-ghost btn-square btn-sm" @click="$emit('close')">
                    <Icon name="close" />
                </button>
            </div>

            <p class="py-4">Are you sure you want to disable the "{{ selectedService?.displayName }}" service?</p>
            <div class="modal-action">
                <button class="btn" @click="$emit('close')">Cancel</button>
                <button class="btn btn-success" @click="$emit('confirm')">Disable</button>
            </div>
        </div>
    </div>
</template>

<script setup>
import Icon from '../Icon.vue'

const props = defineProps({
    showModal: {
        type: Boolean,
        default: false,
    },
    selectedService: {
        type: Object,
        default: null,
    },
})

defineEmits(['close', 'confirm'])
</script>
