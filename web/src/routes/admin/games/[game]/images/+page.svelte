<script lang="ts">
  import { i18n } from '$lib/i18n'
  import { platform } from '$lib/stores/platform'
</script>

<svelte:head><title>{$i18n.t('admin.gameImage')} - {$platform.name}</title></svelte:head>
<div class="flex-1 relative flex justify-center"></div>
