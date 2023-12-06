<script lang="ts">
  import SidebarLayout from '$lib/blocks/SidebarLayout.svelte'
  import { i18n } from '$lib/i18n'
  import { game } from '$lib/stores/game'
  import Sidebar from './Sidebar.svelte'
</script>

<svelte:head>
  <title>{$i18n.t('games.writeups')} - {$game.current?.name}</title>
</svelte:head>

<SidebarLayout leftSidebar={Sidebar}></SidebarLayout>
