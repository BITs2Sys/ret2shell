<script lang="ts">
  import RxButton from '$lib/components/RxButton.svelte'
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'
  import type { Challenge, Tag } from '$lib/models/challenge'
  import type { Submission } from '$lib/models/submission'
  import { game } from '$lib/stores/game'
  import { theme } from '$lib/stores/theme'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte'
  import { onMount } from 'svelte'

  export let challenges: Challenge[]
  export let tags: Tag[]
  export let selfSubmissions: Submission[]
  export let mayHaveMoreChallenges: boolean
  export let loading = false
  export let loadMoreChallengesCallback: () => void

  let tagsChallengesRecord: Record<number, Challenge[]> = {}

  $: {
    tagsChallengesRecord = {}
    tags.forEach((tag) => {
      tagsChallengesRecord[tag.id] = challenges.filter((challenge) => challenge.tag_id === tag.id)
    })
  }

  let tagExpandedRecord: Record<number, boolean> = {}

  onMount(() => {
    tagExpandedRecord = {}
    if (tags.length <= 3)
      tags.forEach((tag) => {
        tagExpandedRecord[tag.id] = true
      })
  })

  function handleTagExpand(tagId: number) {
    tagExpandedRecord[tagId] = !tagExpandedRecord[tagId]
    tagExpandedRecord = tagExpandedRecord
  }
</script>

<div class="w-full h-full flex flex-col">
  <h1
    class="font-bold flex flex-row justify-center items-center h-16 space-x-2 sticky top-0 border-b border-b-base-content/5 z-10"
  >
    <span class="icon-[fluent--flag-20-regular] w-5 h-5" />
    <span>
      {$i18n.t('games.challengeList')}
    </span>
  </h1>
  <OverlayScrollbarsComponent
    options={{
      scrollbars: { theme: $theme.colorScheme === 'light' ? 'os-theme-dark' : 'os-theme-light', autoHide: 'scroll' },
    }}
    class="w-full flex-1 flex flex-col relative print:hidden"
    defer
  >
    {#if loading}
      <div class="flex flex-row justify-center items-center h-16 space-x-2">
        <span class="loading loading-spinner loading-sm" />
        <span class="text-base">{$i18n.t('game.fetchingList')}</span>
      </div>
    {:else if challenges.length === 0}
      <div class="min-h-full flex flex-col justify-center items-center">
        <p class="text-base font-bold p-4 opacity-60 text-center">{$i18n.t('playground.emptyCategory')}</p>
      </div>
    {/if}
    <div class="flex-1 flex-col p-4 overflow-x-hidden">
      <ul class="relative space-y-2">
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
                  class="pl-4 relative before:border-l-2 before:absolute before:h-full before:border-l-base-content/10 overflow-x-hidden space-y-2 mt-2"
                >
                  {#each tagsChallengesRecord[tag.id] as chal}
                    <RxLink
                      ghost
                      class="w-full"
                      justify="start"
                      href={`/games/${$game.current?.id}/challenges#${chal.id}`}
                    >
                      {#if selfSubmissions.find((item) => item.challenge_id === chal.id)}
                        <span class="icon-[fluent--checkmark-circle-20-regular] text-success w-5 h-5" />
                      {:else}
                        <span class="icon-[fluent--question-circle-20-regular] opacity-60 w-5 h-5" />
                      {/if}
                      <span
                        class={`flex-1 text-start text-ellipsis overflow-hidden whitespace-nowrap ${
                          selfSubmissions.find((item) => item.challenge_id === chal.id) ? 'line-through opacity-60' : ''
                        }`}
                      >
                        {chal.name}
                      </span>
                      <span class="opacity-60 font-bold">{chal.current_score} pts</span>
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
              loadMoreChallengesCallback()
            }}
          >
            {$i18n.t('playground.loadMore')}
          </RxButton>
        {/if}
      </ul>
    </div>
  </OverlayScrollbarsComponent>
</div>
