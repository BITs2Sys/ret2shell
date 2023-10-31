<script lang="ts">
  import { platform } from '$lib/stores/platform'
  import { i18n } from '$lib/i18n'
  import RxImage from '$lib/components/RxImage.svelte'
  import type { Game } from '$lib/models/game'
  import Bg from '$lib/assets/background-lines.svg'
  import Logo from '$lib/assets/logo.svg'
  import LogoGray from '$lib/assets/logo-gray.svg'
  import RxTag from '$lib/components/RxTag.svelte'
  import { getGame, getGameList } from '$lib/api/game'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { blur } from 'svelte/transition'
  import RxLink from '$lib/components/RxLink.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import { page } from '$app/stores'
  import { onDestroy } from 'svelte'
  import { goto } from '$app/navigation'
  import { game } from '$lib/stores/game'
  import LogoAnimate from '$lib/assets/logo-animate.svelte'

  let games: Game[] = []
  $: hasCover = $game.cached?.cover_path !== null
  let loading = true
  const perPage = 10
  let currentPage = 1
  let total = 0
  $: hasNextPage = total > currentPage
  $: hasPrevPage = currentPage > 1

  let isStarted = false
  let isEnded = false
  let unknownState = true
  $: {
    if ($game.cached) {
      unknownState = false
      let startTime = new Date($game.cached.start_time * 1000)
      let endTime = new Date($game.cached.end_time * 1000)
      let now = new Date()
      isStarted = now >= startTime
      isEnded = now >= endTime
    } else {
      unknownState = true
    }
  }

  async function fetchPage() {
    loading = true
    return getGameList(currentPage, perPage, true)
      .then((res) => {
        games = res.games
        total = res.total
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('games.fetchGamesError')}: ${(err as AxiosError).response?.data}`, 5000)
      })
      .finally(() => {
        loading = false
      })
  }

  function fetchPrevPage() {
    if (hasPrevPage) {
      currentPage--
      fetchPage()
    }
  }

  function fetchNextPage() {
    if (hasNextPage) {
      currentPage++
      fetchPage()
    }
  }

  fetchPage().then(() => {
    if (games.length !== 0 && $game.cached === null) {
      $game.cached = games[0]
    }
  })

  let unsubscribe = page.subscribe((value) => {
    if (!value.url.hash || value.url.hash === '#' || value.url.hash.trim() === '') {
      if (games.length !== 0) {
        $game.cached = games[0]
      }
      return
    }
    let gameId = value.url.hash ? parseInt(value.url.hash.slice(1)) || null : null
    if (gameId) {
      $game.cached = games.find((item) => item.id === gameId) || null
      if (!$game.cached) loading = true
      getGame(gameId)
        .then((res) => {
          if (!res.host_as_game) {
            goto(`/playground/${res.id}`, { replaceState: true })
            return
          }
          $game.cached = res
        })
        .catch(() => {
          $game.cached = null
        })
        .finally(() => {
          loading = false
        })
    } else {
      if (games.length !== 0) {
        $game.cached = games[0]
      }
      showMessage('error', $i18n.t('games.gameIdInvalid'), 5000)
    }
  })

  onDestroy(() => {
    unsubscribe()
    $game.cached = null
  })
</script>

<svelte:head><title>{$i18n.t('games.title')} - {$platform.name}</title></svelte:head>
<div class="flex-1 flex flex-row p-4 lg:p-6 justify-center">
  <div
    class={`max-w-[32rem] flex-shrink-0 hidden lg:flex flex-col pt-16 pl-16 pr-8 space-y-1 ${
      games.length > 0 ? 'w-1/4 min-w-[24rem]' : 'w-0'
    }`}
  >
    <RxButton class="w-full !bg-transparent" ghost disabled={!hasPrevPage} on:click={fetchPrevPage}>
      <span class="icon-[fluent--chevron-double-up-20-regular] w-5 h-5" />
    </RxButton>
    <div class="divider m-0"></div>
    {#each games as item}
      <RxLink href={`/games#${item.id}`} class="w-full" ghost justify="start">
        <span
          class={`w-5 h-5 ${
            $game.cached?.id === item.id
              ? 'icon-[fluent--flag-20-filled] text-primary'
              : 'icon-[fluent--flag-20-regular]'
          }`}
        />
        <span
          class={`text-base flex-1 text-start ${
            $game.cached?.id === item.id ? 'font-bold text-primary' : 'font-normal'
          }`}
        >
          {item.name}
        </span>
        <span
          class={`icon-[fluent--chevron-double-right-20-regular] w-5 h-5 ${
            $game.cached?.id === item.id && 'text-primary'
          }`}
        />
      </RxLink>
    {/each}
    <div class="divider m-0"></div>
    <RxButton class="w-full !bg-transparent" ghost disabled={!hasNextPage} on:click={fetchNextPage}>
      <span class="icon-[fluent--chevron-double-down-20-regular] w-5 h-5" />
    </RxButton>
  </div>
  <div class="flex-1 flex flex-col p-3 lg:p-6 items-center lg:justify-center">
    <div
      class="w-full lg:w-3/4 h-auto rounded-box bg-base-content/5 backdrop-blur shadow-lg aspect-video transition-all lg:-translate-x-[4rem] rounded-b-none lg:rounded-b-box overflow-clip relative"
    >
      {#if hasCover && games.length !== 0}
        <RxImage class="w-full h-full relative" src={$game.cached?.cover_path} {loading}></RxImage>
      {:else}
        <RxImage class="w-full h-full relative" src={Bg} {loading}>
          <div class="absolute top-0 left-0 w-full h-full flex flex-row justify-center items-center">
            {#if games.length !== 0 && !loading}
              <img alt="CTF" src={Logo} width="128" height="128" transition:blur={{ amount: 20, duration: 300 }} />
            {:else if !loading}
              <img alt="CTF" src={LogoGray} width="128" height="128" transition:blur={{ amount: 20, duration: 300 }} />
            {/if}
          </div>
        </RxImage>
      {/if}
      {#if !loading}
        <RxTag
          class="absolute top-4 right-4 !bg-neutral/60"
          level={isStarted && !isEnded ? 'success' : isEnded ? 'warning' : unknownState ? 'error' : 'info'}
        >
          <span class="text-base font-bold">
            {#if isStarted && !isEnded}
              {$i18n.t('games.started')}
            {:else if isEnded}
              {$i18n.t('games.ended')}
            {:else if unknownState}
              {$i18n.t('games.unknown')}
            {:else}
              {$i18n.t('games.notStarted')}
            {/if}
          </span>
        </RxTag>
      {/if}
    </div>
    <div
      class="h-64 sm:h-40 w-full lg:w-3/4 lg:translate-x-[4rem] lg:-translate-y-[3rem] flex flex-row justify-end transition-all"
    >
      <a
        class="w-full lg:w-2/3 rounded-box bg-neutral/80 backdrop-blur rounded-t-none lg:rounded-t-box overflow-clip flex flex-row relative border border-base-content/5"
        href={$game.cached ? `/games/${$game.cached.id}` : '#'}
      >
        <div class="flex flex-col p-6 space-y-4 sm:space-y-2 flex-1 sm:flex-none">
          <h1 class="text-2xl font-bold flex-1 flex flex-row space-x-4 items-center">
            <LogoAnimate width={48} height={48} />
            <div class="flex flex-col">
              <span>{$game.cached?.name || $i18n.t('games.noGameTitle')}</span>
              {#if games.length > 0}
                <span class="font-bold text-base opacity-60">
                  {$game.cached?.brief ? $game.cached.brief : $i18n.t('games.noBrief')}
                </span>
              {/if}
            </div>
          </h1>
          <div class="flex-1 flex flex-col sm:flex-row space-y-2 sm:space-y-0 sm:space-x-4 items-center justify-center">
            {#if $game.cached}
              <RxTag
                label={new Date($game.cached.start_time * 1000).toLocaleDateString('default', {
                  year: 'numeric',
                  day: '2-digit',
                  month: '2-digit',
                })}
              />
              <span class="text-primary transform transition-all rotate-90 sm:rotate-0">=&gt;</span>
              <RxTag
                label={new Date($game.cached.end_time * 1000).toLocaleDateString('default', {
                  year: 'numeric',
                  day: '2-digit',
                  month: '2-digit',
                })}
              />
            {/if}
          </div>
        </div>
        <div class="md:flex flex-1 flex-row p-8 justify-end items-center hidden">
          <span class="icon-[fluent--chevron-double-right-16-regular] text-primary w-8 h-8" />
        </div>
        {#if loading}
          <div
            class="absolute top-0 left-0 w-full h-full flex flex-col justify-center items-center backdrop-blur"
            transition:blur={{ amount: 20, duration: 300 }}
          >
            <span class="loading" />
          </div>
        {/if}
      </a>
    </div>
  </div>
</div>
