<script lang="ts">
  import { page } from '$app/stores'
  import { getGameTeamExtras, getGameTeamRank, getGameTeamSolves, getTeamInfo, getTeamMembers } from '$lib/api/v1/game'
  import SidebarLayout from '$lib/blocks/SidebarLayout.svelte'
  import { i18n } from '$lib/i18n'
  import type { Team } from '$lib/models/team'
  import { game } from '$lib/stores/game'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { onDestroy, onMount } from 'svelte'
  import Sidebar from './Sidebar.svelte'
  import Error from '$lib/blocks/Error.svelte'
  import type { Submission } from '$lib/models/submission'
  import type { User } from '$lib/models/user'
  import { getChallengeList, getTagList } from '$lib/api/v1/challenge'
  import { colorDefs, theme } from '$lib/stores/theme'
  import { Chart } from 'chart.js/auto'
  import type { Challenge, Tag } from '$lib/models/challenge'
  import RxTag from '$lib/components/RxTag.svelte'
  import type { Extra } from '$lib/models/extra'
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
  // Chart.defaults.animation = false

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

  let team: Team | null = null
  let loading = true
  let error = 200
  let solved: Submission[] = []
  let members: User[] = []
  let challenges = $game.challenges
  let tagsChallengesRecord: Record<number, Challenge[]> = {}
  let tags: Tag[] = []
  let extras: Extra[] = []
  let rank = 0
  $: {
    tagsChallengesRecord = {}
    if (tags.length > 0) {
      tags.forEach((tag) => {
        tagsChallengesRecord[tag.id] = challenges.filter((challenge) => challenge.tag_id === tag.id)
      })
    }
    // console.log(tagsChallengesRecord)
  }
  $: availableTags = tags.filter((tag) => tagsChallengesRecord[tag.id]?.length > 0)

  function refreshDataPie() {
    if (chart) {
      chart.data.labels = availableTags.map((tag) => tag.name)
      // console.log(chart.data.labels)
      chart.data.datasets = [
        {
          label: $i18n.t('game.teamStat'),
          data: availableTags.map((tag) => {
            const solvedChallenges = tagsChallengesRecord[tag.id]?.filter(
              (challenge) => solved.find((x) => x.challenge_id === challenge.id) !== undefined
            )
            return (solvedChallenges.length / tagsChallengesRecord[tag.id]?.length || 0) * 100
          }),
          fill: true,
          pointStyle: false,
          borderWidth: 1,
        },
      ]

      chart.update()
    }
  }

  const unsubscribe = game.subscribe((val) => {
    if (val.current) {
      const teamId = parseInt($page.params['team']) || -1
      if (teamId < 0 || Number.isNaN(teamId)) {
        error = 404
        showMessage('error', $i18n.t('game.teamNotFound'), 5000)
      } else {
        loading = true
        if (challenges.length === 0) {
          getChallengeList(val.current.id, 1, 200)
            .then((data) => {
              challenges = data.challenges
              setTimeout(() => {
                refreshDataPie()
                updateUserStats()
              }, 300)
            })
            .catch((err) => {
              showMessage('error', `${$i18n.t('challenge.fetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
            })
        }
        getTagList()
          .then((res) => {
            tags = res.toSorted((a, b) => (a.name > b.name ? 1 : a.name === b.name ? 0 : -1))
            setTimeout(() => {
              refreshDataPie()
              updateUserStats()
            }, 300)
          })
          .catch((err) => {
            showMessage(
              'error',
              `${$i18n.t('playground.fetchTagsFailed')}: ${(err as AxiosError).response?.data}`,
              5000
            )
          })
        getTeamInfo(val.current.id, teamId)
          .then((data) => {
            team = data
            getGameTeamSolves(team.game_id, team.id)
              .then((data) => {
                solved = data
              })
              .catch((err) => {
                showMessage('error', `${$i18n.t('game.teamFetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
              })
            getTeamMembers(team.game_id, team.id)
              .then((data) => {
                members = data
              })
              .catch((err) => {
                showMessage('error', `${$i18n.t('game.teamFetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
              })
            error = 200
          })
          .catch((err) => {
            showMessage('error', `${$i18n.t('game.teamFetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
            error = (err as AxiosError).response?.status || 500
          })
          .finally(() => {
            loading = false
          })
        getGameTeamExtras(val.current.id, teamId)
          .then((data) => {
            extras = data
          })
          .catch((err) => {
            showMessage('error', `${$i18n.t('game.teamFetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
          })
        getGameTeamRank(val.current.id, teamId)
          .then((data) => {
            rank = data.rank
          })
          .catch((err) => {
            showMessage('error', `${$i18n.t('game.teamFetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
          })
      }
    }
  })

  onMount(() => {
    chart = new Chart(canvas, {
      type: 'radar',
      data: {
        labels: availableTags.map((tag) => tag.name),
        datasets: [],
      },
      options: {
        scales: {
          r: {
            angleLines: {
              display: false,
            },
            suggestedMin: 100,
            suggestedMax: 100,
            pointLabels: {
              display: true,
              centerPointLabels: true,
            },
            ticks: {
              display: false,
            },
          },
        },
        plugins: {
          legend: {
            display: false,
          },
        },
      },
    })
  })

  onDestroy(() => {
    unsubscribe()
  })

  interface UserStat {
    id: number
    name: string
    score: number
    preferedTagId: number
  }

  let userStats: UserStat[] = []
  function updateUserStats() {
    userStats = members.map((member) => {
      const solvedChallenges = challenges.filter((challenge) => solved.find((x) => x.challenge_id === challenge.id))
      const solvedRecordByMember = solved.filter((s) => s.user_id === member.id)
      const score = solvedRecordByMember.reduce((prev, curr) => {
        return prev + (challenges.find((x) => x.id === curr.challenge_id)?.current_score || 0)
      }, 0)
      const preferedTagId = solvedChallenges
        .map((challenge) => challenge.tag_id)
        .reduce(
          (prev, curr) => {
            prev[curr] = (prev[curr] || 0) + 1
            return prev
          },
          {} as Record<number, number>
        )
      let max = 0
      let maxId = -1
      Object.keys(preferedTagId).forEach((key) => {
        if (preferedTagId[parseInt(key)] > max) {
          max = preferedTagId[parseInt(key)]
          maxId = parseInt(key)
        }
      })
      return {
        id: member.id,
        name: member.name,
        score,
        preferedTagId: maxId,
      }
    })
  }
</script>

<svelte:head><title>{team?.name} - {$game.current?.name}</title></svelte:head>
<SidebarLayout
  leftSidebar={Sidebar}
  leftProps={{
    loading,
    team,
    members,
  }}
>
  {#if error - 200 < 100}
    <div class="flex-1 flex flex-col items-center p-4 lg:p-6">
      <div class="w-full max-w-5xl flex flex-col">
        <h2 class="h-12 text-base font-bold flex flex-row space-x-2 items-center border-b-2 border-b-base-content/5">
          <span class="icon-[fluent--data-pie-20-regular] w-5 h-5"></span>
          <span class="text-base font-bold">{$i18n.t('game.teamStat')}</span>
        </h2>
        <div class="min-h-[24rem] flex flex-row p-4">
          <div class="h-full aspect-square relative">
            <canvas bind:this={canvas}></canvas>
          </div>
          <div class="flex-1 flex flex-col p-4">
            <div class="h-12 border-b border-b-base-content/5 flex flex-row items-center px-4 space-x-2">
              <span class="icon-[fluent--flag-20-regular] w-5 h-5 text-error"></span>
              <a class="hover:underline font-bold" href="#">
                {team?.name}
              </a>
              <RxTag class="m-0"><span>{$i18n.t('game.currentRank')} #{rank}</span></RxTag>
              <div class="flex-1"></div>
              <span class="font-bold">
                <span>{team?.score}</span>
                <span class="opacity-60">pts</span>
              </span>
            </div>
            {#each userStats as item}
              <div class="h-12 border-b border-b-base-content/5 flex flex-row items-center px-4 space-x-2">
                <span class="icon-[fluent--person-20-regular] w-5 h-5"></span>
                <a class="hover:underline font-bold" href="/users/{item.id}">
                  {item.name}
                </a>
                <RxTag class="m-0">
                  <span>{tags.find((x) => x.id === item.preferedTagId)?.name}</span>
                </RxTag>
                <div class="flex-1"></div>
                <span class="font-bold">
                  <span>{item.score}</span>
                  <span class="opacity-60">pts</span>
                </span>
              </div>
            {/each}
            <div class="h-12 border-b border-b-base-content/5 flex flex-row items-center px-4 space-x-2">
              <span class="icon-[fluent--trophy-20-regular] w-5 h-5"></span>
              <span>{$i18n.t('game.teamExtraScore')}</span>
              <div class="flex-1"></div>
              <span class="font-bold">
                <span>
                  {extras
                    .map((x) => x.score)
                    .reduce((sum, num) => {
                      return sum + num
                    }, 0)}
                </span>
                <span class="opacity-60">pts</span>
              </span>
            </div>
          </div>
        </div>
        <h2 class="h-12 text-base font-bold flex flex-row space-x-2 items-center border-b-2 border-b-base-content/5">
          <span class="icon-[fluent--notepad-20-regular] w-5 h-5"></span>
          <span class="text-base font-bold">{$i18n.t('game.teamSolved')}</span>
        </h2>
        <p class="flex flex-col space-y-2">
          {#each solved as item}
            <div class="h-12 flex flex-row items-center space-x-2 border-b border-b-base-content/5 mt-2">
              <span class="icon-[fluent--flag-20-regular] w-5 h-5"></span>
              <span class="text-base flex-1">
                <a class="hover:underline font-bold" href="/games/{$game.current?.id}/challenges#{item.challenge_id}">
                  {challenges.find((x) => x.id === item.challenge_id)?.name}
                </a>
                {$i18n.t('game.eachSolvesByMember1')}
                <a class="hover:underline font-bold" href="/users/{members.find((x) => x.id === item.user_id)?.id}">
                  {members.find((x) => x.id === item.user_id)?.name}
                </a>
                {$i18n.t('game.eachSolvesByMember2')}
                <span class="font-bold">{challenges.find((x) => x.id === item.challenge_id)?.current_score}</span>
                pts
              </span>
              <span class="text-base opacity-60 px-4">
                {new Date(item.created_at * 1000).toLocaleString()}
              </span>
            </div>
          {/each}
        </p>
        <h2
          class="h-12 text-base font-bold flex flex-row space-x-2 items-center border-b-2 border-b-base-content/5 mt-8"
        >
          <span class="icon-[fluent--trophy-20-regular] w-5 h-5"></span>
          <span class="text-base font-bold">{$i18n.t('game.teamExtraScore')}</span>
        </h2>
        <p class="flex flex-col space-y-2">
          {#each extras as item}
            <div class="h-12 flex flex-row items-center space-x-2 border-b border-b-base-content/5 mt-2">
              {#if item.score > 0}
                <span class="icon-[fluent--chevron-double-up-20-regular] w-5 h-5 text-success"></span>
              {:else}
                <span class="icon-[fluent--chevron-double-down-20-regular] w-5 h-5 text-warning"></span>
              {/if}
              <span class="text-base flex-1">
                <span class="font-bold">{item.reason}</span>
              </span>
              <span>
                <span class="font-bold">{item.score}</span>
                <span class="font-bold opacity-60">pts</span>
              </span>
              <span class="text-base opacity-60 px-4">
                {new Date(item.created_at * 1000).toLocaleString()}
              </span>
            </div>
          {/each}
        </p>
      </div>
    </div>
  {:else}
    <Error status={error} />
  {/if}
</SidebarLayout>
