<script lang="ts">
  import { goto } from '$app/navigation'
  import { page } from '$app/stores'
  import { getChallengeList, getTagList } from '$lib/api/challenge'
  import { getGameList, getGameSelfSolves } from '$lib/api/game'
  import Sidebar from './Sidebar.svelte'
  import { i18n } from '$lib/i18n'
  import type { Tag } from '$lib/models/challenge'
  import type { Game } from '$lib/models/game'
  import { Permission } from '$lib/models/user'
  import { showMessage } from '$lib/stores/toast'
  import { user } from '$lib/stores/user'
  import type { AxiosError } from 'axios'
  import { onDestroy } from 'svelte'
  import { game } from '$lib/stores/game'
  import SidebarLayout from '$lib/blocks/SidebarLayout.svelte'

  if (!$user.isLoggedIn) {
    goto(`/account/login?redirect=${$page.url.pathname}`, { replaceState: true }).then(() => {
      showMessage('warning', $i18n.t('permissions.beLoggedInToView'), 5000)
    })
  } else if (!$user.permissions.find((p) => p === Permission.Verified)) {
    goto('/account/profile', { replaceState: true }).then(() => {
      showMessage('warning', $i18n.t('permissions.beVerifiedToView'), 5000)
    })
  }

  let playgrounds: Game[] = []
  let games: Game[] = []
  let tags: Tag[] = []

  let activeGameId: number | null = null
  let playgroundTotalPages: number = 0
  let playgroundPage: number = 1
  let playgroundPageSize = 10
  let gameTotalPages: number = 0
  let gamePageSize = 10
  let gamePage: number = 1
  let challengeTotalPages: number = 0
  let challengePageSize = 200
  let challengePage: number = 1
  $: mayHaveMoreChallenges = challengePage < challengeTotalPages
  $: mayHaveMoreGames = gamePage < gameTotalPages
  $: mayHaveMorePlaygrounds = playgroundPage < playgroundTotalPages
  let loading = true

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
    .finally(() => {
      loading = false
    })

  getGameList(gamePage, gamePageSize, true)
    .then((res) => {
      games = res.games
      gameTotalPages = res.total
    })
    .catch((err) => {
      showMessage('error', `${$i18n.t('playground.fetchGamesFailed')}: ${(err as AxiosError).response?.data}`, 5000)
    })
    .finally(() => {
      loading = false
    })

  getTagList()
    .then((res) => {
      tags = res.toSorted((a, b) => (a.name > b.name ? 1 : a.name === b.name ? 0 : -1))
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
          $game.challenges = $game.challenges
            .concat(res.challenges)
            .toSorted((a, b) =>
              a.current_score - b.current_score === 0
                ? a.name > b.name
                  ? 1
                  : a.name === b.name
                    ? 0
                    : -1
                : a.current_score - b.current_score
            )
          challengeTotalPages = res.total
        })
        .catch((err) => {
          showMessage('error', `${$i18n.t('playground.noMoreChallenges')}: ${(err as AxiosError).response?.data}`, 5000)
        })
    }
  }

  function getSelfSubmissions() {
    if (activeGameId)
      getGameSelfSolves(activeGameId)
        .then((res) => {
          $game.submissions = res
        })
        .catch((err) => {
          showMessage(
            'error',
            `${$i18n.t('playground.fetchSelfSubmissionsFailed')}: ${(err as AxiosError).response?.data}`,
            5000
          )
        })
  }

  const unsubscribe = page.subscribe((value) => {
    let newActiveGameId = value.params.game ? parseInt(value.params.game) || null : null
    // console.log(activeGameId)
    if (newActiveGameId && newActiveGameId !== activeGameId) {
      $game.challenges = []
      activeGameId = newActiveGameId
      getChallengeList(activeGameId, challengePage, challengePageSize)
        .then((res) => {
          $game.challenges = res.challenges.toSorted((a, b) =>
            a.current_score - b.current_score === 0
              ? a.name > b.name
                ? 1
                : a.name === b.name
                  ? 0
                  : -1
              : a.current_score - b.current_score
          )
          challengeTotalPages = res.total
        })
        .catch((err) => {
          showMessage(
            'error',
            `${$i18n.t('playground.fetchChallengesFailed')}: ${(err as AxiosError).response?.data}`,
            5000
          )
        })
      getSelfSubmissions()
    }
  })
  onDestroy(() => {
    unsubscribe()
    $game.challenges = []
  })
</script>

<SidebarLayout
  leftSidebar={Sidebar}
  leftProps={{
    loading,
    games,
    playgrounds,
    selfSubmissions: $game.submissions,
    activeGameChallenges: $game.challenges,
    tags,
    mayHaveMoreChallenges,
    mayHaveMoreGames,
    mayHaveMorePlaygrounds,
    loadMoreChallengesCallback: getMoreChallenges,
    loadMoreGamesCallback: getMoreGames,
    loadMorePlaygroundsCallback: getMorePlaygrounds,
  }}
>
  <slot />
</SidebarLayout>
