<script lang="ts">
  import { page } from '$app/stores'
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'
  import { admin } from '$lib/stores/admin'

  $: routes = [
    {
      name: $i18n.t('admin.routes.statistics'),
      icon: 'icon-[fluent--data-pie-20-regular]',
      link: `/admin/games/${$admin.game?.id}/statistics`,
      enabled: true,
    },
    {
      name: $i18n.t('admin.routes.gameInfo'),
      icon: 'icon-[fluent--info-20-regular]',
      link: `/admin/games/${$admin.game?.id}/info`,
      enabled: true,
    },
    {
      name: $i18n.t('admin.routes.challenges'),
      icon: 'icon-[fluent--flag-20-regular]',
      link: `/admin/games/${$admin.game?.id}/challenges`,
      enabled: true,
    },
    {
      name: $i18n.t('admin.routes.tags'),
      icon: 'icon-[fluent--tag-20-regular]',
      link: `/admin/games/${$admin.game?.id}/tags`,
      enabled: true,
    },
    {
      name: $i18n.t('admin.routes.gameNotifications'),
      icon: 'icon-[fluent--chat-20-regular]',
      link: `/admin/games/${$admin.game?.id}/notifications`,
      enabled: $admin.game?.host_as_game,
    },
    {
      name: $i18n.t('admin.routes.gameTeams'),
      icon: 'icon-[fluent--people-20-regular]',
      link: `/admin/games/${$admin.game?.id}/teams`,
      enabled: $admin.game?.host_as_game,
    },
    {
      name: $i18n.t('admin.routes.writeups'),
      icon: 'icon-[fluent--people-20-regular]',
      link: `/admin/games/${$admin.game?.id}/writeups`,
      enabled: $admin.game?.host_as_game,
    },
    {
      name: $i18n.t('admin.routes.submissions'),
      icon: 'icon-[fluent--people-20-regular]',
      link: `/admin/games/${$admin.game?.id}/submissions`,
      enabled: true,
    },
  ]
</script>

{#each routes as item}
  <RxLink
    class="flex-nowrap overflow-hidden"
    href={item.link}
    ghost
    justify="start"
    exactlyMatched={item.link === $page.url.pathname}
    disabled={!item.enabled}
  >
    <span class={`${item.icon} w-5 h-5 flex-shrink-0`} />
    <span class="text-ellipsis whitespace-nowrap overflow-hidden">{item.name}</span>
  </RxLink>
{/each}
