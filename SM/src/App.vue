<template>
  <div class="min-h-screen bg-base-200">
    <div class="container mx-auto p-6">

      <div role="tablist" class="tabs tabs-boxed gap-2 p-0 mb-6">
        <label class="tab gap-1 text-lg font-medium hover:text-info" v-for="tab in tabs" :key="tab.id">
          <input v-model="activeTab" type="radio" name="tabs_main" class="tab" :value="tab.component" />
          <component :is="getIconComponent(tab.icon)" class="w-6 h-6" />
          {{ tab.name }}
        </label>
      </div>

      <div class="card bg-base-100 card-border border-base-300 mb-6">
        <div class="card-body">
          <component :is="activeTab" :servicesByStatus="servicesByStatus" :servicesByStartupType="servicesByStartupType"
            :totalServices="totalServices" :filteredCount="filteredServices.length" :totalCount="totalServices"
            @filter="handleFilter" />
        </div>
      </div>


      <!-- Error State -->
      <div v-if="error" class="alert alert-error shadow-lg mb-6">
        <svg class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd"
            d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
            clip-rule="evenodd" />
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
      <div v-else class="card bg-base-100 card-border border-base-300">
        <div class="overflow-x-auto">
          <table class="table">
            <thead>
              <tr>
                <th>Name</th>
                <th>Status</th>
                <th>Startup Type</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              <template v-for="service in paginatedServices" :key="service.name">
                <tr>
                  <td class="font-medium text-base-content">
                    <span class="tooltip tooltip-right" :data-tip="service.displayName">{{ service.name }}</span>
                  </td>
                  <td>
                    <div :class="getStatusBadgeClass(service.status)" class="badge">
                      {{ service.status }}
                    </div>
                  </td>
                  <td class="text-base-content/70">{{ service.startupType }}</td>
                  <td>
                    <div class="flex items-center space-x-2">
                      <Button :text="service.isExpanded ? 'Hide Info' : 'Show Info'"
                        :class="service.isExpanded ? 'btn btn-ghost btn-sm' : 'btn btn-primary btn-sm'"
                        @clicked="service.isExpanded = !service.isExpanded" />

                      <Button v-if="service.startupType !== 'Disabled'"
                        :text="service.isDisabling ? 'Disabling...' : 'Disable'" :disabled="service.isDisabling"
                        class="btn btn-error btn-sm" @clicked="openModal(service)" />
                    </div>
                  </td>
                </tr>
                <tr v-show="service.isExpanded" class="bg-base-200">
                  <td colspan="4" class="p-4">
                    <div class="space-y-4">
                      <div class="flex items-center justify-between">
                        <h4 class="text-lg font-bold text-base-content">Service Details</h4>
                        <Button :text="service.isReloading ? 'Reloading...' : 'Reload Info'"
                          :disabled="service.isDisabling" class="btn btn-warning btn-sm"
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
                          <a :href="service.info.url" target="_blank" class="link link-primary break-all">{{
                            service.info.url }}</a>
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
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
              d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
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
import { ref, onMounted, computed, watch, markRaw } from 'vue'
import { loadServices, reloadServiceInfo, disableService, refreshServices } from './services/api.js'
import Button from './components/Button.vue'
import FiltersTab from './components/FiltersTab.vue'
import AnalyticsTab from './components/AnalyticsTab.vue'

const allServices = ref([])
const filteredServices = ref([])
const searchQuery = ref('')
const selectedStatus = ref('')
const selectedStartupType = ref('')
const error = ref(false)
const loading = ref(true)
const showModal = ref(false)
const selectedService = ref(null)

// Tab system
const tabs = ref([
  {
    id: 'filters',
    name: 'Filters',
    component: markRaw(FiltersTab),
    icon: 'FunnelIcon'
  },
  {
    id: 'analytics',
    name: 'Analytics',
    component: markRaw(AnalyticsTab),
    icon: 'ChartBarIcon'
  }
])
const activeTab = ref(markRaw(FiltersTab))

// Analytics computed properties
const totalServices = computed(() => allServices.value.length)
const servicesByStatus = computed(() => {
  const counts = {}
  allServices.value.forEach(service => {
    counts[service.status] = (counts[service.status] || 0) + 1
  })
  return counts
})

const servicesByStartupType = computed(() => {
  const counts = {}
  allServices.value.forEach(service => {
    counts[service.startupType] = (counts[service.startupType] || 0) + 1
  })
  return counts
})

const paginatedServices = computed(() => {
  return filteredServices.value
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

  if (selectedStatus.value) {
    filtered = filtered.filter((service) => service.status === selectedStatus.value)
  }

  if (selectedStartupType.value) {
    filtered = filtered.filter((service) => service.startupType === selectedStartupType.value)
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

const getStatusBadgeClass = (status) => {
  const classes = {
    'Running': 'badge-error',
    'Stopped': 'badge-success',
    'Paused': 'badge-warning',
    'Pending': 'badge-info'
  }
  return 'badge ' + (classes[status] || 'badge-neutral')
}

const getSourceBadgeClass = (source) => {
  const classes = {
    'ai': 'badge-warning',
    'scraped': 'badge-info'
  }
  return 'badge ' + (classes[source] || 'badge-neutral')
}

// Icon components
const FunnelIcon = {
  template: `<svg fill="currentColor" viewBox="0 0 24 24">
    <path d="M13.85 22.25h-3.7c-.74 0-1.36-.54-1.45-1.27l-1.01-8.6c-.1-.83.48-1.59 1.3-1.69l3.7-.42c.82-.1 1.59.48 1.69 1.3l1.01 8.6c.1.83-.48 1.59-1.3 1.69z"/>
    <path d="M12 2.25c-.41 0-.75.34-.75.75v2c0 .41.34.75.75.75s.75-.34.75-.75v-2c0-.41-.34-.75-.75-.75z"/>
    <path d="M16.5 5.25c-.19 0-.38-.07-.53-.22l-1.5-1.5c-.29-.29-.29-.77 0-1.06s.77-.29 1.06 0l1.5 1.5c.29.29.29.77 0 1.06-.15.15-.34.22-.53.22z"/>
    <path d="M7.5 5.25c-.19 0-.38-.07-.53-.22-.29-.29-.29-.77 0-1.06l1.5-1.5c.29-.29.77-.29 1.06 0s.29.77 0 1.06l-1.5 1.5c-.15.15-.34.22-.53.22z"/>
  </svg>`
}

const ChartBarIcon = {
  template: `<svg fill="currentColor" viewBox="0 0 24 24">
    <path d="M3 3v18h18V3H3zm16 16H5V5h14v14z"/>
    <path d="M7 7h2v10H7V7zm4 0h2v10h-2V7zm4 0h2v10h-2V7z"/>
  </svg>`
}

// Get icon component function
const getIconComponent = (iconName) => {
  const icons = {
    FunnelIcon,
    ChartBarIcon
  }
  return icons[iconName] || FunnelIcon
}

// Handle filter updates from FiltersTab
const handleFilter = (filterData) => {
  searchQuery.value = filterData.searchQuery
  selectedStatus.value = filterData.selectedStatus
  selectedStartupType.value = filterData.selectedStartupType
  filterServices()
}

// Watch for filter changes
watch([searchQuery, selectedStatus, selectedStartupType], () => {
  filterServices()
})
</script>

<style scoped>
.whitespace-pre-line {
  white-space: pre-line;
}

/* Custom tab styling */
.tabs-box {
  box-shadow: none;
}

.tab {
  border: var(--border) solid var(--color-base-300) !important;
  border-radius: var(--radius-field) !important;
  box-shadow: none;
  transition: all 0.2s ease;
}

.tab:hover {
  border-color: var(--color-primary) !important;
  background-color: var(--color-primary) !important;
  color: var(--color-primary-content) !important;
}

.tab:has(input:checked) {
  border-color: var(--color-primary) !important;
  background-color: var(--color-primary) !important;
  color: var(--color-primary-content) !important;
}
</style>

<style scoped></style>
