export const presets = [
    {
        id: 'no-telemetry',
        name: 'No Telemetry',
        icon: 'eye',
        description:
            'Disables Microsoft data collection and telemetry services to enhance privacy.',
        color: 'info',
        services: [
            {
                name: 'DiagTrack',
                displayName: 'Connected User Experiences and Telemetry',
                reason: 'Sends diagnostic and usage data to Microsoft.',
            },
            {
                name: 'dmwappushservice',
                displayName: 'Device Management Wireless Application Protocol',
                reason: 'Used for telemetry data routing.',
            },
            {
                name: 'WerSvc',
                displayName: 'Windows Error Reporting Service',
                reason: 'Sends crash reports and error data to Microsoft.',
            },
            {
                name: 'PcaSvc',
                displayName: 'Program Compatibility Assistant Service',
                reason: 'Collects application compatibility telemetry.',
            },
            {
                name: 'DPS',
                displayName: 'Diagnostic Policy Service',
                reason: 'Enables problem detection and diagnostics reporting.',
            },
            {
                name: 'WdiServiceHost',
                displayName: 'Diagnostic Service Host',
                reason: 'Runs diagnostic tasks that report data to Microsoft.',
            },
            {
                name: 'WdiSystemHost',
                displayName: 'Diagnostic System Host',
                reason: 'Hosts system-level diagnostics with telemetry.',
            },
            {
                name: 'diagnosticshub.standardcollector.service',
                displayName:
                    'Microsoft Diagnostics Hub Standard Collector Service',
                reason: 'Collects performance and diagnostic data.',
            },
            {
                name: 'lfsvc',
                displayName: 'Geolocation Service',
                reason: 'Tracks and reports device location.',
            },
            {
                name: 'RetailDemo',
                displayName: 'Retail Demo Service',
                reason: 'Collects data for retail demo environments.',
            },
        ],
    },
    {
        id: 'gaming',
        name: 'Gaming',
        icon: 'lightning',
        description:
            'Disables background services that can cause input lag, stuttering, or high CPU usage during gaming.',
        color: 'warning',
        services: [
            {
                name: 'DiagTrack',
                displayName: 'Connected User Experiences and Telemetry',
                reason: 'Consumes CPU and network while sending telemetry.',
            },
            {
                name: 'SysMain',
                displayName: 'SysMain (Superfetch)',
                reason: 'Causes disk thrashing and stuttering on HDDs/low-RAM systems.',
            },
            {
                name: 'WSearch',
                displayName: 'Windows Search',
                reason: 'Indexing process spikes disk I/O and CPU in the background.',
            },
            {
                name: 'WMPNetworkSvc',
                displayName: 'Windows Media Player Network Sharing Service',
                reason: 'Unnecessary network activity in the background.',
            },
            {
                name: 'XblAuthManager',
                displayName: 'Xbox Live Auth Manager',
                reason: 'Xbox authentication background process, unneeded if not using Xbox.',
            },
            {
                name: 'XblGameSave',
                displayName: 'Xbox Live Game Save',
                reason: 'Xbox cloud save syncing, unneeded if not using Xbox.',
            },
            {
                name: 'XboxNetApiSvc',
                displayName: 'Xbox Live Networking Service',
                reason: 'Xbox network service, unneeded if not using Xbox.',
            },
            {
                name: 'Fax',
                displayName: 'Fax',
                reason: 'Fax service consumes resources, rarely needed.',
            },
        ],
    },
    {
        id: 'performance',
        name: 'Performance',
        icon: 'settings',
        description:
            'Disables resource-hungry background services to free up CPU, RAM, and disk for everyday tasks.',
        color: 'success',
        services: [
            {
                name: 'SysMain',
                displayName: 'SysMain (Superfetch)',
                reason: 'Pre-fetching caches consume significant RAM.',
            },
            {
                name: 'WSearch',
                displayName: 'Windows Search',
                reason: 'Background indexing uses disk and CPU.',
            },
            {
                name: 'DiagTrack',
                displayName: 'Connected User Experiences and Telemetry',
                reason: 'Telemetry uses CPU, memory, and network.',
            },
            {
                name: 'WMPNetworkSvc',
                displayName: 'Windows Media Player Network Sharing Service',
                reason: 'Unnecessary network scanning for media.',
            },
            {
                name: 'Fax',
                displayName: 'Fax',
                reason: 'Rarely needed; wastes resources when idle.',
            },
            {
                name: 'TabletInputService',
                displayName: 'Touch Keyboard and Handwriting Panel Service',
                reason: 'Only needed on touch/pen devices.',
            },
            {
                name: 'WerSvc',
                displayName: 'Windows Error Reporting Service',
                reason: 'Sends crash dumps in background, consuming disk I/O.',
            },
            {
                name: 'RetailDemo',
                displayName: 'Retail Demo Service',
                reason: 'Retail-only service, unnecessary on personal PCs.',
            },
        ],
    },
]
