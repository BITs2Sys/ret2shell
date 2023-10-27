<script lang="ts">
  import { page } from '$app/stores'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'
  import type { Challenge, Tag } from '$lib/models/challenge'
  import type { Game } from '$lib/models/game'
  import type { Submission } from '$lib/models/submission'
  import { theme } from '$lib/stores/theme'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte'
  import { createEventDispatcher, onDestroy } from 'svelte'

  export let playgrounds: Game[]
  export let games: Game[]
  $: filteredGames = games.filter((item) => new Date(item.archive_time * 1000) <= new Date())
  let activeGameId: number | null = null
  $: activeGame = filteredGames.concat(playgrounds).find((item) => item.id === activeGameId)
  export let activeGameChallenges: Challenge[]
  export let tags: Tag[]
  export let selfSubmissions: Submission[]
  export let mayHaveMorePlaygrounds: boolean
  export let mayHaveMoreGames: boolean
  export let mayHaveMoreChallenges: boolean
  export let loading = false
  export let loadMoreChallengesCallback: () => void
  export let loadMoreGamesCallback: () => void
  export let loadMorePlaygroundsCallback: () => void

  let tagsChallengesRecord: Record<number, Challenge[]> = {}

  $: {
    tagsChallengesRecord = {}
    tags.forEach((tag) => {
      tagsChallengesRecord[tag.id] = activeGameChallenges.filter((challenge) => challenge.tag_id === tag.id)
    })
  }

  let tagExpandedRecord: Record<number, boolean> = {}

  function handleTagExpand(tagId: number) {
    tagExpandedRecord[tagId] = !tagExpandedRecord[tagId]
    tagExpandedRecord = tagExpandedRecord
  }

  const unsubscribe = page.subscribe((value) => {
    activeGameId = value.params.game ? parseInt(value.params.game) || null : null
    // console.log(activeGameId)
  })
  onDestroy(unsubscribe)
</script>

<div class="w-full h-full flex flex-col">
  <h1
    class="font-bold flex flex-row justify-center items-center h-16 space-x-2 sticky top-0 border-b border-b-base-content/5 z-10"
  >
    <span class="icon-[fluent--dumbbell-20-regular] w-5 h-5" />
    {#if activeGame}
      <span>
        {activeGame.name}
      </span>
    {:else}
      <span>
        {$i18n.t('playground.sideToc')}
      </span>
    {/if}
  </h1>
  <OverlayScrollbarsComponent
    options={{
      scrollbars: { theme: $theme.colorScheme === 'light' ? 'os-theme-dark' : 'os-theme-light', autoHide: 'scroll' },
    }}
    class="w-full flex-1 flex flex-col relative print:hidden"
    defer
  >
    {#if activeGameId}
      <div class="flex-1 flex-col p-4 overflow-x-hidden space-y-2">
        <RxLink ghost class="w-full" justify="start" href="/playground">
          <span class="icon-[fluent--arrow-hook-down-left-20-regular] w-5 h-5" />
          <span class="flex-1 text-start text-ellipsis overflow-hidden whitespace-nowrap">
            {$i18n.t('playground.returnToList')}
          </span>
          <span class="icon-[fluent--chevron-down-20-regular] w-5 h-5" />
        </RxLink>
        <ul class="relative flex flex-col space-y-2">
          {#each tags as tag}
            {#if tagsChallengesRecord[tag.id] && tagsChallengesRecord[tag.id].length > 0}
              <li>
                <RxButton
                  class="w-full"
                  justify="start"
                  ghost
                  on:click={() => {
                    handleTagExpand(tag.id)
                  }}
                >
                  <span class="icon-[fluent--tag-20-regular] w-5 h-5" />
                  <span class="flex-1 text-start text-ellipsis overflow-hidden whitespace-nowrap">{tag.name}</span>
                  <span
                    class="icon-[fluent--chevron-down-20-regular] w-5 h-5 flex-shrink-0 transition-all {tagExpandedRecord[
                      tag.id
                    ]
                      ? ' rotate-0'
                      : '-rotate-90'}"
                  />
                </RxButton>
                {#if tagExpandedRecord[tag.id]}
                  <ul
                    class="pl-4 relative before:border-l-2 before:absolute before:h-full before:border-l-base-content/10 overflow-x-hidden flex flex-col space-y-2 mt-2"
                  >
                    {#each tagsChallengesRecord[tag.id] as chal}
                      <RxLink ghost class="w-full" justify="start" href={`/playground/${activeGameId}#${chal.id}`}>
                        {#if selfSubmissions.find((item) => item.challenge_id === chal.id)}
                          <span class="icon-[fluent--checkmark-circle-20-regular] text-success w-5 h-5" />
                        {:else}
                          <span class="icon-[fluent--question-circle-20-regular] opacity-60 w-5 h-5" />
                        {/if}
                        <span
                          class={`flex-1 text-start text-ellipsis overflow-hidden whitespace-nowrap ${
                            selfSubmissions.find((item) => item.challenge_id === chal.id)
                              ? 'line-through opacity-60'
                              : ''
                          }`}
                        >
                          {chal.name}
                        </span>
                      </RxLink>
                    {/each}
                  </ul>
                {/if}
              </li>
            {/if}
          {/each}
          {#if mayHaveMoreChallenges}
            <RxButton class="w-full opacity-60" ghost on:click={loadMoreChallengesCallback}>
              {$i18n.t('playground.loadMore')}
            </RxButton>
          {/if}
        </ul>
      </div>
    {:else}
      <div class="flex-1 flex-col">
        {#if loading}
          <div class="flex flex-row justify-center items-center h-16 space-x-2">
            <span class="loading loading-spinner loading-sm" />
            <span class="text-base">{$i18n.t('playground.fetchingList')}</span>
          </div>
        {:else if playgrounds.length === 0 && filteredGames.length === 0}
          <p class="text-base font-semibold p-4 opacity-60 text-center">{$i18n.t('playground.emptyCategory')}</p>
        {/if}
        {#if playgrounds.length > 0}
          <h2 class="text-base font-semibold p-4 pb-0 flex flex-row items-center justify-center space-x-2 opacity-60">
            <span class="icon-[fluent--beaker-20-regular] w-5 h-5" />
            <span>
              {$i18n.t('playground.persistTitle')}
            </span>
          </h2>
          <div class="flex flex-col p-4 space-y-2">
            {#each playgrounds as item}
              <RxLink class="w-full" justify="start" ghost href={`/playground/${item.id}`}>
                <span class="icon-[fluent--bookmark-20-regular] w-5 h-5" />
                <span class="flex-1 text-start">{item.name}</span>
                <span class="icon-[fluent--chevron-right-20-regular] w-5 h-5" />
              </RxLink>
            {/each}
            {#if mayHaveMorePlaygrounds}
              <RxButton class="w-full opacity-60" ghost on:click={loadMorePlaygroundsCallback}>
                {$i18n.t('playground.loadMore')}
              </RxButton>
            {/if}
          </div>
        {/if}
        {#if filteredGames.length > 0}
          <h2 class="text-base font-semibold p-4 pb-0 flex flex-row items-center justify-center space-x-2 opacity-60">
            <span class="icon-[fluent--flag-20-regular] w-5 h-5" />
            <span>
              {$i18n.t('playground.gamesTitle')}
            </span>
          </h2>
          <div class="flex flex-col p-4 space-y-2">
            {#each filteredGames as item}
              <RxLink class="w-full" justify="start" ghost href={`/playground/${item.id}`}>
                <span class="icon-[fluent--archive-20-regular] w-5 h-5" />
                <span class="flex-1 text-start">{item.name}</span>
                <span class="icon-[fluent--chevron-right-20-regular] w-5 h-5" />
              </RxLink>
            {/each}
            {#if mayHaveMoreGames}
              <RxButton class="w-full opacity-60" ghost on:click={loadMoreGamesCallback}>
                {$i18n.t('playground.loadMore')}
              </RxButton>
            {/if}
          </div>
        {/if}
      </div>
    {/if}
  </OverlayScrollbarsComponent>
</div>
