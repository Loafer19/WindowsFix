<template>
    <div>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 mb-6">
            <div class="bg-base-200 rounded-lg p-4 flex items-center gap-3">
                <Icon name="arrowDown" class="w-8 h-8 text-info" />
                <div>
                    <div class="text-base-content/70 text-sm">Download</div>
                    <div class="text-2xl font-bold text-info">{{ formatSpeed(currentDownload) }}</div>
                </div>
            </div>
            <div class="bg-base-200 rounded-lg p-4 flex items-center gap-3">
                <Icon name="arrowUp" class="w-8 h-8 text-success" />
                <div>
                    <div class="text-base-content/70 text-sm">Upload</div>
                    <div class="text-2xl font-bold text-success">{{ formatSpeed(currentUpload) }}</div>
                </div>
            </div>
        </div>

        <div class="bg-base-200 rounded-lg p-4 mb-6">
            <Line :data="chartData" :options="chartOptions" />
        </div>

        <div class="bg-base-200 rounded-lg p-4">
            <div class="flex items-center gap-3 mb-2">
                <Icon name="speedometer" class="w-6 h-6 text-warning" />
                <span class="font-semibold">Global Speed Limit</span>
                <span class="badge badge-warning ml-auto">
                    {{ limitLabel }}
                </span>
            </div>
            <input
                type="range"
                class="range range-warning w-full"
                min="0"
                :max="limitSteps.length - 1"
                :value="limitIndex"
                @input="onLimitChange"
            />
            <div class="flex justify-between text-xs text-base-content/50 mt-1">
                <span>Unlimited</span>
                <span>128 KB/s</span>
                <span>512 KB/s</span>
                <span>1 MB/s</span>
                <span>5 MB/s</span>
                <span>10 MB/s</span>
            </div>
        </div>
    </div>
</template>

<script setup>
import { computed, ref } from 'vue'
import { Line } from 'vue-chartjs'
import {
    CategoryScale,
    Chart as ChartJS,
    Filler,
    Legend,
    LineElement,
    LinearScale,
    PointElement,
    Title,
    Tooltip,
} from 'chart.js'
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

const props = defineProps({
    downloadHistory: { type: Array, default: () => [] },
    uploadHistory: { type: Array, default: () => [] },
    labels: { type: Array, default: () => [] },
    formatSpeed: { type: Function, required: true },
})

const emit = defineEmits(['limit-change'])

const limitSteps = [0, 131072, 524288, 1048576, 5242880, 10485760]
const limitLabels = ['Unlimited', '128 KB/s', '512 KB/s', '1 MB/s', '5 MB/s', '10 MB/s']
const limitIndex = ref(0)

const currentDownload = computed(() => props.downloadHistory.at(-1) ?? 0)
const currentUpload = computed(() => props.uploadHistory.at(-1) ?? 0)
const limitLabel = computed(() => limitLabels[limitIndex.value])

const chartData = computed(() => ({
    labels: props.labels,
    datasets: [
        {
            label: 'Download',
            data: props.downloadHistory,
            borderColor: 'oklch(74% 0.16 232.661)',
            backgroundColor: 'oklch(74% 0.16 232.661 / 0.15)',
            borderWidth: 2,
            pointRadius: 0,
            fill: true,
            tension: 0.4,
        },
        {
            label: 'Upload',
            data: props.uploadHistory,
            borderColor: 'oklch(76% 0.177 163.223)',
            backgroundColor: 'oklch(76% 0.177 163.223 / 0.15)',
            borderWidth: 2,
            pointRadius: 0,
            fill: true,
            tension: 0.4,
        },
    ],
}))

const chartOptions = {
    responsive: true,
    animation: false,
    interaction: { mode: 'index', intersect: false },
    plugins: {
        legend: {
            labels: { color: 'oklch(82.901% 0.031 222.959)' },
        },
        title: {
            display: true,
            text: 'Bandwidth (bytes/s)',
            color: 'oklch(82.901% 0.031 222.959)',
        },
        tooltip: {
            callbacks: {
                label: (ctx) => {
                    const bps = ctx.raw ?? 0
                    if (bps >= 1_048_576) return `${ctx.dataset.label}: ${(bps / 1_048_576).toFixed(2)} MB/s`
                    if (bps >= 1024) return `${ctx.dataset.label}: ${(bps / 1024).toFixed(1)} KB/s`
                    return `${ctx.dataset.label}: ${bps} B/s`
                },
            },
        },
    },
    scales: {
        x: { ticks: { color: 'oklch(82.901% 0.031 222.959)', maxTicksLimit: 10 }, grid: { color: 'oklch(26.346% 0.018 262.177)' } },
        y: { ticks: { color: 'oklch(82.901% 0.031 222.959)' }, grid: { color: 'oklch(26.346% 0.018 262.177)' }, min: 0 },
    },
}

function onLimitChange(e) {
    limitIndex.value = Number(e.target.value)
    emit('limit-change', limitSteps[limitIndex.value])
}
</script>
