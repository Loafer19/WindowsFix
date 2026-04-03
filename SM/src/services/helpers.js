export const getStatusColor = (status) => {
    const colors = {
        Running: 'error',
        Stopped: 'neutral',
        Paused: 'warning',
        Pending: 'warning',
    }
    return colors[status] || 'neutral'
}

export const getStartupTypeColor = (type) => {
    const colors = {
        Automatic: 'error',
        Manual: 'warning',
        Disabled: 'neutral',
        System: 'info',
        Boot: 'info',
    }
    return colors[type] || 'neutral'
}

export const formatTime = (timestamp) => {
    const date = new Date(timestamp)
    const now = new Date()
    const diff = now - date

    if (diff < 60000) return 'just now'
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`
    return date.toLocaleDateString()
}

export const getActionColor = (action) => {
    const map = {
        'Started': 'success',
        'Stopped': 'warning',
        'Restarted': 'info',
        'Disabled': 'error',
        'Added': 'success',
        'Removed': 'error',
        'enable': 'warning',
        'disable': 'success',
    }
    return map[action] || 'neutral'
}

export const getLocationColor = (location) => {
    const map = {
        'hkeyLocalMachine': 'primary',
        'hkeyCurrentUser': 'info',
        'startupFolder': 'warning',
    }
    return map[location] || 'neutral'
}

export const formatLocation = (location) => {
    const map = {
        'hkeyLocalMachine': 'HKLM Registry',
        'hkeyCurrentUser': 'HKCU Registry',
        'startupFolder': 'Startup Folder',
    }
    return map[location] || location
}
