<template>
    <div v-if="showModal" class="modal modal-open">
        <div class="modal-box max-w-4xl">
            <div class="flex items-center justify-between mb-4">
                <div>
                    <h3 class="font-bold text-lg">{{ selectedService?.name }}</h3>
                    <p class="text-base-content/60 text-sm">{{ selectedService?.displayName }}</p>
                </div>
                <div class="flex gap-2">
                    <Button :text="'Reload Info'" class="btn btn-secondary btn-square btn-sm"
                        :is-loading="selectedService?.isReloading" :disabled="selectedService?.isReloading"
                        @clicked="$emit('reload', selectedService)">
                        <Icon name="resetRight" />
                    </Button>
                    <button class="btn btn-ghost btn-square btn-sm" @click="$emit('close')">
                        <Icon name="close" />
                    </button>
                </div>
            </div>

            <!-- Current State -->
            <div class="flex gap-3 mb-4">
                <div :class="`badge badge-${getStatusColor(selectedService?.status)}`">
                    {{ selectedService?.status }}
                </div>
                <div :class="`badge badge-${getStartupTypeColor(selectedService?.startupType)}`">
                    {{ selectedService?.startupType }}
                </div>
            </div>

            <!-- Service Controls -->
            <div class="card bg-base-200 card-border border-base-300 mb-4">
                <div class="card-body p-4">
                    <h4 class="font-semibold text-sm text-base-content/70 uppercase tracking-wide mb-3">Service Controls</h4>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                            <label class="label label-text text-xs">Status</label>
                            <div class="flex gap-2">
                                <Button
                                    class="btn btn-success btn-sm flex-1"
                                    :disabled="selectedService?.isActioning || selectedService?.status === 'Running'"
                                    :is-loading="selectedService?.isActioning && pendingAction === 'start'"
                                    @clicked="doAction('start')">
                                    <Icon name="play" />
                                    Start
                                </Button>
                                <Button
                                    class="btn btn-warning btn-sm flex-1"
                                    :disabled="selectedService?.isActioning || selectedService?.status === 'Stopped'"
                                    :is-loading="selectedService?.isActioning && pendingAction === 'stop'"
                                    @clicked="doAction('stop')">
                                    <Icon name="stop" />
                                    Stop
                                </Button>
                                <Button
                                    class="btn btn-info btn-sm flex-1"
                                    :disabled="selectedService?.isActioning || selectedService?.status !== 'Running'"
                                    :is-loading="selectedService?.isActioning && pendingAction === 'restart'"
                                    @clicked="doAction('restart')">
                                    <Icon name="restart" />
                                    Restart
                                </Button>
                            </div>
                        </div>
                        <div>
                            <label class="label label-text text-xs">Startup Type</label>
                            <div class="flex gap-2">
                                <select
                                    v-model="newStartupType"
                                    class="select select-bordered select-sm flex-1"
                                    :disabled="selectedService?.isActioning">
                                    <option value="Automatic">Automatic</option>
                                    <option value="Manual">Manual</option>
                                    <option value="Disabled">Disabled</option>
                                </select>
                                <Button
                                    class="btn btn-primary btn-sm"
                                    :disabled="selectedService?.isActioning || newStartupType === selectedService?.startupType"
                                    :is-loading="selectedService?.isActioning && pendingAction === 'startup'"
                                    @clicked="doAction('startup')">
                                    Apply
                                </Button>
                            </div>
                        </div>
                    </div>
                    <div v-if="actionError" class="alert alert-error mt-3 py-2">
                        <Icon name="alarmWarning" class="w-4 h-4" />
                        <span class="text-sm">{{ actionError }}</span>
                    </div>
                </div>
            </div>

            <!-- Service Info -->
            <div class="space-y-4">
                <div v-if="selectedService.info?.description">
                    <label class="label label-text text-base-content/70">Description</label>
                    <p class="text-base-content">{{ selectedService.info.description }}</p>
                </div>

                <div v-if="selectedService.info?.explained">
                    <label class="label label-text text-base-content/70">Explanation</label>
                    <p class="text-base-content">{{ selectedService.info.explained }}</p>
                </div>

                <div v-if="selectedService.info?.recommendation" class="md:col-span-2">
                    <label class="label label-text text-base-content/70">Recommendation</label>
                    <div class="alert alert-info">
                        <div class="whitespace-pre-line">{{ selectedService.info.recommendation }}</div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup>
import { ref, watch } from 'vue'
import { getStartupTypeColor, getStatusColor } from '../../services/helpers.js'
import Button from '../Button.vue'
import Icon from '../Icon.vue'

const props = defineProps({
    showModal: {
        type: Boolean,
        default: false,
    },
    selectedService: {
        type: Object,
        default: null,
    },
})

const emit = defineEmits(['close', 'reload', 'action'])

const newStartupType = ref('')
const pendingAction = ref('')
const actionError = ref('')

watch(
    () => props.selectedService,
    (svc) => {
        if (svc) {
            newStartupType.value = svc.startupType || 'Manual'
            actionError.value = ''
            pendingAction.value = ''
        }
    },
    { immediate: true },
)

// Sync error from parent (set via service.actionError in App.vue)
watch(
    () => props.selectedService?.actionError,
    (err) => {
        if (err) actionError.value = err
    },
)

// Sync startup type when action completes (e.g. after set_startup_type)
watch(
    () => props.selectedService?.startupType,
    (type) => {
        if (type && !props.selectedService?.isActioning) {
            newStartupType.value = type
        }
    },
)

const doAction = (action) => {
    actionError.value = ''
    pendingAction.value = action
    const payload =
        action === 'startup'
            ? { action, startupType: newStartupType.value }
            : { action }
    emit('action', props.selectedService, payload)
}
</script>
