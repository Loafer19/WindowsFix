import { CONFIG } from './config.js'

// Input validation function
export function validateServiceName(serviceName) {
  if (!serviceName || typeof serviceName !== 'string') {
    return false
  }

  // Allow alphanumeric characters, spaces, underscores, hyphens, and common punctuation
  // Windows service names can contain spaces and various characters
  const validPattern = /^[a-zA-Z0-9\s_\-().&]+$/

  // Additional checks for security
  const trimmed = serviceName.trim()
  if (trimmed.length === 0 || trimmed.length > 256) {
    return false
  }

  // Prevent potential path traversal or command injection
  if (trimmed.includes('..') || trimmed.includes('|') || trimmed.includes('&') ||
      trimmed.includes(';') || trimmed.includes('`')) {
    return false
  }

  return validPattern.test(trimmed)
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
