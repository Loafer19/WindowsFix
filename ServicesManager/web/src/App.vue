<template>
  <div class="container mx-auto my-6">
    <h1 class="text-3xl font-bold text-gray-800 mb-6">Windows Services</h1>

    <div class="mb-6">
      <label for="startModeFilter" class="text-gray-700 font-semibold mr-2">Filter by Start Mode:</label>
      <select id="startModeFilter" v-model="selectedStartMode" @change="filterServices"
        class="p-2 border border-gray-200 rounded-lg text-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500">
        <option value="all">All</option>
        <option value="Auto">Auto</option>
        <option value="Manual">Manual</option>
        <option value="Disabled">Disabled</option>
        <option value="System">System</option>
        <option value="Boot">Boot</option>
      </select>
    </div>

    <div class="overflow-x-auto">
      <table v-if="!error" class="w-full">
        <thead class="bg-gray-200 text-left text-gray-700">
          <tr>
            <th class="py-3 px-4">Name</th>
            <th class="py-3 px-4">Display Name</th>
            <th class="py-3 px-4">State</th>
            <th class="py-3 px-4">Start Mode</th>
            <th class="py-3 px-4">Details</th>
          </tr>
        </thead>
        <tbody class="text-gray-600">
          <template v-for="(service, index) in filteredServices" :key="service.name">
            <tr class="border-b hover:bg-gray-100">
              <td class="py-3 px-4">{{ service.name }}</td>
              <td class="py-3 px-4">{{ service.displayName }}</td>
              <td class="py-3 px-4">{{ service.state }}</td>
              <td class="py-3 px-4">{{ service.startMode }}</td>
              <td class="py-3 px-4">
                <span @click="service.isExpanded = !service.isExpanded" class="text-blue-500 underline cursor-pointer">
                  {{ service.isExpanded ? 'Hide' : 'Show' }}
                </span>
                <button v-if="service.info.error" @click="reloadInfo(service)"
                  class="ml-2 text-red-500 underline cursor-pointer" :disabled="service.isReloading">
                  {{ service.isReloading ? 'Reloading...' : 'Reload Info' }}
                </button>
              </td>
            </tr>
            <tr :class="{ 'hidden': !service.isExpanded }" class="details border-b">
              <td colspan="5" class="py-3 px-4 bg-gray-50">
                <div v-if="!service.info?.error">
                  <p><strong>URL:</strong> <a :href="service.info.url" target="_blank" class="underline">Open</a></p>
                  <p><strong>Description:</strong> {{ service.info.description }}</p>
                  <p><strong>Explained:</strong> {{ service.info.explained }}</p>
                  <p><strong>Recommendation:</strong> {{ service.info.recommendation }}</p>
                </div>
                <div v-else>
                  <p class="text-red-500 font-semibold">{{ service.info.message }}</p>
                </div>
              </td>
            </tr>
          </template>
        </tbody>
      </table>
      <div v-else class="py-3 px-4 bg-gray-200 text-red-500 font-semibold">Failed to load services data!</div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { loadServices, reloadServiceInfo } from './services/api.js'

const allServices = ref({})
const filteredServices = ref([])
const selectedStartMode = ref('all')
const error = ref(false)

onMounted(async () => {
  loadServices()
    .then(data => {
      allServices.value = data
      filterServices()
    })
    .catch(err => {
      error.value = true
    })
})

const reloadInfo = async (service) => {
  service.isReloading = true

  reloadServiceInfo(service.name)
    .then(data => {
      service.info = data
    })

  service.isReloading = false
}

const filterServices = () => {
  if (selectedStartMode.value === 'all') {
    filteredServices.value = [...allServices.value]
  } else {
    filteredServices.value = allServices.value.filter(
      service => service.startMode === selectedStartMode.value
    )
  }
}
</script>

<style scoped>
.hidden {
  display: none;
}
</style>
