<template>
    <div class="card bg-base-100 card-border border-base-300 mb-6">
        <div class="card-body">
            <div class="collapse collapse-arrow bg-base-200 mb-4">
                <input type="checkbox" />
                <div class="collapse-title text-lg font-semibold text-base-content">
                    Analytics
                </div>
                <div class="collapse-content">
                    <div class="grid grid-cols-2 md:grid-cols-4 gap-3">
                        <div class="bg-base-100 rounded-lg p-4">
                            <div class="text-sm font-medium text-base-content/70">Total Apps</div>
                            <div class="text-3xl font-bold text-primary">{{ stats.total }}</div>
                        </div>
                        <div class="bg-base-100 rounded-lg p-4">
                            <div class="text-sm font-medium text-base-content/70">Enabled</div>
                            <div class="text-3xl font-bold text-success">{{ stats.enabled }}</div>
                        </div>
                        <div class="bg-base-100 rounded-lg p-4">
                            <div class="text-sm font-medium text-base-content/70">Disabled</div>
                            <div class="text-3xl font-bold text-warning">{{ stats.disabled }}</div>
                        </div>
                        <div class="bg-base-100 rounded-lg p-4">
                            <div class="text-sm font-medium text-base-content/70">Registry</div>
                            <div class="text-3xl font-bold text-info">{{ stats.fromRegistry }}</div>
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
        <Icon name="search" />
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
                            <div class="badge" :class="`badge-${getLocationColor(app.location)}`">
                                {{ formatLocation(app.location) }}
                            </div>
                        </td>
                        <td style="word-break: break-word; max-width: 300px;">
                            <code class="text-xs bg-base-200 px-2 py-1 rounded tooltip" :data-tip="fullCommand(app)">
                                {{ fullCommand(app) }}
                            </code>
                        </td>
                        <td>
                            <div :class="`badge ${app.enabled ? 'badge-success' : 'badge-warning'}`">
                                {{ app.enabled ? 'Enabled' : 'Disabled' }}
                            </div>
                        </td>
                        <td>
                            <div class="flex gap-2">
                                <Button class="btn btn-info btn-sm btn-square" @clicked="editApp(app)">
                                    <Icon name="edit" />
                                </Button>
                                <Button class="btn btn-success btn-sm btn-square" @clicked="confirmRemove(app)"
                                    :is-loading="removingApp === `${app.name}-${app.location}`">
                                    <Icon name="shutDown" />
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

const {
    filteredApps,
    loading,
    error,
    searchQuery,
    selectedLocation,
    selectedStatus,
    stats,
    locationOptions,
    loadStartupApps,
    addApp,
    removeApp,
    clearFilters,
} = useStartupApps()

const showAddModal = ref(false)
const showConfirmRemove = ref(false)
const removeAppConfirm = ref(null)
const removingApp = ref(null)
const editingApp = ref(null)

const formatLocation = (location) => {
    const map = {
        'HkeyLocalMachine': 'HKLM Registry',
        'HkeyCurrentUser': 'HKCU Registry',
        'StartupFolder': 'Startup Folder',
    }
    return map[location] || location
}

const getLocationColor = (location) => {
    const map = {
        'HkeyLocalMachine': 'primary',
        'HkeyCurrentUser': 'info',
        'StartupFolder': 'warning',
    }
    return map[location] || 'neutral'
}

const fullCommand = (app) => {
    return app.arguments ? `${app.path} ${app.arguments}` : app.path
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
