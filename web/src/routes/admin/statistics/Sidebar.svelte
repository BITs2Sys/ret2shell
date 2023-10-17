<script lang="ts">
  import { page } from '$app/stores'
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'

  const routes = [
    {
      name: $i18n.t('admin.statisticsSummary'),
      icon: 'icon-[fluent--data-histogram-20-regular]',
      link: '/admin/statistics/overview',
    },
    {
      name: $i18n.t('admin.serverLogs'),
      icon: 'icon-[fluent--code-20-regular]',
      link: '/admin/statistics/logs',
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
