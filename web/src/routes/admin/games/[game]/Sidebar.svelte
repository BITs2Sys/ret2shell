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
    },
    {
      name: $i18n.t('admin.routes.gameInfo'),
      icon: 'icon-[fluent--info-20-regular]',
      link: `/admin/games/${$admin.game?.id}/info`,
    },
    {
      name: $i18n.t('admin.routes.challenges'),
      icon: 'icon-[fluent--flag-20-regular]',
      link: `/admin/games/${$admin.game?.id}/challenges`,
    },
    {
      name: $i18n.t('admin.routes.tags'),
      icon: 'icon-[fluent--tag-20-regular]',
      link: `/admin/games/${$admin.game?.id}/tags`,
    },
    {
      name: $i18n.t('admin.routes.gameNotifications'),
      icon: 'icon-[fluent--chat-20-regular]',
      link: `/admin/games/${$admin.game?.id}/notifications`,
    },
    {
      name: $i18n.t('admin.routes.gameTeams'),
      icon: 'icon-[fluent--people-20-regular]',
      link: `/admin/games/${$admin.game?.id}/teams`,
    },
    {
      name: $i18n.t('admin.routes.writeups'),
      icon: 'icon-[fluent--people-20-regular]',
      link: `/admin/games/${$admin.game?.id}/writeups`,
    },
    {
      name: $i18n.t('admin.routes.submissions'),
      icon: 'icon-[fluent--people-20-regular]',
      link: `/admin/games/${$admin.game?.id}/submissions`,
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
  >
    <span class={`${item.icon} w-5 h-5 flex-shrink-0`} />
    <span class="text-ellipsis whitespace-nowrap overflow-hidden">{item.name}</span>
  </RxLink>
{/each}
