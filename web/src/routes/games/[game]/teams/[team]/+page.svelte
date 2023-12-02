<script lang="ts">
  import { page } from '$app/stores'
  import { getGameTeamSolves, getTeamInfo, getTeamMembers } from '$lib/api/game'
  import SidebarLayout from '$lib/blocks/SidebarLayout.svelte'
  import { i18n } from '$lib/i18n'
  import type { Team } from '$lib/models/team'
  import { game } from '$lib/stores/game'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { onDestroy } from 'svelte'
  import Sidebar from './Sidebar.svelte'
  import Error from '$lib/blocks/Error.svelte'
  import type { Submission } from '$lib/models/submission'
  import type { User } from '$lib/models/user'
  import { getChallengeList } from '$lib/api/challenge'

  let team: Team | null = null
  let loading = true
  let error = 200
  let solved: Submission[] = []
  let members: User[] = []
  let challenges = $game.challenges

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
            })
            .catch((err) => {
              showMessage('error', `${$i18n.t('challenge.fetchFailed')}: ${(err as AxiosError).response?.data}`, 5000)
            })
        }
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
      }
    }
  })

  onDestroy(() => {
    unsubscribe()
  })
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
      </div>
    </div>
  {:else}
    <Error status={error} />
  {/if}
</SidebarLayout>
