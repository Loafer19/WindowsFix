<template>
  <!-- Service Statistics -->
  <div class="mb-6">
    <div class="stats stats-vertical lg:stats-horizontal shadow bg-base-200">
      <div class="stat">
        <div class="stat-title">Filtered Services</div>
        <div class="stat-value text-primary">{{ filteredCount }}</div>
        <div class="stat-desc">
          of {{ totalCount }} total
          <span v-if="filteredCount !== totalCount" class="text-success">
            ({{ Math.round((filteredCount / totalCount) * 100) }}% shown)
          </span>
        </div>
      </div>
    </div>
  </div>

  <!-- Filters Grid -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
    <div class="form-control">
      <label for="searchFilter" class="label">
        <span class="label-text">Search</span>
      </label>
      <input id="searchFilter" v-model="searchQuery" @input="filterServices"
        class="input input-bordered"
        placeholder="Name or Display Name" />
    </div>

    <div class="form-control">
      <label for="stateFilter" class="label">
        <span class="label-text">State</span>
      </label>
      <select id="stateFilter" v-model="selectedState" @change="filterServices"
        class="select select-bordered">
        <option value="">All States</option>
        <option value="Running">Running</option>
        <option value="Stopped">Stopped</option>
        <option value="Paused">Paused</option>
        <option value="Pending">Pending</option>
      </select>
    </div>

    <div class="form-control">
      <label for="startModeFilter" class="label">
        <span class="label-text">Start Mode</span>
      </label>
      <select id="startModeFilter" v-model="selectedStartMode" @change="filterServices"
        class="select select-bordered">
        <option value="">All Modes</option>
        <option value="Auto">Auto</option>
        <option value="Manual">Manual</option>
        <option value="Disabled">Disabled</option>
        <option value="System">System</option>
        <option value="Boot">Boot</option>
      </select>
    </div>

    <div class="form-control">
      <label class="label">
        <span class="label-text">Actions</span>
      </label>
      <div class="flex gap-2">
        <Button :text="'Refresh'" @clicked="refresh"
          class="btn btn-success"></Button>
        <Button :text="'Clear Filters'" @clicked="clearFilters"
          class="btn btn-ghost" ></Button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, watch } from 'vue'
import { refreshServices } from '../services/api.js'
import Button from './Button.vue'

// Reactive data
const searchQuery = ref('')
const selectedState = ref('')
const selectedStartMode = ref('')

// Props from parent
const props = defineProps({
  filteredCount: {
    type: Number,
    default: 0
  },
  totalCount: {
    type: Number,
    default: 0
  }
})

// Emits for parent communication
const emit = defineEmits(['update:searchQuery', 'update:selectedState', 'update:selectedStartMode', 'filter'])

// Filter services function
const filterServices = () => {
  emit('filter', {
    searchQuery: searchQuery.value,
    selectedState: selectedState.value,
    selectedStartMode: selectedStartMode.value
  })
}

// Watch for changes and emit filter events
watch([searchQuery, selectedState, selectedStartMode], () => {
  filterServices()
}, { immediate: false })

// Refresh services
const refresh = async () => {
  try {
    await refreshServices()
    // Reload data would be handled by parent
  } catch (error) {
    console.error('Failed to refresh services:', error)
  }
}

// Clear filters
const clearFilters = () => {
  searchQuery.value = ''
  selectedState.value = ''
  selectedStartMode.value = ''
  filterServices()
}

// Expose functions for parent access
defineExpose({
  searchQuery,
  selectedState,
  selectedStartMode,
  filterServices,
  clearFilters
})
</script>
