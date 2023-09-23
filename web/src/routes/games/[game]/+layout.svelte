<script lang="ts">
  import { goto } from '$app/navigation'
  import { page } from '$app/stores'
  import { getGame, getSelfTeamInfo } from '$lib/api/game'
  import Background from '$lib/blocks/Background.svelte'
  import { i18n } from '$lib/i18n'
  import { game } from '$lib/stores/game'
  import { platform } from '$lib/stores/platform'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { onDestroy, onMount } from 'svelte'
  import { blur } from 'svelte/transition'
  import Logo from '$lib/assets/logo.svg'
  import { user } from '$lib/stores/user'
  import { Permission } from '$lib/models/user'

  let loading = false
  let delayedLoading = false
  setTimeout(() => {
    delayedLoading = false
  }, 3000)

  onMount(() => {
    loading = true
    delayedLoading = true
    if ($page.params.game) {
      let gameId = parseInt($page.params.game) || null
      if (!gameId) {
        showMessage('error', `${$i18n.t('games.invalidGameId')}: ${$page.params.game}`, 5000)
        goto('/errors/404')
      } else if ($game.current?.id !== gameId) {
        getGame(gameId)
          .then((res) => {
            game.update((value) => {
              value.current = res
              return value
            })
          })
          .catch((err) => {
            if ($user.permissions.find((p) => p === Permission.Verified)) {
              goto(`/errors/${err.response?.status || 500}`).then(() => {
                showMessage('error', `${$i18n.t('games.fetchGameError')}: ${(err as AxiosError).response?.data}`, 5000)
              })
            } else {
              goto('/account/profile').then(() => {
                showMessage('warning', $i18n.t('games.verifyFirst'), 5000)
              })
            }
          })
          .finally(() => {
            loading = false
          })

        getSelfTeamInfo(gameId)
          .then((res) => {
            game.update((value) => {
              value.team = res
              return value
            })
          })
          .catch((err) => {
            if ((err as AxiosError).response?.status !== 404) {
              showMessage('error', `${$i18n.t('games.fetchTeamError')}: ${(err as AxiosError).response?.data}`, 5000)
            }
            game.update((value) => {
              value.team = null
              return value
            })
          })
      }
    }
  })

  onDestroy(() => {
    game.update((value) => {
      value.current = null
      value.cached = null
      value.team = null
      return value
    })
  })
</script>

<svelte:head><title>{$game.current?.name} - {$platform.name}</title></svelte:head>

<slot />

{#if loading || delayedLoading}
  <div class="w-screen h-screen fixed bg-base-100 z-50 flex flex-col items-center justify-center"></div>
  <div
    class="w-screen h-screen fixed z-50 flex flex-col items-center justify-center space-y-8"
    transition:blur={{ amount: 20, duration: 300 }}
  >
    <Background />
    <img src={Logo} alt="Ret2Shell" width="128" height="128" />
    <p class="text-2xl opacity-60 font-bold" transition:blur={{ amount: 20, duration: 300 }}>
      W E L C O M E&nbsp;&nbsp;&nbsp;T O
    </p>
    <h1 class="text-3xl font-bold">{$game.current?.name || $game.cached?.name || $i18n.t('games.loading')}</h1>
    <div class="h-32"></div>
  </div>
{/if}
