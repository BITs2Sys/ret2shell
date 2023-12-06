<script lang="ts">
  import { platform } from '$lib/stores/platform'
  import { i18n } from '$lib/i18n'
  import Sidebar from './Sidebar.svelte'
  import Error from '$lib/blocks/Error.svelte'
  import { onMount } from 'svelte'
  import { getUserInfo, getUserTeams } from '$lib/api/v1/user'
  import { user } from '$lib/stores/user'
  import type { AxiosError } from 'axios'
  import RxArticle from '$lib/components/RxArticle.svelte'
  import type { TeamWithGameName } from '$lib/models/team'
  import { showMessage } from '$lib/stores/toast'
  import SidebarLayout from '$lib/blocks/SidebarLayout.svelte'

  let loading = false
  let error = 200
  let teams: TeamWithGameName[] = []

  onMount(() => {
    loading = true
    getUserInfo($user.id)
      .then((value) => {
        user.update((val) => {
          val = {
            ...val,
            info: value,
          }
          return val
        })
        loading = false
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('account.fetchInfoFailed')}: ${(err as AxiosError).response?.data}`, 5000)
        error = (err as AxiosError).response?.status || 500
      })
    getUserTeams($user.id)
      .then((value) => {
        teams = value
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('account.fetchInfoFailed')}: ${(err as AxiosError).response?.data}`, 5000)
        error = (err as AxiosError).response?.status || 500
      })
  })
</script>

<svelte:head><title>{$i18n.t('account.profile')} - {$platform.name}</title></svelte:head>

<SidebarLayout
  leftSidebar={Sidebar}
  leftProps={{
    loading,
    user: $user.info,
  }}
>
  {#if error - 200 < 100}
    <div class="flex-1 flex flex-col items-center p-4 lg:p-6">
      <div class="w-full max-w-5xl flex flex-col">
        <h2 class="h-12 text-base font-bold flex flex-row space-x-2 items-center border-b-2 border-b-base-content/5">
          <span class="icon-[fluent--notepad-20-regular] w-5 h-5"></span>
          <span class="text-base font-bold">{$i18n.t('account.intro')}</span>
        </h2>
        <RxArticle class="mt-4" content={$user.info?.intro || $i18n.t('account.noIntro')}></RxArticle>
        <h2
          class="h-12 text-base font-bold flex flex-row space-x-2 items-center mt-4 border-b-2 border-b-base-content/5"
        >
          <span class="icon-[fluent--data-area-20-regular] w-5 h-5"></span>
          <span class="text-base font-bold">{$i18n.t('account.recentActivities')}</span>
        </h2>
        <p class="flex flex-col space-y-2">
          {#each teams as team}
            <div class="h-12 flex flex-row items-center space-x-2 border-b border-b-base-content/5 mt-2">
              <span class="icon-[fluent--trophy-20-regular] w-5 h-5"></span>
              <span class="text-base flex-1">
                {$i18n.t('account.takePartAs', { team: team.name, game: team.game_name, score: team.score })}
              </span>
              <span class="text-base opacity-60 px-4">
                {new Date(team.last_active_at * 1000).toLocaleDateString('default', {
                  year: 'numeric',
                  day: '2-digit',
                  month: '2-digit',
                })}
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
