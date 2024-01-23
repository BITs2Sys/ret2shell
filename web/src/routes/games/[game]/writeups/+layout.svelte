<script lang="ts">
  import SidebarLayout from '$lib/blocks/SidebarLayout.svelte'
  import Sidebar from './Sidebar.svelte'
</script>

<SidebarLayout leftSidebar={Sidebar}>
  <slot />
</SidebarLayout>
