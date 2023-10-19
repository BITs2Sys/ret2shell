<script lang="ts">
  import { getPlatformStat } from '$lib/api/platform'
  import { i18n } from '$lib/i18n'
  import type { CPULoad, PlatformStat } from '$lib/models/config'
  import { humanFileSize } from '$lib/shell/utils/size'
  import { platform } from '$lib/stores/platform'
  import { colorDefs, theme } from '$lib/stores/theme'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import Chart from 'chart.js/auto'
  import { blur } from 'svelte/transition'
  import { onDestroy, onMount } from 'svelte'

  let loading = true

  Chart.defaults.font.family =
    '"JetBrains Mono", Menlo, -apple-system, "Noto Sans", "Helvetica Neue", Helvetica, "Nimbus Sans L", Arial, "Liberation Sans", "PingFang SC", "Hiragino Sans GB", "Noto Sans CJK SC", "Source Han Sans SC", "Source Han Sans CN", "Microsoft YaHei", Consolas, Courier, monospace'
  Chart.defaults.font.size = 16
  Chart.defaults.color = colorDefs()['base-content']
  Chart.defaults.borderColor = '#80808020'
  Chart.defaults.plugins.tooltip.borderWidth = 1
  Chart.defaults.plugins.tooltip.padding = 16
  Chart.defaults.plugins.tooltip.bodySpacing = 6
  Chart.defaults.plugins.tooltip.caretSize = 0
  Chart.defaults.plugins.tooltip.intersect = false
  Chart.defaults.plugins.tooltip.mode = 'index'
  Chart.defaults.plugins.tooltip.backgroundColor = colorDefs().neutral
  Chart.defaults.plugins.tooltip.borderColor = colorDefs().border
  Chart.defaults.plugins.tooltip.titleColor = colorDefs()['base-content']
  Chart.defaults.plugins.tooltip.bodyColor = colorDefs()['base-content']
  Chart.defaults.plugins.tooltip.footerColor = colorDefs()['base-content']
  Chart.defaults.animation = false
  let canvas: HTMLCanvasElement
  let chart: Chart
  theme.subscribe(() => {
    if (chart) {
      chart.options.color = colorDefs()['base-content']
      if (chart.options.plugins?.tooltip) {
        chart.options.plugins.tooltip.backgroundColor = colorDefs().neutral
        chart.options.plugins.tooltip.borderColor = colorDefs().border
        chart.options.plugins.tooltip.titleColor = colorDefs()['base-content']
        chart.options.plugins.tooltip.bodyColor = colorDefs()['base-content']
        chart.options.plugins.tooltip.footerColor = colorDefs()['base-content']
      }
      if (chart.options.scales?.y?.ticks) {
        chart.options.scales.y.ticks.color = colorDefs()['base-content']
      }
      if (chart.options.scales?.x?.ticks) {
        chart.options.scales.x.ticks.color = colorDefs()['base-content']
      }
      chart.update()
    }
  })
  let stat: PlatformStat = {
    cpu: [],
    memory: { total: 0, free: 0 },
    swap: { total: 0, free: 0 },
    disks: [],
    uptime: 0,
  }

  let cpusHistory: Record<number, number[]> = {}

  $: pushCpusHistory(stat.cpu)
  function pushCpusHistory(cpus: CPULoad[]) {
    cpus.forEach((cpu, index) => {
      if (!cpusHistory[index]) {
        cpusHistory[index] = Array(50).fill(0)
      }
      cpusHistory[index].push((cpu.system + cpu.user) * 100)
      if (cpusHistory[index].length > 50) {
        cpusHistory[index].shift()
      }
    })

    if (chart) {
      chart.data.datasets = [
        ...Object.entries(cpusHistory).map(([key, value]) => {
          return {
            label: '',
            data: value,
            fill: true,
            tension: 0.1,
          }
        }),
      ]
      chart.update()
    }
  }

  onMount(() => {
    chart = new Chart(canvas, {
      type: 'line',
      data: {
        labels: Array(50).fill(''),
        datasets: [],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          legend: {
            display: false,
          },
          tooltip: {
            enabled: false,
          },
        },
        scales: {
          y: {
            suggestedMax: 100.0,
            suggestedMin: 0.0,
          },
        },
      },
    })
  })

  function refreshStatistics() {
    getPlatformStat()
      .then((res) => {
        stat = res
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('platform.fetchStatFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
      .finally(() => {
        loading = false
      })
  }

  const timer = setInterval(() => {
    refreshStatistics()
  }, 3000)

  onMount(() => {
    loading = true
    refreshStatistics()
  })

  onDestroy(() => {
    clearInterval(timer)
  })

  $: systemCpuUsage = (stat.cpu.reduce((acc, cur) => acc + cur.system, 0) / stat.cpu.length) * 100
  $: userCpuUsage = (stat.cpu.reduce((acc, cur) => acc + cur.user, 0) / stat.cpu.length) * 100
  $: totalCpuUsage = systemCpuUsage + userCpuUsage
  $: validDisks = stat.disks
    .filter((disk) => disk.total > 0)
    .filter(
      (disk) =>
        !disk.fs_mounted_on.startsWith('/sys') &&
        !disk.fs_mounted_on.startsWith('/dev') &&
        !disk.fs_mounted_on.startsWith('/proc') &&
        !disk.fs_type.includes('tmpfs')
    )
</script>

<svelte:head><title>{$i18n.t('admin.serverLogs')} - {$platform.name}</title></svelte:head>

<div class="flex-1 flex flex-col p-6 space-y-4 relative">
  <h2 class="py-2 text-base font-bold border-b border-b-base-content/10">CPU Load</h2>
  <div class="flex flex-row space-x-2">
    <div class="w-48 flex flex-col justify-start space-y-2 text-base font-bold">
      <div class="flex flex-row rounded-lg bg-base-content/5 p-4 backdrop-blur">
        <div class="w-24 opacity-60">SYSTEM:</div>
        <span>{systemCpuUsage.toFixed(2)}%</span>
      </div>
      <div class="flex flex-row rounded-lg bg-base-content/5 p-4 backdrop-blur">
        <div class="w-24 opacity-60">USER:</div>
        <span>{userCpuUsage.toFixed(2)}%</span>
      </div>
      <div class="flex flex-row rounded-lg bg-base-content/5 p-4 backdrop-blur">
        <div class="w-24 opacity-60">TOTAL:</div>
        <span>{totalCpuUsage.toFixed(2)}%</span>
      </div>
    </div>
    <div class="h-64 flex-1 relative">
      <canvas bind:this={canvas}></canvas>
    </div>
  </div>
  <div class="flex flex-row space-x-4">
    <div class="flex-1 flex flex-col space-y-4">
      <h2 class="py-2 text-base font-bold border-b border-b-base-content/10">Memory</h2>
      <div class="flex flex-row space-x-4 items-center">
        <progress
          class="progress progress-success"
          max="100"
          value={((stat.memory.total - stat.memory.free || 0) / (stat.memory.total || 1)) * 100}
        ></progress>
        <span class="flex-grow flex-shrink-0">
          <span class="flex-grow flex-shrink-0">{humanFileSize(stat.memory.total - stat.memory.free)}</span>
          <span>/</span>
          <span class="flex-grow flex-shrink-0">{humanFileSize(stat.memory.total)}</span>
        </span>
      </div>
    </div>
    <div class="flex-1 flex flex-col space-y-4">
      <h2 class="py-2 text-base font-bold border-b border-b-base-content/10">Swap</h2>
      <div class="flex flex-row space-x-4 items-center">
        <progress
          class="progress progress-info"
          max="100"
          value={((stat.swap.total - stat.swap.free || 0) / (stat.swap.total || 1)) * 100}
        ></progress>
        <span class="flex-grow flex-shrink-0">
          <span class="flex-grow flex-shrink-0">{humanFileSize(stat.swap.total - stat.swap.free)}</span>
          <span>/</span>
          <span class="flex-grow flex-shrink-0">{humanFileSize(stat.swap.total)}</span>
        </span>
      </div>
    </div>
  </div>
  <h2 class="py-2 text-base font-bold border-b border-b-base-content/10">Disks</h2>
  {#each validDisks as disk}
    <div class="flex flex-col">
      <h3 class="flex flex-row justify-between">
        <span>
          <span class="opacity-80">{disk.fs_mounted_from}</span>
          <span class="opacity-60">-&gt;</span>
          <span class="opacity-80">{disk.fs_mounted_on}</span>
        </span>
        <span class="font-bold text-primary uppercase">{disk.fs_type}</span>
      </h3>
      <div class="flex flex-row space-x-4 items-center">
        <progress
          class={`progress ${
            disk.free / disk.total > 0.1
              ? disk.free / disk.total > 0.6
                ? 'progress-success'
                : 'progress-warning'
              : 'progress-error'
          }`}
          value={((disk.total - disk.free) / disk.total) * 100}
          max="100"
        ></progress>
        <span class="flex-grow flex-shrink-0">
          <span class="flex-grow flex-shrink-0">{humanFileSize(disk.total - disk.free)}</span>
          <span>/</span>
          <span class="flex-grow flex-shrink-0">{humanFileSize(disk.total)}</span>
        </span>
      </div>
    </div>
  {/each}
  {#if loading}
    <div
      class="absolute top-0 left-0 w-full h-full z-20 backdrop-blur flex flex-row justify-center items-center"
      transition:blur={{ amount: 20, duration: 300 }}
    >
      <span class="loading loading-spinner loading-sm" />
    </div>
  {/if}
</div>
