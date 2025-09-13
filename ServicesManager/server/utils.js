import { CONFIG } from './config.js'

// Input validation function
export function validateServiceName(serviceName) {
  if (!serviceName || typeof serviceName !== 'string') {
    return false
  }
  // Allow only alphanumeric characters, underscores, and hyphens
  // Windows service names typically follow this pattern
  const validPattern = /^[a-zA-Z0-9_-]+$/
  return validPattern.test(serviceName) && serviceName.length <= 256
}

// Security headers
export function createSecureHeaders() {
  return {
    'Access-Control-Allow-Origin': CONFIG.ALLOWED_ORIGIN,
    'Access-Control-Allow-Methods': 'GET, OPTIONS',
    'Access-Control-Allow-Headers': 'Content-Type',
    'Access-Control-Max-Age': '86400', // 24 hours
    'X-Content-Type-Options': 'nosniff',
    'X-Frame-Options': 'DENY',
    'X-XSS-Protection': '1; mode=block',
  }
}

// Error response helper
export function createErrorResponse(message, status = 400, headers = {}) {
  const secureHeaders = { ...createSecureHeaders(), ...headers }
  return new Response(JSON.stringify({ error: message }), {
    status,
    headers: { 'Content-Type': 'application/json', ...secureHeaders }
  })
}

// Success response helper
export function createSuccessResponse(data, headers = {}) {
  const secureHeaders = { ...createSecureHeaders(), ...headers }
  return new Response(JSON.stringify(data), {
    status: 200,
    headers: { 'Content-Type': 'application/json', ...secureHeaders }
  })
}

// Logger utility
export class Logger {
  static info(message, data = {}) {
    console.log(`[${new Date().toISOString()}] INFO: ${message}`, data)
  }

  static error(message, error = {}) {
    console.error(`[${new Date().toISOString()}] ERROR: ${message}`, error)
  }

  static warn(message, data = {}) {
    console.warn(`[${new Date().toISOString()}] WARN: ${message}`, data)
  }
}
