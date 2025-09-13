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

  const searchUrl = `https://win10tweaker.ru/?s=${serviceName}`

  try {
    const searchResponse = await fetchWithRetry(searchUrl)
    const $search = cheerio.load(await searchResponse.text())

    const targetPost = $search('.fusion-post-grid').filter((i, element) => {
      return $search(element).text().includes('Имя службы: ' + serviceName)
    }).first()

    const targetLink = targetPost.find('.fusion-post-title a').attr('href')

    if (!targetLink || !targetLink.includes('/twikinarium/services/')) {
      additionalInfo.message = 'Not found'
      return await updateServiceInJson(serviceName, additionalInfo)
    }

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
      recommendation: recommendation || 'Not found'
    }

    Logger.info(`Fetched service info for ${serviceName}`)
  } catch (error) {
    Logger.error(`Failed to fetch service info for ${serviceName}`, error)
  }

  return await updateServiceInJson(serviceName, additionalInfo)
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
