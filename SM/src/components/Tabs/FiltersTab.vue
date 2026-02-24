<template>
  <div class="mb-4">
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
      <div class="bg-base-200 rounded-lg p-4">
        <div class="capitalize font-medium">Showing</div>
        <div class="text-2xl text-primary">{{ filteredCount }}</div>
        <div class="text-base-content/70">
          {{ ((filteredCount / totalCount) * 100).toFixed(1) }}% of total
        </div>
        <progress class="progress progress-primary mt-2" :value="((filteredCount / totalCount) * 100)"
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
        <option value="">All</option>
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
      <select id="startupTypeFilter" v-model="selectedStartupType" @change="filterServices"
        class="select select-bordered">
        <option value="">All</option>
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
        <Button :text="'Refresh'" @clicked="refresh" class="btn btn-info btn-square">
          <Icon name="refresh" />
        </Button>
        <Button :text="'Clear Filters'" @clicked="clearFilters" class="btn btn-neutral btn-square">
          <Icon name="filterOff" />
        </Button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import Button from '../Button.vue'
import Icon from '../Icon.vue'

const searchQuery = ref('')
const selectedStatus = ref('')
const selectedStartupType = ref('')

const props = defineProps({
    filteredCount: {
        type: Number,
        default: 0,
    },
    totalCount: {
        type: Number,
        default: 0,
    },
})

const emit = defineEmits(['filter', 'refresh'])

const filterServices = () => {
    emit('filter', {
        searchQuery: searchQuery.value,
        selectedStatus: selectedStatus.value,
        selectedStartupType: selectedStartupType.value,
    })
}

const refresh = () => {
    emit('refresh')
}

const clearFilters = () => {
    searchQuery.value = ''
    selectedStatus.value = ''
    selectedStartupType.value = ''
    filterServices()
}
</script>
