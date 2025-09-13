import { exec } from 'child_process'
import * as cheerio from 'cheerio'
import fs from 'fs'
import { CONFIG } from './config.js'
import { Logger } from './utils.js'

let services = []
let servicesInfo = {}
let servicesCache = {
  data: [],
  lastUpdated: 0,
  ttl: 5 * 60 * 1000, // 5 minutes
}

export async function getServices() {
  // Check if cache is valid
  if (servicesCache.data.length > 0 && Date.now() - servicesCache.lastUpdated < servicesCache.ttl) {
    return servicesCache.data
  }

  // Cache expired or empty, refresh data
  await refreshServicesCache()
  return servicesCache.data
}

export function getServicesInfo() {
  return servicesInfo
}

export async function loadServicesData() {
  try {
    const data = await fs.promises.readFile(CONFIG.SERVICES_DATA_FILE, 'utf8')
    servicesInfo = JSON.parse(data)
    Logger.info('Services data loaded successfully')
  } catch (error) {
    Logger.error('Failed to load services data', error)
    servicesInfo = {}
  }
}

export async function fetchRawServices() {
  return new Promise((resolve, reject) => {
    const command = 'powershell -Command "Get-CimInstance -ClassName Win32_Service | Select-Object Name,DisplayName,State,StartMode | ConvertTo-Json"'
    exec(command, (err, stdout) => {
      if (err) {
        Logger.error('Failed to fetch raw services', err)
        reject(err)
        return
      }

      try {
        const servicesRaw = JSON.parse(stdout)
        resolve(servicesRaw)
      } catch (parseError) {
        Logger.error('Failed to parse services JSON', parseError)
        reject(parseError)
      }
    })
  })
}

export async function initializeServices() {
  try {
    await refreshServicesCache()
    Logger.info(`Initialized ${servicesCache.data.length} services`)
    Logger.info('Sample service data:', servicesCache.data.slice(0, 2)) // Log first 2 services for debugging
  } catch (error) {
    Logger.error('Failed to initialize services', error)
    servicesCache.data = []
  }
}

export async function refreshServicesCache() {
  try {
    const data = await fetchRawServices()
    const processedServices = data.map((item) => {
      let name = item.Name.split('_')[0]
      return {
        name: name,
        displayName: item.DisplayName || 'Unknown',
        state: item.State,
        startMode: item.StartMode,
        info: servicesInfo[name] || {
          error: true,
          message: 'Not loaded'
        }
      }
    })

    servicesCache.data = processedServices
    servicesCache.lastUpdated = Date.now()
    services = processedServices // Keep backward compatibility

    Logger.info(`Refreshed services cache with ${processedServices.length} services`)
  } catch (error) {
    Logger.error('Failed to refresh services cache', error)
    throw error
  }
}

export async function disableService(serviceName) {
  return new Promise((resolve, reject) => {
    const command = `powershell -Command "Stop-Service -Name '${serviceName}' -Force; Set-Service -Name '${serviceName}' -StartupType Disabled; Get-CimInstance -ClassName Win32_Service -Filter \\"Name='${serviceName}'\\" | Select-Object Name,DisplayName,State,StartMode | ConvertTo-Json"`
    exec(command, (err, stdout) => {
      if (err) {
        Logger.error(`Failed to disable service ${serviceName}`, err)
        reject(err)
        return
      }

      try {
        const updatedService = JSON.parse(stdout || '{"error":"Something went wrong while shutting down the service :("}')
        const index = services.findIndex((service) => service.name === serviceName)

        if (updatedService.error) {
          services[index].info.error = true
          services[index].info.message = updatedService.error
        } else {
          Object.assign(services[index], {
            state: updatedService.State,
            startMode: updatedService.StartMode
          })
        }

        Logger.info(`Service ${serviceName} disabled successfully`)
        resolve(services[index])
      } catch (parseError) {
        Logger.error(`Failed to parse disable response for ${serviceName}`, parseError)
        reject(parseError)
      }
    })
  })
}

export async function updateServiceInJson(serviceName, additionalInfo) {
  const index = services.findIndex((service) => service.name === serviceName)
  if (index !== -1) {
    services[index].info = additionalInfo
  }

  servicesInfo[serviceName] = additionalInfo

  try {
    await fs.promises.writeFile(CONFIG.SERVICES_DATA_FILE, JSON.stringify(servicesInfo, null, 2))
    Logger.info(`Updated service info for ${serviceName}`)
  } catch (error) {
    Logger.error(`Failed to update service info for ${serviceName}`, error)
  }

  return additionalInfo
}

export async function fetchServiceInfo(serviceName) {
  let additionalInfo = {
    error: true,
    message: 'Fetch error'
  }

  // First try scraping the existing source
  const searchUrl = `https://win10tweaker.ru/?s=${serviceName}`

  try {
    const searchResponse = await fetchWithRetry(searchUrl)
    const $search = cheerio.load(await searchResponse.text())

    const targetPost = $search('.fusion-post-grid').filter((i, element) => {
      return $search(element).text().includes('Имя службы: ' + serviceName)
    }).first()

    const targetLink = targetPost.find('.fusion-post-title a').attr('href')

    if (targetLink && targetLink.includes('/twikinarium/services/')) {
      const itemResponse = await fetchWithRetry(targetLink)
      const $detail = cheerio.load(await itemResponse.text())

      const description = $detail('p:contains("Описание по умолчанию")').next('p').text().trim()
      const explained = $detail('p:contains("Нормальное описание")').next('p').text().trim()
      const recommendation = $detail('p:contains("Рекомендации")')
        .nextAll()
        .text()
        .trim()
        .replace('Учитывая следующее:\n', '')

      additionalInfo = {
        url: targetLink,
        description: description || 'Not found',
        explained: explained || 'Not found',
        recommendation: recommendation || 'Not found',
        source: 'scraped'
      }

      Logger.info(`Fetched service info for ${serviceName} from scraping`)
      return await updateServiceInJson(serviceName, additionalInfo)
    }
  } catch (error) {
    Logger.warn(`Scraping failed for ${serviceName}, trying AI:`, error.message)
  }

  // If scraping fails or service not found, try Grok AI
  try {
    Logger.info(`Attempting AI fetch for service: ${serviceName}`)
    const aiInfo = await fetchServiceInfoFromAI(serviceName)
    Logger.info(`AI fetch result for ${serviceName}:`, aiInfo)

    if (aiInfo) {
      additionalInfo = {
        ...aiInfo,
        source: 'ai'
      }
      Logger.info(`Final AI info object for ${serviceName}:`, additionalInfo)
      Logger.info(`Fetched service info for ${serviceName} from AI`)
      return await updateServiceInJson(serviceName, additionalInfo)
    } else {
      Logger.warn(`AI returned null for ${serviceName}`)
    }
  } catch (error) {
    Logger.error(`AI fetch failed for ${serviceName}:`, error.message)
  }

  // If both fail, provide a generic response
  additionalInfo = {
    description: `Windows service: ${serviceName}`,
    explained: `This is a Windows system service named ${serviceName}. Specific information about this service could not be retrieved from available sources.`,
    recommendation: `Unable to provide specific recommendations for this service. Please research this service carefully before making changes, as disabling system services can affect system stability.`,
    source: 'fallback'
  }
  Logger.warn(`Using fallback info for ${serviceName}`)
  return await updateServiceInJson(serviceName, additionalInfo)
}

async function fetchServiceInfoFromAI(serviceName) {
  if (!CONFIG.GROK_API_KEY) {
    Logger.warn('Grok API key not configured, skipping AI fetch')
    return null
  }

  const prompt = `What is the Windows service "${serviceName}"? Please provide:
1. A brief description of what this service does
2. A detailed explanation of its purpose and functionality
3. A recommendation on whether users should disable it and why

Format your response as JSON with keys: "description", "explained", "recommendation"`

  try {
    Logger.info(`Attempting to fetch AI info for service: ${serviceName}`)
    Logger.info(`Using API endpoint: ${CONFIG.GROK_API_URL}`)

    // Add timeout to prevent hanging requests
    const controller = new AbortController()
    const timeoutId = setTimeout(() => controller.abort(), 10000) // 10 second timeout

    const response = await fetch(CONFIG.GROK_API_URL, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${CONFIG.GROK_API_KEY}`
      },
      body: JSON.stringify({
        model: 'grok-3',
        messages: [
          {
            role: 'user',
            content: prompt
          }
        ],
        max_tokens: 1000,
        temperature: 0.7,
        stream: false
      }),
      signal: controller.signal
    })

    clearTimeout(timeoutId)

    Logger.info(`Grok API response status: ${response.status}`)

    if (!response.ok) {
      const errorText = await response.text()
      Logger.error(`Grok API error response: ${errorText}`)
      throw new Error(`Grok API error: ${response.status} - ${errorText}`)
    }

    const data = await response.json()
    Logger.info('Grok API response data:', JSON.stringify(data, null, 2))

    const aiResponse = data.choices?.[0]?.message?.content

    if (!aiResponse) {
      Logger.warn('No content in AI response')
      throw new Error('No response content from Grok API')
    }

    Logger.info(`AI response content (full): ${aiResponse}`)
    Logger.info(`AI response content length: ${aiResponse.length} characters`)

    // Try to parse JSON response
    try {
      const parsed = JSON.parse(aiResponse)
      Logger.info('Successfully parsed AI response as JSON:', parsed)

      // Ensure recommendation is properly handled
      let recommendation = parsed.recommendation || 'AI-generated recommendation not available'
      if (typeof recommendation === 'object' && recommendation.reason) {
        recommendation = recommendation.reason
      } else if (typeof recommendation === 'object') {
        recommendation = 'AI-generated recommendation not available'
      }

      return {
        description: parsed.description || 'AI-generated description not available',
        explained: parsed.explained || 'AI-generated explanation not available',
        recommendation: recommendation
      }
    } catch (parseError) {
      Logger.warn('Failed to parse AI response as JSON, using text extraction. Parse error:', parseError.message)
      Logger.info('AI response that failed JSON parsing:', aiResponse)
      const textParsed = parseAIResponseText(aiResponse)
      Logger.info('Text parsing result:', textParsed)
      return textParsed
    }
  } catch (error) {
    if (error.name === 'AbortError') {
      Logger.error(`Grok API request timed out for ${serviceName}`)
      throw new Error('AI API request timed out')
    }
    Logger.error(`Grok API request failed for ${serviceName}:`, error.message)
    throw error
  }
}

function parseAIResponseText(text) {
  Logger.info('Starting text parsing of AI response')
  Logger.info('Raw text to parse:', text)

  // Simple text parsing for AI response
  const lines = text.split('\n')
  Logger.info(`Text has ${lines.length} lines`)

  let description = ''
  let explained = ''
  let recommendation = ''

  let currentSection = ''

  for (const line of lines) {
    const lowerLine = line.toLowerCase()
    Logger.info(`Processing line: "${line}" (section: ${currentSection})`)

    if (lowerLine.includes('description') || lowerLine.includes('1.')) {
      currentSection = 'description'
      Logger.info('Switched to description section')
      continue
    } else if (lowerLine.includes('explanation') || lowerLine.includes('detailed') || lowerLine.includes('2.')) {
      currentSection = 'explained'
      Logger.info('Switched to explained section')
      continue
    } else if (lowerLine.includes('recommendation') || lowerLine.includes('3.')) {
      currentSection = 'recommendation'
      Logger.info('Switched to recommendation section')
      continue
    }

    if (currentSection === 'description' && line.trim()) {
      description += line.trim() + ' '
      Logger.info(`Added to description: "${line.trim()}"`)
    } else if (currentSection === 'explained' && line.trim()) {
      explained += line.trim() + ' '
      Logger.info(`Added to explained: "${line.trim()}"`)
    } else if (currentSection === 'recommendation' && line.trim()) {
      recommendation += line.trim() + ' '
      Logger.info(`Added to recommendation: "${line.trim()}"`)
    }
  }

  const result = {
    description: description.trim() || 'AI-generated description',
    explained: explained.trim() || 'AI-generated explanation',
    recommendation: recommendation.trim() || 'AI-generated recommendation'
  }

  Logger.info('Text parsing completed. Result:', result)
  return result
}

async function fetchWithRetry(url, retries = CONFIG.SCRAPE_RETRIES) {
  try {
    const response = await fetch(url)
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`)
    }
    return response
  } catch (error) {
    if (retries > 0) {
      Logger.warn(`Retrying fetch for ${url}, attempts left: ${retries}`)
      await new Promise((resolve) => setTimeout(resolve, CONFIG.SCRAPE_TIMEOUT))
      return await fetchWithRetry(url, retries - 1)
    }
    throw error
  }
}
