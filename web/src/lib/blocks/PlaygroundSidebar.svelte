<script lang="ts">
  import { page } from '$app/stores'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxInput from '$lib/components/RxInput.svelte'
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

  const dispatch = createEventDispatcher()

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

<OverlayScrollbarsComponent
  options={{
    scrollbars: { theme: $theme.colorScheme === 'light' ? 'os-theme-dark' : 'os-theme-light', autoHide: 'scroll' },
  }}
  class="w-full h-full flex flex-col relative print:hidden"
  defer
>
  <div class={`bg-base-100 border-b border-b-base-content/5 flex flex-col sticky top-0 h-32`}>
    <h1 class="font-bold flex flex-row justify-center items-center h-16 space-x-2">
      <span class="icon-[fluent--dumbbell-16-regular] w-6 h-6" />
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
    <div class="h-16 join px-4">
      <RxInput class="join-item" />
      <RxButton square class="join-item ml-0">
        <span class="icon-[fluent--filter-16-regular] w-6 h-6" />
      </RxButton>
    </div>
  </div>
  {#if activeGameId}
    <div class="flex-1 flex-col p-4 overflow-x-hidden">
      <RxLink ghost class="w-full" justify="start" href="/playground">
        <span class="icon-[fluent--arrow-hook-down-left-16-regular] w-6 h-6" />
        <span class="flex-1 text-start text-ellipsis overflow-hidden whitespace-nowrap">{$i18n.t('playground.returnToList')}</span>
        <span class="icon-[fluent--chevron-down-16-regular] w-5 h-5" />
      </RxLink>
      <ul class="relative">
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
                <span class="icon-[fluent--tag-16-regular] w-6 h-6" />
                <span class="flex-1 text-start text-ellipsis overflow-hidden whitespace-nowrap">{tag.name}</span>
                <span
                  class="icon-[fluent--chevron-down-16-regular] w-5 h-5 flex-shrink-0 transition-all {tagExpandedRecord[
                    tag.id
                  ]
                    ? ' rotate-0'
                    : '-rotate-90'}"
                />
              </RxButton>
              {#if tagExpandedRecord[tag.id]}
                <ul
                  class="pl-4 relative before:border-l-2 before:absolute before:h-full before:border-l-base-content/10 overflow-x-hidden"
                >
                  {#each tagsChallengesRecord[tag.id] as chal}
                    <RxLink ghost class="w-full" justify="start" href={`/playground/${activeGameId}#${chal.id}`}>
                      <span class="icon-[fluent--flag-16-regular] w-6 h-6" />
                      <span class="flex-1 text-start text-ellipsis overflow-hidden whitespace-nowrap">{chal.name}</span>
                      {#if selfSubmissions.find((item) => item.challenge_id === chal.id)}
                        <span class="icon-[fluent--checkmark-circle-16-regular] text-success w-5 h-5" />
                      {/if}
                    </RxLink>
                  {/each}
                </ul>
              {/if}
            </li>
          {/if}
        {/each}
        {#if mayHaveMoreChallenges}
          <RxButton
            class="w-full opacity-60"
            ghost
            on:click={() => {
              dispatch('loadMoreChallenges')
            }}
          >
            {$i18n.t('playground.loadMore')}
          </RxButton>
        {/if}
      </ul>
    </div>
  {:else}
    <div class="flex-1 flex-col">
      {#if playgrounds.length === 0 && filteredGames.length === 0}
        <p class="text-base font-semibold p-4 opacity-60 text-center">{$i18n.t('playground.emptyCategory')}</p>
      {/if}
      {#if playgrounds.length > 0}
        <h2 class="text-base font-semibold p-4 pb-0 flex flex-row items-center justify-center space-x-2 opacity-60">
          <span class="icon-[fluent--beaker-16-regular] w-6 h-6" />
          <span>
            {$i18n.t('playground.persistTitle')}
          </span>
        </h2>
        <div class="flex flex-col p-4">
          {#each playgrounds as item}
            <RxLink class="w-full" justify="start" ghost href={`/playground/${item.id}`}>
              <span class="icon-[fluent--bookmark-16-regular] w-6 h-6" />
              <span class="flex-1 text-start">{item.name}</span>
              <span class="icon-[fluent--chevron-right-16-regular] w-5 h-5" />
            </RxLink>
          {/each}
          {#if mayHaveMorePlaygrounds}
            <RxButton
              class="w-full opacity-60"
              ghost
              on:click={() => {
                dispatch('loadMorePlaygrounds')
              }}
            >
              {$i18n.t('playground.loadMore')}
            </RxButton>
          {/if}
        </div>
      {/if}
      {#if filteredGames.length > 0}
        <h2 class="text-base font-semibold p-4 pb-0 flex flex-row items-center justify-center space-x-2 opacity-60">
          <span class="icon-[fluent--flag-16-regular] w-6 h-6" />
          <span>
            {$i18n.t('playground.gamesTitle')}
          </span>
        </h2>
        <div class="flex flex-col p-4">
          {#each filteredGames as item}
            <RxLink class="w-full" justify="start" ghost href={`/playground/${item.id}`}>
              <span class="icon-[fluent--archive-16-regular] w-6 h-6" />
              <span class="flex-1 text-start">{item.name}</span>
              <span class="icon-[fluent--chevron-right-16-regular] w-5 h-5" />
            </RxLink>
          {/each}
          {#if mayHaveMoreGames}
            <RxButton
              class="w-full opacity-60"
              ghost
              on:click={() => {
                dispatch('loadMoreGames')
              }}
            >
              {$i18n.t('playground.loadMore')}
            </RxButton>
          {/if}
        </div>
      {/if}
    </div>
  {/if}
</OverlayScrollbarsComponent>
