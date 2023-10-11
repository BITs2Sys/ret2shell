<script lang="ts">
  import { goto } from '$app/navigation'
  import { admin } from '$lib/stores/admin'
  import { onDestroy } from 'svelte'

  const unsubscribe = admin.subscribe(() => {
    if ($admin.game) {
      goto(`/admin/games/${$admin.game?.id}/statistics`)
    }
  })

  onDestroy(() => {
    unsubscribe()
  })
</script>
