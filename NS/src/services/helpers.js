export const getBlockStatusInfo = (proc) => ({
    btnClass: proc.blocked ? 'btn btn-success btn-sm btn-square' : 'btn btn-warning btn-sm btn-square',
    btnTip: proc.blocked ? 'Unblock' : 'Block traffic',
    icon: proc.blocked ? 'errorWarning' : 'forbid',
    rowClass: proc.blocked ? 'opacity-50' : '',
    nameClass: proc.blocked ? 'line-through text-error' : '',
})

export const calcSpeed = (bps) => {
    if (bps >= 1_048_576) return `${(bps / 1_048_576).toFixed(1)} MB/s`
    if (bps >= 1024) return `${(bps / 1024).toFixed(1)} KB/s`
    return `${bps} B/s`
}

export const calcBytes = (bytes) => {
    if (bytes >= 1_073_741_824) return `${(bytes / 1_073_741_824).toFixed(2)} GB`
    if (bytes >= 1_048_576) return `${(bytes / 1_048_576).toFixed(1)} MB`
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(1)} KB`
    return `${bytes} B`
}

export const sortIcon = (currentField, sortField, sortDir) => {
    if (currentField !== sortField) return 'arrowUpDown'
    return sortDir === 'asc' ? 'sortAsc' : 'sortDesc'
}
