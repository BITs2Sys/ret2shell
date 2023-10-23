<script lang="ts">
  import { getChallengeList, getTagList } from '$lib/api/challenge'
  import { getGameScoreboard } from '$lib/api/game'
  import RxPaginator from '$lib/components/RxPaginator.svelte'
  import { i18n } from '$lib/i18n'
  import type { Challenge, Tag } from '$lib/models/challenge'
  import type { ScoreHistory, Team } from '$lib/models/team'
  import { game } from '$lib/stores/game'
  import { colorDefs, theme } from '$lib/stores/theme'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { Chart } from 'chart.js/auto'
  import 'chartjs-adapter-luxon'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte'
  import { onDestroy, onMount } from 'svelte'
  import type { Unsubscriber } from 'svelte/store'

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
  let page = 1
  let perPage = 15
  let total = 0
  let instituteId: number | null = null
  let showAll = false
  let scoreboardTeams: Team[] = []
  let cachedGameId = 0

  let tagsChallengesRecord: Record<number, Challenge[]> = {}
  let tags: Tag[] = []

  $: {
    tagsChallengesRecord = {}
    if (tags.length > 0) {
      tags.forEach((tag) => {
        tagsChallengesRecord[tag.id] = $game.challenges.filter((challenge) => challenge.tag_id === tag.id)
      })
    }
  }

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

  let unsubscribe: Unsubscriber

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
        scales: {
          x: {
            type: 'time',
            time: {
              displayFormats: {
                day: 'MM.dd',
              },
              tooltipFormat: 'DD T',
            },
            ticks: {
              autoSkip: false,
              maxRotation: 0,
              major: {
                enabled: true,
              },
              font: function (context) {
                if (context.tick && context.tick.major) {
                  return {
                    weight: 'bold',
                  }
                }
              },
            },
          },
          y: {
            suggestedMin: 0,
            suggestedMax: 2000,
          },
        },
      },
    })

    unsubscribe = game.subscribe((value) => {
      if (value.current && value.current.id !== cachedGameId) {
        cachedGameId = value.current.id
        getChallengeList(value.current.id, 1, 200)
          .then((res) => {
            game.update((value) => {
              value.challenges = res.challenges
              return value
            })
          })
          .catch((err) => {
            showMessage(
              'error',
              `${$i18n.t('playground.fetchChallengesFailed')}: ${(err as AxiosError).response?.data}`,
              5000
            )
          })
        refreshScoreboard()
        refreshTop10()
      }
    })
  })

  onMount(() => {
    getTagList()
      .then((res) => {
        tags = res.toSorted((a, b) => (a.name > b.name ? 1 : a.name === b.name ? 0 : -1))
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('playground.fetchTagsFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
  })

  function refreshScoreboard() {
    getGameScoreboard($game.current?.id || 0, page, perPage, showAll, instituteId)
      .then((value) => {
        teams = value.teams
        total = value.total
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('game.fetchScoreboardFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
  }

  function refreshTop10() {
    getGameScoreboard($game.current?.id || 0, 1, 10, showAll, instituteId)
      .then((value) => {
        scoreboardTeams = value.teams
        updateChart()
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('game.fetchScoreboardFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
  }

  function updateChart() {
    let labels = []
    // from $game.current.register_time to $game.current.end_time, label every day
    let start = $game.current?.register_time || 0
    let end = $game.current?.end_time || 0
    let days = (end - start) / 86400
    let step = days / 6
    for (let i = 0; i <= 6; i++) {
      labels.push(new Date((start + step * i * 86400) * 1000))
    }
    chart.data.labels = labels
    let datasets = []
    for (let team of scoreboardTeams) {
      datasets.push({
        label: team.name,
        data: team.history.map((value) => {
          return {
            x: value.time * 1000,
            y: value.score,
          }
        }),
        fill: false,
        stepped: true,
      })
    }
    chart.data.datasets = datasets
    chart.update()
  }

  onDestroy(() => {
    unsubscribe()
  })

  function getSolveState(historyItem: ScoreHistory[], challengeId: number) {
    const item = historyItem.find((i) => i.challenge_id === challengeId)
    if (item) {
      return item.blood_state
    } else {
      return null
    }
  }
</script>

<svelte:head>
  <title>{$i18n.t('game.scoreboard')} - {$game.current?.name}</title>
</svelte:head>
<div class="flex-1 flex flex-col p-6 lg:p-12 space-y-6">
  <div class="h-80 rounded-box overflow-hidden bg-neutral/30 backdrop-blur p-6 pb-4">
    <div class="w-full h-full relative">
      <canvas bind:this={canvas}></canvas>
      {#if teams.length === 0}
        <div class="absolute top-4 left-4 right-4 bottom-4 flex flex-row items-center justify-center">
          <span class="font-bold text-base opacity-80">{$i18n.t('game.noScoreboard')}</span>
        </div>
      {/if}
    </div>
  </div>
  <div class="flex flex-row">
    <table class="table-auto flex-1 flex-shrink-0 min-w-[32rem]">
      <thead class="border-b-4 border-b-base-content/10">
        <tr class="h-12"></tr>
        <tr class="border-b border-b-base-content/10 h-12">
          <th class="text-base font-bold">
            <div class="w-16"></div>
          </th>
          <th class="text-base font-bold max-w-0 w-full overflow-hidden">
            <div class="text-start whitespace-nowrap flex-nowrap truncate max-w-lg px-2">
              {$i18n.t('game.team')}
            </div>
          </th>
          <th class="text-base font-bold">
            <div class="w-32 text-start px-2">
              {$i18n.t('game.score')}
            </div>
          </th>
        </tr>
      </thead>
      <tbody>
        {#each teams as item, index}
          <tr class="h-12 border-b border-b-base-content/10">
            <td class="text-base font-bold sticky z-10 left-0">
              <div class="flex flex-row justify-center items-center">
                {#if index + 1 + (page - 1) * perPage === 1}
                  <span class="icon-[fluent--trophy-20-filled] text-yellow-500 w-5 h-5"></span>
                {:else if index + 1 + (page - 1) * perPage === 2}
                  <span class="icon-[fluent--trophy-20-filled] text-zinc-500 w-5 h-5"></span>
                {:else if index + 1 + (page - 1) * perPage === 3}
                  <span class="icon-[fluent--trophy-20-filled] text-orange-500 w-5 h-5"></span>
                {:else}
                  <span>{index + 1 + (page - 1) * perPage}</span>
                {/if}
              </div>
            </td>
            <td class="text-base font-bold sticky z-10 left-16 min-w-64 max-w-lg overflow-hidden">
              <div class="min-w-64 text-start whitespace-nowrap flex-nowrap truncate px-2">
                <a class="hover:underline" href={`/games/${$game.current?.id}/teams/${item.id}`}>{item.name}</a>
              </div>
            </td>
            <td class="text-base font-bold sticky z-10 left-80">
              <div class="px-2">
                {item.score}
                <span class="opacity-60">pts</span>
              </div>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
    <OverlayScrollbarsComponent
      options={{
        scrollbars: { theme: $theme.colorScheme === 'light' ? 'os-theme-dark' : 'os-theme-light', autoHide: 'scroll' },
      }}
      class="relative h-auto print:h-auto print:overflow-auto"
      defer
    >
      <table>
        <thead class="border-b-4 border-b-base-content/10">
          <tr class="h-12 border-b border-b-base-content/10">
            {#each tags as item}
              {#if tagsChallengesRecord[item.id].length > 0}
                <th
                  class="text-base font-bold h-12 border-l border-l-base-content/5"
                  colspan={tagsChallengesRecord[item.id].length}
                >
                  <span class="opacity-60">{item.name}</span>
                </th>
              {/if}
            {/each}
          </tr>
          <tr class="h-12">
            {#each tags as item}
              {#if tagsChallengesRecord[item.id].length > 0}
                {#each tagsChallengesRecord[item.id] as challenge}
                  <th class="text-base border-l border-l-base-content/5" title={challenge.name}>
                    <div class="w-20 overflow-hidden whitespace-nowrap truncate opacity-80">{challenge.name}</div>
                  </th>
                {/each}
              {/if}
            {/each}
          </tr>
        </thead>
        <tbody>
          {#each teams as item}
            <tr class="h-12 border-b border-b-base-content/10">
              {#each tags as tag}
                {#each tagsChallengesRecord[tag.id] as challenge}
                  <td class="text-base font-bold">
                    <div class="flex flex-row justify-center items-center">
                      {#if getSolveState(item.history, challenge.id) === 1}
                        <span class="icon-[fluent--trophy-20-filled] text-yellow-500 w-5 h-5"></span>
                      {:else if getSolveState(item.history, challenge.id) === 2}
                        <span class="icon-[fluent--trophy-20-filled] text-zinc-500 w-5 h-5"></span>
                      {:else if getSolveState(item.history, challenge.id) === 3}
                        <span class="icon-[fluent--trophy-20-filled] text-orange-500 w-5 h-5"></span>
                      {:else if getSolveState(item.history, challenge.id) !== null}
                        <span class="icon-[fluent--flag-20-filled] text-error w-5 h-5"></span>
                      {/if}
                    </div>
                  </td>
                {/each}
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </OverlayScrollbarsComponent>
  </div>
  <div class="flex-1"></div>
  <RxPaginator bind:page {total} />
</div>
