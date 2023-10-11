<script lang="ts">
  import { goto } from '$app/navigation'
  import { page } from '$app/stores'
  import { getGame } from '$lib/api/game'
  import { i18n } from '$lib/i18n'
  import { admin, refreshAdminRoute } from '$lib/stores/admin'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { onDestroy, onMount } from 'svelte'
  import Sidebar from './Sidebar.svelte'

  const gameId = parseInt($page.params.game)
  if (isNaN(gameId)) {
    goto('/errors/404')
  }

  getGame(gameId)
    .then((game) => {
      $admin.game = game
      refreshAdminRoute($page.url.pathname)
    })
    .catch((error) => {
      goto('/errors/404').then(() => {
        showMessage('error', $i18n.t('games.fetchGameError') + ': ' + (error as AxiosError).response?.data, 5000)
      })
    })

  onMount(() => {
    $admin.secondLevelComponent = Sidebar
  })

  onDestroy(() => {
    $admin.game = null
    $admin.secondLevelComponent = null
  })
</script>

<slot />
