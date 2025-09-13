<template>
  <div class="min-h-screen bg-base-300">
    <div class="container mx-auto px-4 py-8 max-w-7xl">
      <header class="mb-8">
        <h1 class="text-4xl font-bold text-base-content mb-2">Windows Services Manager</h1>
        <p class="text-base-content/70">Manage and monitor Windows services with detailed information</p>
      </header>

      <!-- Filters Section -->
      <div class="card bg-base-100 shadow-lg mb-4">
        <div class="card-body">
          <h3 class="card-title text-base-content">Filters</h3>
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
        </div>
      </div>

      <!-- Analytics Section -->
      <div class="card bg-base-100 shadow-lg mb-6">
        <div class="card-body">
          <h3 class="card-title text-base-content">Service Analytics</h3>

          <!-- Services by State -->
          <div class="mb-6">
            <h4 class="text-lg font-semibold text-base-content mb-4">By State</h4>
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
              <div v-for="(count, state) in servicesByState" :key="state"
                   class="stat bg-base-200 rounded-lg p-4">
                <div class="stat-title capitalize">{{ state }}</div>
                <div class="stat-value text-2xl text-primary">{{ count }}</div>
                <div class="stat-desc">
                  {{ ((count / totalServices) * 100).toFixed(1) }}% of total
                </div>
              </div>
            </div>
          </div>

          <!-- Services by Start Mode -->
          <div>
            <h4 class="text-lg font-semibold text-base-content mb-4">By Start Mode</h4>
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-4">
              <div v-for="(count, mode) in servicesByStartMode" :key="mode"
                   class="stat bg-base-200 rounded-lg p-4">
                <div class="stat-title capitalize">{{ mode }}</div>
                <div class="stat-value text-xl text-secondary">{{ count }}</div>
                <div class="stat-desc">
                  {{ ((count / totalServices) * 100).toFixed(1) }}% of total
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Pagination Section -->
      <div class="card bg-base-100 shadow-lg mb-6">
        <div class="card-body">
          <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
            <div class="stats stats-vertical lg:stats-horizontal shadow">
              <div class="stat">
                <div class="stat-title">Services</div>
                <div class="stat-value text-primary">{{ paginatedServices.length }}</div>
                <div class="stat-desc">
                  of {{ filteredServices.length }}
                  <span v-if="filteredServices.length !== totalServices" class="text-base-content/60">
                    ({{ totalServices }} total)
                  </span>
                </div>
              </div>
            </div>

            <div class="flex flex-col sm:flex-row items-start sm:items-center gap-4">
              <div class="flex items-center gap-2">
                <label for="pageSize" class="label label-text text-base-content">Show:</label>
                <select id="pageSize" v-model="pageSize" @change="resetPagination"
                  class="select select-bordered select-sm">
                  <option :value="10">10</option>
                  <option :value="25">25</option>
                  <option :value="50">50</option>
                  <option :value="100">100</option>
                </select>
              </div>

              <div class="join">
                <button @click="previousPage" :disabled="currentPage === 1"
                  class="btn join-item btn-outline" :class="{ 'btn-disabled': currentPage === 1 }">
                  ← Prev
                </button>
                <button class="btn join-item no-animation pointer-events-none">
                  {{ currentPage }} / {{ totalPages }}
                </button>
                <button @click="nextPage" :disabled="currentPage === totalPages"
                  class="btn join-item btn-outline" :class="{ 'btn-disabled': currentPage === totalPages }">
                  Next →
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Error State -->
      <div v-if="error" class="alert alert-error shadow-lg mb-6">
        <svg class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
        </svg>
        <div>
          <h3 class="font-bold">Error Loading Services</h3>
          <div class="text-xs">Failed to load services data. Please try refreshing the page.</div>
        </div>
      </div>

      <!-- Loading State -->
      <div v-else-if="loading" class="card bg-base-100 shadow-lg mb-6">
        <div class="card-body flex items-center justify-center">
          <span class="loading loading-spinner loading-lg"></span>
          <span class="ml-3 text-base-content">Loading services...</span>
        </div>
      </div>

      <!-- Services Table -->
      <div v-else class="card bg-base-100 shadow-lg overflow-hidden">
        <div class="overflow-x-auto">
          <table class="table table-zebra">
            <thead>
              <tr>
                <th>Name</th>
                <th>Display Name</th>
                <th>State</th>
                <th>Start Mode</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              <template v-for="service in paginatedServices" :key="service.name">
                <tr class="hover">
                  <td class="font-medium text-base-content">{{ service.name }}</td>
                  <td class="text-base-content/70">{{ service.displayName }}</td>
                  <td>
                    <div :class="getStateBadgeClass(service.state)" class="badge">
                      {{ service.state }}
                    </div>
                  </td>
                  <td class="text-base-content/70">{{ service.startMode }}</td>
                  <td>
                    <div class="flex items-center space-x-2">
                      <Button :text="service.isExpanded ? 'Hide Info' : 'Show Info'"
                        :class="service.isExpanded ? 'btn btn-ghost btn-sm' : 'btn btn-primary btn-sm'"
                        @clicked="service.isExpanded = !service.isExpanded" />

                      <Button v-if="service.startMode !== 'Disabled'"
                        :text="service.isDisabling ? 'Disabling...' : 'Disable'"
                        :disabled="service.isDisabling"
                        class="btn btn-error btn-sm"
                        @clicked="openModal(service)" />
                    </div>
                  </td>
                </tr>
                <tr v-show="service.isExpanded" class="bg-base-200">
                  <td colspan="5" class="p-4">
                    <div class="space-y-4">
                      <div class="flex items-center justify-between">
                        <h4 class="text-lg font-bold text-base-content">Service Details</h4>
                        <Button :text="service.isReloading ? 'Reloading...' : 'Reload Info'"
                          :disabled="service.isDisabling"
                          class="btn btn-warning btn-sm"
                          @clicked="reloadInfo(service)" />
                      </div>

                      <div v-if="service.info && !service.info.error" class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div class="md:col-span-2">
                          <div class="flex items-center gap-2 mb-2">
                            <label class="label label-text text-base-content/70">Information Source:</label>
                            <div :class="getSourceBadgeClass(service.info.source)" class="badge badge-sm">
                              {{ service.info.source === 'ai' ? 'AI Generated' : 'Web Scraped' }}
                            </div>
                          </div>
                        </div>

                        <div v-if="service.info.url" class="md:col-span-2">
                          <label class="label label-text text-base-content/70">Source URL</label>
                          <a :href="service.info.url" target="_blank"
                            class="link link-primary break-all">{{ service.info.url }}</a>
                        </div>

                        <div v-if="service.info.description">
                          <label class="label label-text text-base-content/70">Description</label>
                          <p class="text-base-content">{{ service.info.description }}</p>
                        </div>

                        <div v-if="service.info.explained">
                          <label class="label label-text text-base-content/70">Explanation</label>
                          <p class="text-base-content">{{ service.info.explained }}</p>
                        </div>

                        <div v-if="service.info.recommendation" class="md:col-span-2">
                          <label class="label label-text text-base-content/70">Recommendation</label>
                          <div class="alert alert-info">
                            <div class="whitespace-pre-line">{{ service.info.recommendation }}</div>
                          </div>
                        </div>
                      </div>

                      <div v-else class="text-center py-4">
                        <p class="text-error font-medium">
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
          <svg class="mx-auto h-12 w-12 text-base-content/50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          <h3 class="mt-2 text-lg font-bold text-base-content">No services found</h3>
          <p class="mt-1 text-base-content/70">Try adjusting your search or filter criteria.</p>
        </div>
        </div>
      </div>
    </div>

  <!-- Confirmation Modal -->
  <div v-if="showModal" class="modal modal-open">
    <div class="modal-box">
      <h3 class="font-bold text-lg">Confirm Disable Service</h3>
      <p class="py-4">Are you sure you want to disable the service "{{ selectedService?.displayName }}"?</p>
      <div class="modal-action">
        <button class="btn" @click="showModal = false">Cancel</button>
        <button class="btn btn-error" @click="confirmDisable">Disable</button>
      </div>
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
const showModal = ref(false)
const selectedService = ref(null)

// Pagination
const currentPage = ref(1)
const pageSize = ref(25)

const totalServices = computed(() => allServices.value.length)
const totalPages = computed(() => Math.ceil(filteredServices.value.length / pageSize.value))

// Analytics computed properties
const servicesByState = computed(() => {
  const counts = {}
  allServices.value.forEach(service => {
    counts[service.state] = (counts[service.state] || 0) + 1
  })
  return counts
})

const servicesByStartMode = computed(() => {
  const counts = {}
  allServices.value.forEach(service => {
    counts[service.startMode] = (counts[service.startMode] || 0) + 1
  })
  return counts
})

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

const openModal = (service) => {
  selectedService.value = service
  showModal.value = true
}

const confirmDisable = () => {
  if (selectedService.value) {
    disable(selectedService.value)
    showModal.value = false
    selectedService.value = null
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
    'Running': 'badge-success',
    'Stopped': 'badge-error',
    'Paused': 'badge-warning',
    'Pending': 'badge-info'
  }
  return 'badge ' + (classes[state] || 'badge-neutral')
}

const getSourceBadgeClass = (source) => {
  const classes = {
    'ai': 'badge-warning',
    'scraped': 'badge-info'
  }
  return 'badge ' + (classes[source] || 'badge-neutral')
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
