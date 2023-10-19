<script lang="ts">
  import { i18n } from '$lib/i18n'
  import type { Team } from '$lib/models/team'
  import { game } from '$lib/stores/game'
  import { colorDefs, theme } from '$lib/stores/theme'
  import { Chart } from 'chart.js/auto'
  import { onMount } from 'svelte'

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
  let teams: Team[] = []

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

  onMount(() => {
    chart = new Chart(canvas, {
      type: 'line',
      data: {
        labels: [],
        datasets: [],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          legend: {
            position: 'bottom',
            labels: {
              usePointStyle: true,
              pointStyle: 'rectRounded',
            },
          },
          tooltip: {
            enabled: true,
            usePointStyle: true,
          },
        },
      },
    })
  })
</script>

<svelte:head>
  <title>{$i18n.t('game.scoreboard')} - {$game.current?.name}</title>
</svelte:head>
<div class="flex-1 flex flex-col p-6 lg:p-12">
  <div class="h-80 rounded-box overflow-hidden bg-neutral/30 backdrop-blur">
    <div class="w-full h-full relative">
      <canvas bind:this={canvas}></canvas>
      {#if teams.length === 0}
        <div class="absolute w-full h-full top-0 left-0 flex flex-row items-center justify-center">
          <span class="font-bold text-base opacity-80">{$i18n.t('game.noScoreboard')}</span>
        </div>
      {/if}
    </div>
  </div>
</div>
