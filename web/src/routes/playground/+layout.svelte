<script lang="ts">
  import { goto } from '$app/navigation'
  import { page } from '$app/stores'
  import { getChallengeList, getTagList } from '$lib/api/challenge'
  import { getGameList, getGameSelfSubmission } from '$lib/api/game'
  import PlaygroundSidebar from '$lib/blocks/PlaygroundSidebar.svelte'
  import { i18n } from '$lib/i18n'
  import type { Challenge, Tag } from '$lib/models/challenge'
  import type { Game } from '$lib/models/game'
  import type { Submission } from '$lib/models/submission'
  import { Permission } from '$lib/models/user'
  import { showMessage } from '$lib/stores/toast'
  import { user } from '$lib/stores/user'
  import type { AxiosError } from 'axios'
  import { onDestroy } from 'svelte'
  import { quintOut } from 'svelte/easing'
  import { fly } from 'svelte/transition'
  import { game } from '$lib/stores/game'

  if (!$user.isLoggedIn) {
    goto(`/account/login?redirect=${$page.url.pathname}`).then(() => {
      showMessage('warning', $i18n.t('permissions.beLoggedInToView'), 5000)
    })
  } else if (!$user.permissions.find((p) => p === Permission.Verified)) {
    goto('/account/profile').then(() => {
      showMessage('warning', $i18n.t('permissions.beVerifiedToView'), 5000)
    })
  }

  let screenWidth: number
  let toggleSidebar = false
  $: showSidebar = screenWidth > 1024 // lg

  let playgrounds: Game[] = []
  let games: Game[] = []
  let tags: Tag[] = []

  let activeGameId: number | null = null
  let activeGameChallenges: Challenge[] = []
  let playgroundTotalPages: number = 0
  let playgroundPage: number = 1
  let playgroundPageSize = 10
  let gameTotalPages: number = 0
  let gamePageSize = 10
  let gamePage: number = 1
  let challengeTotalPages: number = 0
  let challengePageSize = 200
  let challengePage: number = 1
  let selfSubmissions: Submission[] = []
  $: mayHaveMoreChallenges = challengePage < challengeTotalPages
  $: mayHaveMoreGames = gamePage < gameTotalPages
  $: mayHaveMorePlaygrounds = playgroundPage < playgroundTotalPages

  getGameList(playgroundPage, playgroundPageSize, false)
    .then((res) => {
      playgrounds = res.games
      playgroundTotalPages = res.total
    })
    .catch((err) => {
      showMessage(
        'error',
        `${$i18n.t('playground.fetchPlaygroundFailed')}: ${(err as AxiosError).response?.data}`,
        5000
      )
    })

  getGameList(gamePage, gamePageSize, true)
    .then((res) => {
      games = res.games
      gameTotalPages = res.total
    })
    .catch((err) => {
      showMessage('error', `${$i18n.t('playground.fetchGamesFailed')}: ${(err as AxiosError).response?.data}`, 5000)
    })

  getTagList()
    .then((res) => {
      tags = res
    })
    .catch((err) => {
      showMessage('error', `${$i18n.t('playground.fetchTagsFailed')}: ${(err as AxiosError).response?.data}`, 5000)
    })

  function getMoreGames() {
    if (mayHaveMoreGames) {
      getGameList(++gamePage, gamePageSize, true)
        .then((res) => {
          games = games.concat(res.games)
          gameTotalPages = res.total
        })
        .catch((err) => {
          showMessage('error', `${$i18n.t('playground.noMoreGames')}: ${(err as AxiosError).response?.data}`, 5000)
        })
    }
  }

  function getMorePlaygrounds() {
    if (mayHaveMorePlaygrounds) {
      getGameList(++playgroundPage, playgroundPageSize, false)
        .then((res) => {
          playgrounds = playgrounds.concat(res.games)
          playgroundTotalPages = res.total
        })
        .catch((err) => {
          showMessage(
            'error',
            `${$i18n.t('playground.noMorePlaygrounds')}: ${(err as AxiosError).response?.data}`,
            5000
          )
        })
    }
  }

  function getMoreChallenges() {
    if (mayHaveMoreChallenges && activeGameId) {
      getChallengeList(activeGameId, ++challengePage, challengePageSize)
        .then((res) => {
          activeGameChallenges = activeGameChallenges.concat(res.challenges)
          challengeTotalPages = res.total
        })
        .catch((err) => {
          showMessage('error', `${$i18n.t('playground.noMoreChallenges')}: ${(err as AxiosError).response?.data}`, 5000)
        })
    }
  }

  const unsubscribe = page.subscribe((value) => {
    let newActiveGameId = value.params.game ? parseInt(value.params.game) || null : null
    // console.log(activeGameId)
    if (newActiveGameId && newActiveGameId !== activeGameId) {
      activeGameChallenges = []
      activeGameId = newActiveGameId

      getGameSelfSubmission(activeGameId)
        .then((res) => {
          selfSubmissions = res
        })
        .catch((err) => {
          showMessage(
            'error',
            `${$i18n.t('playground.fetchSelfSubmissionsFailed')}: ${(err as AxiosError).response?.data}`,
            5000
          )
        })
      getChallengeList(activeGameId, challengePage, challengePageSize)
        .then((res) => {
          // console.log(res)
          activeGameChallenges = res.challenges
          $game.challenges = res.challenges
          challengeTotalPages = res.total
        })
        .catch((err) => {
          showMessage(
            'error',
            `${$i18n.t('playground.fetchChallengesFailed')}: ${(err as AxiosError).response?.data}`,
            5000
          )
        })
    }
  })
  onDestroy(() => {
    unsubscribe()
    $game.challenges = []
  })
</script>

<svelte:window bind:innerWidth={screenWidth} />

<div class="flex-1 flex flex-row overflow-x-hidden">
  {#if showSidebar}
    <div
      class="w-1/5 h-[calc(100vh_-_4rem)] flex-shrink-0 min-w-[24rem] max-w-[32rem] bg-base-100/60 backdrop-blur border-r border-r-base-content/10 overflow-hidden"
    >
      <PlaygroundSidebar
        {games}
        {playgrounds}
        {selfSubmissions}
        {activeGameChallenges}
        {tags}
        {mayHaveMoreChallenges}
        {mayHaveMoreGames}
        {mayHaveMorePlaygrounds}
        on:loadMoreChallenges={getMoreChallenges}
        on:loadMoreGames={getMoreGames}
        on:loadMorePlaygrounds={getMorePlaygrounds}
      />
    </div>
  {:else}
    <label
      class="btn bg-base-content/5 border-none backdrop-blur btn-square btn-lg fixed right-6 bottom-6 z-10 swap swap-rotate"
    >
      <input
        type="checkbox"
        on:click={() => {
          toggleSidebar = !toggleSidebar
        }}
      />
      <span class="swap-off icon-[fluent--navigation-16-regular] fill-current w-6 h-6"></span>
      <span class="swap-on icon-[fluent--dismiss-16-regular] fill-current w-6 h-6"></span>
    </label>
  {/if}
  <slot />
  {#if toggleSidebar && !showSidebar}
    <div
      class="fixed w-full max-w-[24rem] h-[calc(100vh_-_4rem)] overflow-hidden backdrop-blur bg-base-100/40 border-r border-r-base-content/10"
      transition:fly={{ delay: 100, duration: 300, x: -256, y: 0, opacity: 0, easing: quintOut }}
    >
      <PlaygroundSidebar
        {games}
        {playgrounds}
        {selfSubmissions}
        {activeGameChallenges}
        {tags}
        {mayHaveMoreChallenges}
        {mayHaveMoreGames}
        {mayHaveMorePlaygrounds}
        on:loadMoreChallenges={getMoreChallenges}
        on:loadMoreGames={getMoreGames}
        on:loadMorePlaygrounds={getMorePlaygrounds}
      />
    </div>
  {/if}
</div>
