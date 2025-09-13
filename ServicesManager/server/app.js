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

      if (url.pathname === '/services/reload') {
        const serviceName = url.searchParams.get('name')
        if (!serviceName) {
          return createErrorResponse('Service name parameter is required', 400)
        }
        return await handleServiceReloadRequest(decodeURIComponent(serviceName))
      }

      if (url.pathname === '/services/disable') {
        const serviceName = url.searchParams.get('name')
        if (!serviceName) {
          return createErrorResponse('Service name parameter is required', 400)
        }
        return await handleServiceDisableRequest(decodeURIComponent(serviceName))
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
