<!-- src/Services.vue -->
<template>
  <div class="container mx-auto p-6">
    <h1 class="text-3xl font-bold text-gray-800 mb-6">Windows Services</h1>

    <!-- Filter Dropdown -->
    <div class="mb-6">
      <label for="typeFilter" class="text-gray-700 font-semibold mr-2">Filter by Start Mode:</label>
      <select id="typeFilter" v-model="selectedFilter" @change="filterServices"
        class="p-2 border rounded-lg text-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500">
        <option value="all">All</option>
        <option value="Auto">Auto</option>
        <option value="Manual">Manual</option>
        <option value="Disabled">Disabled</option>
        <option value="System">System</option>
        <option value="Boot">Boot</option>
      </select>
    </div>

    <!-- Services Table -->
    <div class="overflow-x-auto">
      <table v-if="!error" class="w-full bg-white shadow-md rounded-lg">
        <thead class="bg-gray-200 text-gray-700">
          <tr>
            <th class="py-3 px-4 text-left">Name</th>
            <th class="py-3 px-4 text-left">Display Name</th>
            <th class="py-3 px-4 text-left">State</th>
            <th class="py-3 px-4 text-left">Start Mode</th>
            <th class="py-3 px-4 text-left">Site Details</th>
          </tr>
        </thead>
        <tbody class="text-gray-600">
          <template v-for="(service, index) in filteredServices" :key="service.name">
            <!-- Service Row -->
            <tr class="border-b expandable">
              <td class="py-3 px-4">{{ service.name }}</td>
              <td class="py-3 px-4">{{ service.displayName }}</td>
              <td class="py-3 px-4">{{ service.state }}</td>
              <td class="py-3 px-4">{{ service.startMode }}</td>
              <td class="py-3 px-4">
                <span @click="toggleDetails(index)" class="toggle-details text-blue-500 underline cursor-pointer">
                  {{ isExpanded(index) ? 'Hide Details' : 'Show Details' }}
                </span>
                <button v-if="hasFetchError(service.site)" @click="reloadInfo(service, index)"
                  class="reload-info ml-2 text-red-500 underline" :disabled="service.isReloading">
                  {{ service.isReloading ? 'Reloading...' : 'Reload Info' }}
                </button>
              </td>
            </tr>
            <!-- Details Row (immediately follows the service row) -->
            <tr :class="{ 'hidden': !isExpanded(index) }" class="details">
              <td colspan="5" class="py-3 px-4 bg-gray-50">
                <div>
                  <p><strong>Default Description:</strong> {{ service.site.defaultDescription }}</p>
                  <p><strong>Normal Description:</strong> {{ service.site.normalDescription }}</p>
                  <p><strong>Recommendations:</strong> {{ service.site.recommendations }}</p>
                </div>
              </td>
            </tr>
          </template>
        </tbody>
      </table>
      <div v-else class="text-red-500 py-3 px-4">Failed to load services data.</div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { loadServices, reloadServiceInfo, hasFetchError } from './services.js';

// Reactive state
const allServices = ref([]);
const filteredServices = ref([]);
const selectedFilter = ref('all');
const expanded = ref({}); // Tracks expanded state by index
const error = ref(false);

// Load services on component mount
onMounted(async () => {
  try {
    allServices.value = await loadServices();
    filterServices();
  } catch (err) {
    error.value = true;
    console.error('Failed to load services:', err);
  }
});

// Toggle details for a service
const toggleDetails = (index) => {
  console.log(`Toggling details for index ${index}`); // Debug log
  expanded.value[index] = !expanded.value[index];
};

// Helper to check if a row is expanded
const isExpanded = (index) => {
  return expanded.value[index] || false;
};

// Reload service info
const reloadInfo = async (service, index) => {
  console.log(`Reloading info for ${service.name}`); // Debug log
  service.isReloading = true;
  const newSiteData = await reloadServiceInfo(service.name);
  service.site = newSiteData;
  service.isReloading = false;
};

// Filter services based on selected start mode
const filterServices = () => {
  console.log(`Filtering services with ${selectedFilter.value}`); // Debug log
  if (selectedFilter.value === 'all') {
    filteredServices.value = [...allServices.value];
  } else {
    filteredServices.value = allServices.value.filter(
      service => service.startMode.toLowerCase() === selectedFilter.value.toLowerCase()
    );
  }
};
</script>

<style scoped>
.expandable:hover {
  cursor: pointer;
  background-color: #f1f5f9;
}

.hidden {
  display: none;
}
</style>
