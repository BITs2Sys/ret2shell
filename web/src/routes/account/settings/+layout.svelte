<script lang="ts">
  import { goto } from '$app/navigation'
  import { page } from '$app/stores'
  import { i18n } from '$lib/i18n'
  import { platform } from '$lib/stores/platform'
  import { user } from '$lib/stores/user'
  import Sidebar from './Sidebar.svelte'
  import Error from '$lib/blocks/Error.svelte'
  import SidebarLayout from '$lib/blocks/SidebarLayout.svelte'

  let error = 200

  if (!$user.isLoggedIn) {
    goto(`/account/login?redirect=${$page.url.pathname}`, { replaceState: true })
  }
</script>

<svelte:head><title>{$i18n.t('account.settings')} - {$platform.name}</title></svelte:head>

<SidebarLayout leftSidebar={Sidebar}>
  {#if error - 200 < 100}
    <slot />
  {:else}
    <Error status={error} />
  {/if}
</SidebarLayout>
