<script lang="ts">
    import { i18n } from "$lib/i18n"
    import { platform } from "$lib/stores/platform"

</script>

<svelte:head><title>{$i18n.t('account.3rdAuth')} - {$platform.name}</title></svelte:head>

