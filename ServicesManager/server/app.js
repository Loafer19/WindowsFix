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
    Logger.info(`Incoming request: ${req.method} ${url.pathname}`)

    // Handle preflight requests
    if (req.method === 'OPTIONS') {
      Logger.info('Handling OPTIONS request')
      return new Response(null, { headers: createSecureHeaders() })
    }

    // Only allow GET requests
    if (req.method !== 'GET') {
      Logger.info(`Rejecting ${req.method} request - only GET allowed`)
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
        Logger.info(`Handling reload request for service: ${serviceName}`)
        try {
          const response = await handleServiceReloadRequest(decodeURIComponent(serviceName))
          Logger.info(`Reload response status: ${response.status}, headers:`, Object.fromEntries(response.headers.entries()))

          // Clone the response to read the body for logging
          const clonedResponse = response.clone()
          const bodyText = await clonedResponse.text()
          Logger.info(`Reload response body length: ${bodyText.length} characters`)

          Logger.info(`About to return response for ${serviceName}`)
          return response
        } catch (error) {
          Logger.error(`Error handling reload request for ${serviceName}:`, error)
          return createErrorResponse('Internal server error during reload', 500)
        }
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
