<script lang="ts">
  import { goto } from '$app/navigation'
  import { page } from '$app/stores'
  import { getGame } from '$lib/api/game'
  import Background from '$lib/blocks/Background.svelte'
  import { i18n } from '$lib/i18n'
  import { game } from '$lib/stores/game'
  import { platform } from '$lib/stores/platform'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { onDestroy, onMount } from 'svelte'
  import { blur } from 'svelte/transition'
  import Logo from '$lib/assets/logo.svg'

  let loading = false
  let delayedLoading = false
  setTimeout(() => {
    delayedLoading = false
  }, 2000)

  onMount(() => {
    loading = true
    delayedLoading = true
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
  })

  onDestroy(() => {
    game.update((value) => {
      value.current = null
      value.cached = null
      return value
    })
  })
</script>

<svelte:head><title>{$game.current?.name} - {$platform.name}</title></svelte:head>

<slot />

{#if loading || delayedLoading}
  <div class="w-screen h-screen fixed bg-base-100 z-50 flex flex-col items-center justify-center">
    <p class="text-3xl opacity-60 font-bold" transition:blur={{ amount: 20 }}>W E L C O M E&nbsp;&nbsp;&nbsp;T O</p>
    <div class="h-64"></div>
  </div>
  <div
    class="w-screen h-screen fixed z-50 flex flex-col items-center justify-center space-y-8"
    transition:blur={{ amount: 20 }}
  >
    <Background />
    <img src={Logo} alt="Ret2Shell" width="128" height="128" />
    <h1 class="text-3xl font-bold">{$game.current?.name || $game.cached?.name || $i18n.t('games.loading')}</h1>
  </div>
{/if}
