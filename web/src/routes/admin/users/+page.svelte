<script lang="ts">
  import { goto } from '$app/navigation'
  import { i18n } from '$lib/i18n'
  import { platform } from '$lib/stores/platform'

  goto('/admin/users/persons', { replaceState: true })
</script>

<svelte:head><title>{$i18n.t('admin.usersSettings')} - {$platform.name}</title></svelte:head>
