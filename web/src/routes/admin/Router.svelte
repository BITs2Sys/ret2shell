<script lang="ts">
  import { page } from '$app/stores'
  import RxLink from '$lib/components/RxLink.svelte'
  import { admin, refreshAdminRoute } from '$lib/stores/admin'
  import { onDestroy } from 'svelte'

  const unsubscribe = page.subscribe((value) => {
    if (value.url.pathname.startsWith('/admin')) {
      refreshAdminRoute(value.url.pathname)
    }
  })

  onDestroy(() => {
    unsubscribe()
  })
</script>

<div
  class="flex-shrink-0 h-16 sticky top-16 bg-neutral/30 backdrop-blur border-b border-b-base-content/5 z-30 px-2 flex flex-row items-center space-x-2"
>
  {#each $admin.route as route}
    <RxLink ghost href={route.path} exactlyMatched>{route.name}</RxLink>
    <span class="icon-[fluent--chevron-double-right-16-regular] opacity-60" />
  {/each}
</div>
