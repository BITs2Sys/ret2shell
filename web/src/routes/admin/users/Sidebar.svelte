<script lang="ts">
  import { page } from '$app/stores'
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'

  const routes = [
    {
      name: $i18n.t('admin.routes.persons'),
      icon: 'icon-[fluent--person-20-regular]',
      link: '/admin/users/persons',
    },
    {
      name: $i18n.t('admin.routes.institutes'),
      icon: 'icon-[fluent--people-20-regular]',
      link: '/admin/users/institutes',
    },
    {
      name: $i18n.t('admin.routes.permissions'),
      icon: 'icon-[fluent--shield-20-regular]',
      link: '/admin/users/permissions',
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
