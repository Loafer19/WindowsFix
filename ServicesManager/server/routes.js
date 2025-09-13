import { getServices, fetchServiceInfo, disableService, refreshServicesCache } from './services.js'
import { validateServiceName, createSuccessResponse, createErrorResponse, Logger } from './utils.js'
import fs from 'fs'
import { CONFIG } from './config.js'

export async function handleServicesRequest() {
  try {
    const services = await getServices()
    Logger.info(`Returning ${services.length} services in API response`)
    Logger.info('First service sample:', services[0])
    return createSuccessResponse(services)
  } catch (error) {
    Logger.error('Failed to handle services request', error)
    return createErrorResponse('Failed to retrieve services', 500)
  }
}

export async function handleServiceReloadRequest(serviceName) {
  if (!validateServiceName(serviceName)) {
    return createErrorResponse('Invalid service name', 400)
  }

  try {
    const additionalInfo = await fetchServiceInfo(serviceName)
    return createSuccessResponse(additionalInfo)
  } catch (error) {
    Logger.error(`Failed to reload service info for ${serviceName}`, error)
    return createErrorResponse('Failed to reload service information', 500)
  }
}

export async function handleServiceDisableRequest(serviceName) {
  if (!validateServiceName(serviceName)) {
    return createErrorResponse('Invalid service name', 400)
  }

  try {
    // Log the disable action
    const logEntry = `${new Date().toISOString()}: ${serviceName}\n`
    await fs.promises.writeFile(CONFIG.LOG_FILE, logEntry, { flag: 'a' })

    const service = await disableService(serviceName)
    Logger.info(`Service ${serviceName} disabled successfully`)
    return createSuccessResponse(service)
  } catch (error) {
    Logger.error(`Failed to disable service ${serviceName}`, error)
    return createErrorResponse('Failed to disable service', 500)
  }
}

export async function handleServicesRefreshRequest() {
  try {
    await refreshServicesCache()
    const services = await getServices()
    Logger.info('Services cache refreshed successfully')
    return createSuccessResponse({
      message: 'Services cache refreshed',
      count: services.length,
      timestamp: new Date().toISOString()
    })
  } catch (error) {
    Logger.error('Failed to refresh services cache', error)
    return createErrorResponse('Failed to refresh services cache', 500)
  }
}
