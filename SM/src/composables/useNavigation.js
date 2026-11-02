import { ref } from 'vue'

// Shared active tab so nested components (e.g. the service details modal)
// can send the user to another tab
const activeTab = ref('filters')

export function useNavigation() {
    const goTo = (tabId) => {
        activeTab.value = tabId
    }
    return { activeTab, goTo }
}
