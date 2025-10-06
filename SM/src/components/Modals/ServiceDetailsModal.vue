<template>
    <div v-if="showModal" class="modal modal-open">
        <div class="modal-box max-w-4xl">
            <div class="flex items-center justify-between">
                <h3 class="font-bold text-lg">{{ selectedService?.name }} - {{ selectedService?.displayName }}</h3>
                <Button :text="'Reload Info'" class="btn btn-secondary btn-square btn-sm"
                    :is-loading="selectedService?.isReloading"
                    :disabled="selectedService?.isReloading"
                    @clicked="$emit('reload', selectedService)">
                    <Icon name="resetRight" />
                </Button>
            </div>
            <div class="space-y-4">
                <div v-if="selectedService.info.description">
                    <label class="label label-text text-base-content/70">Description</label>
                    <p class="text-base-content">{{ selectedService.info.description }}</p>
                </div>

                <div v-if="selectedService.info.explained">
                    <label class="label label-text text-base-content/70">Explanation</label>
                    <p class="text-base-content">{{ selectedService.info.explained }}</p>
                </div>

                <div v-if="selectedService.info.recommendation" class="md:col-span-2">
                    <label class="label label-text text-base-content/70">Recommendation</label>
                    <div class="alert alert-info">
                        <div class="whitespace-pre-line">{{ selectedService.info.recommendation }}</div>
                    </div>
                </div>
            </div>
            <div class="modal-action">
                <button class="btn" @click="$emit('close')">Close</button>
            </div>
        </div>
    </div>
</template>

<script setup>
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

defineEmits(['close', 'reload'])
</script>

<style scoped>
.whitespace-pre-line {
    white-space: pre-line;
}
</style>
