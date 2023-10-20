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
  import { getGameNotifications, getGameSelfSubmission } from '$lib/api/game'
  import ChallengeSidebar from './ChallengeSidebar.svelte'
  import TeamSidebar from './TeamSidebar.svelte'
  import HintsPanel from '$lib/blocks/challenge/HintsPanel.svelte'
  import TerminalPanel from '$lib/blocks/challenge/TerminalPanel.svelte'
  import RxArticle from '$lib/components/RxArticle.svelte'
  import RxButton from '$lib/components/RxButton.svelte'
  import RxLink from '$lib/components/RxLink.svelte'
  import { i18n } from '$lib/i18n'
  import type { Challenge, Tag } from '$lib/models/challenge'
  import { Permission } from '$lib/models/user'
  import { game } from '$lib/stores/game'
  import { theme } from '$lib/stores/theme'
  import { showMessage } from '$lib/stores/toast'
  import { user } from '$lib/stores/user'
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

  let screenWidth: number
  let toggleSidebar = false
  let toggleTeamSidebar = false
  $: showSidebar = screenWidth > 1024 // lg
  $: showTeamSidebar = screenWidth > 1280 // lg

  let challengeTotalPages: number = 0
  let challengePageSize = 200
  let challengePage: number = 1
  $: mayHaveMoreChallenges = challengePage < challengeTotalPages
  let tags: Tag[] = []
  let currentGameIdCache: number | null = null

  function getChallenges() {
    if ($game.current?.id) {
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
  }

  onMount(() => {
    if (!$game.team && !$user.permissions.find((p) => p === Permission.Devops || p === Permission.Organize)) {
      goto(`/games/${$game.current?.id}`).then(() => {
        showMessage('warning', $i18n.t('games.takePartInFirst'), 5000)
      })
    } else if ($user.permissions.find((p) => p === Permission.Devops || p === Permission.Organize)) {
      // showMessage('info', $i18n.t('games.adminWarning'), 5000)
    }
  })

  let gameUnsubscribe = game.subscribe((value) => {
    if (value.current?.id && currentGameIdCache !== value.current.id) {
      currentGameIdCache = value.current.id
      challengePage = 1
      $game.challenges = []
      getChallenges()
      getSelfSubmissions()
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
    $game.challenges = []
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
<svelte:window bind:innerWidth={screenWidth} />

<div class="flex-1 flex flex-row overflow-x-hidden">
  {#if showSidebar}
    <div
      class="w-1/5 h-[calc(100vh_-_4rem)] flex-shrink-0 flex flex-col min-w-[24rem] max-w-[32rem] bg-neutral/20 backdrop-blur border-r border-r-base-content/10 overflow-hidden"
    >
      <ChallengeSidebar
        selfSubmissions={$game.submissions}
        challenges={$game.challenges}
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
      <span class="swap-off icon-[fluent--navigation-20-regular] fill-current w-5 h-5"></span>
      <span class="swap-on icon-[fluent--dismiss-20-regular] fill-current w-5 h-5"></span>
    </label>
  {/if}
  <div class="flex-1 flex flex-col overflow-x-hidden">
    <div id="info-stack" class="flex flex-col overflow-x-hidden">
      <div class="border-b border-b-base-content/10 flex flex-row h-16 overflow-hidden bg-neutral/30 backdrop-blur">
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
      <div class="border-b border-b-base-content/5 bg-neutral/30 flex flex-row items-center p-2 space-x-2">
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
              getSelfSubmissions()
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
  {#if showTeamSidebar}
    <div
      class="w-1/5 h-[calc(100vh_-_4rem)] flex-shrink-0 flex flex-col min-w-[24rem] max-w-[32rem] bg-neutral/20 backdrop-blur border-l border-l-base-content/10 overflow-hidden"
    >
      <TeamSidebar {notifications} />
    </div>
  {:else}
    <label
      class="btn no-animation bg-base-content/5 border-none backdrop-blur btn-square btn-lg fixed right-6 bottom-6 z-10 swap swap-rotate"
    >
      <input
        type="checkbox"
        on:click={() => {
          toggleTeamSidebar = !toggleTeamSidebar
        }}
      />
      <span class="swap-off icon-[fluent--flag-20-regular] fill-current w-5 h-5"></span>
      <span class="swap-on icon-[fluent--dismiss-20-regular] fill-current w-5 h-5"></span>
    </label>
  {/if}
  {#if toggleTeamSidebar && !showTeamSidebar}
    <div
      class="fixed right-0 w-full flex flex-col max-w-[24rem] h-[calc(100vh_-_4rem)] overflow-hidden backdrop-blur bg-base-100/40 border-l border-l-base-content/10"
      transition:fly={{ delay: 100, duration: 300, x: 256, y: 0, opacity: 0, easing: quintOut }}
    >
      <TeamSidebar {notifications} />
    </div>
  {/if}
  {#if toggleSidebar && !showSidebar}
    <div
      class="fixed left-0 w-full flex flex-col max-w-[24rem] h-[calc(100vh_-_4rem)] overflow-hidden backdrop-blur bg-base-100/40 border-r border-r-base-content/10"
      transition:fly={{ delay: 100, duration: 300, x: -256, y: 0, opacity: 0, easing: quintOut }}
    >
      <ChallengeSidebar
        selfSubmissions={$game.submissions}
        challenges={$game.challenges}
        {tags}
        {mayHaveMoreChallenges}
        on:loadMoreChallenges={getMoreChallenges}
      />
    </div>
  {/if}
</div>
