<script lang="ts">
  import { i18n } from '$lib/i18n'
  import { platform } from '$lib/stores/platform'
  let title = $i18n.t('announcements.titleLoadingPlaceholder')
</script>

<svelte:head><title>{title} - {$platform.name}</title></svelte:head>

<div class="flex-1 flex flex-row p-4 lg:p-6 justify-center">
  <div class="flex-1 max-w-5xl" />
</div>
