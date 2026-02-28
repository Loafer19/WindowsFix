<template>
    <div v-if="showModal" class="modal modal-open">
        <div class="modal-box">
            <h3 class="font-bold text-lg">Confirm Disable Service</h3>
            <p class="py-4">Are you sure you want to disable the "{{ selectedService?.displayName }}" service?</p>
            <div class="modal-action">
                <button class="btn" @click="$emit('close')">Cancel</button>
                <button class="btn btn-success" @click="$emit('confirm')">Disable</button>
            </div>
        </div>
    </div>
</template>

<script setup>
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
