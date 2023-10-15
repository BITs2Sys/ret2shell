<script lang="ts">
  import { page } from '$app/stores'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'
  import { Permission } from '$lib/models/user'
  import { admin } from '$lib/stores/admin'
  import { theme } from '$lib/stores/theme'
  import { user } from '$lib/stores/user'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte'
  import { onDestroy } from 'svelte'

  let firstLevelExpanded = true
  $: secondLevelExpanded = !firstLevelExpanded

  const routes = [
    {
      name: $i18n.t('admin.statistics'),
      icon: 'icon-[fluent--data-pie-24-regular]',
      link: '/admin/statistics',
      permissions: [Permission.Statistics, Permission.Devops, Permission.Audit, Permission.Organize],
    },
    {
      name: $i18n.t('admin.platformSettings'),
      icon: 'icon-[fluent--home-24-regular]',
      link: '/admin/platform',
      permissions: [Permission.Devops],
    },
    {
      name: $i18n.t('admin.gamesSettings'),
      icon: 'icon-[fluent--flag-24-regular]',
      link: '/admin/games',
      permissions: [Permission.Devops, Permission.Organize, Permission.Audit],
    },
    {
      name: $i18n.t('admin.announcementsSettings'),
      icon: 'icon-[fluent--megaphone-24-regular]',
      link: '/admin/announcements',
      permissions: [Permission.Publish],
    },
    {
      name: $i18n.t('admin.calendarSettings'),
      icon: 'icon-[fluent--calendar-24-regular]',
      link: '/admin/calendar',
      permissions: [Permission.Calendar],
    },
    {
      name: $i18n.t('admin.wikiSettings'),
      icon: 'icon-[fluent--book-number-24-regular]',
      link: '/admin/wiki',
      permissions: [Permission.Publish],
    },
    {
      name: $i18n.t('admin.usersSettings'),
      icon: 'icon-[fluent--person-24-regular]',
      link: '/admin/users',
      permissions: [Permission.Organize, Permission.Devops, Permission.Audit],
    },
  ]

  $: {
    if ($admin.secondLevelComponent) {
      firstLevelExpanded = false
    } else {
      firstLevelExpanded = true
    }
  }
  let secondTitle = ''

  const pageUnsubscribe = page.subscribe((value) => {
    if (value.url.pathname) {
      const path = value.url.pathname
      const route = routes.find((r) => path.startsWith(r.link))
      if (route) {
        secondTitle = route.name
      } else {
        secondTitle = ''
      }
    }
  })

  onDestroy(() => {
    pageUnsubscribe()
  })
</script>

<div
  class="fixed top-16 left-0 w-1/5 min-w-[24rem] max-w-[32rem] h-[calc(100vh_-_4rem)] flex-shrink-0 bg-neutral/20 backdrop-blur border-r border-r-base-content/10 overflow-hidden flex flex-row"
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
        disabled={$admin.secondLevelComponent === null}
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
    {#each routes as item}
      <RxLink
        class="flex-nowrap overflow-hidden"
        href={item.link}
        ghost
        disabled={!$user.permissions.find((p) => item.permissions?.includes(p))}
        justify={firstLevelExpanded ? 'start' : 'center'}
        square={!firstLevelExpanded}
        title={item.name}
      >
        <span class={`${item.icon} w-6 h-6 flex-shrink-0`} />
        {#if firstLevelExpanded}
          <span class="text-ellipsis whitespace-nowrap overflow-hidden">{item.name}</span>
        {/if}
      </RxLink>
    {/each}
  </div>
  <div class={`${secondLevelExpanded ? 'w-[calc(100%_-_4rem)]' : 'w-0'} transition-all flex flex-col duration-200`}>
    <div class="h-16 flex-shrink-0 bg-neutral/10 border-b border-b-base-content/5 flex flex-row px-4 items-center">
      <h2 class="font-bold text-base flex flex-row space-x-2 items-center justify-center w-full overflow-hidden">
        <span class="text-ellipsis whitespace-nowrap overflow-hidden">{secondTitle}</span>
      </h2>
    </div>
    <OverlayScrollbarsComponent
      options={{
        scrollbars: { theme: $theme.colorScheme === 'light' ? 'os-theme-dark' : 'os-theme-light', autoHide: 'scroll' },
      }}
      class="w-full flex-1 relative print:hidden"
      defer
    >
      <div class="flex flex-col space-y-2 p-4">
        <svelte:component this={$admin.secondLevelComponent} />
      </div>
    </OverlayScrollbarsComponent>
  </div>
</div>
<div class="w-1/5 min-w-[24rem] max-w-[32rem]"></div>
