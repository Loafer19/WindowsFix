<template>
    <div v-if="showPrompt && update" class="modal modal-open">
        <div class="modal-box">
            <div class="flex items-center justify-between mb-2">
                <h3 class="font-bold text-lg">Update available</h3>
                <button class="btn btn-ghost btn-square btn-sm" :disabled="installing" @click="dismissPrompt">
                    <Icon name="close" />
                </button>
            </div>

            <p class="py-2">
                Version <span class="font-mono font-bold">{{ update.version }}</span> is ready to install
                <span v-if="currentVersion" class="text-base-content/60">(you have {{ currentVersion }})</span>
            </p>
            <p class="text-sm text-base-content/60">
                The app will close, install the update and start again
            </p>

            <div v-if="update.body" class="mt-4 max-h-40 overflow-auto rounded bg-base-200 p-3 text-xs whitespace-pre-wrap">
                {{ update.body }}
            </div>

            <progress v-if="installing" class="progress progress-primary w-full mt-4" :value="progress" max="100"></progress>
            <p v-if="error" class="text-sm text-error mt-4">{{ error }}</p>

            <div class="modal-action">
                <button class="btn" :disabled="installing" @click="dismissPrompt">Later</button>
                <button class="btn btn-primary" :disabled="installing" @click="installUpdate">
                    <span v-if="installing" class="loading loading-spinner loading-sm"></span>
                    Install and restart
                </button>
            </div>
        </div>
    </div>
</template>

<script setup>
import { useUpdater } from '../../composables/useUpdater.js'
import Icon from '../Icon.vue'

const {
    update,
    currentVersion,
    showPrompt,
    installing,
    progress,
    error,
    installUpdate,
    dismissPrompt,
} = useUpdater()
</script>
