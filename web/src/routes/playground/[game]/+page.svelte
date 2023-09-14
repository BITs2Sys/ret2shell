<script lang="ts">
  import { page } from '$app/stores'
  import { getChallenge } from '$lib/api/challenge'
  import { getGame } from '$lib/api/game'
  import Error from '$lib/blocks/Error.svelte'
  import RxArticle from '$lib/components/RxArticle.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxLink from '$lib/components/RxLink.svelte'
  import RxTag from '$lib/components/RxTag.svelte'
  import { i18n } from '$lib/i18n'
  import type { Challenge } from '$lib/models/challenge'
  import type { Game } from '$lib/models/game'
  import { platform } from '$lib/stores/platform'
  import { theme } from '$lib/stores/theme'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte'
  import Split from 'split.js'
  import { onMount } from 'svelte'
  import { quintOut } from 'svelte/easing'
  import { blur, fly } from 'svelte/transition'

  onMount(() => {
    Split(['#info-stack', '#work-stack'], {
      direction: 'vertical',
      gutterSize: 8,
      gutterAlign: 'center',
      minSize: 200,
      gutterStyle: (_dimension, gutterSize) => {
        return {
          height: `${gutterSize}px`,
          cursor: 'row-resize',
        }
      },
      gutter: (_index, direction) => {
        const gutter = document.createElement('div')
        gutter.className = `gutter gutter-${direction} border-b border-b-base-content/5`
        return gutter
      },
    })
  })

  let game: Game | null = null
  let error = 200

  let gameId = $page.params.game ? parseInt($page.params.game) || null : null
  if (gameId) {
    getGame(gameId)
      .then((res) => {
        game = res
      })
      .catch((err) => {
        error = (err as AxiosError).response?.status || 500
      })
  } else {
    error = 404
  }

  // challenge reactive
  let openedChallenges: Challenge[] = []
  let activeChallenge: Challenge | null = null
  let loadingNewChallenge = false
  let loadingPlaceHolder: HTMLDivElement
  let openedTabDivRecord: Record<number, HTMLDivElement> = {}

  page.subscribe((value) => {
    let challengeId = value.url.hash ? parseInt(value.url.hash.slice(1)) || null : null
    if (challengeId) {
      if (openedChallenges.find((chal) => chal.id === challengeId)) {
        activeChallenge = openedChallenges.find((chal) => chal.id === challengeId) || null
        if (openedTabDivRecord[challengeId]) {
          setTimeout(() => {
            if (challengeId && openedTabDivRecord[challengeId])
              openedTabDivRecord[challengeId].scrollIntoView({ behavior: 'smooth', inline: 'center' })
          }, 0)
        }
        return
      }
      loadingNewChallenge = true
      setTimeout(() => {
        if (loadingPlaceHolder) loadingPlaceHolder.scrollIntoView({ behavior: 'smooth' })
      }, 0)
      getChallenge(challengeId)
        .then((value) => {
          openedChallenges.push(value)
          activeChallenge = value
        })
        .catch((err) => {
          showMessage(
            'error',
            `${$i18n.t('playground.fetchChallengeFailed')}: ${(err as AxiosError).response?.data}`,
            5000
          )
        })
        .finally(() => {
          loadingNewChallenge = false
        })
    } else {
      activeChallenge = null
    }
  })
</script>

<svelte:head><title>{game?.name || $i18n.t('playground.gameLoading')} - {$platform.name}</title></svelte:head>

{#if error - 200 < 100}
  <div class="flex-1 flex flex-col overflow-x-hidden">
    <div id="info-stack" class="flex flex-col overflow-x-hidden">
      <div
        class="border-b border-b-base-content/5 flex flex-row items-center pr-2 space-x-2 backdrop-blur relative overflow-x-scroll flex-shrink-0"
        on:wheel={(e) => {
          e.currentTarget.scrollLeft += e.deltaY
        }}
      >
        <div class="bg-base-100 sticky left-0 p-2 flex-shrink-0 z-20">
          <RxLink ghost active={activeChallenge === null} href="#">
            <span class="w-4 h-4 icon-[fluent--pin-16-regular]" />
            {$i18n.t('playground.gameIntro')}
          </RxLink>
        </div>
        {#each openedChallenges as chal}
          <div
            class="join flex-shrink-0 transition-all"
            transition:fly={{
              x: -100,
              duration: 300,
              delay: 0,
              easing: quintOut,
            }}
            bind:this={openedTabDivRecord[chal.id]}
          >
            <RxLink
              class="join-item overflow-x-hidden max-w-[240px] flex-nowrap"
              ghost
              active={activeChallenge?.id === chal.id}
              href={`#${chal.id}`}
            >
              <span class="w-4 h-4 icon-[fluent--braces-16-regular] flex-shrink-0" />
              <span class="text-ellipsis overflow-hidden whitespace-nowrap">
                {chal.name}
              </span>
            </RxLink>
            <RxButton
              class="join-item ml-0"
              ghost
              on:click={() => {
                openedChallenges = openedChallenges.filter((c) => c.id !== chal.id)
                if (activeChallenge?.id === chal.id) {
                  activeChallenge = null
                  window.location.hash = '#'
                }
              }}
            >
              <span class="w-4 h-4 icon-[fluent--dismiss-16-regular]" />
            </RxButton>
          </div>
        {/each}
        {#if loadingNewChallenge}
          <div
            class="flex flex-row items-center space-x-2 opacity-80 w-48 flex-shrink-0"
            bind:this={loadingPlaceHolder}
          >
            <span class="loading loading-spinner loading-sm"></span>
            <span>{$i18n.t('playground.challengeLoading')}</span>
          </div>
        {/if}
      </div>
      <div class="flex-1 relative">
        <div class="absolute w-full h-full">
          <OverlayScrollbarsComponent
            options={{
              scrollbars: {
                theme: $theme.colorScheme === 'light' ? 'os-theme-dark' : 'os-theme-light',
                autoHide: 'scroll',
              },
            }}
            class="relative w-full h-full print:h-auto print:overflow-auto"
            defer
          >
            <div class="w-full flex p-6 pt-0 justify-center relative">
              {#if activeChallenge}
                <div class="flex flex-col w-full max-w-5xl px-6">
                  <RxArticle class="mt-4" content={activeChallenge?.content || $i18n.t('playground.emptyContent')} />
                </div>
              {:else}
                <div class="flex flex-col w-full max-w-5xl px-6">
                  <h1 class="font-bold text-center text-3xl p-6 pb-0">
                    {game?.name}
                  </h1>
                  <RxArticle class="mt-4" content={game?.introduction || $i18n.t('playground.emptyContent')} />
                  <div class="h-32" />
                </div>
              {/if}
            </div>
          </OverlayScrollbarsComponent>
        </div>
        {#if loadingNewChallenge}
          <div
            class="absolute bg-base-100/60 w-full h-full backdrop-blur-xl flex justify-center items-center space-x-2"
            transition:blur
          >
            <span class="loading loading-sm loading-spinner"></span>
            <span>{$i18n.t('playground.challengeLoading')}</span>
          </div>
        {/if}
      </div>
    </div>
    <div id="work-stack" class="flex flex-col backdrop-blur">
      <div class="border-b border-b-base-content/5 flex flex-row items-center p-2 space-x-2">
        <RxButton ghost active>
          <span class="w-4 h-4 icon-[fluent--code-16-regular]" />
          {$i18n.t('playground.terminal')}
        </RxButton>
        <RxButton ghost>
          <span class="w-4 h-4 icon-[fluent--checkmark-16-regular]" />
          {$i18n.t('playground.challengeAnswer')}
        </RxButton>
      </div>
    </div>
  </div>
{:else}
  <Error status={error} />
{/if}
