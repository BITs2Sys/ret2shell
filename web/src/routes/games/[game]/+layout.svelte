<script lang="ts">
  import { goto } from '$app/navigation'
  import { page } from '$app/stores'
  import { getGame } from '$lib/api/game'
  import Background from '$lib/blocks/Background.svelte'
  import BgBlur from '$lib/assets/imgs/bg-blur.webp'
  import { i18n } from '$lib/i18n'
  import { game, refreshTeam } from '$lib/stores/game'
  import { platform } from '$lib/stores/platform'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { onDestroy, onMount } from 'svelte'
  import { blur, fly } from 'svelte/transition'
  import { user } from '$lib/stores/user'
  import { Permission } from '$lib/models/user'
  import LogoAnimate from '$lib/assets/animates/logo-animate.svelte'
  import '$lib/styles/transitions.scss'

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
        goto('/errors/404', { replaceState: true })
      } else if ($game.current?.id !== gameId) {
        getGame(gameId)
          .then((res) => {
            game.update((value) => {
              value.current = res
              value.showGameNav = true
              return value
            })
            refreshTeam()
          })
          .catch((err) => {
            if ($user.permissions.find((p) => p === Permission.Verified)) {
              goto(`/errors/${err.response?.status || 500}`, { replaceState: true }).then(() => {
                showMessage('error', `${$i18n.t('games.fetchGameError')}: ${(err as AxiosError).response?.data}`, 5000)
              })
            } else {
              goto('/account/profile', { replaceState: true }).then(() => {
                showMessage('warning', $i18n.t('games.verifyFirst'), 5000)
              })
            }
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
      value.team = null
      value.showGameNav = false
      value.challenges = []
      return value
    })
  })
</script>

<svelte:head><title>{$game.current?.name} - {$platform.name}</title></svelte:head>

<slot />

{#if loading || delayedLoading}
  <div class="fixed -left-1 -right-1 -top-1 -bottom-1 z-50">
    <img
      src={BgBlur}
      alt=""
      class="w-full h-full object-fill"
      on:contextmenu={() => {
        return false
      }}
    />
    <div class="fixed left-0 right-0 top-0 bottom-0 bg-base-100/80 backdrop-blur"></div>
  </div>
  <div
    class="w-screen h-screen fixed z-50 flex flex-col items-center justify-center space-y-8"
    transition:blur={{ amount: 20, duration: 300 }}
  >
    <Background showBlurBg={false} />
    <div in:fly={{ x: 0, y: -64, duration: 1500 }}>
      <LogoAnimate width={128} height={128} />
    </div>
    <div class="flex flex-col space-y-8 items-center justify-center">
      <div in:fly={{ x: 0, y: 32, duration: 1500, opacity: 0, delay: 300 }}>
        <p class="text-2xl opacity-60 font-bold expand-then-shrink">WELCOME TO</p>
      </div>
      <h1 class="text-3xl font-bold" in:fly={{ x: 0, y: 32, duration: 1500, opacity: 0, delay: 700 }}>
        {$game.current?.name || $game.cached?.name || $i18n.t('games.loading')}
      </h1>
    </div>
    <div class="h-32"></div>
  </div>
{/if}
