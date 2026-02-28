<template>
    <button :class="class" :disabled="disabled" @click="emit('clicked')">
        <slot v-if="!isLoading">{{ text }}</slot>
        <span v-else class="loading loading-spinner"></span>
    </button>
</template>

<script setup>
const props = defineProps({
    class: {
        type: String,
        default: 'btn btn-primary',
    },
    text: {
        type: String,
        default: '',
    },
    disabled: {
        type: Boolean,
        default: false,
    },
    isLoading: {
        type: Boolean,
        default: false,
    },
})

const emit = defineEmits(['clicked'])
</script>
