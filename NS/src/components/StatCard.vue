<template>
    <div class="bg-base-200 rounded-lg p-4 flex items-center gap-3">
        <Icon :name="icon" class="w-8 h-8 shrink-0" :class="colorClass" />
        <div class="min-w-0">
            <div class="text-base-content/70 text-sm">{{ label }}</div>
            <div class="font-bold font-mono truncate" :class="[colorClass, sizeClass]">{{ value }}</div>
        </div>
    </div>
</template>

<script setup>
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = defineProps({
    icon: { type: String, required: true },
    label: { type: String, required: true },
    value: { type: String, required: true },
    color: { type: String, default: 'primary' },
    size: { type: String, default: '2xl' },
})

const colorClass = computed(() => `text-${props.color}`)
const sizeClass = computed(() => `text-${props.size}`)
</script>
