<script lang="ts">
  import RxButton from '$lib/components/RxButton.svelte'
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'
  import { Permission } from '$lib/models/user'
  import { game } from '$lib/stores/game'
  import { user } from '$lib/stores/user'

  export let canTakePartIn: boolean
</script>

<div class="p-2 flex flex-col">
  {#if $game.team}
    <RxLink justify="start" class="h-16 overflow-hidden" href="/account/profile">
      <div class="avatar">
        <div
          class="w-10 rounded-full ring-2 ring-offset-base-100 ring-offset-2 !flex flex-col justify-center items-center"
        >
          <span class="w-6 h-6 icon-[fluent--flag-16-regular]" />
        </div>
      </div>
      <div class="ml-2 flex flex-col items-start overflow-hidden">
        <span class="font-bold text-ellipsis overflow-hidden whitespace-nowrap w-full">{$game.team?.name}</span>
        <span class="opacity-60">0x{$game.team?.id.toString(16).padStart(6, '0')}</span>
      </div>
    </RxLink>
    <div class="divider m-0 ml-2 mr-2" />
    <RxButton justify="start" ghost>
      <span class="w-6 h-6 icon-[fluent--copy-16-regular] text-success" />
      {$i18n.t('games.copyInviteLink')}
    </RxButton>
  {:else if $user.permissions.find((p) => p === Permission.Devops || p === Permission.Organize)}
    <RxButton disabled justify="start">
      <span class="w-6 h-6 icon-[fluent--thumb-dislike-16-regular]" />
      {$i18n.t('games.adminCanNotTakePartIn')}
    </RxButton>
  {:else if canTakePartIn}
    <RxLink href={`/games/${$game.current?.id}/participate`} justify="start" ghost>
      <span class="w-6 h-6 icon-[fluent--thumb-like-16-regular]" />
      {$i18n.t('games.takePartIn')}
    </RxLink>
  {:else}
    <RxButton disabled justify="start">
      <span class="w-6 h-6 icon-[fluent--thumb-dislike-16-regular]" />
      {$i18n.t('games.cantTakePartIn')}
    </RxButton>
  {/if}
</div>
