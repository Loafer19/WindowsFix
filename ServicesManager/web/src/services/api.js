async function loadServices() {
  const response = await fetch('http://localhost:3000/services')

  return await response.json()
}

async function reloadServiceInfo(serviceName) {
  try {
    const response = await fetch(`http://localhost:3000/services/${serviceName}/reload`)

    return await response.json()
  } catch (error) {
    return {
      error: true,
      message: 'Reload failed'
    }
  }
}

async function disableService(serviceName) {
  try {
    const response = await fetch(`http://localhost:3000/services/${serviceName}/disable`)

    return await response.json()
  } catch (error) {
    return {
      error: true,
      message: 'Disabling failed'
    }
  }
}

export { loadServices, reloadServiceInfo, disableService }
