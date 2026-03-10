<template>
    <div class="mb-4">
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
            <div class="bg-base-200 rounded-box p-4">
                <div class="capitalize font-medium">Showing</div>
                <div class="text-2xl text-primary">{{ filteredCount }}</div>
                <div class="text-base-content/70">
                    {{ ((filteredCount / totalCount) * 100).toFixed(1) }}% of total
                </div>
                <progress class="progress progress-primary mt-2" :value="((filteredCount / totalCount) * 100)"
                    max="100"></progress>
            </div>
        </div>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-4">
        <div class="form-control">
            <label for="searchFilter" class="label">
                Search
            </label>
            <input id="searchFilter" :value="searchQuery" @input="updateSearch" class="input input-bordered"
                placeholder="Name or Display Name" />
        </div>

        <div class="form-control">
            <label for="statusFilter" class="label">
                Status
            </label>
            <select id="statusFilter" :value="selectedStatus" @change="updateStatus" class="select select-bordered">
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
            <select id="startupTypeFilter" :value="selectedStartupType" @change="updateStartupType"
                class="select select-bordered">
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
                <Button :text="'Clear Filters'" @clicked="clearFilters" class="btn btn-neutral btn-square">
                    <Icon name="filterOff" />
                </Button>
            </div>
        </div>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div class="form-control">
            <label for="sortByFilter" class="label">
                Sort By
            </label>
            <select id="sortByFilter" :value="sortBy" @change="updateSortBy" class="select select-bordered">
                <option value="status">Status (Running first)</option>
                <option value="name">Name (A–Z)</option>
                <option value="startupType">Startup Type</option>
            </select>
        </div>

        <div class="form-control">
            <label for="sortDirFilter" class="label">
                Sort Direction
            </label>
            <div class="flex gap-2" id="sortDirFilter">
                <Button
                    :class="`btn btn-sm flex-1 ${sortDir === 'asc' ? 'btn-primary' : 'btn-neutral'}`"
                    @clicked="updateSortDir('asc')">
                    <Icon name="sortAsc" />
                    Ascending
                </Button>
                <Button
                    :class="`btn btn-sm flex-1 ${sortDir === 'desc' ? 'btn-primary' : 'btn-neutral'}`"
                    @clicked="updateSortDir('desc')">
                    <Icon name="sortDesc" />
                    Descending
                </Button>
            </div>
        </div>
    </div>
</template>

<script setup>
import Button from '../Button.vue'
import Icon from '../Icon.vue'

const props = defineProps({
    filteredCount: {
        type: Number,
        default: 0,
    },
    totalCount: {
        type: Number,
        default: 0,
    },
    searchQuery: {
        type: String,
        default: '',
    },
    selectedStatus: {
        type: String,
        default: '',
    },
    selectedStartupType: {
        type: String,
        default: '',
    },
    sortBy: {
        type: String,
        default: 'status',
    },
    sortDir: {
        type: String,
        default: 'asc',
    },
})

const emit = defineEmits(['filter', 'refresh', 'clear-filters'])

const refresh = () => {
    emit('refresh')
}

const updateSearch = (event) => {
    emit('filter', {
        searchQuery: event.target.value,
        selectedStatus: props.selectedStatus,
        selectedStartupType: props.selectedStartupType,
        sortBy: props.sortBy,
        sortDir: props.sortDir,
    })
}

const updateStatus = (event) => {
    emit('filter', {
        searchQuery: props.searchQuery,
        selectedStatus: event.target.value,
        selectedStartupType: props.selectedStartupType,
        sortBy: props.sortBy,
        sortDir: props.sortDir,
    })
}

const updateStartupType = (event) => {
    emit('filter', {
        searchQuery: props.searchQuery,
        selectedStatus: props.selectedStatus,
        selectedStartupType: event.target.value,
        sortBy: props.sortBy,
        sortDir: props.sortDir,
    })
}

const updateSortBy = (event) => {
    emit('filter', {
        searchQuery: props.searchQuery,
        selectedStatus: props.selectedStatus,
        selectedStartupType: props.selectedStartupType,
        sortBy: event.target.value,
        sortDir: props.sortDir,
    })
}

const updateSortDir = (dir) => {
    emit('filter', {
        searchQuery: props.searchQuery,
        selectedStatus: props.selectedStatus,
        selectedStartupType: props.selectedStartupType,
        sortBy: props.sortBy,
        sortDir: dir,
    })
}

const clearFilters = () => {
    emit('clear-filters')
}
</script>
