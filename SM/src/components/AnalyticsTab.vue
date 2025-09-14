<template>
  <div>
    <!-- Services by State -->
    <div class="mb-6">
      <h4 class="text-lg font-semibold text-base-content mb-4 flex items-center gap-2">
        By State
      </h4>
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <div v-for="(count, state) in servicesByState" :key="state"
             class="stat bg-base-200 rounded-lg p-4">
          <div class="stat-title capitalize font-medium">{{ state }}</div>
          <div class="stat-value text-2xl text-primary">{{ count }}</div>
          <div class="stat-desc text-base-content/70">
            {{ ((count / totalServices) * 100).toFixed(1) }}% of total
          </div>
          <progress  class="progress progress-primary mt-2" :value="((count / totalServices) * 100)" max="100"></progress >
        </div>
      </div>
    </div>

    <!-- Services by Start Mode -->
    <div>
      <h4 class="text-lg font-semibold text-base-content mb-4 flex items-center gap-2">
        By Start Mode
      </h4>
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-4">
        <div v-for="(count, mode) in servicesByStartMode" :key="mode"
             class="stat bg-base-200 rounded-lg p-4">
          <div class="stat-title capitalize font-medium">{{ mode }}</div>
          <div class="stat-value text-xl text-secondary">{{ count }}</div>
          <div class="stat-desc text-base-content/70">
            {{ ((count / totalServices) * 100).toFixed(1) }}% of total
          </div>
          <progress  class="progress progress-secondary mt-2" :value="((count / totalServices) * 100)" max="100"></progress >
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'

// Props from parent
const props = defineProps({
  servicesByState: {
    type: Object,
    default: () => ({})
  },
  servicesByStartMode: {
    type: Object,
    default: () => ({})
  },
  totalServices: {
    type: Number,
    default: 0
  }
})

</script>
