<template>
  <div>
    <!-- Services by Status -->
    <div class="mb-6">
      <h4 class="text-lg font-semibold text-base-content mb-4 flex items-center gap-2">
        By Status
      </h4>
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <div v-for="(count, status) in servicesByStatus" :key="status"
             class="bg-base-200 rounded-lg p-4">
          <div class="capitalize font-medium">{{ status }}</div>
          <div class="text-2xl text-primary">{{ count }}</div>
          <div class="text-base-content/70">
            {{ ((count / totalServices) * 100).toFixed(1) }}% of total
          </div>
          <progress  class="progress progress-primary mt-2" :value="((count / totalServices) * 100)" max="100"></progress >
        </div>
      </div>
    </div>

    <!-- Services by Startup Type -->
    <div>
      <h4 class="text-lg font-semibold text-base-content mb-4 flex items-center gap-2">
        By Startup Type
      </h4>
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-4">
        <div v-for="(count, type) in servicesByStartupType" :key="type"
             class=" bg-base-200 rounded-lg p-4">
          <div class="capitalize font-medium">{{ type }}</div>
          <div class="text-xl text-secondary">{{ count }}</div>
          <div class="text-base-content/70">
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
  servicesByStatus: {
    type: Object,
    default: () => ({})
  },
  servicesByStartupType: {
    type: Object,
    default: () => ({})
  },
  totalServices: {
    type: Number,
    default: 0
  }
})

</script>
