<script lang="ts">
  import { getUserIpAddresses, getUserTeams } from '$lib/api/user'
  import RxTag from '$lib/components/RxTag.svelte'
  import { i18n } from '$lib/i18n'
  import type { IpAddress } from '$lib/models/ip'
  import type { TeamWithGameName } from '$lib/models/team'
  import type { User } from '$lib/models/user'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'

  export let user: User | null
  export let ipAddrs: IpAddress[] = []
  let teams: TeamWithGameName[] = []
  export let loadingIp = false
  export let loadingGames = false

  $: if (user) {
    loadingIp = true
    loadingGames = true
    getUserIpAddresses(user.id)
      .then((value) => {
        ipAddrs = value
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('account.fetchInfoFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
      .finally(() => {
        loadingIp = false
      })
    getUserTeams(user.id)
      .then((value) => {
        teams = value
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('account.fetchInfoFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
      .finally(() => {
        loadingGames = false
      })
  }
</script>

<div class="flex flex-col p-6">
  <div class="h-12 flex flex-row items-center space-x-2 border-b-2 border-b-base-content/5 px-4">
    <span class="icon-[fluent--globe-location-20-regular] w-5 h-5"></span>
    <span class="text-base font-bold">{$i18n.t('admin.userIpAddresses')}</span>
  </div>
  {#if loadingIp}
    <div class="h-16 px-4 flex flex-row space-x-2 items-center">
      <span class="loading loading-spinner loading-sm"></span>
      <span>{$i18n.t('account.loading')}</span>
    </div>
  {/if}
  <div class="p-2 flex flex-row flex-wrap">
    {#each ipAddrs as ip}
      <RxTag label={ip.address} class="m-1"></RxTag>
    {/each}
  </div>
  <div class="h-12 flex flex-row items-center space-x-2 border-b-2 border-b-base-content/5 px-4 mt-6">
    <span class="icon-[fluent--data-area-20-regular] w-5 h-5"></span>
    <span class="text-base font-bold">{$i18n.t('account.recentActivities')}</span>
  </div>
  {#if loadingGames}
    <div class="h-16 px-4 flex flex-row space-x-2 items-center">
      <span class="loading loading-spinner loading-sm"></span>
      <span>{$i18n.t('account.loading')}</span>
    </div>
  {/if}
  {#each teams as team}
    <div class="h-12 flex flex-row items-center space-x-2 border-b border-b-base-content/5 px-4 mt-4">
      <span class="icon-[fluent--trophy-20-regular] w-5 h-5"></span>
      <span class="text-base flex-1">
        {$i18n.t('account.takePartAs', { team: team.name, game: team.game_name, score: team.score })}
      </span>
      <span class="text-base opacity-60">
        {new Date(team.last_active_at * 1000).toLocaleDateString('default', {
          year: 'numeric',
          day: '2-digit',
          month: '2-digit',
        })}
      </span>
    </div>
  {/each}
</div>
