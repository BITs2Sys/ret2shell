<script lang="ts">
  import { page } from '$app/stores'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'
  import { Permission } from '$lib/models/user'
  import { user } from '$lib/stores/user'
  import { onDestroy } from 'svelte'

  let firstLevelExpanded = true
  $: secondLevelExpanded = !firstLevelExpanded

  interface RouteItem {
    name: string
    icon: string
    link: string
  }

  let statisticsRoutes = [
    {
      name: $i18n.t('admin.statisticsSummary'),
      icon: 'icon-[fluent--data-histogram-24-regular]',
      link: '/admin/statistics/overview',
    },
    {
      name: $i18n.t('admin.serverLogs'),
      icon: 'icon-[fluent--code-24-regular]',
      link: '/admin/statistics/logs',
    },
  ] as RouteItem[]

  let platformRoutes = [
    {
      name: $i18n.t('admin.basicInfoSettings'),
      icon: 'icon-[fluent--info-24-regular]',
      link: '/admin/platform/info',
    },
    {
      name: $i18n.t('admin.captchaSettings'),
      icon: 'icon-[fluent--beaker-24-regular]',
      link: '/admin/platform/captcha',
    },
    {
      name: $i18n.t('admin.emailSettings'),
      icon: 'icon-[fluent--mail-24-regular]',
      link: '/admin/platform/email',
    },
    {
      name: $i18n.t('admin.mediaSettings'),
      icon: 'icon-[fluent--image-24-regular]',
      link: '/admin/platform/media',
    },
    {
      name: $i18n.t('admin.pusherSettings'),
      icon: 'icon-[fluent--bot-24-regular]',
      link: '/admin/platform/pusher',
    },
  ] as RouteItem[]

  let currentRoutes: RouteItem[] = []
  let secondTitle = ''
  let haveSecondLevel = false

  const pageUnsubscribe = page.subscribe((value) => {
    if (value.url.pathname) {
      if (value.url.pathname.startsWith('/admin/statistics')) {
        currentRoutes = statisticsRoutes
        firstLevelExpanded = false
        haveSecondLevel = true
        secondTitle = $i18n.t('admin.statistics')
      } else if (value.url.pathname.startsWith('/admin/platform')) {
        currentRoutes = platformRoutes
        firstLevelExpanded = false
        haveSecondLevel = true
        secondTitle = $i18n.t('admin.platformSettings')
      } else if (value.url.pathname === '/admin/games') {
        currentRoutes = []
        firstLevelExpanded = true
        haveSecondLevel = false
        // show games, when user choose it then we construct the second level
      } else if (value.url.pathname.startsWith('/admin/announcements')) {
        currentRoutes = []
        firstLevelExpanded = true
        haveSecondLevel = false
        secondTitle = $i18n.t('admin.announcementsSettings')
      } else if (value.url.pathname.startsWith('/admin/calendar')) {
        currentRoutes = []
        firstLevelExpanded = true
        haveSecondLevel = false
        secondTitle = $i18n.t('admin.calendarSettings')
      } else if (value.url.pathname.startsWith('/admin/wiki')) {
        currentRoutes = []
        firstLevelExpanded = false
        haveSecondLevel = true
        secondTitle = $i18n.t('admin.wikiSettings')
      } else if (value.url.pathname === '/admin/users') {
        currentRoutes = []
        firstLevelExpanded = false
        haveSecondLevel = true
        secondTitle = $i18n.t('admin.usersSettings')
      } else {
        currentRoutes = []
        firstLevelExpanded = true
        haveSecondLevel = false
        secondTitle = ''
      }
    }
  })

  onDestroy(() => {
    pageUnsubscribe()
  })
</script>

<div
  class="fixed top-16 left-0 w-1/5 min-w-[24rem] max-w-[32rem] h-[calc(100vh_-_4rem)] flex-shrink-0 bg-base-100/60 backdrop-blur border-r border-r-base-content/10 overflow-hidden flex flex-row"
>
  <div
    class={`${
      firstLevelExpanded ? 'w-full p-4' : 'w-16 border-r p-2'
    } transition-all flex flex-col space-y-2 duration-200 border-r-base-content/10`}
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
        class="join-item disabled:bg-transparent"
        disabled={!haveSecondLevel}
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
        class="flex-nowrap overflow-hidden"
        href="/admin/statistics"
        ghost
        justify={firstLevelExpanded ? 'start' : 'center'}
        square={!firstLevelExpanded}
        title={$i18n.t('admin.statistics')}
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
        title={$i18n.t('admin.platformSettings')}
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
        title={$i18n.t('admin.gamesSettings')}
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
        title={$i18n.t('admin.announcementsSettings')}
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
        title={$i18n.t('admin.calendarSettings')}
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
        title={$i18n.t('admin.wikiSettings')}
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
        title={$i18n.t('admin.usersSettings')}
      >
        <span class="icon-[fluent--person-24-regular] w-6 h-6 flex-shrink-0" />
        {#if firstLevelExpanded}
          <span class="text-ellipsis whitespace-nowrap overflow-hidden">{$i18n.t('admin.usersSettings')}</span>
        {/if}
      </RxLink>
    {/if}
  </div>
  <div class={`${secondLevelExpanded ? 'w-[calc(100%_-_4rem)]' : 'w-0'} transition-all flex flex-col duration-200`}>
    <div class="h-16 bg-base-100 border-b border-b-base-content/5 flex flex-row px-4 items-center">
      <h2 class="font-bold text-base flex flex-row space-x-2 items-center justify-center w-full overflow-hidden">
        <span class="text-ellipsis whitespace-nowrap overflow-hidden">{secondTitle}</span>
      </h2>
    </div>
    <div class="flex-1 flex flex-col space-y-2 p-4">
      {#each currentRoutes as item}
        <RxLink
          class="flex-nowrap overflow-hidden"
          href={item.link}
          ghost
          justify="start"
          exactlyMatched={item.link === $page.url.pathname}
        >
          <span class={`${item.icon} w-6 h-6 flex-shrink-0`} />
          <span class="text-ellipsis whitespace-nowrap overflow-hidden">{item.name}</span>
        </RxLink>
      {/each}
    </div>
  </div>
</div>
<div class="w-1/5 min-w-[24rem] max-w-[32rem]"></div>
