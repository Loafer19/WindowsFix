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
