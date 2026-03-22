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
    /** Remix icon name to display. */
    icon: { type: String, required: true },
    /** Label text shown above the value. */
    label: { type: String, required: true },
    /** Formatted value string (e.g. "1.5 MB/s"). */
    value: { type: String, required: true },
    /** DaisyUI colour token: 'primary' | 'info' | 'success' | 'warning' | 'error'. */
    color: { type: String, default: 'primary' },
    /** Tailwind text-size suffix: 'sm' | 'base' | 'lg' | 'xl' | '2xl'. */
    size: { type: String, default: '2xl' },
})

const colorClass = computed(() => `text-${props.color}`)
const sizeClass = computed(() => `text-${props.size}`)
</script>
