const WMIClient = require('wmi-client');
const fs = require('fs');
const axios = require('axios');
const cheerio = require('cheerio');
const express = require('express');
const cors = require('cors');

const app = express();
const wmi = new WMIClient({ host: 'localhost' });

app.use(cors({
    origin: 'http://localhost:5173' // Adjust if using Vite (e.g., 'http://localhost:5173')
}));

const SERVICE_START_TYPES = {
    AUTO: 'Auto',
    MANUAL: 'Manual',
    DISABLED: 'Disabled',
    SYSTEM: 'System',
    BOOT: 'Boot'
};

const delay = ms => {
    console.log(`Delaying for ${ms}ms...`);
    return new Promise(resolve => {
        setTimeout(() => {
            console.log(`Delay of ${ms}ms complete`);
            resolve();
        }, ms);
    });
};

async function fetchAdditionalInfo(serviceName) {
    const searchUrl = `https://win10tweaker.ru/?s=${encodeURIComponent(serviceName)}`;

    async function fetchWithRetry(url, retries = 3) {
        console.log(`Fetching ${url} with ${retries} retries remaining`);
        try {
            const response = await axios.get(url);
            console.log(`Successfully fetched ${url}`);
            return response;
        } catch (error) {
            if (error.response && error.response.status === 503 && retries > 0) {
                console.log(`503 Service Unavailable for ${url}. Retrying in 1 second... (${retries} retries left)`);
                await delay(1000);
                return await fetchWithRetry(url, retries - 1);
            }
            console.error(`Fetch failed for ${url}: ${error.message}`);
            throw error;
        }
    }

    try {
        const searchResponse = await fetchWithRetry(searchUrl);
        const $search = cheerio.load(searchResponse.data);
        const firstLink = $search('.fusion-post-grid .fusion-post-title a').attr('href');
        if (!firstLink || !firstLink.includes('/twikinarium/services/')) {
            console.log(`No relevant result for ${serviceName}`);
            return {
                defaultDescription: 'Not found',
                normalDescription: 'Not found',
                recommendations: 'Not found'
            };
        }

        const detailResponse = await fetchWithRetry(firstLink);
        const $detail = cheerio.load(detailResponse.data);

        const defaultDescription = $detail('p:contains("Описание по умолчанию")').next('p').text().trim();
        const normalDescription = $detail('p:contains("Нормальное описание")').next('p').text().trim();
        const recommendations = $detail('p:contains("Рекомендации")').nextAll().text().trim();

        return {
            defaultDescription: defaultDescription || 'Not found',
            normalDescription: normalDescription || 'Not found',
            recommendations: recommendations || 'Not found'
        };
    } catch (error) {
        console.error(`Error fetching info for ${serviceName}: ${error.message}`);
        return {
            defaultDescription: 'Error fetching data',
            normalDescription: 'Error fetching data',
            recommendations: 'Error fetching data'
        };
    }
}

// Store services data in memory
let servicesData = null;

app.get('/reload/:serviceName', async (req, res) => {
    const serviceName = req.params.serviceName;
    console.log(`Received reload request for ${serviceName}`);
    const siteData = await fetchAdditionalInfo(serviceName);
    res.json(siteData);
});

// New route to return the full services JSON
app.get('/services', (req, res) => {
    if (servicesData) {
        console.log('Returning services data from memory');
        res.json(servicesData);
    } else {
        console.log('Services data not yet loaded');
        res.status(503).json({ error: 'Services data not available yet' });
    }
});

// Initial WMI query and JSON generation
(async () => {
    wmi.query('SELECT * FROM Win32_Service', async (err, result) => {
        if (err) {
            console.error(`Error: ${err}`);
            return;
        }

        const allServices = await Promise.all(result.map(async service => {
            const additionalInfo = await fetchAdditionalInfo(service.Name);
            return {
                name: service.Name,
                displayName: service.DisplayName || service.Caption || 'Unknown',
                state: service.State,
                startMode: service.StartMode,
                site: additionalInfo
            };
        }));

        const autoServices = filterServicesByStartMode(allServices, SERVICE_START_TYPES.AUTO);
        servicesData = { allServices, autoServices };

        fs.writeFile('services-enhanced.json', JSON.stringify(servicesData, null, 2), { encoding: 'utf8' }, (err) => {
            if (err) {
                console.error(`Error writing to file: ${err}`);
            } else {
                console.log('Services saved to services-enhanced.json');
                app.listen(3000, () => console.log('Server running on http://localhost:3000'));
            }
        });
    });
})();

function filterServicesByStartMode(services, startMode) {
    return services.filter(service => service.startMode.toLowerCase() === startMode.toLowerCase());
}
