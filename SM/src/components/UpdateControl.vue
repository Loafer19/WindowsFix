<template>
    <div class="flex items-center gap-2 ml-auto self-center">
        <span v-if="currentVersion" class="badge badge-ghost font-mono text-xs">v{{ currentVersion }}</span>
        <span v-if="error" class="text-xs text-error max-w-xs truncate" :title="error">Update check failed</span>
        <span v-else-if="checked && !update" class="text-xs text-success">Up to date</span>
        <button v-if="update" class="btn btn-sm btn-primary" @click="openPrompt">
            Update to v{{ update.version }}
        </button>
        <button v-else class="btn btn-sm btn-outline" :disabled="checking" @click="checkForUpdates()">
            <span v-if="checking" class="loading loading-spinner loading-xs"></span>
            Check for updates
        </button>
    </div>
</template>

<script setup>
import { onMounted } from 'vue'
import { useUpdater } from '../composables/useUpdater.js'

const {
    currentVersion,
    update,
    checking,
    checked,
    error,
    loadVersion,
    checkForUpdates,
    openPrompt,
} = useUpdater()

onMounted(loadVersion)
</script>
