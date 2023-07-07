<script lang="ts">
  import { platform } from '$lib/stores/platform'
  import { i18n } from '$lib/i18n'
</script>

<svelte:head><title>{$i18n.t('announcements.title')} - {$platform.name}</title></svelte:head>

<div class="flex-1 flex flex-row p-4 lg:p-6 justify-center">
  <div class="flex-1 max-w-5xl" />
</div>
