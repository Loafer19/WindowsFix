<template>
    <!-- Modal backdrop -->
    <div class="modal modal-open" @click.self="emit('close')">
        <div class="modal-box max-w-2xl w-full">
            <!-- Header -->
            <div class="flex items-start justify-between mb-2">
                <div>
                    <h3 class="font-bold text-xl">{{ proc.name }}</h3>
                </div>
                <Button class="btn btn-ghost btn-sm btn-square" @clicked="emit('close')">
                    <Icon name="close" />
                </Button>
            </div>

            <div class="flex items-center gap-3 mb-3">
                <span class="badge badge-ghost font-mono text-xs">{{ proc.pid ? `PID ${proc.pid}` : 'Not running' }}</span>
                <span class="badge badge-ghost font-mono text-xs truncate max-w-xs" :title="proc.exePath">{{ proc.exePath }}</span>
                <span v-if="proc.blocked" class="badge badge-error text-xs">Blocked</span>
            </div>

            <!-- Live stats row -->
            <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-6">
                <div class="bg-base-200 rounded-lg p-3 text-center">
                    <div class="text-xs text-base-content/60 mb-1">Download</div>
                    <div class="font-bold text-primary font-mono">{{ formatSpeed(proc.downloadBps) }}</div>
                </div>
                <div class="bg-base-200 rounded-lg p-3 text-center">
                    <div class="text-xs text-base-content/60 mb-1">Upload</div>
                    <div class="font-bold text-info font-mono">{{ formatSpeed(proc.uploadBps) }}</div>
                </div>
                <div class="bg-base-200 rounded-lg p-3 text-center">
                    <div class="text-xs text-base-content/60 mb-1">Total DL</div>
                    <div class="font-bold font-mono text-sm text-primary">{{ formatBytes(proc.totalDownloadBytes) }}</div>
                </div>
                <div class="bg-base-200 rounded-lg p-3 text-center">
                    <div class="text-xs text-base-content/60 mb-1">Total UL</div>
                    <div class="font-bold font-mono text-sm text-info">{{ formatBytes(proc.totalUploadBytes) }}</div>
                </div>
            </div>

            <!-- 24h hourly chart -->
            <div class="bg-base-200 rounded-lg p-4 mb-4">
                <div class="text-sm font-semibold mb-3 text-base-content/70">Last 24 hours - hourly usage</div>
                <div v-if="history.length === 0" class="text-center py-8 text-base-content/40 text-sm">
                    No hourly history yet. Data accumulates over time.
                </div>
                <Bar v-else :data="chartData" :options="chartOptions" />
            </div>

            <!-- Throttle / info row -->
            <div class="flex items-center gap-4 flex-wrap">
                <div v-if="proc.pid" class="flex items-center gap-2">
                    <span class="text-sm text-base-content/70">Throttle:</span>
                    <input
                        type="number"
                        class="input input-bordered input-sm w-24 font-mono"
                        min="0"
                        placeholder="no limit"
                        :value="proc.limitBps ? Math.round(proc.limitBps / 1024) : ''"
                        @change="onThrottleChange"
                        @keydown.enter="$event.target.blur()"
                    />
                    <span class="text-xs text-base-content/50">KB/s</span>
                </div>
                <span v-if="proc.limitBps" class="badge badge-warning font-mono text-xs">
                    Limited to {{ formatSpeed(proc.limitBps) }}
                </span>
            </div>
        </div>
    </div>
</template>

<script setup>
import {
    BarElement,
    CategoryScale,
    Chart as ChartJS,
    Legend,
    LinearScale,
    Title,
    Tooltip,
} from 'chart.js'
import { computed, onMounted, ref } from 'vue'
import { Bar } from 'vue-chartjs'
import { formatBytes, formatSpeed } from '../../composables/useNetwork.js'
import { getProcessHistory } from '../../services/api.js'
import Button from '../Button.vue'
import Icon from '../Icon.vue'

ChartJS.register(CategoryScale, LinearScale, BarElement, Title, Tooltip, Legend)

const props = defineProps({
    proc: { type: Object, required: true },
})

const emit = defineEmits(['close', 'throttle'])

const history = ref([])

onMounted(async () => {
    try {
        history.value = await getProcessHistory(props.proc.exePath)
    } catch {
        history.value = []
    }
})


const hourLabels = computed(() => {
    const n = history.value.length
    return Array.from({ length: n }, (_, i) => {
        const offset = n - 1 - i
        return offset === 0 ? 'now' : `-${offset}h`
    })
})

const CHART_GRID_COLOR = 'oklch(26.346% 0.018 262.177)'
const CHART_TEXT_COLOR = 'oklch(82.901% 0.031 222.959)'

const chartData = computed(() => ({
    labels: hourLabels.value,
    datasets: [
        {
            label: 'Download',
            data: history.value.map((p) => p.downloadBytes),
            backgroundColor: 'oklch(71% 0.203 305.504 / 0.7)',
            borderColor: 'oklch(71% 0.203 305.504)',
            borderWidth: 1,
            borderRadius: 3,
        },
        {
            label: 'Upload',
            data: history.value.map((p) => p.uploadBytes),
            backgroundColor: 'oklch(74% 0.16 232.661 / 0.7)',
            borderColor: 'oklch(74% 0.16 232.661)',
            borderWidth: 1,
            borderRadius: 3,
        },
    ],
}))

const chartOptions = {
    responsive: true,
    animation: false,
    interaction: { mode: 'index', intersect: false },
    plugins: {
        legend: { labels: { color: CHART_TEXT_COLOR } },
        title: { display: false },
        tooltip: {
            callbacks: {
                label: (ctx) => {
                    return `${ctx.dataset.label}: ${formatBytes(ctx.raw ?? 0)}`
                },
            },
        },
    },
    scales: {
        x: {
            ticks: { color: CHART_TEXT_COLOR, maxTicksLimit: 12 },
            grid: { color: CHART_GRID_COLOR },
        },
        y: {
            min: 0,
            ticks: {
                color: CHART_TEXT_COLOR,
                callback: (v) => formatBytes(v),
            },
            grid: { color: CHART_GRID_COLOR },
        },
    },
}

function onThrottleChange(event) {
    const kb = Number(event.target.value)
    const bps = kb > 0 ? kb * 1024 : 0
    emit('throttle', { proc: props.proc, bps })
}
</script>
