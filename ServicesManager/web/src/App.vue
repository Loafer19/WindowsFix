<template>
  <div class="min-h-screen bg-gray-50">
    <div class="container mx-auto px-4 py-8 max-w-7xl">
      <header class="mb-8">
        <h1 class="text-4xl font-bold text-gray-900 mb-2">Windows Services Manager</h1>
        <p class="text-gray-600">Manage and monitor Windows services with detailed information</p>
      </header>

      <!-- Controls Section -->
      <section class="bg-white rounded-lg shadow-sm border p-6 mb-6">
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-4">
          <div class="flex flex-col">
            <label for="searchFilter" class="text-sm font-medium text-gray-700 mb-1">Search</label>
            <input id="searchFilter" v-model="searchQuery" @input="filterServices"
              class="p-3 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              placeholder="Name or Display Name" />
          </div>

          <div class="flex flex-col">
            <label for="stateFilter" class="text-sm font-medium text-gray-700 mb-1">State</label>
            <select id="stateFilter" v-model="selectedState" @change="filterServices"
              class="p-3 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500">
              <option value="">All States</option>
              <option value="Running">Running</option>
              <option value="Stopped">Stopped</option>
              <option value="Paused">Paused</option>
              <option value="Pending">Pending</option>
            </select>
          </div>

          <div class="flex flex-col">
            <label for="startModeFilter" class="text-sm font-medium text-gray-700 mb-1">Start Mode</label>
            <select id="startModeFilter" v-model="selectedStartMode" @change="filterServices"
              class="p-3 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500">
              <option value="">All Modes</option>
              <option value="Auto">Auto</option>
              <option value="Manual">Manual</option>
              <option value="Disabled">Disabled</option>
              <option value="System">System</option>
              <option value="Boot">Boot</option>
            </select>
          </div>

          <div class="flex flex-col">
            <label class="text-sm font-medium text-gray-700 mb-1">Actions</label>
            <div class="flex gap-2">
              <Button :text="'Refresh'" @clicked="refresh"
                class="bg-green-600 hover:bg-green-700 text-white px-4 py-2 text-sm" />
              <Button :text="'Clear Filters'" @clicked="clearFilters"
                class="bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 text-sm" />
            </div>
          </div>
        </div>

        <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
          <div class="text-sm text-gray-600">
            Showing {{ paginatedServices.length }} of {{ filteredServices.length }} services
            <span v-if="filteredServices.length !== totalServices">
              (filtered from {{ totalServices }} total)
            </span>
          </div>

          <div class="flex items-center gap-4">
            <div class="flex items-center gap-2">
              <label for="pageSize" class="text-sm text-gray-700">Show:</label>
              <select id="pageSize" v-model="pageSize" @change="resetPagination"
                class="p-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500">
                <option :value="10">10</option>
                <option :value="25">25</option>
                <option :value="50">50</option>
                <option :value="100">100</option>
              </select>
            </div>

            <div class="flex items-center gap-2">
              <button @click="previousPage" :disabled="currentPage === 1"
                class="px-3 py-1 border border-gray-300 rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed">
                ← Prev
              </button>
              <span class="text-sm text-gray-700">
                Page {{ currentPage }} of {{ totalPages }}
              </span>
              <button @click="nextPage" :disabled="currentPage === totalPages"
                class="px-3 py-1 border border-gray-300 rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed">
                Next →
              </button>
            </div>
          </div>
        </div>
      </section>

      <!-- Error State -->
      <section v-if="error" class="bg-red-50 border border-red-200 rounded-lg p-6 mb-6">
        <div class="flex items-center">
          <div class="flex-shrink-0">
            <svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
            </svg>
          </div>
          <div class="ml-3">
            <h3 class="text-sm font-medium text-red-800">Error Loading Services</h3>
            <p class="mt-1 text-sm text-red-700">Failed to load services data. Please try refreshing the page.</p>
          </div>
        </div>
      </section>

      <!-- Loading State -->
      <section v-else-if="loading" class="bg-white rounded-lg shadow-sm border p-12 mb-6">
        <div class="flex items-center justify-center">
          <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
          <span class="ml-3 text-gray-600">Loading services...</span>
        </div>
      </section>

      <!-- Services Table -->
      <section v-else class="bg-white rounded-lg shadow-sm border overflow-hidden">
        <div class="overflow-x-auto">
          <table class="w-full">
            <thead class="bg-gray-50 border-b border-gray-200">
              <tr>
                <th class="px-6 py-4 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Name</th>
                <th class="px-6 py-4 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Display Name</th>
                <th class="px-6 py-4 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">State</th>
                <th class="px-6 py-4 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Start Mode</th>
                <th class="px-6 py-4 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Actions</th>
              </tr>
            </thead>
            <tbody class="bg-white divide-y divide-gray-200">
              <template v-for="service in paginatedServices" :key="service.name">
                <tr class="hover:bg-gray-50 transition-colors">
                  <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">{{ service.name }}</td>
                  <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{{ service.displayName }}</td>
                  <td class="px-6 py-4 whitespace-nowrap">
                    <span :class="getStateBadgeClass(service.state)" class="inline-flex px-2 py-1 text-xs font-semibold rounded-full">
                      {{ service.state }}
                    </span>
                  </td>
                  <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{{ service.startMode }}</td>
                  <td class="px-6 py-4 whitespace-nowrap text-sm font-medium">
                    <div class="flex items-center space-x-2">
                      <Button :text="service.isExpanded ? 'Hide Info' : 'Show Info'"
                        :class="service.isExpanded ? 'text-gray-600 hover:text-gray-800' : 'text-blue-600 hover:text-blue-800'"
                        @clicked="service.isExpanded = !service.isExpanded" />

                      <Button v-if="service.startMode !== 'Disabled'"
                        :text="service.isDisabling ? 'Disabling...' : 'Disable'"
                        :disabled="service.isDisabling"
                        class="text-red-600 hover:text-red-800 disabled:opacity-50"
                        @clicked="disable(service)" />
                    </div>
                  </td>
                </tr>
                <tr v-show="service.isExpanded" class="bg-gray-50">
                  <td colspan="5" class="px-6 py-4">
                    <div class="space-y-4">
                      <div class="flex items-center justify-between">
                        <h4 class="text-sm font-medium text-gray-900">Service Details</h4>
                        <Button :text="service.isReloading ? 'Reloading...' : 'Reload Info'"
                          :disabled="service.isDisabling"
                          class="text-orange-600 hover:text-orange-800 disabled:opacity-50"
                          @clicked="reloadInfo(service)" />
                      </div>

                      <div v-if="service.info && !service.info.error" class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div v-if="service.info.url" class="md:col-span-2">
                          <label class="block text-xs font-medium text-gray-500 mb-1">Source URL</label>
                          <a :href="service.info.url" target="_blank"
                            class="text-blue-600 hover:text-blue-800 underline break-all">{{ service.info.url }}</a>
                        </div>

                        <div v-if="service.info.description">
                          <label class="block text-xs font-medium text-gray-500 mb-1">Description</label>
                          <p class="text-sm text-gray-900">{{ service.info.description }}</p>
                        </div>

                        <div v-if="service.info.explained">
                          <label class="block text-xs font-medium text-gray-500 mb-1">Explanation</label>
                          <p class="text-sm text-gray-900">{{ service.info.explained }}</p>
                        </div>

                        <div v-if="service.info.recommendation" class="md:col-span-2">
                          <label class="block text-xs font-medium text-gray-500 mb-1">Recommendation</label>
                          <div class="text-sm text-gray-900 whitespace-pre-line bg-blue-50 p-3 rounded-md border-l-4 border-blue-400">
                            {{ service.info.recommendation }}
                          </div>
                        </div>
                      </div>

                      <div v-else class="text-center py-4">
                        <p class="text-red-600 font-medium">
                          {{ service.info?.message || 'No information available for this service' }}
                        </p>
                      </div>
                    </div>
                  </td>
                </tr>
              </template>
            </tbody>
          </table>
        </div>

        <!-- Empty State -->
        <div v-if="paginatedServices.length === 0" class="text-center py-12">
          <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          <h3 class="mt-2 text-sm font-medium text-gray-900">No services found</h3>
          <p class="mt-1 text-sm text-gray-500">Try adjusting your search or filter criteria.</p>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, computed, watch } from 'vue'
import { loadServices, reloadServiceInfo, disableService, refreshServices } from './services/api.js'
import Button from './components/Button.vue'

const allServices = ref([])
const filteredServices = ref([])
const searchQuery = ref('')
const selectedState = ref('')
const selectedStartMode = ref('')
const error = ref(false)
const loading = ref(true)

// Pagination
const currentPage = ref(1)
const pageSize = ref(25)

const totalServices = computed(() => allServices.value.length)
const totalPages = computed(() => Math.ceil(filteredServices.value.length / pageSize.value))

const paginatedServices = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  const end = start + pageSize.value
  return filteredServices.value.slice(start, end)
})

onMounted(async () => {
  await loadServicesData()
})

const loadServicesData = async () => {
  try {
    loading.value = true
    error.value = false
    const data = await loadServices()
    allServices.value = data
    filterServices()
  } catch (err) {
    console.error('Failed to load services:', err)
    error.value = true
  } finally {
    loading.value = false
  }
}

const refresh = async () => {
  try {
    loading.value = true
    await refreshServices()
    await loadServicesData()
  } catch (err) {
    console.error('Failed to refresh services:', err)
    // Keep existing data on refresh failure
    loading.value = false
  }
}

const reloadInfo = async (service) => {
  service.isReloading = true

  try {
    const data = await reloadServiceInfo(service.name)
    service.info = data
  } catch (error) {
    console.error('Failed to reload service info:', error)
    service.info = { error: true, message: 'Failed to reload information' }
  } finally {
    service.isReloading = false
  }
}

const disable = async (service) => {
  service.isDisabling = true

  try {
    const data = await disableService(service.name)
    Object.assign(service, data)
  } catch (error) {
    console.error('Failed to disable service:', error)
    service.info = { error: true, message: 'Failed to disable service' }
  } finally {
    service.isDisabling = false
  }
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
  resetPagination()
}

const clearFilters = () => {
  searchQuery.value = ''
  selectedState.value = ''
  selectedStartMode.value = ''
  filterServices()
}

const resetPagination = () => {
  currentPage.value = 1
}

const nextPage = () => {
  if (currentPage.value < totalPages.value) {
    currentPage.value++
  }
}

const previousPage = () => {
  if (currentPage.value > 1) {
    currentPage.value--
  }
}

const getStateBadgeClass = (state) => {
  const classes = {
    'Running': 'bg-green-100 text-green-800',
    'Stopped': 'bg-red-100 text-red-800',
    'Paused': 'bg-yellow-100 text-yellow-800',
    'Pending': 'bg-blue-100 text-blue-800'
  }
  return classes[state] || 'bg-gray-100 text-gray-800'
}

// Watch for filter changes to reset pagination
watch([searchQuery, selectedState, selectedStartMode], () => {
  resetPagination()
})
</script>

<style scoped>
.whitespace-pre-line {
  white-space: pre-line;
}
</style>
