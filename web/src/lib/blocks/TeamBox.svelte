<script lang="ts">
    import RxLink from '$lib/components/RxLink.svelte'
    import { i18n } from '$lib/i18n'
    import { game } from '$lib/stores/game'
  </script>
  
  <div class="p-2 flex flex-col">
    <RxLink justify="start" class="h-16" href="/account/profile">
      <div class="avatar">
        <div
          class="w-10 rounded-full ring-2 ring-offset-base-100 ring-offset-2 !flex flex-col justify-center items-center"
        >
          <span class="w-6 h-6 icon-[fluent--flag-16-regular]" />
        </div>
      </div>
      <div class="ml-2 flex flex-col items-start">
        <span class="font-bold">{$game.team?.name}</span>
        <span class="opacity-60">0x{$game.team?.id.toString(16).padStart(6, '0')}</span>
      </div>
    </RxLink>
    <div class="divider m-0 ml-2 mr-2" />
    <RxLink href="/account/settings" justify="start" ghost>
      <span class="w-6 h-6 icon-[fluent--settings-16-regular]" />
      {$i18n.t('games.copyInviteLink')}
    </RxLink>
  </div>
  