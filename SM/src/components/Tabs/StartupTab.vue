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
                            By Location
                        </h4>
                        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                            <div v-for="(count, type) in appsByLocation" :key="type" class="bg-base-100 rounded-lg p-4">
                                <div class="flex items-center gap-2 capitalize font-medium">
                                    {{ formatLocation(type) }}
                                </div>
                                <div class="text-xl" :class="`text-${getLocationColor(type)}`">{{ count.length }}</div>
                                <div class="text-base-content/70">
                                    {{ stats.total > 0 ? ((count.length / stats.total) * 100).toFixed(1) : 0 }}% of
                                    total
                                </div>
                                <progress class="progress h-2 mt-2" :class="`progress-${getLocationColor(type)}`"
                                    :value="stats.total > 0 ? ((count.length / stats.total) * 100) : 0"
                                    max="100"></progress>
                            </div>
                        </div>
                    </div>

                    <div>
                        <h4 class="text-lg font-semibold text-base-content mb-4">
                            By Status
                        </h4>
                        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                            <div class="bg-base-100 rounded-lg p-4">
                                <div class="flex items-center gap-2 capitalize font-medium">
                                    Enabled
                                </div>
                                <div class="text-2xl text-success">{{ stats.enabled }}</div>
                                <div class="text-base-content/70">
                                    {{ stats.total > 0 ? ((stats.enabled / stats.total) * 100).toFixed(1) : 0 }}% of
                                    total
                                </div>
                                <progress class="progress h-2 mt-2 progress-success"
                                    :value="stats.total > 0 ? ((stats.enabled / stats.total) * 100) : 0"
                                    max="100"></progress>
                            </div>
                            <div class="bg-base-100 rounded-lg p-4">
                                <div class="flex items-center gap-2 capitalize font-medium">
                                    Disabled
                                </div>
                                <div class="text-2xl text-warning">{{ stats.disabled }}</div>
                                <div class="text-base-content/70">
                                    {{ stats.total > 0 ? ((stats.disabled / stats.total) * 100).toFixed(1) : 0 }}% of
                                    total
                                </div>
                                <progress class="progress h-2 mt-2 progress-warning"
                                    :value="stats.total > 0 ? ((stats.disabled / stats.total) * 100) : 0"
                                    max="100"></progress>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="collapse collapse-arrow bg-base-200">
                <input type="checkbox" checked />
                <div class="collapse-title text-lg font-semibold text-base-content flex items-center gap-2">
                    Filters
                </div>
                <div class="collapse-content">
                    <div class="mb-4">
                        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                            <div class="bg-base-100 rounded-lg p-4">
                                <div class="capitalize font-medium">Showing</div>
                                <div class="text-2xl text-primary">{{ filteredApps.length }}</div>
                                <div class="text-base-content/70">
                                    {{ stats.total > 0 ? ((filteredApps.length / stats.total) * 100).toFixed(1)
                                        : 0 }}% of total
                                </div>
                                <progress class="progress progress-primary mt-2"
                                    :value="stats.total > 0 ? ((filteredApps.length / stats.total) * 100) : 0"
                                    max="100"></progress>
                            </div>
                        </div>
                    </div>

                    <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
                        <div class="form-control">
                            <label for="searchFilter" class="label">
                                Search
                            </label>
                            <input id="searchFilter" v-model="searchQuery" type="text" placeholder="Name or Path"
                                class="input input-bordered" />
                        </div>

                        <div class="form-control">
                            <label class="label">
                                Location
                            </label>
                            <select v-model="selectedLocation" class="select select-bordered">
                                <option value="">All Locations</option>
                                <option v-for="loc in locationOptions" :key="loc.value" :value="loc.value">
                                    {{ loc.label }}
                                </option>
                            </select>
                        </div>

                        <div class="form-control">
                            <label class="label">
                                Status
                            </label>
                            <select v-model="selectedStatus" class="select select-bordered">
                                <option value="">All Statuses</option>
                                <option value="true">Enabled</option>
                                <option value="false">Disabled</option>
                            </select>
                        </div>

                        <div class="form-control">
                            <label class="label">
                                Actions
                            </label>
                            <div class="flex gap-2">
                                <Button :text="'Refresh'" @clicked="refresh" class="btn btn-info btn-square"
                                    :is-loading="loading">
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

    <div v-if="loading" class="alert alert-info">
        <div class="flex items-center justify-center">
            <span class="loading loading-spinner loading-lg"></span>
        </div>
    </div>

    <div v-else-if="error" class="alert alert-error">
        <Icon name="alarmWarning" />
        <h3 class="font-bold">Failed to load startup apps</h3>
        <div class="text-xs">{{ error }}</div>
    </div>

    <div v-else-if="filteredApps.length === 0" class="alert alert-warning">
        <Icon name="alarmWarning" />
        <h3 class="font-bold">No startup apps found</h3>
        <div class="text-xs">Try adjusting your filters</div>
    </div>

    <div v-else class="card bg-base-100 card-border border-base-300">
        <div class="overflow-x-auto">
            <table class="table">
                <thead>
                    <tr>
                        <th>Name</th>
                        <th>Location</th>
                        <th>Command</th>
                        <th>Status</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>
                    <tr v-for="app in filteredApps" :key="`${app.name}-${app.location}`">
                        <td class="font-medium">
                            {{ app.name }}
                        </td>
                        <td>
                            <div class="badge text-nowrap" :class="`badge-${getLocationColor(app.location)}`">
                                {{ formatLocation(app.location) }}
                            </div>
                        </td>
                        <td>
                            <code class="text-xs bg-base-200 p-1 rounded break-all">
                                {{ app.path }}
                            </code>
                        </td>
                        <td>
                            <div :class="`badge ${app.enabled ? 'badge-warning' : 'badge-success'}`">
                                {{ app.enabled ? 'Enabled' : 'Disabled' }}
                            </div>
                        </td>
                        <td>
                            <div class="flex gap-2">
                                <Button class="btn btn-sm btn-square tooltip"
                                    :class="app.enabled ? 'btn-success' : 'btn-warning'" @clicked="handleToggle(app)"
                                    :is-loading="togglingApp === `${app.name}-${app.location}`"
                                    :data-tip="app.enabled ? 'Disable' : 'Enable'">
                                    <Icon :name="app.enabled ? 'shutDown' : 'checkBoxCircle'" />
                                </Button>
                                <Button class="btn btn-info btn-sm btn-square" @clicked="editApp(app)">
                                    <Icon name="edit" />
                                </Button>
                                <Button class="btn btn-warning btn-sm btn-square" @clicked="confirmRemove(app)"
                                    :is-loading="removingApp === `${app.name}-${app.location}`">
                                    <Icon name="deleteBin" />
                                </Button>
                            </div>
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>
    </div>

    <StartupAppModal :show="showAddModal" :app="editingApp" @close="handleModalClose" @confirm="handleAddApp" />

    <ConfirmModal :show="showConfirmRemove" :title="`Remove ${removeAppConfirm?.name}?`"
        :message="`This will remove '${removeAppConfirm?.name}' from startup.`" @close="showConfirmRemove = false"
        @confirm="handleRemoveApp" />
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useStartupApps } from '../../composables/useStartupApps.js'
import Button from '../Button.vue'
import Icon from '../Icon.vue'
import StartupAppModal from '../Modals/StartupAppModal.vue'
import ConfirmModal from '../Modals/ConfirmModal.vue'
import { getLocationColor, formatLocation } from '../../services/helpers.js'

const {
    filteredApps,
    loading,
    error,
    searchQuery,
    selectedLocation,
    selectedStatus,
    stats,
    appsByLocation,
    locationOptions,
    loadStartupApps,
    addApp,
    removeApp,
    toggleApp,
    clearFilters,
} = useStartupApps()

const showAddModal = ref(false)
const showConfirmRemove = ref(false)
const removeAppConfirm = ref(null)
const removingApp = ref(null)
const togglingApp = ref(null)
const editingApp = ref(null)

const handleToggle = async (app) => {
    try {
        togglingApp.value = `${app.name}-${app.location}`
        await toggleApp(app)
    } catch (err) {
        console.error('Failed to toggle app:', err)
    } finally {
        togglingApp.value = null
    }
}

const editApp = (app) => {
    editingApp.value = app
    showAddModal.value = true
}

const confirmRemove = (app) => {
    removeAppConfirm.value = app
    showConfirmRemove.value = true
}

const handleRemoveApp = async () => {
    if (!removeAppConfirm.value) return

    try {
        removingApp.value = `${removeAppConfirm.value.name}-${removeAppConfirm.value.location}`
        await removeApp(removeAppConfirm.value)
        showConfirmRemove.value = false
    } catch (err) {
        console.error('Failed to remove app:', err)
    } finally {
        removingApp.value = null
    }
}

const handleModalClose = () => {
    showAddModal.value = false
    editingApp.value = null
}

const handleAddApp = async (app) => {
    try {
        if (editingApp.value) {
            await removeApp(editingApp.value)
            await addApp(app)
        } else {
            await addApp(app)
        }
        showAddModal.value = false
        editingApp.value = null
    } catch (err) {
        console.error('Failed to save app:', err)
    }
}

const refresh = async () => {
    await loadStartupApps()
}

// Load startup apps on mount
onMounted(() => {
    // Delay loading to allow UI to render first
    setTimeout(async () => {
        await loadStartupApps()
    }, 100)
})
</script>
