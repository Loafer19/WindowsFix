import { invoke } from '@tauri-apps/api/core'

async function apiRequest(endpoint, options = {}) {
  try {
    // Map HTTP endpoints to Tauri commands
    const commandMap = {
      '/services': 'get_services',
      '/services/refresh': 'refresh_services',
      '/services/reload': 'reload_service_info',
      '/services/disable': 'disable_service'
    }

    const command = commandMap[endpoint]
    if (!command) {
      throw new Error(`Unknown endpoint: ${endpoint}`)
    }

    // Handle different parameter formats
    let params = {}
    if (options.body) {
      params = JSON.parse(options.body)
    }

    // Handle query parameters for reload and disable
    if (endpoint.includes('?')) {
      const url = new URL(endpoint, 'http://localhost')
      if (endpoint.includes('/services/reload')) {
        params.service_name = url.searchParams.get('name')
      } else if (endpoint.includes('/services/disable')) {
        params.service_name = url.searchParams.get('name')
      }
    }

    const result = await invoke(command, params)

    // Transform response format to match HTTP API
    if (command === 'get_services') {
      return result.services
    } else if (command === 'refresh_services') {
      return result
    } else if (command === 'reload_service_info') {
      return result
    } else if (command === 'disable_service') {
      return result.service
    }

    return result
  } catch (error) {
    console.error(`API request failed for ${endpoint}:`, error)
    throw error
  }
}

async function loadServices() {
  return await apiRequest('/services')
}

async function refreshServices() {
  return await apiRequest('/services/refresh')
}

async function reloadServiceInfo(serviceName) {
  try {
    return await apiRequest(`/services/reload?name=${encodeURIComponent(serviceName)}`)
  } catch (error) {
    return {
      error: true,
      message: 'Failed to reload service information'
    }
  }
}

async function disableService(serviceName) {
  try {
    return await apiRequest(`/services/disable?name=${encodeURIComponent(serviceName)}`)
  } catch (error) {
    return {
      error: true,
      message: 'Failed to disable service'
    }
  }
}

export { loadServices, refreshServices, reloadServiceInfo, disableService }
