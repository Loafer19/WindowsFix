<template>
    <div class="min-h-screen bg-base-200">
        <div class="container mx-auto p-6">

            <div role="tablist" class="tabs tabs-boxed gap-2 p-0 mb-6">
                <label class="tab gap-1 text-lg font-medium hover:text-info" v-for="tab in tabs" :key="tab.id">
                    <input v-model="activeTab" type="radio" name="tabs_main" class="tab" :value="tab.component" />
                    <Icon :name="tab.icon" />
                    {{ tab.name }}
                </label>
            </div>

            <component :is="activeTab" />
        </div>
    </div>
</template>

<script setup>
import { markRaw, ref } from 'vue'

import FiltersTab from './components/Tabs/FiltersTab.vue'
import HistoryTab from './components/Tabs/HistoryTab.vue'
import PresetsTab from './components/Tabs/PresetsTab.vue'
import StartupTab from './components/Tabs/StartupTab.vue'

const tabs = ref([
    {
        id: 'filters',
        name: 'Services',
        component: markRaw(FiltersTab),
        icon: 'equalizer',
    },
    {
        id: 'startup',
        name: 'Startup Apps',
        component: markRaw(StartupTab),
        icon: 'rocket',
    },
    {
        id: 'presets',
        name: 'Presets',
        component: markRaw(PresetsTab),
        icon: 'flashlight',
    },
    {
        id: 'history',
        name: 'History',
        component: markRaw(HistoryTab),
        icon: 'history',
    },
])

const activeTab = ref(markRaw(FiltersTab))

</script>

<style scoped>
.tabs-box {
    box-shadow: none;
}

.tab {
    border: var(--border) solid var(--color-base-300) !important;
    border-radius: var(--radius-field) !important;
    box-shadow: none;
    transition: all 0.2s ease;
}

.tab:hover {
    border-color: var(--color-primary) !important;
    background-color: var(--color-primary) !important;
    color: var(--color-primary-content) !important;
}

.tab:has(input:checked) {
    border-color: var(--color-primary) !important;
    background-color: var(--color-primary) !important;
    color: var(--color-primary-content) !important;
}
</style>
