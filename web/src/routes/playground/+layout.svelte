<script lang="ts">
  import PlaygroundSidebar from '$lib/blocks/PlaygroundSidebar.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import type { Challenge, Tag } from '$lib/models/challenge'
  import type { Game } from '$lib/models/game'
  import { quintOut } from 'svelte/easing'
  import { fly } from 'svelte/transition'

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
  $: mayHaveMoreChallenges = challengePage < challengeTotalPages
  $: mayHaveMoreGames = gamePage < gameTotalPages
  $: mayHaveMorePlaygrounds = playgroundPage < playgroundTotalPages
</script>

<svelte:window bind:innerWidth={screenWidth} />

<div class="flex-1 flex flex-row">
  {#if showSidebar}
    <div
      class="fixed w-1/5 h-[calc(100vh_-_4rem)] min-w-[24rem] max-w-[32rem] bg-base-100/60 backdrop-blur border-r border-r-base-content/10"
    >
      <PlaygroundSidebar
        {games}
        {playgrounds}
        {activeGameId}
        {activeGameChallenges}
        {tags}
        {mayHaveMoreChallenges}
        {mayHaveMoreGames}
        {mayHaveMorePlaygrounds}
      />
    </div>
    <div class="w-1/5 min-w-[24rem] max-w-[32rem] flex-shrink-0" />
  {:else}
    <RxButton
      size="lg"
      class="fixed right-6 bottom-6 z-10"
      on:click={() => {
        toggleSidebar = !toggleSidebar
      }}
    >
      <span class="icon-[fluent--navigation-16-regular] w-5 h-5" />
    </RxButton>
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
        {activeGameId}
        {activeGameChallenges}
        {tags}
        {mayHaveMoreChallenges}
        {mayHaveMoreGames}
        {mayHaveMorePlaygrounds}
      />
    </div>
    <RxButton
      size="lg"
      class="fixed right-6 bottom-6"
      on:click={() => {
        toggleSidebar = !toggleSidebar
      }}
    >
      <span class="icon-[fluent--dismiss-16-regular] w-5 h-5" />
    </RxButton>
  {/if}
</div>
