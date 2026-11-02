<template>
    <div class="card bg-base-100 card-border border-base-300 mb-6">
        <div class="card-body">
            <div class="flex items-center justify-between flex-wrap gap-2 mb-2">
                <h3 class="text-lg font-semibold text-base-content">AI Insights</h3>
                <div v-if="aiStatus" :class="`badge ${aiStatus.configured ? 'badge-success' : 'badge-warning'}`">
                    {{ statusLabel }}
                </div>
            </div>
            <p class="text-base-content/70 text-sm mb-4">
                Service explanations and recommendations are generated with the xAI API using your own key
            </p>

            <div v-if="aiStatusError" class="alert alert-error mb-4 py-2">
                <Icon name="alarmWarning" class="w-4 h-4" />
                <span class="text-sm">{{ aiStatusError }}</span>
            </div>

            <!-- API key -->
            <div class="bg-base-200 rounded-lg p-4 mb-4">
                <div class="flex items-center justify-between flex-wrap gap-2 mb-2">
                    <label class="label label-text font-medium" for="ai-key">xAI API Key</label>
                    <button class="btn btn-link btn-sm px-0" @click="openConsole">
                        Get a key at console.x.ai
                    </button>
                </div>

                <div v-if="aiStatus?.configured" class="text-sm text-base-content/70 mb-2">
                    Current key: <span class="font-mono">{{ aiStatus.maskedKey }}</span>
                    <span v-if="aiStatus.source === 'environment'"> (from the GROK_API_KEY environment variable)</span>
                    <span v-else> (stored in Windows Credential Manager)</span>
                </div>

                <div class="join w-full">
                    <input
                        id="ai-key"
                        v-model="keyInput"
                        :type="showKey ? 'text' : 'password'"
                        class="input input-bordered input-sm join-item flex-1 font-mono"
                        :placeholder="aiStatus?.source === 'credentialManager' ? 'Paste a new key to replace the saved one' : 'xai-…'"
                        autocomplete="off"
                        spellcheck="false"
                        @keyup.enter="saveKey" />
                    <button class="btn btn-sm join-item" :title="showKey ? 'Hide key' : 'Show key'" @click="showKey = !showKey">
                        <Icon name="eye" class="w-4 h-4" />
                        {{ showKey ? 'Hide' : 'Show' }}
                    </button>
                </div>

                <div class="flex gap-2 flex-wrap mt-3">
                    <Button class="btn btn-primary btn-sm" :disabled="!keyInput.trim() || busy !== ''"
                        :is-loading="busy === 'save'" @clicked="saveKey">
                        Save Key
                    </Button>
                    <Button class="btn btn-info btn-sm" :disabled="busy !== '' || (!keyInput.trim() && !aiStatus?.configured)"
                        :is-loading="busy === 'test'" @clicked="testConnection">
                        <Icon name="router" class="w-4 h-4" />
                        Test Connection
                    </Button>
                    <Button class="btn btn-error btn-outline btn-sm"
                        :disabled="busy !== '' || aiStatus?.source !== 'credentialManager'"
                        :is-loading="busy === 'clear'" @clicked="clearKey">
                        <Icon name="deleteBin" class="w-4 h-4" />
                        Clear Key
                    </Button>
                </div>

                <p class="text-xs text-base-content/60 mt-3">
                    The key is stored only in Windows Credential Manager on this PC and is sent only to api.x.ai
                </p>
            </div>

            <!-- Model and limits -->
            <div class="bg-base-200 rounded-lg p-4">
                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div>
                        <label class="label label-text text-xs" for="ai-model">Model</label>
                        <input id="ai-model" v-model="form.model" list="ai-models"
                            class="input input-bordered input-sm w-full font-mono" spellcheck="false" />
                        <datalist id="ai-models">
                            <option v-for="m in suggestedModels" :key="m" :value="m" />
                        </datalist>
                    </div>
                    <div>
                        <label class="label label-text text-xs" for="ai-timeout">Timeout (seconds)</label>
                        <input id="ai-timeout" v-model.number="form.timeoutSecs" type="number" min="5" max="120"
                            class="input input-bordered input-sm w-full" />
                    </div>
                    <div>
                        <label class="label label-text text-xs" for="ai-tokens">Max Tokens</label>
                        <input id="ai-tokens" v-model.number="form.maxTokens" type="number" min="100" max="8000"
                            step="100" class="input input-bordered input-sm w-full" />
                    </div>
                </div>
                <div class="flex gap-2 flex-wrap mt-3">
                    <Button class="btn btn-primary btn-sm" :disabled="busy !== '' || !settingsChanged"
                        :is-loading="busy === 'settings'" @clicked="saveSettings">
                        Save Settings
                    </Button>
                    <button class="btn btn-ghost btn-sm" :disabled="busy !== ''" @click="resetDefaults">
                        <Icon name="resetRight" class="w-4 h-4" />
                        Defaults
                    </button>
                </div>
            </div>

            <div v-if="message" :class="`alert ${messageType === 'error' ? 'alert-error' : 'alert-success'} mt-4 py-2`">
                <Icon :name="messageType === 'error' ? 'alarmWarning' : 'checkBoxCircle'" class="w-4 h-4" />
                <span class="text-sm">{{ message }}</span>
            </div>
        </div>
    </div>
</template>

<script setup>
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { useAiSettings } from '../../composables/useAiSettings.js'
import {
    clearAiKey,
    errorMessage,
    saveAiKey,
    saveAiSettings,
    testAiConnection,
} from '../../services/api.js'
import Button from '../Button.vue'

const DEFAULTS = { model: 'grok-4.3', timeoutSecs: 30, maxTokens: 1000 }
const suggestedModels = ['grok-4.3', 'grok-4.7', 'grok-4.20-0309-non-reasoning']

const { aiStatus, aiStatusError, loadAiStatus, setAiStatus } = useAiSettings()

const keyInput = ref('')
const showKey = ref(false)
const busy = ref('')
const message = ref('')
const messageType = ref('success')
const form = reactive({ ...DEFAULTS })

const statusLabel = computed(() => {
    if (!aiStatus.value?.configured) return 'Not configured'
    return aiStatus.value.source === 'environment'
        ? 'Using environment key'
        : 'Configured'
})

const settingsChanged = computed(() => {
    const s = aiStatus.value?.settings
    if (!s) return true
    return (
        s.model !== form.model.trim() ||
        s.timeoutSecs !== form.timeoutSecs ||
        s.maxTokens !== form.maxTokens
    )
})

watch(
    () => aiStatus.value?.settings,
    (s) => {
        if (s) Object.assign(form, s)
    },
    { immediate: true },
)

const notify = (text, type = 'success') => {
    message.value = text
    messageType.value = type
}

const run = async (name, fn) => {
    busy.value = name
    message.value = ''
    try {
        await fn()
    } catch (err) {
        notify(errorMessage(err), 'error')
    } finally {
        busy.value = ''
    }
}

const openConsole = async () => {
    try {
        await openUrl('https://console.x.ai')
    } catch (err) {
        notify(errorMessage(err, 'Could not open the browser'), 'error')
    }
}

const saveKey = () =>
    run('save', async () => {
        if (!keyInput.value.trim()) return
        setAiStatus(await saveAiKey(keyInput.value))
        keyInput.value = ''
        showKey.value = false
        notify('API key saved to Windows Credential Manager')
    })

const testConnection = () =>
    run('test', async () => {
        notify(await testAiConnection(keyInput.value.trim(), form.model.trim()))
    })

const clearKey = () =>
    run('clear', async () => {
        const status = await clearAiKey()
        setAiStatus(status)
        notify(
            status.configured
                ? 'Saved key removed, the environment key is still used'
                : 'API key removed',
        )
    })

const saveSettings = () =>
    run('settings', async () => {
        const saved = await saveAiSettings({
            model: form.model.trim(),
            timeoutSecs: Number(form.timeoutSecs),
            maxTokens: Number(form.maxTokens),
        })
        if (aiStatus.value)
            aiStatus.value = { ...aiStatus.value, settings: saved }
        notify('Settings saved')
    })

const resetDefaults = () => {
    Object.assign(form, DEFAULTS)
}

onMounted(() => {
    loadAiStatus()
})
</script>
