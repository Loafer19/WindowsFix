<script setup>
const emit = defineEmits(['clicked']);

defineProps({
    text: String,
    disabled: Boolean,
    class: String
})
</script>

<template>
    <button @click="emit('clicked')" :class="class" class="underline cursor-pointer" :disabled="disabled">
        {{ text }}
    </button>
</template>
