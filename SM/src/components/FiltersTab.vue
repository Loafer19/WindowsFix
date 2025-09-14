<template>
  <!-- Service Statistics -->
  <div class="mb-6">
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
      <div class="bg-base-200 rounded-lg p-4">
        <div class="capitalize font-medium">Showing</div>
        <div class="text-2xl text-warning">{{ filteredCount }}</div>
        <div class="text-base-content/70">
          {{ ((filteredCount / totalCount) * 100).toFixed(1) }}% of total
        </div>
        <progress class="progress progress-warning mt-2" :value="((filteredCount / totalCount) * 100)"
          max="100"></progress>
      </div>
    </div>
  </div>

  <!-- Filters Grid -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
    <div class="form-control">
      <label for="searchFilter" class="label">
        Search
      </label>
      <input id="searchFilter" v-model="searchQuery" @input="filterServices" class="input input-bordered"
        placeholder="Name or Display Name" />
    </div>

    <div class="form-control">
      <label for="statusFilter" class="label">
        Status
      </label>
      <select id="statusFilter" v-model="selectedStatus" @change="filterServices" class="select select-bordered">
        <option value="">All Statuses</option>
        <option value="Running">Running</option>
        <option value="Stopped">Stopped</option>
        <option value="Paused">Paused</option>
        <option value="Pending">Pending</option>
      </select>
    </div>

    <div class="form-control">
      <label for="startupTypeFilter" class="label">
        Startup Type
      </label>
      <select id="startupTypeFilter" v-model="selectedStartupType" @change="filterServices" class="select select-bordered">
        <option value="">All Startup Types</option>
        <option value="Automatic">Automatic</option>
        <option value="Manual">Manual</option>
        <option value="Disabled">Disabled</option>
        <option value="System">System</option>
        <option value="Boot">Boot</option>
      </select>
    </div>

    <div class="form-control">
      <label class="label">
        Actions
      </label>
      <div class="flex gap-2">
        <Button :text="'Refresh'" @clicked="refresh" class="btn btn-success"></Button>
        <Button :text="'Clear Filters'" @clicked="clearFilters" class="btn btn-ghost"></Button>
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
const selectedStatus = ref('')
const selectedStartupType = ref('')

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
const emit = defineEmits(['update:searchQuery', 'update:selectedStatus', 'update:selectedStartupType', 'filter'])

// Filter services function
const filterServices = () => {
  emit('filter', {
    searchQuery: searchQuery.value,
    selectedStatus: selectedStatus.value,
    selectedStartupType: selectedStartupType.value
  })
}

// Watch for changes and emit filter events
watch([searchQuery, selectedStatus, selectedStartupType], () => {
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
  selectedStatus.value = ''
  selectedStartupType.value = ''
  filterServices()
}

// Expose functions for parent access
defineExpose({
  searchQuery,
  selectedStatus,
  selectedStartupType,
  filterServices,
  clearFilters
})
</script>
