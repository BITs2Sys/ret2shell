<script lang="ts">
  import RxButton from '$lib/components/RxButton.svelte'
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'
  import { Permission } from '$lib/models/user'
  import { user } from '$lib/stores/user'

  let firstLevelExpanded = true
  $: secondLevelExpanded = !firstLevelExpanded
</script>

<div
  class="h-[calc(100vh_-_4rem)] flex-shrink-0 bg-base-100/60 backdrop-blur border-r border-r-base-content/10 overflow-hidden flex flex-row"
>
  <div
    class={`${
      firstLevelExpanded ? 'w-[24rem]' : 'w-16 border-r'
    } transition-all flex flex-col p-2 space-y-2 duration-200 border-r-base-content/10`}
  >
    <div class="join">
      {#if firstLevelExpanded}
        <RxLink href="/admin" exactlyMatched ghost class="join-item flex-1 flex-nowrap overflow-hidden" justify="start">
          <span class="icon-[fluent--organization-16-regular] w-6 h-6 text-error" />
          <span class="text-ellipsis whitespace-nowrap overflow-hidden">{$i18n.t('admin.title')}</span>
        </RxLink>
      {/if}
      <RxButton
        square
        ghost
        class="join-item"
        on:click={() => {
          firstLevelExpanded = !firstLevelExpanded
        }}
      >
        <span
          class={`icon-[fluent--chevron-double-left-16-regular] w-6 h-6 transition-all duration-300 ${
            !firstLevelExpanded && 'rotate-180'
          }`}
        />
      </RxButton>
    </div>
    {#if $user.permissions.find((p) => p === Permission.Statistics || p === Permission.Devops || p === Permission.Audit || p === Permission.Organize)}
      <RxLink
        class="!mt-4 flex-nowrap overflow-hidden"
        href="/admin/statistics"
        ghost
        justify={firstLevelExpanded ? 'start' : 'center'}
        square={!firstLevelExpanded}
      >
        <span class="icon-[fluent--data-pie-24-regular] w-6 h-6 flex-shrink-0" />
        {#if firstLevelExpanded}
          <span class="text-ellipsis whitespace-nowrap overflow-hidden">{$i18n.t('admin.statistics')}</span>
        {/if}
      </RxLink>
    {/if}
    {#if $user.permissions.find((p) => p === Permission.Devops)}
      <RxLink
        class="flex-nowrap overflow-hidden"
        href="/admin/platform"
        ghost
        justify={firstLevelExpanded ? 'start' : 'center'}
        square={!firstLevelExpanded}
      >
        <span class="icon-[fluent--home-24-regular] w-6 h-6 flex-shrink-0" />
        {#if firstLevelExpanded}
          <span class="text-ellipsis whitespace-nowrap overflow-hidden">{$i18n.t('admin.platformSettings')}</span>
        {/if}
      </RxLink>
    {/if}
    {#if $user.permissions.find((p) => p === Permission.Devops || p === Permission.Organize || p === Permission.Audit)}
      <RxLink
        class="flex-nowrap overflow-hidden"
        href="/admin/games"
        ghost
        justify={firstLevelExpanded ? 'start' : 'center'}
        square={!firstLevelExpanded}
      >
        <span class="icon-[fluent--flag-24-regular] w-6 h-6 flex-shrink-0" />
        {#if firstLevelExpanded}
          <span class="text-ellipsis whitespace-nowrap overflow-hidden">{$i18n.t('admin.gamesSettings')}</span>
        {/if}
      </RxLink>
    {/if}
    {#if $user.permissions.find((p) => p === Permission.Publish)}
      <RxLink
        class="flex-nowrap overflow-hidden"
        href="/admin/announcements"
        ghost
        justify={firstLevelExpanded ? 'start' : 'center'}
        square={!firstLevelExpanded}
      >
        <span class="icon-[fluent--megaphone-24-regular] w-6 h-6 flex-shrink-0" />
        {#if firstLevelExpanded}
          <span class="text-ellipsis whitespace-nowrap overflow-hidden">{$i18n.t('admin.announcementsSettings')}</span>
        {/if}
      </RxLink>
    {/if}
    {#if $user.permissions.find((p) => p === Permission.Publish)}
      <RxLink
        class="flex-nowrap overflow-hidden"
        href="/admin/calendar"
        ghost
        justify={firstLevelExpanded ? 'start' : 'center'}
        square={!firstLevelExpanded}
      >
        <span class="icon-[fluent--calendar-24-regular] w-6 h-6 flex-shrink-0" />
        {#if firstLevelExpanded}
          <span class="text-ellipsis whitespace-nowrap overflow-hidden">{$i18n.t('admin.calendarSettings')}</span>
        {/if}
      </RxLink>
    {/if}
    {#if $user.permissions.find((p) => p === Permission.Publish)}
      <RxLink
        class="flex-nowrap overflow-hidden"
        href="/admin/wiki"
        ghost
        justify={firstLevelExpanded ? 'start' : 'center'}
        square={!firstLevelExpanded}
      >
        <span class="icon-[fluent--book-number-24-regular] w-6 h-6 flex-shrink-0" />
        {#if firstLevelExpanded}
          <span class="text-ellipsis whitespace-nowrap overflow-hidden">{$i18n.t('admin.wikiSettings')}</span>
        {/if}
      </RxLink>
    {/if}
    {#if $user.permissions.find((p) => p === Permission.Organize || p === Permission.Devops || p === Permission.Audit)}
      <RxLink
        class="flex-nowrap overflow-hidden"
        href="/admin/users"
        ghost
        justify={firstLevelExpanded ? 'start' : 'center'}
        square={!firstLevelExpanded}
      >
        <span class="icon-[fluent--person-24-regular] w-6 h-6 flex-shrink-0" />
        {#if firstLevelExpanded}
          <span class="text-ellipsis whitespace-nowrap overflow-hidden">{$i18n.t('admin.usersSettings')}</span>
        {/if}
      </RxLink>
    {/if}
  </div>
  <div class={`${secondLevelExpanded ? 'w-[20rem]' : 'w-0'} transition-all flex flex-col duration-200`}></div>
</div>
