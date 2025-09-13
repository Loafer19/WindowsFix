export const CONFIG = {
  PORT: 3000,
  ALLOWED_ORIGIN: 'http://localhost:5173',
  LOG_FILE: 'logs/services-disabling.log',
  SERVICES_DATA_FILE: 'public/services-info.json',
  SCRAPE_RETRIES: 3,
  SCRAPE_TIMEOUT: 500,
  GROK_API_KEY: process.env.GROK_API_KEY || '',
  GROK_API_URL: 'https://api.x.ai/v1/chat/completions',
}
