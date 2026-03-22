<template>
    <div v-if="show" class="modal modal-open">
        <div class="modal-box">
            <h3 class="font-bold text-lg">{{ title }}</h3>
            <p class="py-4">{{ message }}</p>
            <div class="modal-action">
                <button class="btn" @click="$emit('close')">Cancel</button>
                <button class="btn btn-error" @click="$emit('confirm')">
                    {{ confirmText || 'Confirm' }}
                </button>
            </div>
        </div>
    </div>
</template>

<script setup>
defineProps({
    show: Boolean,
    title: String,
    message: String,
    confirmText: String,
})

defineEmits(['close', 'confirm'])
</script>
