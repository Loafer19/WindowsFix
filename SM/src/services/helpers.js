const STATUS_COLORS = Object.freeze({
    Running: 'error',
    Stopped: 'neutral',
    Paused: 'warning',
    Pending: 'warning',
})

const STARTUP_TYPE_COLORS = Object.freeze({
    Automatic: 'error',
    Manual: 'warning',
    Disabled: 'neutral',
    System: 'info',
    Boot: 'info',
})

const ACTION_COLORS = Object.freeze({
    'Started': 'success',
    'Stopped': 'warning',
    'Restarted': 'info',
    'Disabled': 'error',
    'Added': 'success',
    'Removed': 'error',
    'enable': 'warning',
    'disable': 'success',
})

const LOCATION_CONFIG = Object.freeze({
    hkeyLocalMachine: { color: 'primary', label: 'HKLM Registry' },
    hkeyCurrentUser: { color: 'info', label: 'HKCU Registry' },
    startupFolder: { color: 'warning', label: 'Startup Folder' },
})

export const STARTUP_LOCATIONS = Object.freeze([
    { value: 'hkeyLocalMachine', label: 'HKLM Registry (System)' },
    { value: 'hkeyCurrentUser', label: 'HKCU Registry (User)' },
    { value: 'startupFolder', label: 'Startup Folder' },
])

export const getStatusColor = (status) => STATUS_COLORS[status] || 'neutral'

export const getStartupTypeColor = (type) => STARTUP_TYPE_COLORS[type] || 'neutral'

export const getActionColor = (action) => ACTION_COLORS[action] || 'neutral'

export const getLocationInfo = (location) => LOCATION_CONFIG[location] || { color: 'neutral', label: location }

export const getLocationColor = (location) => getLocationInfo(location).color

export const formatLocation = (location) => getLocationInfo(location).label

export const calcPercentage = (part, total) => total > 0 ? ((part / total) * 100).toFixed(1) : 0

export const formatTime = (timestamp) => {
    const diff = Date.now() - new Date(timestamp).getTime()
    if (diff < 60000) return 'just now'
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`
    return new Date(timestamp).toLocaleDateString()
}
