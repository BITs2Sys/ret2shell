<script lang="ts">
  import { getChallengeList, getTagList } from '$lib/api/challenge'
  import { getGameSelfSubmission } from '$lib/api/game'
  import GameChallengeSidebar from '$lib/blocks/GameChallengeSidebar.svelte'
  import GameTeamSidebar from '$lib/blocks/GameTeamSidebar.svelte'
  import { i18n } from '$lib/i18n'
  import type { Challenge, Tag } from '$lib/models/challenge'
  import type { Submission } from '$lib/models/submission'
  import { game } from '$lib/stores/game'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { onDestroy, onMount } from 'svelte'
  import { quintOut } from 'svelte/easing'
  import { fly } from 'svelte/transition'

  let screenWidth: number
  let toggleSidebar = false
  let toggleTeamSidebar = false
  $: showSidebar = screenWidth > 1024 // lg
  $: showTeamSidebar = screenWidth > 1280 // lg

  let challengeTotalPages: number = 0
  let challengePageSize = 200
  let challengePage: number = 1
  let selfSubmissions: Submission[] = []
  let challenges: Challenge[] = []
  $: mayHaveMoreChallenges = challengePage < challengeTotalPages
  let tags: Tag[] = []

  function getChallenges() {
    if ($game.current?.id) {
      getChallengeList($game.current?.id, challengePage, challengePageSize)
        .then((res) => {
          challenges = challenges.concat(res.challenges)
          challengeTotalPages = res.total
        })
        .catch((err) => {
          showMessage('error', `${$i18n.t('playground.noMoreChallenges')}: ${(err as AxiosError).response?.data}`, 5000)
        })
    }
  }

  function getMoreChallenges() {
    challengePage++
    if (challengePage <= challengeTotalPages) getChallenges()
  }

  function getSelfSubmissions() {
    if ($game.current?.id) {
      getGameSelfSubmission($game.current?.id)
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
    }
  }

  onMount(() => {
    getTagList()
      .then((res) => {
        tags = res
      })
      .catch((err) => {
        showMessage('error', `${$i18n.t('playground.fetchTagsFailed')}: ${(err as AxiosError).response?.data}`, 5000)
      })
  })

  let gameUnsubscribe = game.subscribe((value) => {
    if (value.current?.id) {
      challengePage = 1
      challenges = []
      getChallenges()
      getSelfSubmissions()
    }
  })

  onDestroy(() => {
    gameUnsubscribe()
  })
</script>

<svelte:window bind:innerWidth={screenWidth} />

<div class="flex-1 flex flex-row overflow-x-hidden">
  {#if showSidebar}
    <div
      class="w-1/5 h-[calc(100vh_-_4rem)] flex-shrink-0 min-w-[24rem] max-w-[32rem] bg-base-100/60 backdrop-blur border-r border-r-base-content/10 overflow-hidden"
    >
      <GameChallengeSidebar
        {selfSubmissions}
        {challenges}
        {tags}
        {mayHaveMoreChallenges}
        on:loadMoreChallenges={getMoreChallenges}
      />
    </div>
  {:else}
    <label
      class="btn bg-base-content/5 border-none backdrop-blur btn-square btn-lg fixed left-6 bottom-6 z-10 swap swap-rotate"
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
  <div class="flex-1"></div>
  {#if showTeamSidebar}
    <div
      class="w-1/5 h-[calc(100vh_-_4rem)] flex-shrink-0 min-w-[24rem] max-w-[32rem] bg-base-100/60 backdrop-blur border-l border-l-base-content/10 overflow-hidden"
    >
      <GameTeamSidebar />
    </div>
  {:else}
    <label
      class="btn bg-base-content/5 border-none backdrop-blur btn-square btn-lg fixed right-6 bottom-6 z-10 swap swap-rotate"
    >
      <input
        type="checkbox"
        on:click={() => {
          toggleTeamSidebar = !toggleTeamSidebar
        }}
      />
      <span class="swap-off icon-[fluent--flag-16-regular] fill-current w-6 h-6"></span>
      <span class="swap-on icon-[fluent--dismiss-16-regular] fill-current w-6 h-6"></span>
    </label>
  {/if}
  {#if toggleTeamSidebar && !showTeamSidebar}
    <div
      class="fixed right-0 w-full max-w-[24rem] h-[calc(100vh_-_4rem)] overflow-hidden backdrop-blur bg-base-100/40 border-l border-l-base-content/10"
      transition:fly={{ delay: 100, duration: 300, x: 256, y: 0, opacity: 0, easing: quintOut }}
    >
      <GameTeamSidebar />
    </div>
  {/if}
  {#if toggleSidebar && !showSidebar}
    <div
      class="fixed left-0 w-full max-w-[24rem] h-[calc(100vh_-_4rem)] overflow-hidden backdrop-blur bg-base-100/40 border-r border-r-base-content/10"
      transition:fly={{ delay: 100, duration: 300, x: -256, y: 0, opacity: 0, easing: quintOut }}
    >
      <GameChallengeSidebar
        {selfSubmissions}
        {challenges}
        {tags}
        {mayHaveMoreChallenges}
        on:loadMoreChallenges={getMoreChallenges}
      />
    </div>
  {/if}
</div>
