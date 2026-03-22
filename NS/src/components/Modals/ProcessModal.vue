<template>
    <div class="modal modal-open" @click.self="emit('close')">
        <div class="modal-box max-w-2xl w-full">
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
                <span class="badge badge-ghost font-mono text-xs">{{ proc.exePath }}</span>
            </div>

            <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-4">
                <StatCard icon="arrowDownCircle" label="Download" :value="formatSpeed(proc.downloadBps)" color="primary" size="lg" />
                <StatCard icon="arrowUpCircle" label="Upload" :value="formatSpeed(proc.uploadBps)" color="info" size="lg" />
                <StatCard icon="arrowDownCircle" label="Total DL" :value="formatBytes(proc.totalDownloadBytes)" color="primary" size="base" />
                <StatCard icon="arrowUpCircle" label="Total UL" :value="formatBytes(proc.totalUploadBytes)" color="info" size="base" />
            </div>

            <div class="bg-base-200 rounded-lg p-4 mb-4">
                <div class="flex items-center justify-between mb-3">
                    <div class="text-sm font-semibold text-base-content/70">{{ periodLabel }}</div>
                    <div class="flex gap-1">
                        <button
                            v-for="p in periods"
                            :key="p.key"
                            class="btn btn-xs"
                            :class="{ 'btn-primary': selectedPeriod === p.key, 'btn-ghost': selectedPeriod !== p.key }"
                            @click="selectedPeriod = p.key; loadHistory()"
                        >
                            {{ p.label }}
                        </button>
                    </div>
                </div>
                <div v-if="history.length === 0" class="text-center py-8 text-base-content/40 text-sm">
                    No history yet. Data accumulates over time.
                </div>
                <Bar v-else :data="chartData" :options="chartOptions" />
            </div>

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
                <span v-if="proc.blocked" class="badge badge-error text-xs">Blocked</span>
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
import { rustService } from '../../services/rust.js'
import Button from '../Button.vue'
import Icon from '../Icon.vue'
import StatCard from '../StatCard.vue'

ChartJS.register(CategoryScale, LinearScale, BarElement, Title, Tooltip, Legend)

const props = defineProps({
    proc: { type: Object, required: true },
})

const emit = defineEmits(['close', 'throttle'])

const history = ref([])
const selectedPeriod = ref('24h')

const periods = [
    { key: '24h', label: '24h' },
    { key: '7d', label: '7d' },
    { key: '30d', label: '30d' },
]

const periodLabel = computed(() => {
    switch (selectedPeriod.value) {
        case '24h':
            return 'Last 24 hours - hourly usage'
        case '7d':
            return 'Last 7 days - daily usage'
        case '30d':
            return 'Last 30 days - daily usage'
        default:
            return 'Usage'
    }
})

async function loadHistory() {
    try {
        const result = await rustService.getProcessHistory(
            props.proc.exePath,
            selectedPeriod.value,
        )
        history.value = result ?? []
    } catch {
        history.value = []
    }
}

onMounted(loadHistory)

const MONTHS = [
    'Jan',
    'Feb',
    'Mar',
    'Apr',
    'May',
    'Jun',
    'Jul',
    'Aug',
    'Sep',
    'Oct',
    'Nov',
    'Dec',
]

const hourLabels = computed(() => {
    const n = history.value.length
    if (n === 0) return []
    const now = new Date()

    if (selectedPeriod.value === '24h') {
        return Array.from({ length: n }, (_, i) => {
            const offset = n - 1 - i
            if (offset === 0) return 'now'
            const d = new Date(now.getTime() - offset * 3_600_000)
            return `${String(d.getHours()).padStart(2, '0')}:00`
        })
    }

    const MS_PER_DAY = 86_400_000
    return Array.from({ length: n }, (_, i) => {
        const offset = n - 1 - i
        if (offset === 0) return 'today'
        const d = new Date(now.getTime() - offset * MS_PER_DAY)
        return `${MONTHS[d.getMonth()]} ${d.getDate()}`
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

const chartOptions = computed(() => ({
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
            ticks: {
                color: CHART_TEXT_COLOR,
                maxTicksLimit: selectedPeriod.value === '24h' ? 12 : 7,
            },
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
}))

function onThrottleChange(event) {
    const kb = Number(event.target.value)
    const bps = kb > 0 ? kb * 1024 : 0
    emit('throttle', { proc: props.proc, bps })
}
</script>
