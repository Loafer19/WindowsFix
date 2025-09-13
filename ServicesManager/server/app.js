import { CONFIG } from './config.js'
import { createSecureHeaders, createErrorResponse, Logger } from './utils.js'
import { loadServicesData, initializeServices } from './services.js'
import { handleServicesRequest, handleServiceReloadRequest, handleServiceDisableRequest, handleServicesRefreshRequest } from './routes.js'

// Initialize services data
await loadServicesData()
await initializeServices()

const server = Bun.serve({
  port: CONFIG.PORT,
  async fetch(req) {
    const url = new URL(req.url)

    // Handle preflight requests
    if (req.method === 'OPTIONS') {
      return new Response(null, { headers: createSecureHeaders() })
    }

    // Only allow GET requests
    if (req.method !== 'GET') {
      return createErrorResponse('Method not allowed', 405)
    }

    try {
      if (url.pathname === '/services') {
        return await handleServicesRequest()
      }

      if (url.pathname === '/services/refresh') {
        return await handleServicesRefreshRequest()
      }

      const reloadMatch = url.pathname.match(/^\/services\/([^\/]+)\/reload$/)
      if (reloadMatch) {
        const serviceName = decodeURIComponent(reloadMatch[1])
        return await handleServiceReloadRequest(serviceName)
      }

      const disableMatch = url.pathname.match(/^\/services\/([^\/]+)\/disable$/)
      if (disableMatch) {
        const serviceName = decodeURIComponent(disableMatch[1])
        return await handleServiceDisableRequest(serviceName)
      }

      // 404 for unknown routes
      return createErrorResponse('Not Found', 404)
    } catch (error) {
      Logger.error('Server error:', error)
      return createErrorResponse('Internal Server Error', 500)
    }
  },
})

Logger.info(`Server running on port ${CONFIG.PORT}`)
