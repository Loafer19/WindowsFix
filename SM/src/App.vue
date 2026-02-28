<template>
    <div class="min-h-screen bg-base-200">
        <div class="container mx-auto p-6">

            <div role="tablist" class="tabs tabs-boxed gap-2 p-0 mb-6">
                <label class="tab gap-1 text-lg font-medium hover:text-info" v-for="tab in tabs" :key="tab.id">
                    <input v-model="activeTab" type="radio" name="tabs_main" class="tab" :value="tab.component" />
                    <Icon :name="tab.icon" />
                    {{ tab.name }}
                </label>
            </div>

            <div class="card bg-base-100 card-border border-base-300 mb-6">
                <div class="card-body">
                    <component :is="activeTab" :servicesByStatus="servicesByStatus"
                        :servicesByStartupType="servicesByStartupType" :totalServices="totalServices"
                        :filteredCount="filteredServices.length" :totalCount="totalServices" :searchQuery="searchQuery"
                        :selectedStatus="selectedStatus" :selectedStartupType="selectedStartupType"
                        @filter="handleFilter" @refresh="refresh" @clear-filters="clearFilters" />
                </div>
            </div>

            <div v-if="error" class="alert alert-error">
                <Icon name="alarmWarning" />
                <h3 class="font-bold">Failed to load services data</h3>
                <div class="text-xs">Please try refreshing the page :(</div>
            </div>

            <div v-else-if="loading" class="card bg-base-100 card-border border-base-300">
                <div class="card-body flex items-center justify-center">
                    <span class="loading loading-spinner loading-lg"></span>
                </div>
            </div>

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
                            <template v-for="service in filteredServices" :key="service.name">
                                <tr>
                                    <td class="font-medium text-base-content">
                                        <span class="tooltip tooltip-right" :data-tip="service.displayName">{{
                                            service.name }}</span>
                                    </td>
                                    <td>
                                        <div :class="getStatusBadgeClass(service.status)" class="badge">
                                            {{ service.status }}
                                        </div>
                                    </td>
                                    <td>
                                        <div :class="getStartupTypeBadgeClass(service.startupType)" class="badge">
                                            {{ service.startupType }}
                                        </div>
                                    </td>
                                    <td>
                                        <div class="flex items-center space-x-2">
                                            <Button class="btn btn-info btn-sm btn-square"
                                                @clicked="openModalForDetails(service)">
                                                <Icon name="eye" />
                                            </Button>

                                            <Button v-if="service.startupType != 'Disabled'"
                                                :disabled="service.isDisabling" :is-loading="service.isDisabling"
                                                class="btn btn-success btn-sm btn-square" @clicked="openModal(service)">
                                                <Icon name="shutDown" />
                                            </Button>
                                        </div>
                                    </td>
                                </tr>
                            </template>
                        </tbody>
                    </table>
                </div>

                <div v-if="filteredServices.length === 0" class="text-center py-12">
                    <h3 class="mt-2 text-lg font-bold text-base-content">No services found</h3>
                    <p class="mt-1 text-base-content/70">Try adjusting your search or filter criteria.</p>
                </div>
            </div>
        </div>
    </div>

    <ConfirmDisableModal :showModal="showModal" :selectedService="selectedService" @close="showModal = false"
        @confirm="confirmDisable" />

    <ServiceDetailsModal :showModal="showDetailsModal" :selectedService="selectedServiceForDetails"
        @close="showDetailsModal = false" @reload="reloadInfo" />
</template>

<script setup>
import { markRaw, onMounted, ref } from 'vue'
import Button from './components/Button.vue'
import Icon from './components/Icon.vue'
import ConfirmDisableModal from './components/Modals/ConfirmDisableModal.vue'
import ServiceDetailsModal from './components/Modals/ServiceDetailsModal.vue'
import AnalyticsTab from './components/Tabs/AnalyticsTab.vue'
import FiltersTab from './components/Tabs/FiltersTab.vue'
import { useAnalytics } from './composables/useAnalytics.js'
import { useFiltering } from './composables/useFiltering.js'
import { useModals } from './composables/useModals.js'
import {
    disableService,
    loadServices,
    refreshServices,
    reloadServiceInfo,
} from './services/api.js'

const tabs = ref([
    {
        id: 'filters',
        name: 'Filters',
        component: markRaw(FiltersTab),
        icon: 'equalizer',
    },
    {
        id: 'analytics',
        name: 'Analytics',
        component: markRaw(AnalyticsTab),
        icon: 'fileChart',
    },
])
const activeTab = ref(markRaw(FiltersTab))

const allServices = ref([])
const error = ref(false)
const loading = ref(true)

const { totalServices, servicesByStatus, servicesByStartupType } =
    useAnalytics(allServices)
const { filteredServices, searchQuery, selectedStatus, selectedStartupType, filterServices, handleFilter, clearFilters } =
    useFiltering(allServices)
const {
    showModal,
    selectedService,
    showDetailsModal,
    selectedServiceForDetails,
    openModal,
    confirmDisable: confirmDisableModal,
    openModalForDetails,
} = useModals()

const confirmDisable = () => confirmDisableModal(disable)

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

        // Also update the original service in allServices array
        const originalService = allServices.value.find(
            (s) => s.name === service.name,
        )
        if (originalService) {
            originalService.info = data
        }

        console.log(
            `Successfully reloaded information for service: ${service.name}`,
        )
    } catch (error) {
        console.error('Failed to reload service info:', error)
        service.info = { error: true, message: 'Failed to reload information' }

        // Also update the original service in allServices array
        const originalService = allServices.value.find(
            (s) => s.name === service.name,
        )
        if (originalService) {
            originalService.info = service.info
        }
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

const getStatusBadgeClass = (status) => {
    const classes = {
        Running: 'badge-error',
        Stopped: 'badge-neutral',
        Paused: 'badge-warning',
        Pending: 'badge-warning',
    }
    return classes[status] || 'badge-neutral'
}

const getStartupTypeBadgeClass = (startupType) => {
    const classes = {
        Automatic: 'badge-error',
        Manual: 'badge-warning',
        Disabled: 'badge-neutral',
        System: 'badge-info',
        Boot: 'badge-info',
    }
    return classes[startupType] || 'badge-neutral'
}
</script>

<style scoped>
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
