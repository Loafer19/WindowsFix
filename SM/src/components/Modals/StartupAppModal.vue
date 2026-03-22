<template>
    <div v-if="show" class="modal modal-open">
        <div class="modal-box w-11/12 max-w-2xl">
            <h3 class="font-bold text-lg mb-4">{{ props.app ? 'Edit Startup Application' : 'Add Startup Application' }}</h3>

            <div class="space-y-4">
                <!-- App Name -->
                <div class="form-control">
                    <label class="label">
                        <span class="label-text">Application Name *</span>
                    </label>
                    <input
                        v-model="formData.name"
                        type="text"
                        placeholder="e.g., MyApp"
                        class="input input-bordered"
                    />
                </div>

                <!-- App Path -->
                <div class="form-control">
                    <label class="label">
                        <span class="label-text">Executable Path *</span>
                    </label>
                    <input
                        v-model="formData.path"
                        type="text"
                        placeholder="C:\Program Files\MyApp\app.exe"
                        class="input input-bordered"
                    />
                </div>

                <!-- Arguments -->
                <div class="form-control">
                    <label class="label">
                        <span class="label-text">Arguments (Optional)</span>
                    </label>
                    <input
                        v-model="formData.arguments"
                        type="text"
                        placeholder="-silent -minimize"
                        class="input input-bordered"
                    />
                </div>

                <!-- Location -->
                <div class="form-control">
                    <label class="label">
                        <span class="label-text">Startup Location *</span>
                    </label>
                    <select v-model="formData.location" class="select select-bordered">
                        <option value="">Select location...</option>
                        <option value="HkeyCurrentUser">Current User Registry</option>
                        <option value="HkeyLocalMachine">All Users Registry (Admin Required)</option>
                        <option value="StartupFolder">User Startup Folder</option>
                    </select>
                </div>

                <!-- Info Box -->
                <div class="alert alert-info">
                    <Icon name="info" class="w-5 h-5" />
                    <div>
                        <h3 class="font-bold">Startup Locations</h3>
                        <ul class="text-sm mt-2 space-y-1">
                            <li>• <strong>Current User:</strong> Only for your account</li>
                            <li>• <strong>All Users:</strong> For all accounts (requires admin)</li>
                            <li>• <strong>Startup Folder:</strong> Quick access folder in Start Menu</li>
                        </ul>
                    </div>
                </div>
            </div>

            <div class="modal-action">
                <button class="btn" @click="$emit('close')">Cancel</button>
                <Button
                    class="btn btn-primary"
                    @clicked="submit"
                    :disabled="!isFormValid"
                    :is-loading="isSubmitting"
                >
                    {{ props.app ? 'Update Application' : 'Add Application' }}
                </Button>
            </div>
        </div>
    </div>
</template>

<script setup>
import { ref, computed, watch } from 'vue'
import Button from '../Button.vue'
import Icon from '../Icon.vue'

const props = defineProps({
    show: Boolean,
    app: Object,
})

const emit = defineEmits(['close', 'confirm'])

const formData = ref({
    name: '',
    path: '',
    arguments: '',
    location: '',
    enabled: true,
    description: null,
})

const isSubmitting = ref(false)

// Watch for app changes to pre-fill form when editing
watch(() => props.app, (newApp) => {
    if (newApp) {
        formData.value = {
            name: newApp.name || '',
            path: newApp.path || '',
            arguments: newApp.arguments || '',
            location: newApp.location || '',
            enabled: newApp.enabled !== undefined ? newApp.enabled : true,
            description: newApp.description || null,
        }
    } else {
        // Reset form for new app
        formData.value = {
            name: '',
            path: '',
            arguments: '',
            location: '',
            enabled: true,
            description: null,
        }
    }
}, { immediate: true })

const isFormValid = computed(() => {
    return formData.value.name.trim() &&
           formData.value.path.trim() &&
           formData.value.location
})

const submit = async () => {
    if (!isFormValid.value) return

    try {
        isSubmitting.value = true
        emit('confirm', { ...formData.value })
        formData.value = {
            name: '',
            path: '',
            arguments: '',
            location: '',
            enabled: true,
            description: null,
        }
    } finally {
        isSubmitting.value = false
    }
}
</script>
