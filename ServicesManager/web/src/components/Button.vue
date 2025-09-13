<script setup>
const emit = defineEmits(['clicked']);

defineProps({
    text: String,
    disabled: Boolean,
    class: String
})
</script>

<template>
    <button @click="emit('clicked')" class="btn" :class="class" :disabled="disabled">
        {{ text }}
    </button>
</template>
