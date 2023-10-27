<script>
  import SidebarLayout from '$lib/blocks/SidebarLayout.svelte'
  import Router from './Router.svelte'
  import Sidebar from './Sidebar.svelte'
</script>

<SidebarLayout leftSidebar={Sidebar}>
  <div class="flex-1 flex flex-col">
    <Router />
    <slot />
  </div>
</SidebarLayout>
