<script lang="ts">
  import { i18n } from '$lib/i18n'
  import { Permission } from '$lib/models/user'
  import { game } from '$lib/stores/game'
  import { theme } from '$lib/stores/theme'
  import { user } from '$lib/stores/user'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte'
  import type { Notification } from '$lib/models/game'

  export let notifications: Notification[]
</script>

<div class="flex flex-col h-32 sticky top-0 bg-neutral/30 border-b border-b-base-content/5 z-10 p-6">
  <div class="flex-1 flex flex-row items-center space-x-2">
    {#if $game.team}
      <span class="icon-[fluent--people-team-16-regular] w-6 h-6" />
      <span>{$game.team?.name}</span>
      <span class="flex-1" />
      <span>#{$game.team?.id}</span>
    {:else if $user.permissions.find((p) => p === Permission.Devops || p === Permission.Organize)}
      <span class="icon-[fluent--person-wrench-20-regular] w-6 h-6 text-info flex-shrink-0" />
      <span class="text-info font-bold">{$i18n.t('games.teamAsAdmin')}</span>
    {/if}
  </div>
  <div class="flex-1 flex flex-row items-center space-x-2">
    {#if $game.team}
      <div></div>
    {:else if $user.permissions.find((p) => p === Permission.Devops || p === Permission.Organize)}
      <span class="icon-[fluent--info-20-regular] w-6 h-6 opacity-60 flex-shrink-0" />
      <span class="text-base font-bold opacity-60">{$i18n.t('games.teamAsAdminTips')}</span>
    {/if}
  </div>
</div>
<OverlayScrollbarsComponent
  options={{
    scrollbars: { theme: $theme.colorScheme === 'light' ? 'os-theme-dark' : 'os-theme-light', autoHide: 'scroll' },
  }}
  class="w-full flex-1 relative print:hidden"
  defer
>
  <div class="flex flex-col min-h-full">
    <div class="flex-1 flex flex-col">
      {#if notifications.length === 0}
        <div class="flex-1 flex flex-col items-center justify-center">
          <span class="text-base font-bold opacity-60">{$i18n.t('playground.noNotifications')}</span>
        </div>
      {/if}
      {#each notifications as item}
        <div class="flex flex-col p-6">
          <h2 class="text-base font-bold flex flex-row items-center space-x-2" title={item.title}>
            <span class="icon-[fluent--info-20-regular] w-6 h-6 text-info" />
            <span class="flex-1 text-ellipsis text-start whitespace-nowrap overflow-hidden">{item.title}</span>
            <span class="text-end opacity-60">
              {new Date(item.published_at * 1000).toLocaleString()}
            </span>
          </h2>
          <div class="h-px bg-base-content/5 my-2"></div>
          <p class="opacity-80">{item.content}</p>
        </div>
      {/each}
    </div>
  </div>
</OverlayScrollbarsComponent>
