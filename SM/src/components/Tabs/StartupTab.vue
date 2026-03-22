<template>
    <div>
        <!-- Stats Section -->
        <div class="mb-6">
            <div class="grid grid-cols-2 md:grid-cols-4 gap-3">
                <div class="bg-base-200 rounded-lg p-4">
                    <div class="text-sm font-medium text-base-content/70">Total Apps</div>
                    <div class="text-3xl font-bold text-primary">{{ stats.total }}</div>
                </div>
                <div class="bg-base-200 rounded-lg p-4">
                    <div class="text-sm font-medium text-base-content/70">Enabled</div>
                    <div class="text-3xl font-bold text-success">{{ stats.enabled }}</div>
                </div>
                <div class="bg-base-200 rounded-lg p-4">
                    <div class="text-sm font-medium text-base-content/70">Disabled</div>
                    <div class="text-3xl font-bold text-warning">{{ stats.disabled }}</div>
                </div>
                <div class="bg-base-200 rounded-lg p-4">
                    <div class="text-sm font-medium text-base-content/70">Registry</div>
                    <div class="text-3xl font-bold text-info">{{ stats.fromRegistry }}</div>
                </div>
            </div>
        </div>

        <!-- Filters Section -->
        <div class="card bg-base-100 card-border border-base-300 mb-6">
            <div class="card-body">
                <h3 class="card-title text-base mb-4 flex items-center gap-2">
                    <Icon name="funnel" class="w-5 h-5" />
                    Filters
                </h3>

                <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
                    <!-- Search -->
                    <div class="form-control">
                        <label class="label">
                            <span class="label-text">Search</span>
                        </label>
                        <input
                            v-model="searchQuery"
                            type="text"
                            placeholder="Search by name or path..."
                            class="input input-bordered"
                        />
                    </div>

                    <!-- Location Filter -->
                    <div class="form-control">
                        <label class="label">
                            <span class="label-text">Location</span>
                        </label>
                        <select v-model="selectedLocation" class="select select-bordered">
                            <option :value="null">All Locations</option>
                            <option v-for="loc in locationOptions" :key="loc.value" :value="loc.value">
                                {{ loc.label }}
                            </option>
                        </select>
                    </div>

                    <!-- Status Filter -->
                    <div class="form-control">
                        <label class="label">
                            <span class="label-text">Status</span>
                        </label>
                        <select v-model.number="selectedStatus" class="select select-bordered">
                            <option :value="null">All Statuses</option>
                            <option :value="true">Enabled</option>
                            <option :value="false">Disabled</option>
                        </select>
                    </div>

                    <!-- Actions -->
                    <div class="form-control">
                        <label class="label">
                            <span class="label-text">Actions</span>
                        </label>
                        <div class="flex gap-2">
                            <Button
                                class="btn btn-info btn-square"
                                @clicked="refresh"
                                :is-loading="loading"
                            >
                                <Icon name="refresh" class="w-4 h-4" />
                            </Button>
                            <Button
                                class="btn btn-neutral btn-square"
                                @clicked="clearFilters"
                            >
                                <Icon name="filterOff" class="w-4 h-4" />
                            </Button>
                            <Button
                                class="btn btn-primary btn-square"
                                @clicked="showAddModal = true"
                            >
                                <Icon name="plus" class="w-4 h-4" />
                            </Button>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <!-- Apps Table -->
        <div v-if="loading" class="card bg-base-100 card-border border-base-300">
            <div class="card-body flex items-center justify-center h-40">
                <span class="loading loading-spinner loading-lg"></span>
            </div>
        </div>

        <div v-else-if="error" class="alert alert-error">
            <Icon name="alarmWarning" />
            <div>
                <h3 class="font-bold">Failed to load startup apps</h3>
                <div class="text-xs">{{ error }}</div>
            </div>
        </div>

        <div v-else-if="filteredApps.length === 0" class="text-center py-12">
            <h3 class="text-lg font-bold text-base-content">No startup apps found</h3>
            <p class="text-base-content/70">Try adjusting your filters</p>
        </div>

        <div v-else class="card bg-base-100 card-border border-base-300">
            <div class="overflow-x-auto">
                <table class="table">
                    <thead>
                        <tr>
                            <th>Name</th>
                            <th>Location</th>
                            <th>Path</th>
                            <th>Arguments</th>
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
                            <td class="max-w-xs">
                                <span class="tooltip tooltip-right" :data-tip="app.path">
                                    {{ truncatePath(app.path) }}
                                </span>
                            </td>
                            <td>
                                <code v-if="app.arguments" class="text-xs bg-base-200 px-2 py-1 rounded">
                                    {{ app.arguments }}
                                </code>
                                <span v-else class="text-base-content/50">—</span>
                            </td>
                            <td>
                                <div :class="`badge ${app.enabled ? 'badge-success' : 'badge-warning'}`">
                                    {{ app.enabled ? 'Enabled' : 'Disabled' }}
                                </div>
                            </td>
                            <td>
                                <div class="flex gap-2">
                                    <Button
                                        class="btn btn-info btn-xs"
                                        @clicked="editApp(app)"
                                    >
                                        <Icon name="pencil" class="w-3 h-3" />
                                    </Button>
                                    <Button
                                        class="btn btn-error btn-xs"
                                        @clicked="confirmRemove(app)"
                                        :is-loading="removingApp === `${app.name}-${app.location}`"
                                    >
                                        <Icon name="trash" class="w-3 h-3" />
                                    </Button>
                                </div>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>

        <!-- Add/Edit App Modal -->
        <StartupAppModal
            :show="showAddModal"
            :app="editingApp"
            @close="handleModalClose"
            @confirm="handleAddApp"
        />

        <!-- Confirm Remove Modal -->
        <ConfirmModal
            :show="showConfirmRemove"
            :title="`Remove ${removeAppConfirm?.name}?`"
            :message="`This will remove '${removeAppConfirm?.name}' from startup.`"
            @close="showConfirmRemove = false"
            @confirm="handleRemoveApp"
        />
    </div>
</template>

<script setup>
import { ref } from 'vue'
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

const truncatePath = (path) => {
    return path.length > 50 ? `${path.substring(0, 50)}...` : path
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
            // Editing: remove old app first, then add new one
            await removeApp(editingApp.value)
            await addApp(app)
        } else {
            // Adding new app
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

loadStartupApps()
</script>
