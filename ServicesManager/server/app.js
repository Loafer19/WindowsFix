const { exec } = require('child_process')
const cheerio = require('cheerio')
const cors = require('cors')
const express = require('express')
const fs = require('fs')

let services = []
let servicesInfo = JSON.parse(fs.readFileSync('public/services-info.json'))

express()
  .use(cors({ origin: 'http://localhost:5173' }))
  .get('/services', async (req, res) => {
    res.json(services)
  })
  .get('/services/:serviceName/reload', async (req, res) => {
    const serviceName = req.params.serviceName

    const additionalInfo = await fetchServiceInfo(serviceName)

    res.json(additionalInfo)
  })
  .listen(3000)

fetchRawServices()
  .then(data => {
    data.forEach(item => {
      services.push({
        name: item.Name,
        displayName: item.DisplayName || 'Unknown',
        state: item.State,
        startMode: item.StartMode,
        info: servicesInfo[item.Name] || {
          error: true,
          message: 'Not loaded'
        }
      })
    })
  })

async function fetchRawServices() {
  return new Promise((resolve) => {
    exec('powershell -Command "Get-CimInstance -ClassName Win32_Service | Select-Object Name,DisplayName,State,StartMode | ConvertTo-Json"', (err, stdout) => {
      const servicesRaw = JSON.parse(stdout)

      resolve(servicesRaw)
    })
  })
}

async function updateServiceInJson(serviceName, additionalInfo) {
  const index = services.findIndex(service => service.name == serviceName)
  services[index].info = additionalInfo

  servicesInfo[serviceName] = additionalInfo

  await fs.promises.writeFile('public/services-info.json', JSON.stringify(servicesInfo, 0, 2))

  return additionalInfo
}

async function fetchServiceInfo(serviceName) {
  if (servicesInfo[serviceName] && !servicesInfo[serviceName].error) {
    return servicesInfo[serviceName]
  }

  let additionalInfo = {
    error: true,
    message: 'Fetch error'
  }

  const searchUrl = `https://win10tweaker.ru/?s=${serviceName}`

  try {
    const searchResponse = await fetchWithRetry(searchUrl)
    const $search = cheerio.load(await searchResponse.text())
    const firstLink = $search('.fusion-post-grid .fusion-post-title a').attr('href')

    if (!firstLink || !firstLink.includes('/twikinarium/services/')) {
      additionalInfo.message = 'Not found'

      return await updateServiceInJson(serviceName, additionalInfo)
    }

    const itemResponse = await fetchWithRetry(firstLink)
    const $detail = cheerio.load(await itemResponse.text())

    const description = $detail('p:contains("Описание по умолчанию")').next('p').text().trim()
    const explained = $detail('p:contains("Нормальное описание")').next('p').text().trim()
    const recommendation = $detail('p:contains("Рекомендации")').nextAll().text().trim()

    additionalInfo = {
      url: firstLink,
      description: description || 'Not found',
      explained: explained || 'Not found',
      recommendation: recommendation || 'Not found'
    }
  } catch (error) {
    console.error(error);
  }

  return await updateServiceInJson(serviceName, additionalInfo)
}

async function fetchWithRetry(url, retries = 3) {
  try {
    return await fetch(url)
  } catch (error) {
    if (retries > 0) {
      await new Promise(resolve => setTimeout(resolve, 500));
      return await fetchWithRetry(url, retries - 1)
    }
    throw error
  }
}
