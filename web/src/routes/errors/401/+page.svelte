<script lang="ts">
  import Error from '$lib/blocks/Error.svelte'
  import { i18n } from '$lib/i18n'
  import { platform } from '$lib/stores/platform'
</script>

<svelte:head><title>{$i18n.t('errors.401')} - {$platform.name}</title></svelte:head>
<Error status={401} />
