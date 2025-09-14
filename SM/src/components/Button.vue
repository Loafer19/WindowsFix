<template>
  <button
    :class="buttonClass"
    :disabled="disabled"
    @click="emit('clicked')"
  >
    <slot>{{ text }}</slot>
  </button>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  text: {
    type: String,
    default: ''
  },
  disabled: {
    type: Boolean,
    default: false
  },
  class: {
    type: String,
    default: ''
  }
})

const emit = defineEmits(['clicked'])

const buttonClass = computed(() => {
  return props.class || 'btn btn-primary'
})
</script>
