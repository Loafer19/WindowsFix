<template>
    <div>
        <!-- 24-hour total tiles -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 mb-6">
        <div class="bg-base-200 rounded-lg p-4 flex items-center gap-3">
            <Icon name="arrowDown" class="w-8 h-8 text-info" />
            <div>
                <div class="text-base-content/70 text-sm">24h Download</div>
                <div class="text-2xl font-bold text-info">{{ formatBytes(totals.downloadBytes) }}</div>
                <div class="text-xs text-base-content/50">completed hours only</div>
            </div>
        </div>
        <div class="bg-base-200 rounded-lg p-4 flex items-center gap-3">
            <Icon name="arrowUp" class="w-8 h-8 text-success" />
            <div>
                <div class="text-base-content/70 text-sm">24h Upload</div>
                <div class="text-2xl font-bold text-success">{{ formatBytes(totals.uploadBytes) }}</div>
                <div class="text-xs text-base-content/50">completed hours only</div>
            </div>
        </div>
    </div>

    <!-- Live speed tiles -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 mb-6">
            <div class="bg-base-200 rounded-lg p-4 flex items-center gap-3">
                <Icon name="arrowDown" class="w-8 h-8 text-info" />
                <div>
                    <div class="text-base-content/70 text-sm">Download</div>
                    <div class="text-2xl font-bold text-info">{{ formatSpeed(currentDownload) }}</div>
                    <div class="text-xs text-base-content/50">{{ currentDownloadDb.toFixed(1) }} dBbps</div>
                </div>
            </div>
            <div class="bg-base-200 rounded-lg p-4 flex items-center gap-3">
                <Icon name="arrowUp" class="w-8 h-8 text-success" />
                <div>
                    <div class="text-base-content/70 text-sm">Upload</div>
                    <div class="text-2xl font-bold text-success">{{ formatSpeed(currentUpload) }}</div>
                    <div class="text-xs text-base-content/50">{{ currentUploadDb.toFixed(1) }} dBbps</div>
                </div>
            </div>
        </div>

        <!-- dB bandwidth chart -->
        <div class="bg-base-200 rounded-lg p-4 mb-6">
            <Line :data="chartData" :options="chartOptions" />
        </div>

        <!-- Global speed limit slider -->
        <div class="bg-base-200 rounded-lg p-4">
            <div class="flex items-center gap-3 mb-2">
                <Icon name="speedometer" class="w-6 h-6 text-warning" />
                <span class="font-semibold">Global Speed Limit</span>
                <span class="badge badge-warning ml-auto">{{ limitLabel }}</span>
            </div>
            <input
                type="range"
                class="range range-warning w-full"
                min="0"
                :max="LIMIT_PRESETS.length - 1"
                :value="limitIndex"
                @input="onLimitChange"
            />
            <div class="flex justify-between text-xs text-base-content/50 mt-1">
                <span v-for="preset in LIMIT_PRESETS" :key="preset.value">{{ preset.label }}</span>
            </div>
        </div>
    </div>
</template>

<script setup>
import {
    CategoryScale,
    Chart as ChartJS,
    Filler,
    Legend,
    LinearScale,
    LineElement,
    PointElement,
    Title,
    Tooltip,
} from 'chart.js'
import { computed, ref } from 'vue'
import { Line } from 'vue-chartjs'
import { formatBytes, formatSpeed, toDb } from '../../composables/useNetwork.js'
import Icon from '../Icon.vue'

ChartJS.register(
    CategoryScale,
    LinearScale,
    PointElement,
    LineElement,
    Title,
    Tooltip,
    Legend,
    Filler,
)

// Bandwidth-limit presets: { value: bytes/s (0 = unlimited), label: display string }
const LIMIT_PRESETS = Object.freeze([
    { value: 0, label: 'Unlimited' },
    { value: 131_072, label: '128 KB/s' },
    { value: 524_288, label: '512 KB/s' },
    { value: 1_048_576, label: '1 MB/s' },
    { value: 5_242_880, label: '5 MB/s' },
    { value: 10_485_760, label: '10 MB/s' },
])

const props = defineProps({
    downloadHistory: { type: Array, default: () => [] },
    uploadHistory: { type: Array, default: () => [] },
    labels: { type: Array, default: () => [] },
    totals: {
        type: Object,
        default: () => ({ downloadBytes: 0, uploadBytes: 0 }),
    },
})

const emit = defineEmits(['limit-change'])

const limitIndex = ref(0)
const limitLabel = computed(() => LIMIT_PRESETS[limitIndex.value].label)

const currentDownload = computed(() => props.downloadHistory.at(-1) ?? 0)
const currentUpload = computed(() => props.uploadHistory.at(-1) ?? 0)
const currentDownloadDb = computed(() => toDb(currentDownload.value))
const currentUploadDb = computed(() => toDb(currentUpload.value))

// Chart datasets use dB values so the Y-axis shows dBbps
const chartData = computed(() => ({
    labels: props.labels,
    datasets: [
        {
            label: 'Download (dBbps)',
            data: props.downloadHistory.map(toDb),
            borderColor: 'oklch(74% 0.16 232.661)',
            backgroundColor: 'oklch(74% 0.16 232.661 / 0.15)',
            borderWidth: 2,
            pointRadius: 0,
            fill: true,
            tension: 0.4,
        },
        {
            label: 'Upload (dBbps)',
            data: props.uploadHistory.map(toDb),
            borderColor: 'oklch(76% 0.177 163.223)',
            backgroundColor: 'oklch(76% 0.177 163.223 / 0.15)',
            borderWidth: 2,
            pointRadius: 0,
            fill: true,
            tension: 0.4,
        },
    ],
}))

const CHART_GRID_COLOR = 'oklch(26.346% 0.018 262.177)'
const CHART_TEXT_COLOR = 'oklch(82.901% 0.031 222.959)'

const chartOptions = {
    responsive: true,
    animation: false,
    interaction: { mode: 'index', intersect: false },
    plugins: {
        legend: { labels: { color: CHART_TEXT_COLOR } },
        title: {
            display: true,
            text: 'Bandwidth (dBbps  —  0 dB ≈ 1 B/s  ·  30 dB ≈ 1 KB/s  ·  60 dB ≈ 1 MB/s)',
            color: CHART_TEXT_COLOR,
            font: { size: 11 },
        },
        tooltip: {
            callbacks: {
                label: (ctx) => {
                    const db = ctx.raw ?? 0
                    // Recover approximate raw speed for tooltip
                    const bps = db > 0 ? 10 ** (db / 10) : 0
                    return `${ctx.dataset.label.split(' ')[0]}: ${db.toFixed(1)} dBbps (${formatSpeed(Math.round(bps))})`
                },
            },
        },
    },
    scales: {
        x: {
            ticks: { color: CHART_TEXT_COLOR, maxTicksLimit: 10 },
            grid: { color: CHART_GRID_COLOR },
        },
        y: {
            min: 0,
            suggestedMax: 70,
            ticks: {
                color: CHART_TEXT_COLOR,
                callback: (v) => `${v} dB`,
            },
            grid: { color: CHART_GRID_COLOR },
            title: { display: true, text: 'dBbps', color: CHART_TEXT_COLOR },
        },
    },
}

function onLimitChange(e) {
    limitIndex.value = Number(e.target.value)
    emit('limit-change', LIMIT_PRESETS[limitIndex.value].value)
}
</script>
