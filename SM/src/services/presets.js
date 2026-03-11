export const presets = [
    {
        id: 'privacy-first',
        name: 'Privacy First',
        icon: 'eye',
        description:
            'Aggressively disables Microsoft telemetry, diagnostics, data collection, location tracking, and related cloud-sync services for maximum privacy.',
        color: 'info',
        services: [
            {
                name: 'DiagTrack',
                displayName: 'Connected User Experiences and Telemetry',
                reason: 'Core service sending diagnostic and usage data to Microsoft.',
            },
            {
                name: 'dmwappushservice',
                displayName: 'Device Management Wireless Application Protocol',
                reason: 'Routes telemetry and WAP push messages to Microsoft servers.',
            },
            {
                name: 'WerSvc',
                displayName: 'Windows Error Reporting Service',
                reason: 'Automatically uploads crash reports and error data.',
            },
            {
                name: 'PcaSvc',
                displayName: 'Program Compatibility Assistant Service',
                reason: 'Collects application compatibility telemetry.',
            },
            {
                name: 'DPS',
                displayName: 'Diagnostic Policy Service',
                reason: 'Enables diagnostics and problem reporting.',
            },
            {
                name: 'WdiServiceHost',
                displayName: 'Diagnostic Service Host',
                reason: 'Hosts diagnostic tasks that can report data.',
            },
            {
                name: 'WdiSystemHost',
                displayName: 'Diagnostic System Host',
                reason: 'System-level diagnostics with potential telemetry.',
            },
            {
                name: 'diagnosticshub.standardcollector.service',
                displayName: 'Microsoft Diagnostics Hub Standard Collector',
                reason: 'Collects detailed performance and diagnostic data.',
            },
            {
                name: 'lfsvc',
                displayName: 'Geolocation Service',
                reason: 'Tracks and reports device location data.',
            },
            {
                name: 'RetailDemo',
                displayName: 'Retail Demo Service',
                reason: 'Used in retail environments for demo data collection.',
            },
            {
                name: 'wisvc',
                displayName: 'Windows Insider Service',
                reason: 'Handles Insider builds and sends feedback/telemetry.',
            },
            {
                name: 'WpnService',
                displayName: 'Windows Push Notifications System Service',
                reason: 'Push notification platform often tied to telemetry and ads.',
            },
            {
                name: 'CDPSvc',
                displayName: 'Connected Devices Platform Service',
                reason: 'Enables device syncing and cross-device experiences with data sharing.',
            },
            {
                name: 'AJRouter',
                displayName: 'AllJoyn Router Service',
                reason: 'IoT device discovery protocol (unnecessary for most users).',
            },
        ],
    },
    {
        id: 'gaming-optimized',
        name: 'Gaming Optimized',
        icon: 'flashlight',
        description:
            'Reduces input lag, stuttering, disk I/O spikes, and background network activity for smoother gameplay and lower latency.',
        color: 'primary',
        services: [
            {
                name: 'DiagTrack',
                displayName: 'Connected User Experiences and Telemetry',
                reason: 'Background telemetry causes CPU/network spikes during play.',
            },
            {
                name: 'SysMain',
                displayName: 'SysMain (Superfetch)',
                reason: 'Aggressive preloading leads to disk thrashing and stutters (especially on HDDs).',
            },
            {
                name: 'WSearch',
                displayName: 'Windows Search',
                reason: 'Constant indexing creates high disk/CPU usage.',
            },
            {
                name: 'TrkWks',
                displayName: 'Distributed Link Tracking Client',
                reason: 'Constantly polls drives for NTFS file movement, causing micro-stutters.',
            },
            {
                name: 'WMPNetworkSvc',
                displayName: 'Windows Media Player Network Sharing Service',
                reason: 'Unnecessary network scanning in the background.',
            },
            {
                name: 'XblAuthManager',
                displayName: 'Xbox Live Auth Manager',
                reason: 'Xbox authentication background process (disable if not using Xbox).',
            },
            {
                name: 'XblGameSave',
                displayName: 'Xbox Live Game Save',
                reason: 'Xbox cloud save syncing.',
            },
            {
                name: 'XboxNetApiSvc',
                displayName: 'Xbox Live Networking Service',
                reason: 'Xbox network service.',
            },
            {
                name: 'Fax',
                displayName: 'Fax',
                reason: 'Legacy service consuming resources, rarely needed.',
            },
            {
                name: 'Spooler',
                displayName: 'Print Spooler',
                reason: 'Print queue management (safe to disable if you have no printers).',
            },
            {
                name: 'bthserv',
                displayName: 'Bluetooth Support Service',
                reason: 'Bluetooth stack overhead (safe if you have no Bluetooth devices).',
            },
            {
                name: 'ssdpsrv',
                displayName: 'SSDP Discovery',
                reason: 'Network device discovery creates background traffic and jitter.',
            },
        ],
    },
    {
        id: 'performance',
        name: 'Performance Boost',
        icon: 'settings',
        description:
            'Frees up CPU, RAM, and disk resources by disabling non-essential background services for snappier everyday use.',
        color: 'error',
        services: [
            {
                name: 'SysMain',
                displayName: 'SysMain (Superfetch)',
                reason: 'Pre-fetching and caching can consume significant RAM and disk I/O.',
            },
            {
                name: 'WSearch',
                displayName: 'Windows Search',
                reason: 'Background indexing is heavy on disk and CPU.',
            },
            {
                name: 'DiagTrack',
                displayName: 'Connected User Experiences and Telemetry',
                reason: 'Constant telemetry processing uses CPU/memory/network.',
            },
            {
                name: 'WMPNetworkSvc',
                displayName: 'Windows Media Player Network Sharing Service',
                reason: 'Unneeded media discovery.',
            },
            {
                name: 'Fax',
                displayName: 'Fax',
                reason: 'Completely unnecessary on most systems.',
            },
            {
                name: 'TabletInputService',
                displayName: 'Touch Keyboard and Handwriting Panel Service',
                reason: 'Only needed on touch/pen devices.',
            },
            {
                name: 'WerSvc',
                displayName: 'Windows Error Reporting Service',
                reason: 'Background crash dump processing uses disk I/O.',
            },
            {
                name: 'RetailDemo',
                displayName: 'Retail Demo Service',
                reason: 'Unnecessary on personal devices.',
            },
            {
                name: 'MapsBroker',
                displayName: 'Downloaded Maps Manager',
                reason: 'Background map updates and location checks.',
            },
            {
                name: 'TrkWks',
                displayName: 'Distributed Link Tracking Client',
                reason: 'Unnecessary file tracking I/O.',
            },
            {
                name: 'Spooler',
                displayName: 'Print Spooler',
                reason: 'Printing service (safe to disable if no printers).',
            },
            {
                name: 'AJRouter',
                displayName: 'AllJoyn Router Service',
                reason: 'IoT device communication protocol (rarely used).',
            },
            {
                name: 'AppVClient',
                displayName: 'Microsoft App-V Client',
                reason: 'Application virtualization client (rarely used).',
            },
        ],
    },
    {
        id: 'battery-saver',
        name: 'Battery Saver',
        icon: 'batteryLow',
        description:
            'Stops services that prevent the CPU, disk, and network from idling, significantly extending laptop battery life.',
        color: 'success',
        services: [
            {
                name: 'WSearch',
                displayName: 'Windows Search',
                reason: 'Frequent indexing prevents disk and CPU from sleeping.',
            },
            {
                name: 'DiagTrack',
                displayName: 'Connected User Experiences and Telemetry',
                reason: 'Constant wake-ups to send diagnostic packets.',
            },
            {
                name: 'SysMain',
                displayName: 'SysMain (Superfetch)',
                reason: 'Aggressive memory/disk operations drain battery.',
            },
            {
                name: 'DPS',
                displayName: 'Diagnostic Policy Service',
                reason: 'Background diagnostics interfere with deep sleep states.',
            },
            {
                name: 'WMPNetworkSvc',
                displayName: 'Windows Media Player Network Sharing Service',
                reason: 'Keeps network hardware active.',
            },
            {
                name: 'lfsvc',
                displayName: 'Geolocation Service',
                reason: 'Constant hardware polling for location.',
            },
            {
                name: 'WpnService',
                displayName: 'Windows Push Notifications System Service',
                reason: 'Frequent network checks and wake-ups.',
            },
            {
                name: 'CDPSvc',
                displayName: 'Connected Devices Platform Service',
                reason: 'Device discovery and syncing activity.',
            },
            {
                name: 'iphlpsvc',
                displayName: 'IP Helper',
                reason: 'IPv6 tunneling can cause periodic network activity.',
            },
        ],
    },
    {
        id: 'network-bandwidth',
        name: 'Network & Bandwidth',
        icon: 'router',
        description:
            'Disables background data uploads/downloads, network discovery, UPnP, and sharing services. Ideal for metered connections, slow networks, or security-focused setups.',
        color: 'warning',
        services: [
            {
                name: 'DoSvc',
                displayName: 'Delivery Optimization',
                reason: 'Uploads Windows update files to other PCs on the internet.',
            },
            {
                name: 'MapsBroker',
                displayName: 'Downloaded Maps Manager',
                reason: 'Automatic offline map downloads.',
            },
            {
                name: 'WerSvc',
                displayName: 'Windows Error Reporting Service',
                reason: 'Uploads large crash dumps.',
            },
            {
                name: 'XblGameSave',
                displayName: 'Xbox Live Game Save',
                reason: 'Cloud game save syncing.',
            },
            {
                name: 'WpnService',
                displayName: 'Windows Push Notifications System Service',
                reason: 'Pulls data and notifications from the cloud.',
            },
            {
                name: 'CDPSvc',
                displayName: 'Connected Devices Platform Service',
                reason: 'Cross-device syncing and discovery traffic.',
            },
            {
                name: 'ssdpsrv',
                displayName: 'SSDP Discovery',
                reason: 'Advertises and discovers devices on the local network.',
            },
            {
                name: 'upnphost',
                displayName: 'UPnP Device Host',
                reason: 'Allows automatic device configuration and port forwarding.',
            },
            {
                name: 'FDResPub',
                displayName: 'Function Discovery Resource Publication',
                reason: 'Publishes your PC for network discovery.',
            },
            {
                name: 'fdPHost',
                displayName: 'Function Discovery Provider Host',
                reason: 'Supports network device discovery providers.',
            },
            {
                name: 'lltdsvc',
                displayName: 'Link-Layer Topology Discovery Mapper',
                reason: 'Creates network topology maps.',
            },
            {
                name: 'iphlpsvc',
                displayName: 'IP Helper',
                reason: 'IPv6 transition technologies (Teredo, 6to4).',
            },
            {
                name: 'bthserv',
                displayName: 'Bluetooth Support Service',
                reason: 'Full Bluetooth stack (safe if unused).',
            },
        ],
    },
];
