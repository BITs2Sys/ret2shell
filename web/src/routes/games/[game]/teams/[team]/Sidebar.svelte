<script lang="ts">
  import { getGameTeamSolves, getTeamMembers } from '$lib/api/game'
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'
  import type { Team } from '$lib/models/team'
  import type { User } from '$lib/models/user'
  import { showMessage } from '$lib/stores/toast'
  import RxImage from '$lib/components/RxImage.svelte'
  import type { AxiosError } from 'axios'

  export let loading: boolean
  export let team: Team | null
  export let members: User[]
</script>

<div class="w-full flex flex-col">
  <div class="h-28 flex flex-row items-center p-6 border-b border-b-base-content/5">
    <div class="rounded-full overflow-clip w-12 h-12 mx-2 ring-4 ring-offset-base-100 ring-offset-4">
      <div class="w-full h-full flex flex-col justify-center items-center">
        <span class="text-3xl font-bold">{team?.name.slice(0, 1)}</span>
      </div>
    </div>
    <div class="flex flex-col justify-center space-y-2 ml-6">
      <span class="font-bold text-xl">{loading ? $i18n.t('account.loading') : team?.name}</span>
      <span class="font-bold text-base opacity-60">0x{team?.id.toString(16).padStart(6, '0')}</span>
    </div>
  </div>
  <div class="flex flex-col p-6 space-y-4">
    <h2 class="font-bold text-base opacity-60 flex flex-row space-x-2 items-center">
      <span class="icon-[fluent--person-20-regular] w-5 h-5"></span>
      <span>
        {$i18n.t('game.teamMembers')}
      </span>
    </h2>
    {#each members as item}
      <RxLink justify="start" class="h-16 flex flex-nowrap overflow-hidden" href={`/users/${item.id}`} ghost>
        <div class="avatar">
          <div
            class="w-10 rounded-full ring-2 ring-offset-base-100 ring-offset-2 !flex flex-col justify-center items-center"
          >
            {#if item.cover_path}
              <RxImage src={item.cover_path} loading={false} />
            {:else}
              <span class="w-5 h-5 icon-[fluent--person-20-regular]" />
            {/if}
          </div>
        </div>
        <div class="ml-2 flex flex-col items-start overflow-hidden">
          <span class="font-bold text-ellipsis overflow-hidden whitespace-nowrap w-full">{item.name}</span>
          <span class="opacity-60">0x{item.id.toString(16).padStart(6, '0')}</span>
        </div>
      </RxLink>
    {/each}
  </div>
</div>
