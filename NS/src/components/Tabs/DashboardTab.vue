<template>
    <div>
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
            <StatCard icon="arrowDownCircle" label="24h Download" :value="calcBytes(totals.downloadBytes)" color="primary" size="lg" />
            <StatCard icon="arrowUpCircle" label="24h Upload" :value="calcBytes(totals.uploadBytes)" color="info" size="lg" />
            <StatCard icon="arrowDownCircle" label="Download" :value="calcSpeed(currentDownload)" color="primary" size="lg" />
            <StatCard icon="arrowUpCircle" label="Upload" :value="calcSpeed(currentUpload)" color="info" size="lg" />
        </div>

        <div class="bg-base-200 rounded-lg p-4">
            <Line :data="chartData" :options="chartOptions" />
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
import { computed } from 'vue'
import { Line } from 'vue-chartjs'
import { calcBytes, calcSpeed } from '../../services/helpers.js'
import StatCard from '../StatCard.vue'

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

const props = defineProps({
    downloadHistory: { type: Array, default: () => [] },
    uploadHistory: { type: Array, default: () => [] },
    labels: { type: Array, default: () => [] },
    totals: {
        type: Object,
        default: () => ({ downloadBytes: 0, uploadBytes: 0 }),
    },
})

const currentDownload = computed(() => props.downloadHistory.at(-1) ?? 0)
const currentUpload = computed(() => props.uploadHistory.at(-1) ?? 0)

const chartData = computed(() => ({
    labels: props.labels,
    datasets: [
        {
            label: 'Download',
            data: props.downloadHistory,
            borderColor: 'oklch(71% 0.203 305.504)',
            backgroundColor: 'oklch(71% 0.203 305.504 / 0.15)',
            borderWidth: 2,
            pointRadius: 0,
            fill: true,
            tension: 0.4,
        },
        {
            label: 'Upload',
            data: props.uploadHistory,
            borderColor: 'oklch(74% 0.16 232.661)',
            backgroundColor: 'oklch(74% 0.16 232.661 / 0.15)',
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
        legend: {
            labels: { color: CHART_TEXT_COLOR },
        },
        tooltip: {
            callbacks: {
                label: (ctx) => {
                    const bps = ctx.raw ?? 0
                    return `${ctx.dataset.label}: ${calcSpeed(bps)}`
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
            ticks: {
                color: CHART_TEXT_COLOR,
                callback: (v) => calcSpeed(v),
            },
            grid: { color: CHART_GRID_COLOR },
        },
    },
}
</script>
