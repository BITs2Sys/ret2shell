<script lang="ts">
  import { passiveSupport } from 'passive-events-support/src/utils'
  passiveSupport({
    events: ['touchstart'],
    listeners: [
      {
        element: 'gutter',
        event: 'touchstart',
      },
    ],
  })

  import { goto } from '$app/navigation'
  import { getChallengeHints, getChallengeList, getTagList } from '$lib/api/challenge'
  import { getGameNotifications } from '$lib/api/game'
  import ChallengeSidebar from './ChallengeSidebar.svelte'
  import TeamSidebar from './TeamSidebar.svelte'
  import HintsPanel from '$lib/blocks/challenge/HintsPanel.svelte'
  import TerminalPanel from '$lib/blocks/challenge/TerminalPanel.svelte'
  import RxArticle from '$lib/components/RxArticle.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'
  import type { Challenge, Tag } from '$lib/models/challenge'
  import { game, refreshTeam } from '$lib/stores/game'
  import { theme } from '$lib/stores/theme'
  import { showMessage } from '$lib/stores/toast'
  import type { AxiosError } from 'axios'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte'
  import { onDestroy, onMount } from 'svelte'
  import { quintOut } from 'svelte/easing'
  import { fly, blur } from 'svelte/transition'
  import { page } from '$app/stores'
  import { getChallenge } from '$lib/api/challenge'
  import Split from 'split.js'
  import type { Notification } from '$lib/models/game'
  import type { Hint } from '$lib/models/hint'
  import ChallengePanel from '$lib/blocks/challenge/ChallengePanel.svelte'
  import SolvedPanel from '$lib/blocks/challenge/SolvedPanel.svelte'
  import SidebarLayout from '$lib/blocks/SidebarLayout.svelte'

  let challengeTotalPages: number = 0
  let challengePageSize = 200
  let challengePage: number = 1
  $: mayHaveMoreChallenges = challengePage < challengeTotalPages
  let tags: Tag[] = []
  let currentGameIdCache: number | null = null
  let loading = false

  if (!$game.inProgress()) {
    goto(`/playground/${$game.current?.id}${$page.url.hash}`, { replaceState: true })
  }

  function removeDuplicateChallenges() {
    $game.challenges = $game.challenges.filter((challenge, index, self) => {
      return self.findIndex((c) => c.id === challenge.id) === index
    })
  }

  function getChallenges() {
    if ($game.current?.id) {
      loading = true
      getChallengeList($game.current?.id, challengePage, challengePageSize)
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
          removeDuplicateChallenges()
          challengeTotalPages = res.total
        })
        .catch((err) => {
          if (
            (err as AxiosError).response?.status === 403 &&
            (err as AxiosError).response?.data === 'you have not joined this game'
          ) {
            showMessage('warning', $i18n.t('games.takePartInFirst'), 5000)
            goto(`/games/${$game.current?.id}`, { replaceState: true })
          } else {
            showMessage(
              'error',
              `${$i18n.t('playground.noMoreChallenges')}: ${(err as AxiosError).response?.data}`,
              5000
            )
          }
        })
        .finally(() => {
          loading = false
        })
    }
  }

  function getMoreChallenges() {
    challengePage++
    if (challengePage <= challengeTotalPages) getChallenges()
  }

  let gameUnsubscribe = game.subscribe((value) => {
    if (value.current?.id && currentGameIdCache !== value.current.id) {
      currentGameIdCache = value.current.id
      challengePage = 1
      $game.challenges = []
      getChallenges()
      getTagList()
        .then((res) => {
          tags = res.toSorted((a, b) => (a.name > b.name ? 1 : a.name === b.name ? 0 : -1))
        })
        .catch((err) => {
          showMessage('error', `${$i18n.t('playground.fetchTagsFailed')}: ${(err as AxiosError).response?.data}`, 5000)
        })
    }
  })

  onDestroy(() => {
    gameUnsubscribe()
  })

  onMount(() => {
    Split(['#info-stack', '#work-stack'], {
      direction: 'vertical',
      gutterSize: 8,
      gutterAlign: 'center',
      minSize: 200,
      sizes: [70, 30],
      gutterStyle: (_dimension, gutterSize) => {
        return {
          height: `${gutterSize}px`,
          cursor: 'row-resize',
        }
      },
      gutter: (_index, direction) => {
        const gutter = document.createElement('div')
        gutter.className = `gutter gutter-${direction} border-b border-b-base-content/10 hover:border-b-primary transition-all duration-300`
        return gutter
      },
    })
  })
  // challenge reactive
  let openedChallenges: Challenge[] = []
  let activeChallenge: Challenge | null = null
  let loadingNewChallenge = false
  let loadingPlaceHolder: HTMLDivElement
  let openedTabDivRecord: Record<number, HTMLDivElement> = {}
  let hints: Hint[] = []

  $: watchChallenge(activeChallenge)
  function watchChallenge(challenge: Challenge | null) {
    if (challenge?.id)
      getChallengeHints(challenge.id)
        .then((res) => {
          hints = res
        })
        .catch((err) => {
          showMessage('error', `${$i18n.t('playground.fetchHintsFailed')}: ${(err as AxiosError).response?.data}`, 5000)
        })
    else hints = []
  }

  let unsubscribe = page.subscribe((value) => {
    let challengeId = value.url.hash ? parseInt(value.url.hash.slice(1)) || null : null
    if (challengeId && challengeId !== activeChallenge?.id) {
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
  onDestroy(unsubscribe)

  // bottom panel
  let bottomTab = 0

  // notifications
  let notifications: Notification[] = []

  function fetchNotifications() {
    if ($game.current) {
      getGameNotifications($game.current.id, 1, 200)
        .then((res) => {
          notifications = res.notifications
        })
        .catch((err) => {
          showMessage(
            'error',
            `${$i18n.t('playground.fetchNotificationsFailed')}: ${(err as AxiosError).response?.data}`,
            5000
          )
        })
    }
  }

  fetchNotifications()

  let timer = setInterval(() => {
    fetchNotifications()
  }, 5000)

  onDestroy(() => {
    clearInterval(timer)
  })
</script>

<svelte:head>
  <title>{$i18n.t('game.challenges')} - {$game.current?.name}</title>
</svelte:head>

<SidebarLayout
  leftSidebar={ChallengeSidebar}
  leftProps={{
    loading,
    selfSubmissions: $game.submissions,
    challenges: $game.challenges,
    tags,
    mayHaveMoreChallenges,
    loadMoreChallengesCallback: getMoreChallenges,
  }}
  rightSidebar={TeamSidebar}
  rightProps={{
    notifications,
  }}
>
  <div class="flex-1 flex flex-col overflow-x-hidden">
    <div id="info-stack" class="flex flex-col overflow-x-hidden">
      <div class="border-b border-b-base-content/10 flex flex-row h-16 overflow-hidden bg-neutral/20 backdrop-blur">
        <div class="sticky left-0 p-2 flex-shrink-0 z-20">
          <RxLink ghost active={activeChallenge === null} href="#">
            <span class="w-5 h-5 icon-[fluent--pin-20-regular]" />
            <span>
              {$i18n.t('playground.gameIntro')}
            </span>
          </RxLink>
        </div>
        <div
          class="flex-1 flex flex-row items-center pr-2 space-x-2 relative overflow-x-scroll flex-shrink-0 h-16 overflow-y-hidden"
          on:wheel|passive={(e) => {
            e.currentTarget.scrollLeft += e.deltaY
          }}
        >
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
                <span class="w-5 h-5 icon-[fluent--braces-20-regular] flex-shrink-0" />
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
                <span class="w-5 h-5 icon-[fluent--dismiss-20-regular]" />
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
            <div class="w-full min-h-full flex flex-col items-center">
              {#if activeChallenge}
                <ChallengePanel
                  challenge={activeChallenge}
                  solved={$game.submissions.find((s) => s.challenge_id === activeChallenge?.id) !== undefined}
                />
              {:else}
                <div class="flex flex-col w-full max-w-5xl px-6">
                  <h1 class="font-bold text-center text-3xl p-6 pt-12 pb-0">
                    {$game.current?.name}
                  </h1>
                  <RxArticle class="mt-4" content={$game.current?.introduction || $i18n.t('playground.emptyContent')} />
                  <div class="h-32" />
                </div>
              {/if}
            </div>
          </OverlayScrollbarsComponent>
        </div>
        {#if loadingNewChallenge}
          <div
            class="absolute w-full h-full backdrop-blur-xl flex justify-center items-center space-x-2"
            transition:blur={{ amount: 20, duration: 300 }}
          >
            <span class="loading loading-sm loading-spinner"></span>
            <span>{$i18n.t('playground.challengeLoading')}</span>
          </div>
        {/if}
      </div>
    </div>
    <div id="work-stack" class="flex flex-col backdrop-blur min-h-16">
      <div class="border-b border-b-base-content/5 bg-neutral/20 flex flex-row items-center p-2 space-x-2">
        <RxButton
          ghost
          active={bottomTab === 0}
          on:click={() => {
            bottomTab = 0
          }}
        >
          <span class="w-5 h-5 icon-[fluent--code-20-regular]" />
          {$i18n.t('playground.terminal')}
        </RxButton>
        <RxButton
          ghost
          active={bottomTab === 1}
          on:click={() => {
            bottomTab = 1
          }}
          disabled={hints.length === 0}
        >
          <span class="w-5 h-5 icon-[fluent--info-20-regular]" />
          {$i18n.t('playground.challengeHints')}
        </RxButton>
        <RxButton
          ghost
          active={bottomTab === 2}
          on:click={() => {
            bottomTab = 2
          }}
        >
          <span class="w-5 h-5 icon-[fluent--people-20-regular]" />
          {$i18n.t('playground.challengeSolves')}
        </RxButton>
        <RxButton ghost>
          <span class="w-5 h-5 icon-[fluent--thumb-dislike-20-regular]" />
          {$i18n.t('challenge.gankAuthor')}
        </RxButton>
      </div>
      {#if bottomTab === 0}
        <TerminalPanel
          game={$game.current}
          challenge={activeChallenge}
          availableChallenges={$game.challenges}
          class="p-6"
          on:executed={(e) => {
            if (e.detail.code === 0 && e.detail.cmd === 'submit') {
              refreshTeam()
            }
          }}
        />
      {:else if bottomTab === 1}
        <HintsPanel {hints} />
      {:else if bottomTab === 2}
        <SolvedPanel challenge={activeChallenge} />
      {/if}
    </div>
  </div>
</SidebarLayout>
