<script lang="ts">
  import { platform } from '$lib/stores/platform'
  import { i18n } from '$lib/i18n'
</script>

<svelte:head><title>{$i18n.t('wiki.title')} - {$platform.name}</title></svelte:head>

