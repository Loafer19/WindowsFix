// src/services.js

// Function to load services from JSON
async function loadServices() {
    try {
        const response = await fetch('http://localhost:3000/services');
        if (!response.ok) throw new Error('Failed to fetch JSON');
        const data = await response.json();
        return data.allServices.map(service => ({
            ...service,
            isReloading: false // Add flag for reload state
        }));
    } catch (error) {
        console.error('Error loading services:', error);
        throw error; // Let the caller handle the error
    }
}

// Function to reload service info from the API
async function reloadServiceInfo(serviceName) {
    try {
        const response = await fetch(`http://localhost:3000/reload/${serviceName}`);
        if (!response.ok) throw new Error('API error');
        return await response.json();
    } catch (error) {
        console.error(`Reload failed for ${serviceName}: ${error.message}`);
        return {
            defaultDescription: 'Reload failed',
            normalDescription: 'Reload failed',
            recommendations: 'Reload failed'
        };
    }
}

// Function to check if site data has an error
function hasFetchError(site) {
    return (
        site.defaultDescription === 'Error fetching data' ||
        site.normalDescription === 'Error fetching data' ||
        site.recommendations === 'Error fetching data'
    );
}

export { loadServices, reloadServiceInfo, hasFetchError };
