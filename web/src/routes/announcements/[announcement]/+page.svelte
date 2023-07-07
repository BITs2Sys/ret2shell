<script lang="ts">
    import { platform } from '$lib/stores/platform'
    let title = ''
</script>

<svelte:head><title>{title} - {$platform.name}</title></svelte:head>

<div class="flex-1 flex flex-row p-4 lg:p-6 justify-center">
  <div class="flex-1 max-w-5xl" />
</div>
