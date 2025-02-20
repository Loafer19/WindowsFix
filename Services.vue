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
                    <tr v-for="(service, index) in filteredServices" :key="service.name" class="border-b expandable">
                        <td class="py-3 px-4">{{ service.name }}</td>
                        <td class="py-3 px-4">{{ service.displayName }}</td>
                        <td class="py-3 px-4">{{ service.state }}</td>
                        <td class="py-3 px-4">{{ service.startMode }}</td>
                        <td class="py-3 px-4">
                            <span @click="toggleDetails(index)"
                                class="toggle-details text-blue-500 underline cursor-pointer">
                                {{ expanded[index] ? 'Hide Details' : 'Show Details' }}
                            </span>
                            <button v-if="hasFetchError(service.site)" @click="reloadInfo(service, index)"
                                class="reload-info ml-2 text-red-500 underline" :disabled="service.isReloading">
                                {{ service.isReloading ? 'Reloading...' : 'Reload Info' }}
                            </button>
                        </td>
                    </tr>
                    <tr v-for="(service, index) in filteredServices" :key="`${service.name}-details`"
                        :class="{ hidden: !expanded[index] }" class="details">
                        <td colspan="5" class="py-3 px-4 bg-gray-50">
                            <div>
                                <p><strong>Default Description:</strong> {{ service.site.defaultDescription }}</p>
                                <p><strong>Normal Description:</strong> {{ service.site.normalDescription }}</p>
                                <p><strong>Recommendations:</strong> {{ service.site.recommendations }}</p>
                            </div>
                        </td>
                    </tr>
                </tbody>
            </table>
            <div v-else class="text-red-500 py-3 px-4">Failed to load services data.</div>
        </div>
    </div>
</template>

<script>
export default {
    name: 'Services',
    data() {
        return {
            allServices: [],
            filteredServices: [],
            selectedFilter: 'all',
            expanded: {}, // Tracks which rows are expanded
            error: false
        };
    },
    async created() {
        await this.loadServices();
    },
    methods: {
        async loadServices() {
            try {
                const response = await fetch('services-enhanced.json');
                if (!response.ok) throw new Error('Failed to fetch JSON');
                const data = await response.json();
                this.allServices = data.allServices.map(service => ({
                    ...service,
                    isReloading: false // Add flag for reload state
                }));
                this.filterServices();
            } catch (error) {
                console.error('Error loading services:', error);
                this.error = true;
            }
        },
        hasFetchError(site) {
            return (
                site.defaultDescription === 'Error fetching data' ||
                site.normalDescription === 'Error fetching data' ||
                site.recommendations === 'Error fetching data'
            );
        },
        async reloadInfo(service, index) {
            service.isReloading = true;
            try {
                const response = await fetch(`http://localhost:3000/reload/${service.name}`);
                if (!response.ok) throw new Error('API error');
                const newSiteData = await response.json();
                service.site = newSiteData;
            } catch (error) {
                console.error(`Reload failed for ${service.name}: ${error.message}`);
                service.site = {
                    defaultDescription: 'Reload failed',
                    normalDescription: 'Reload failed',
                    recommendations: 'Reload failed'
                };
            } finally {
                service.isReloading = false;
            }
            this.$forceUpdate(); // Ensure UI updates
        },
        toggleDetails(index) {
            this.$set(this.expanded, index, !this.expanded[index]);
        },
        filterServices() {
            if (this.selectedFilter === 'all') {
                this.filteredServices = [...this.allServices];
            } else {
                this.filteredServices = this.allServices.filter(
                    service => service.startMode.toLowerCase() === this.selectedFilter.toLowerCase()
                );
            }
        }
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
