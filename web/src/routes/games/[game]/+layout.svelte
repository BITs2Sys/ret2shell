<script lang="ts">
  import { goto } from '$app/navigation'
  import { page } from '$app/stores'
  import { getGame } from '$lib/api/game'
  import { i18n } from '$lib/i18n'
  import { game } from '$lib/stores/game'
    import { platform } from '$lib/stores/platform'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'

  let loading = true
  if ($page.params.game) {
    let gameId = parseInt($page.params.game) || null
    if (!gameId) {
      showMessage('error', `${$i18n.t('games.invalidGameId')}: ${$page.params.game}`, 5000)
          goto('/errors/404')
    } else {
      getGame(gameId)
        .then((res) => {
          game.update((value) => {
            value.current = res
            return value
          })
        })
        .catch((err) => {
          showMessage('error', `${$i18n.t('games.fetchGameError')}: ${(err as AxiosError).response?.data}`, 5000)
          goto('/errors/500')
        })
        .finally(() => {
          loading = false
        })
    }
  }
</script>

<svelte:head><title>{$game.current?.name} - {$platform.name}</title></svelte:head>

<slot />
