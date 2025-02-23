<template>
  <div class="container mx-auto my-6 text-gray-700">
    <h1 class="text-3xl font-bold mb-6">Windows Services</h1>

    <section class="mb-6 flex flex-wrap gap-6">
      <div class="flex items-center gap-3">
        <label for="searchFilter" class="font-semibold">Search:</label>
        <input id="searchFilter" v-model="searchQuery" @input="filterServices"
          class="p-2 border border-gray-200 focus:outline-none focus:ring-2 focus:ring-blue-500"
          placeholder="Name or Display Name" />
      </div>

      <div class="flex items-center gap-3">
        <label for="stateFilter" class="font-semibold">State:</label>
        <select id="stateFilter" v-model="selectedState" @change="filterServices"
          class="p-2 border border-gray-200 focus:outline-none focus:ring-2 focus:ring-blue-500">
          <option value="">All</option>
          <option value="Running">Running</option>
          <option value="Stopped">Stopped</option>
          <option value="Paused">Paused</option>
          <option value="Pending">Pending</option>
        </select>
      </div>

      <div class="flex items-center gap-3">
        <label for="startModeFilter" class="font-semibold">Start Mode:</label>
        <select id="startModeFilter" v-model="selectedStartMode" @change="filterServices"
          class="p-2 border border-gray-200 focus:outline-none focus:ring-2 focus:ring-blue-500">
          <option value="">All</option>
          <option value="Auto">Auto</option>
          <option value="Manual">Manual</option>
          <option value="Disabled">Disabled</option>
          <option value="System">System</option>
          <option value="Boot">Boot</option>
        </select>
      </div>
    </section>

    <p class="mb-3">
      Showing {{ filteredServices.length }} of {{ totalServices }} services
    </p>

    <section v-if="error" class="p-3 bg-gray-200 text-red-500 font-semibold">
      Failed to load services data!
    </section>

    <section v-else>
      <table class="w-full border-collapse">
        <thead class="bg-gray-200 text-left">
          <tr>
            <th class="p-3">Name</th>
            <th class="p-3">Display Name</th>
            <th class="p-3">State</th>
            <th class="p-3">Start Mode</th>
            <th class="p-3">Actions</th>
          </tr>
        </thead>
        <tbody>
          <template v-for="service in filteredServices" :key="service.name">
            <tr class="border-b border-gray-200 hover:bg-gray-100">
              <td class="p-3">{{ service.name }}</td>
              <td class="p-3">{{ service.displayName }}</td>
              <td class="p-3">{{ service.state }}</td>
              <td class="p-3">{{ service.startMode }}</td>
              <td class="p-3 flex gap-3">
                <Button :text="service.isExpanded ? 'Hide' : 'Info'" class="text-blue-500"
                  @clicked="service.isExpanded = !service.isExpanded" />

                <Button v-if="service.startMode !== 'Disabled'" :text="service.isDisabling ? 'Disabling...' : 'Disable'"
                  :disabled="service.isDisabling" class="text-red-500" @clicked="disable(service)" />
              </td>
            </tr>
            <tr v-show="service.isExpanded" class="border-b border-gray-200">
              <td colspan="5" class="p-3 bg-gray-50">
                <Button :text="service.isReloading ? 'Reloading...' : 'Reload'" :disabled="service.isDisabling"
                  class="mb-3 text-orange-500" @clicked="reloadInfo(service)" />

                <div v-if="service.info && !service.info.error">
                  <p>
                    <strong>URL:</strong>
                    <a :href="service.info.url" target="_blank" class="ml-1 underline">Open</a>
                  </p>
                  <p><strong>Description:</strong>
                    {{ service.info.description }}
                  </p>
                  <p><strong>Explained:</strong>
                    {{ service.info.explained }}
                  </p>
                  <p class="font-bold mt-3 text-blue-500">Recommendation:</p>
                  <div class="whitespace-pre-line">{{ service.info.recommendation }}</div>
                </div>
                <p v-else class="text-red-500 font-semibold">
                  {{ service.info?.message || 'No info available' }}
                </p>
              </td>
            </tr>
          </template>
        </tbody>
      </table>
    </section>
  </div>
</template>

<script setup>
import { ref, onMounted, computed } from 'vue'
import { loadServices, reloadServiceInfo, disableService } from './services/api.js'
import Button from './components/Button.vue'

const allServices = ref([])
const filteredServices = ref([])
const searchQuery = ref('')
const selectedState = ref('')
const selectedStartMode = ref('')
const error = ref(false)

const totalServices = computed(() => allServices.value.length)

onMounted(async () => {
  loadServices()
    .then((data) => {
      allServices.value = data
      filterServices()
    })
    .catch(() => {
      error.value = true
    })
})

const reloadInfo = async (service) => {
  service.isReloading = true

  reloadServiceInfo(service.name)
    .then((data) => (service.info = data))
    .then(() => {
      service.isReloading = false
    })
}

const disable = async (service) => {
  service.isDisabling = true

  disableService(service.name)
    .then((data) => Object.assign(service, data))
    .then(() => {
      service.isDisabling = false
    })
}

const filterServices = () => {
  let filtered = [...allServices.value]

  if (selectedState.value) {
    filtered = filtered.filter((service) => service.state === selectedState.value)
  }

  if (selectedStartMode.value) {
    filtered = filtered.filter((service) => service.startMode === selectedStartMode.value)
  }

  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    filtered = filtered.filter(
      (service) =>
        service.name.toLowerCase().includes(query) ||
        service.displayName.toLowerCase().includes(query)
    )
  }

  filteredServices.value = filtered
}
</script>

<style scoped>
.whitespace-pre-line {
  white-space: pre-line;
}
</style>
