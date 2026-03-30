<template>
    <div class="card bg-base-100 card-border border-base-300 mb-6">
        <div class="card-body">
            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div v-for="preset in presets" :key="preset.id"
                    :class="`card bg-base-200 card-border border-base-300 hover:border-${preset.color} transition-colors`">
                    <div class="card-body">
                        <div class="flex items-center gap-3 mb-2">
                            <div :class="`bg-${preset.color} text-${preset.color} p-2 rounded-lg`">
                                <Icon :name="preset.icon" class="w-6 h-6" />
                            </div>
                            <h3 class="card-title text-base-content">{{ preset.name }}</h3>
                        </div>

                        <p class="text-base-content/70 text-sm flex-1">{{ preset.description }}</p>

                        <div class="mt-3">
                            <div class="text-xs text-base-content/50 mb-2 font-medium uppercase tracking-wide">
                                {{ preset.services.length }} services affected
                            </div>
                            <div class="flex flex-wrap gap-1 mb-4">
                                <span v-for="svc in preset.services.slice(0, 4)" :key="svc.name"
                                    class="badge badge-neutral badge-sm">
                                    {{ svc.name }}
                                </span>
                                <span v-if="preset.services.length > 4" class="badge badge-ghost badge-sm">
                                    +{{ preset.services.length - 4 }} more
                                </span>
                            </div>
                        </div>

                        <div class="card-actions">
                            <Button :class="`btn btn-${preset.color} btn-sm w-full`"
                                @clicked="openPresetModal(preset)">
                                <Icon :name="preset.icon" />
                                Apply "{{ preset.name }}"
                            </Button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <PresetConfirmModal :showModal="showPresetModal" :preset="selectedPreset" :applying="applyingPreset"
        @close="showPresetModal = false" @confirm="applyPreset" />
</template>

<script setup>
import { onMounted } from 'vue'
import { presets } from '../../services/presets.js'
import Button from '../Button.vue'
import PresetConfirmModal from '../Modals/PresetConfirmModal.vue'
import { usePresets } from '../../composables/usePresets.js'
import { useServices } from '../../composables/useServices.js'

const { allServices, loadServicesData } = useServices()

const {
    showPresetModal,
    selectedPreset,
    applyingPreset,
    openPresetModal,
    applyPreset,
} = usePresets(allServices)

onMounted(() => {
    // Delay loading to allow UI to render first
    setTimeout(async () => {
        await loadServicesData()
    }, 100)
})
</script>
