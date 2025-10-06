<template>
  <div class="mb-4">
    <div class="bg-base-200 rounded-lg p-4 text-center transition-transform hover:scale-105 hover:shadow-lg">
      <h4 class="text-lg font-semibold text-base-content">Total Services</h4>
      <div class="text-3xl text-primary">{{ totalServices }}</div>
    </div>
  </div>

  <div class="mb-4">
    <h4 class="text-lg font-semibold text-base-content mb-4">
      By Status
    </h4>
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
      <div v-for="(count, status) in servicesByStatus" :key="status" class="bg-base-200 rounded-lg p-4 transition-transform hover:scale-105 hover:shadow-lg">
        <div class="flex items-center gap-2 capitalize font-medium">
          <Icon :name="getStatusIcon(status)" />
          {{ status }}
        </div>
        <div class="text-2xl" :class="`text-${getStatusColor(status)}`">{{ count }}</div>
        <div class="text-base-content/70 tooltip" :data-tip="`${((count / totalServices) * 100).toFixed(2)}% of total`">
          {{ ((count / totalServices) * 100).toFixed(1) }}% of total
        </div>
        <progress class="progress h-2 mt-2" :class="`progress-${getStatusColor(status)}`" :value="((count / totalServices) * 100)" max="100"></progress>
      </div>
    </div>
  </div>

  <div>
    <h4 class="text-lg font-semibold text-base-content mb-4">
      By Startup Type
    </h4>
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
      <div v-for="(count, type) in servicesByStartupType" :key="type" class="bg-base-200 rounded-lg p-4 transition-transform hover:scale-105 hover:shadow-lg">
        <div class="flex items-center gap-2 capitalize font-medium">
          <Icon :name="getStartupTypeIcon(type)" />
          {{ type }}
        </div>
        <div class="text-xl" :class="`text-${getStartupTypeColor(type)}`">{{ count }}</div>
        <div class="text-base-content/70 tooltip" :data-tip="`${((count / totalServices) * 100).toFixed(2)}% of total`">
          {{ ((count / totalServices) * 100).toFixed(1) }}% of total
        </div>
        <progress class="progress h-2 mt-2" :class="`progress-${getStartupTypeColor(type)}`" :value="((count / totalServices) * 100)" max="100"></progress>
      </div>
    </div>
  </div>
</template>

<script setup>
import Icon from '../Icon.vue'

const props = defineProps({
    servicesByStatus: {
        type: Object,
        default: () => ({}),
    },
    servicesByStartupType: {
        type: Object,
        default: () => ({}),
    },
    totalServices: {
        type: Number,
        default: 0,
    },
})

const getStatusColor = (status) => {
    const colors = {
        Running: 'error',
        Stopped: 'neutral',
        Paused: 'warning',
        Pending: 'warning',
    }
    return colors[status] || 'neutral'
}

const getStartupTypeColor = (type) => {
    const colors = {
        Automatic: 'error',
        Manual: 'warning',
        Disabled: 'neutral',
        System: 'info',
        Boot: 'info',
    }
    return colors[type] || 'neutral'
}

const getStatusIcon = (status) => {
    const icons = {
        Running: 'shutDown',
        Stopped: 'resetRight',
        Paused: 'alarmWarning',
        Pending: 'refresh',
    }
    return icons[status] || 'eye'
}

const getStartupTypeIcon = (type) => {
    const icons = {
        Automatic: 'refresh',
        Manual: 'eye',
        Disabled: 'filterOff',
        System: 'equalizer',
        Boot: 'shutDown',
    }
    return icons[type] || 'eye'
}
</script>
