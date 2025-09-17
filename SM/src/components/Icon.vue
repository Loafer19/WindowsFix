<template>
  <span v-html="iconSvg" :class="props.class"></span>
</template>

<script setup>
import { computed } from 'vue'
import { getIcon } from '../services/icons'

const props = defineProps({
  name: {
    type: String,
    required: true
  },
  class: {
    type: String,
    default: 'w-6 h-6'
  }
})

const iconSvg = computed(() => {
  return getIcon(props.name)
})
</script>
