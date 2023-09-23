<script>
  import AdminSidebar from '$lib/blocks/AdminSidebar.svelte'
</script>

<div class="flex flex-row w-full flex-1">
  <AdminSidebar />
  <div class="flex-1">
    <slot />
  </div>
</div>
