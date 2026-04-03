use super::models::ServiceInfo;
use std::collections::HashMap;

pub fn get_default_services() -> HashMap<String, ServiceInfo> {
    let mut map = HashMap::new();

    map.insert("W32Time".to_string(), ServiceInfo {
        description: Some("Windows Time Service".to_string()),
        explained: Some("Synchronizes system time with external time servers. Essential for Kerberos authentication and event logging.".to_string()),
        recommendation: Some("Keep enabled for accurate timestamps and network authentication. Can be set to Manual if not using domain.".to_string()),
    });

    map.insert("Spooler".to_string(), ServiceInfo {
        description: Some("Print Spooler".to_string()),
        explained: Some("Manages print jobs and printer communication. Required for printing to local or network printers.".to_string()),
        recommendation: Some("Disable if no printers are used. Re-enable when needed for printing.".to_string()),
    });

    map.insert("Dhcp".to_string(), ServiceInfo {
        description: Some("DHCP Client".to_string()),
        explained: Some("Registers and renews IP addresses for network adapters. Essential for dynamic IP assignment on networks.".to_string()),
        recommendation: Some("Keep enabled for automatic IP configuration. Required for most home/corporate networks.".to_string()),
    });

    map.insert("Dnscache".to_string(), ServiceInfo {
        description: Some("DNS Client".to_string()),
        explained: Some("Caches DNS resolutions for faster network access. Resolves domain names to IP addresses.".to_string()),
        recommendation: Some("Keep enabled. Critical for network connectivity and name resolution.".to_string()),
    });

    map.insert("EventLog".to_string(), ServiceInfo {
        description: Some("Windows Event Log".to_string()),
        explained: Some("Manages event logging and subscriptions. Records system, security, and application events.".to_string()),
        recommendation: Some("Keep enabled. Essential for troubleshooting and security auditing.".to_string()),
    });

    map.insert("PlugPlay".to_string(), ServiceInfo {
        description: Some("Plug and Play".to_string()),
        explained: Some("Enables dynamic hardware detection and configuration. Allows system to recognize new devices.".to_string()),
        recommendation: Some("Keep enabled. Required for hardware detection and hot-plugging.".to_string()),
    });

    map.insert("ProfSvc".to_string(), ServiceInfo {
        description: Some("User Profile Service".to_string()),
        explained: Some("Loads and unloads user profiles. Manages user-specific settings and desktop configuration.".to_string()),
        recommendation: Some("Keep enabled. Required for user login and profile management.".to_string()),
    });

    map.insert("SamSs".to_string(), ServiceInfo {
        description: Some("Security Accounts Manager".to_string()),
        explained: Some("Manages user account information and security policies. Stores password hashes and account data.".to_string()),
        recommendation: Some("Keep enabled. Critical for user authentication and security.".to_string()),
    });

    map.insert("LanmanServer".to_string(), ServiceInfo {
        description: Some("Server".to_string()),
        explained: Some("Provides RPC support, named pipe sharing, and file/printer sharing. Enables network file and printer access.".to_string()),
        recommendation: Some("Disable on standalone machines not sharing files/printers. Keep on network servers.".to_string()),
    });

    map.insert("LanmanWorkstation".to_string(), ServiceInfo {
        description: Some("Workstation".to_string()),
        explained: Some("Provides network connection and file sharing capabilities. Enables accessing network resources.".to_string()),
        recommendation: Some("Keep enabled for network connectivity. Required for accessing shared files and printers.".to_string()),
    });

    map.insert("Schedule".to_string(), ServiceInfo {
        description: Some("Task Scheduler".to_string()),
        explained: Some("Manages scheduled tasks and background jobs. Enables running programs at specific times.".to_string()),
        recommendation: Some("Can be disabled if not using scheduled tasks. Some Windows features may depend on it.".to_string()),
    });

    map.insert("BITS".to_string(), ServiceInfo {
        description: Some("Background Intelligent Transfer Service".to_string()),
        explained: Some("Transfers files in the background using idle bandwidth. Used by Windows Update and other installers.".to_string()),
        recommendation: Some("Keep enabled for Windows Updates. Can be set to Manual if not using automatic updates.".to_string()),
    });

    map.insert("wuauserv".to_string(), ServiceInfo {
        description: Some("Windows Update".to_string()),
        explained: Some("Downloads and installs Windows updates and patches. Keeps system secure and up-to-date.".to_string()),
        recommendation: Some("Keep enabled for security updates. Can be disabled if using other update methods.".to_string()),
    });

    map.insert("WinDefend".to_string(), ServiceInfo {
        description: Some("Windows Defender Antivirus".to_string()),
        explained: Some("Provides real-time malware protection and antivirus scanning. Built-in Windows security.".to_string()),
        recommendation: Some("Keep enabled or use alternative antivirus. Critical for system security.".to_string()),
    });

    map.insert("wscsvc".to_string(), ServiceInfo {
        description: Some("Security Center".to_string()),
        explained: Some("Monitors security settings and alerts about security issues. Reports on antivirus/firewall status.".to_string()),
        recommendation: Some("Keep enabled for security notifications. May be replaced by Windows Security in newer versions.".to_string()),
    });

    map
}
