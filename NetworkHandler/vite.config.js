import tailwindcss from '@tailwindcss/vite'
import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'

const host = process.env.TAURI_DEV_HOST

export default defineConfig(async () => ({
    plugins: [vue(), tailwindcss()],

    clearScreen: false,
    server: {
        port: 1421,
        strictPort: true,
        host: host || false,
        hmr: host
            ? {
                  protocol: 'ws',
                  host,
                  port: 1422,
              }
            : undefined,
        watch: {
            ignored: ['**/src-tauri/**'],
        },
    },
}))
