<template>
    <div class="card bg-base-100 card-border border-base-300 mb-6">
        <div class="card-body">
            <div class="collapse collapse-arrow bg-base-200 mb-4">
                <input type="checkbox" />
                <div class="collapse-title text-lg font-semibold text-base-content">
                    Analytics
                </div>
                <div class="collapse-content">
                    <div class="mb-4">
                        <h4 class="text-lg font-semibold text-base-content mb-4">
                            By Status
                        </h4>
                        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                            <div v-for="(count, status) in servicesByStatus" :key="status"
                                class="bg-base-100 rounded-lg p-4">
                                <div class="flex items-center gap-2 capitalize font-medium">
                                    {{ status }}
                                </div>
                                <div class="text-2xl" :class="`text-${getStatusColor(status)}`">{{ count }}</div>
                                <div class="text-base-content/70">
                                    {{ totalServices > 0 ? ((count / totalServices) * 100).toFixed(1) : 0 }}% of total
                                </div>
                                <progress class="progress h-2 mt-2" :class="`progress-${getStatusColor(status)}`"
                                    :value="totalServices > 0 ? ((count / totalServices) * 100) : 0"
                                    max="100"></progress>
                            </div>
                        </div>
                    </div>

                    <div>
                        <h4 class="text-lg font-semibold text-base-content mb-4">
                            By Startup Type
                        </h4>
                        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                            <div v-for="(count, type) in servicesByStartupType" :key="type"
                                class="bg-base-100 rounded-lg p-4">
                                <div class="flex items-center gap-2 capitalize font-medium">
                                    {{ type }}
                                </div>
                                <div class="text-xl" :class="`text-${getStartupTypeColor(type)}`">{{ count }}</div>
                                <div class="text-base-content/70">
                                    {{ totalServices > 0 ? ((count / totalServices) * 100).toFixed(1) : 0 }}% of total
                                </div>
                                <progress class="progress h-2 mt-2" :class="`progress-${getStartupTypeColor(type)}`"
                                    :value="totalServices > 0 ? ((count / totalServices) * 100) : 0"
                                    max="100"></progress>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="collapse collapse-arrow bg-base-200">
                <input type="checkbox" checked />
                <div class="collapse-title text-lg font-semibold text-base-content">
                    Filters
                </div>
                <div class="collapse-content">
                    <div class="mb-4">
                        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                            <div class="bg-base-100 rounded-lg p-4">
                                <div class="capitalize font-medium">Showing</div>
                                <div class="text-2xl text-primary">{{ filteredServices.length }}</div>
                                <div class="text-base-content/70">
                                    {{ totalServices > 0 ? ((filteredServices.length / totalServices) * 100).toFixed(1)
                                        : 0 }}% of total
                                </div>
                                <progress class="progress progress-primary mt-2"
                                    :value="totalServices > 0 ? ((filteredServices.length / totalServices) * 100) : 0"
                                    max="100"></progress>
                            </div>
                        </div>
                    </div>

                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                        <div class="form-control">
                            <label for="searchFilter" class="label">
                                Search
                            </label>
                            <input id="searchFilter" v-model="searchQuery" class="input input-bordered"
                                placeholder="Name or Display Name" />
                        </div>

                        <div class="form-control">
                            <label for="statusFilter" class="label">
                                Status
                            </label>
                            <select id="statusFilter" v-model="selectedStatus" class="select select-bordered">
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
                            <select id="startupTypeFilter" v-model="selectedStartupType" class="select select-bordered">
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
                                <Button :text="'Clear Filters'" @clicked="clearFilters"
                                    class="btn btn-neutral btn-square">
                                    <Icon name="filterOff" />
                                </Button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
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
                                <div :class="`badge-${getStatusColor(service.status)}`" class="badge">
                                    {{ service.status }}
                                </div>
                            </td>
                            <td>
                                <div :class="`badge-${getStartupTypeColor(service.startupType)}`" class="badge">
                                    {{ service.startupType }}
                                </div>
                            </td>
                            <td>
                                <div class="flex items-center space-x-2">
                                    <Button class="btn btn-info btn-sm btn-square"
                                        @clicked="openModalForDetails(service)">
                                        <Icon name="eye" />
                                    </Button>

                                    <Button v-if="service.startupType != 'Disabled'" :disabled="service.isDisabling"
                                        :is-loading="service.isDisabling" class="btn btn-success btn-sm btn-square"
                                        @clicked="openModal(service)">
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

    <ConfirmDisableModal :showModal="showModal" :selectedService="selectedService" @close="showModal = false"
        @confirm="confirmDisable" />

    <ServiceDetailsModal :showModal="showDetailsModal" :selectedService="selectedServiceForDetails"
        @close="showDetailsModal = false" @reload="reloadInfo" @action="handleServiceAction" />
</template>

<script setup>
import { onMounted } from 'vue'
import Button from '../Button.vue'
import Icon from '../Icon.vue'
import ConfirmDisableModal from '../Modals/ConfirmDisableModal.vue'
import ServiceDetailsModal from '../Modals/ServiceDetailsModal.vue'

import { useServices } from '../../composables/useServices.js'
import { useModals } from '../../composables/useModals.js'
import { getStartupTypeColor, getStatusColor } from '../../services/helpers.js'

const { allServices, loading, error, reloadInfo, handleServiceAction, disable, totalServices, servicesByStatus, servicesByStartupType, filteredServices, searchQuery, selectedStatus, selectedStartupType, clearFilters, loadServicesData, refresh } = useServices()

const { showModal, selectedService, showDetailsModal, selectedServiceForDetails, openModal, confirmDisable: confirmDisableModal, openModalForDetails } = useModals()

const confirmDisable = () => confirmDisableModal(disable)

onMounted(async () => {
    await loadServicesData()
})
</script>
