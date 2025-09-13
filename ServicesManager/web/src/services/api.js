const API_BASE = 'http://localhost:3000'

async function apiRequest(endpoint, options = {}) {
  try {
    const response = await fetch(`${API_BASE}${endpoint}`, {
      headers: {
        'Content-Type': 'application/json',
        ...options.headers
      },
      ...options
    })

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`)
    }

    return await response.json()
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
