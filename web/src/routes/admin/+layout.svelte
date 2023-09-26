<script>
  import Router from './Router.svelte'
  import Sidebar from './Sidebar.svelte'
</script>

<div class="flex flex-row w-full flex-1">
  <Sidebar />
  <div class="flex-1 flex flex-col">
    <Router />
    <slot />
  </div>
</div>
